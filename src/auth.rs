extern crate dirs;
extern crate rocket;
extern crate urlencoding;

use rocket::response::content::RawHtml;
use rocket::{build, custom, Config, Shutdown, Request};
use std::fs::read_to_string;
use std::process::Command;
use urlencoding::encode;

use crate::constants::{CLIENT_ID, REDIRECT, SCOPES};

pub(crate) fn key() -> Result<String, Box<dyn std::error::Error>> {
    let mut f = read_to_string(format!(
        "{}/.config/spottybar/key",
        dirs::home_dir().unwrap().display()
    ))
    .expect("Couldn't get key.");

    if f.contains("\n") {
        f.pop();
    }

    Ok(format!("Bearer {}", f))
}

pub(crate) fn authLink() {
    let link = format!(
        "https://accounts.spotify.com/authorize?client_id={}&redirect_uri={}&scope={}&response_type=token", 
        CLIENT_ID, 
        encode(REDIRECT), 
        encode(SCOPES)
    );
    println!("Opening {} in Firefox.", link);
    Command::new("xdg-open")
        .arg(link)
        .output()
        .expect("Couldn't open link.");
}

#[get("/token")]
pub(crate) async fn token(t: Token, shutdown: Shutdown) -> &'static str {
    println!("fragment: {}", t);
    shutdown.notify();
    "auth page"
}

#[launch]
pub(crate) fn rocket() -> _ {
    let c = Config {
        port: 3000,
        ..Config::default()
    };
    custom(&c).mount("/", routes![token])
}
