extern crate reqwest;
extern crate serde;
extern crate tokio;

use crate::constants::SPOTIFY;
use crate::structs::{CurrentRes, Operation, SpottyBarError};

use reqwest::header::{AUTHORIZATION, CONTENT_LENGTH};
use serde_json::Value;

pub(crate) async fn current(auth: String) -> Result<String, Box<dyn std::error::Error>> {
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

pub(crate) async fn track(
    dir: Operation,
    auth: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let c = reqwest::Client::new();

    let url = format!("{}{}", SPOTIFY, dir.as_str());
    let rb = match dir {
        Operation::Next | Operation::Prev => c.post(url),
        Operation::Play | Operation::Pause => c.put(url),
        _ => return Err(Box::new(SpottyBarError::InvalidDirection)),
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
