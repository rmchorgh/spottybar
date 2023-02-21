extern crate reqwest;
extern crate serde;
extern crate tokio;

use crate::auth::key;
use crate::authorize;
use crate::constants::SPOTIFY;
use crate::structs::{CurrentRes, Operation, SpottyBarError};

use reqwest::header::{AUTHORIZATION, CONTENT_LENGTH};
use reqwest::Client;
use rocket::futures::future::BoxFuture;
use rocket::futures::FutureExt;
use serde_json::Value;

pub(crate) fn current(auth: String) -> BoxFuture<'static, Result<String, SpottyBarError>> {
    async move {
        let c = reqwest::Client::new();
        let body = c
            .get(format!("{}currently-playing", SPOTIFY))
            .header(AUTHORIZATION, auth)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let v: Value = serde_json::from_str(&body).unwrap();

        let a = v["item"]["artists"]
            .as_array()
            .unwrap()
            .into_iter()
            .fold("".to_string(), |acc, x| {
                format!("{}, {}", acc, x["name"].as_str().unwrap())
            });

        let res = CurrentRes {
            remaining: v["item"]["duration_ms"].as_u64().unwrap()
                - v["progress_ms"].as_u64().unwrap(),
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
    .boxed()
}

pub(crate) fn track(
    dir: Operation,
    auth: String,
    tries: i8,
) -> BoxFuture<'static, Result<String, SpottyBarError>> {
    async move {
        let c = Client::new();
        let url = format!("{}{}", SPOTIFY, dir.as_str());
        let rb = match dir {
            Operation::Next | Operation::Prev => c.post(url),
            Operation::Play | Operation::Pause => c.put(url),
            _ => return Err(SpottyBarError::InvalidDirection),
        };

        let body = rb
            .header(AUTHORIZATION, auth.clone())
            .header(CONTENT_LENGTH, 0)
            .body("")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        if body.len() == 0 {
            return Ok(current(auth).await.unwrap());
        } else {
            let contains_expired = body.contains("access token");
            if contains_expired && tries >= 0 && authorize().is_ok() {
                let new_auth = key().unwrap();
                return track(dir, new_auth, tries - 1).await;
            } else if contains_expired && tries < 0 {
                return Err(SpottyBarError::TokenExpired);
            } else {
                println!("Body:\n{}", body);
                return Err(SpottyBarError::RequestError);
            }
        }
    }
    .boxed()
}
