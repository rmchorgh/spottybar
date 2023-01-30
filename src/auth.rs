extern crate dirs;
extern crate rocket;
extern crate urlencoding;

use rocket::response::content::RawHtml;
use rocket::{custom, Config, Shutdown};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::fs::read_to_string;
use std::process::Command;
use urlencoding::encode;

use crate::constants::{CLIENT_ID, REDIRECT, SCOPES};

fn key_path() -> String {
    format!(
        "{}/.config/spottybar/key",
        dirs::home_dir().unwrap().display()
    )
}

pub(crate) fn key() -> Result<String, Box<dyn std::error::Error>> {
    let mut f = read_to_string(key_path())
    .expect("Couldn't get key.");

    if f.contains("\n") {
        f.pop();
    }

    Ok(format!("Bearer {}", f))
}

pub(crate) fn auth_link() {
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
pub(crate) fn token_page() -> RawHtml<&'static str> {
    RawHtml(r#"
    <!DOCTYPE html>
    <html>
        <head>
            <title>Spotify Web API</title>
        </head>
        <body>
            <p>Recieved</p>
            <script>
                fetch(`http://localhost:3000/token/${window.location.hash.substr(14, window.location.hash.length - 48)}`, {
                    method: 'POST',
                    body: window.location.hash.substr(14, window.location.hash.length - 48)
                })
                .catch(console.error)
            </script>
        </body>
    </html>
    "#)
}

#[post("/token/<token>")]
pub(crate) async fn save_token(token: String, shutdown: Shutdown) -> &'static str {
    println!("{}", token);
    let mut f = File::create(key_path()).await.expect("Should've created a file.");
    f.write_all(token.as_bytes()).await.expect("Couldn't write to file.");

    shutdown.notify();
    "saved token"
}

#[launch]
pub(crate) fn rocket() -> _ {
    let c = Config {
        port: 3000,
        ..Config::default()
    };
    custom(&c).mount("/", routes![token_page])
    .mount("/", routes![save_token])
}
