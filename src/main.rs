extern crate dirs;
extern crate reqwest;
extern crate tokio;

use snafu::prelude::*;
use std::convert::TryFrom;
use std::env::args;
use std::fs::read_to_string;

static SPOTIFY: &str = "https://api.spotify.com/v1/me/player/";

#[tokio::main]
async fn main() {
    let auth = key().unwrap();
    println!("{}", auth);

    let a: Vec<String> = args().collect();
    println!("{:?}", a);

    if a.len() < 2 {
        panic!("No verb included.")
    }

    if a.len() > 2 {
        panic!("Too many verbs included.")
    }

    let verb: TrackOp = a[1].as_str().try_into().unwrap();
    let v = match verb {
        TrackOp::Next | TrackOp::Prev => track(verb, auth).await,
        TrackOp::Play | TrackOp::Pause => state(verb, auth).await,
        TrackOp::Current => current(auth).await,
    };

    println!("{}", v.expect("Should have changed something."));
}

fn key() -> Result<String, Box<dyn std::error::Error>> {
    let f = read_to_string(format!(
        "{}/.config/spottybar/key",
        dirs::home_dir().unwrap().display()
    ))
    .expect("Couldn't get key.");

    Ok(format!("Bearer: {}", f))
}

#[derive(Debug, Snafu)]
enum SpottyBarError {
    #[snafu(display("Invalid direction"))]
    InvalidDirection,

    #[snafu(display("Invalid conversion"))]
    InvalidConversion,
}

async fn current(auth: String) -> Result<String, Box<dyn std::error::Error>> {
    let c = reqwest::Client::new();
    let body = c
        .get(format!("{}/currently-playing", SPOTIFY))
        .header("Authorization", auth)
        .send()
        .await?
        .text()
        .await?;

    println!("Body:\n {}", body);

    Ok(body)
}

enum TrackOp {
    Prev,
    Next,
    Current,
    Pause,
    Play,
}

impl TrackOp {
    fn as_str(&self) -> &'static str {
        match self {
            TrackOp::Prev => "previous",
            TrackOp::Next => "next",
            TrackOp::Pause => "pause",
            TrackOp::Play => "play",
            Self::Current => "currently-playing",
        }
    }
}

impl<'a> TryFrom<&'a str> for TrackOp {
    type Error = SpottyBarError;
    fn try_from(v: &'a str) -> Result<Self, Self::Error> {
        match v {
            "play" => Ok(TrackOp::Play),
            "pause" => Ok(TrackOp::Pause),
            "next" => Ok(TrackOp::Next),
            "prev" | "previous" => Ok(TrackOp::Prev),
            "current" | "currently-playing" => Ok(TrackOp::Current),
            _ => Err(SpottyBarError::InvalidConversion),
        }
    }
}

async fn track(dir: TrackOp, auth: String) -> Result<String, Box<dyn std::error::Error>> {
    if let TrackOp::Play | TrackOp::Pause = dir {
        return Err(Box::new(SpottyBarError::InvalidDirection));
    }

    let c = reqwest::Client::new();
    let body = c
        .post(dir.as_str())
        .header("Authorization", auth)
        .send()
        .await?
        .text()
        .await?;

    println!("Body:\n {}", body);

    Ok(body)
}

async fn state(dir: TrackOp, auth: String) -> Result<String, Box<dyn std::error::Error>> {
    if let TrackOp::Next | TrackOp::Prev = dir {
        return Err(Box::new(SpottyBarError::InvalidDirection));
    }

    let c = reqwest::Client::new();
    let body = c
        .put(dir.as_str())
        .header("Authorization", auth)
        .send()
        .await?
        .text()
        .await?;

    println!("Body:\n {}", body);

    Ok(body)
}
