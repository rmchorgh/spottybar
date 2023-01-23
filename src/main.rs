extern crate dirs;
extern crate reqwest;
extern crate serde;
extern crate tokio;
#[macro_use]
extern crate rocket;

use reqwest::header::{AUTHORIZATION, CONTENT_LENGTH};
use serde_json::Value;
use snafu::prelude::*;
use std::convert::TryFrom;
use std::env::args;
use std::fs::read_to_string;

static SPOTIFY: &str = "https://api.spotify.com/v1/me/player/";

#[tokio::main]
async fn main() {
    let auth = key().unwrap();

    let a: Vec<String> = args().collect();
    if a.len() < 2 {
        panic!("No verb included.")
    } else if a.len() > 2 {
        panic!("Too many verbs included.")
    }

    let verb: TrackOp = a[1].as_str().try_into().unwrap();
    let v = match verb {
        TrackOp::Current => current(auth).await,
        _ => match track(verb, auth.clone()).await {
            Ok(_) => current(auth).await,
            Err(x) => Err(x),
        },
    };

    println!("{}", v.expect("Should have changed something."));
}

fn key() -> Result<String, Box<dyn std::error::Error>> {
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

#[derive(Debug, Snafu)]
enum SpottyBarError {
    #[snafu(display("Invalid direction"))]
    InvalidDirection,

    #[snafu(display("Invalid conversion"))]
    InvalidConversion,

    #[snafu(display("Request Error"))]
    RequestError,
}

struct CurrentRes {
    artists: String,
    name: String,
    remaining: u64,
    state: bool,
}

impl CurrentRes {
    fn to_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.artists, self.name, self.remaining, self.state
        )
    }
}

async fn current(auth: String) -> Result<String, Box<dyn std::error::Error>> {
    let c = reqwest::Client::new();
    let body = c
        .get(format!("{}currently-playing", SPOTIFY))
        .header(AUTHORIZATION, auth)
        .send()
        .await?
        .text()
        .await?;

    // println!("Body:\n {}", body);

    let v: Value = serde_json::from_str(&body)?;

    let a = v["item"]["artists"]
        .as_array()
        .unwrap()
        .into_iter()
        .fold("".to_string(), |acc, x| {
            format!("{}, {}", acc, x["name"].as_str().unwrap())
        });

    let res = CurrentRes {
        remaining: v["item"]["duration_ms"].as_u64().unwrap() - v["progress_ms"].as_u64().unwrap(),
        state: if v["is_playing"] == "true" {
            true
        } else {
            false
        },
        name: v["item"]["name"].as_str().unwrap().to_string(),
        artists: (&a[2..]).to_string(),
    };

    Ok(res.to_string())
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
            Self::Prev => "previous",
            Self::Next => "next",
            Self::Pause => "pause",
            Self::Play => "play",
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
            "curr" | "current" | "currently-playing" => Ok(TrackOp::Current),
            _ => Err(SpottyBarError::InvalidConversion),
        }
    }
}

async fn track(dir: TrackOp, auth: String) -> Result<String, Box<dyn std::error::Error>> {
    let c = reqwest::Client::new();

    let url = format!("{}{}", SPOTIFY, dir.as_str());
    let rb = match dir {
        TrackOp::Next | TrackOp::Prev => c.post(url),
        TrackOp::Play | TrackOp::Pause => c.put(url),
        TrackOp::Current => return Err(Box::new(SpottyBarError::InvalidDirection)),
    };

    let body = rb
        .header(AUTHORIZATION, auth)
        .header(CONTENT_LENGTH, 0)
        .body("")
        .send()
        .await?
        .text()
        .await?;

    if body.len() == 0 {
        Ok(format!("{}ed track", dir.as_str()))
    } else {
        println!("Body:\n {}", body);
        Err(Box::new(SpottyBarError::RequestError))
    }
}
