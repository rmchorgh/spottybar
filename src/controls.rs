extern crate reqwest;
extern crate serde;
extern crate tokio;

use std::fmt::format;
use std::process::Command;

use crate::auth::key;
use crate::authorize;
use crate::constants::SPOTIFY;
use crate::structs::{DevicesRes, Device, CurrentRes, Operation, SpottyBarError};
use crate::utils::wait;

use reqwest::header::{AUTHORIZATION, CONTENT_LENGTH};
use reqwest::Client;
use rocket::futures::future::BoxFuture;
use rocket::futures::FutureExt;
use serde_json::{Value, json};

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
        println!("{}", v);

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
        println!("TRACK");
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
            println!("FOUND ERROR IN TRACK");
            if tries < 0 {
                println!("Body:\n{}", body);
                return Err(SpottyBarError::RequestError);
            }

            let expired = body.contains("access token");
            if expired && authorize().is_ok() {
                let new_auth = key().unwrap();
                return track(dir, new_auth, tries - 1).await;
            }

            let nodevice = body.contains("NO_ACTIVE_DEVICE");
            if nodevice {
                println!("NO DEVICE IN TRACK");
                let thisDevice = devices(auth.clone(), 1).await.unwrap();
                println!("GOT DEVICE ID");

                match setDevice(auth.clone(), thisDevice).await {
                    Ok(_) => return track(Operation::Next, auth, tries - 1).await,
                    Err(_) => return Err(SpottyBarError::RequestError)
                }
            }

            Err(SpottyBarError::RequestError)
        }
    }
    .boxed()
}

pub(crate) fn setDevice(auth: String, device: Device) -> BoxFuture<'static, Result<(), SpottyBarError>> {
    async move {
        println!("SETTING DEVICE");

        let c = Client::new();
        let url = format!("{}{}", SPOTIFY, "me/player/queue");
        let rb = c.post(url);
        let body = rb
            .header(AUTHORIZATION, auth.clone())
            .query(&[("uri", "spotify:track:4iV5W9uYEdYUVa79Axb7Rh"), ("device_id", device.id.as_str())])
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        println!("setdevice body: {}", body);
        return Ok(());
    }
    .boxed()
}

pub(crate) fn devices(auth: String, tries: i8) -> BoxFuture<'static, Result<Device, SpottyBarError>> {
    async move {
        println!("GETTING DEVICES");
        if tries < 0 {
            return Err(SpottyBarError::RequestError)
        }

        let c = Client::new();
        let url = format!("{}{}", SPOTIFY, "devices");
        let rb = c.get(url);
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

        println!("devices body:\n{}", body);

        match serde_json::from_str::<DevicesRes>(&body) {
            Ok(x) => {
                if x.devices.len() == 0 {
                    println!("NO DEVICES IN JSON RES");
                    // TODO: make this an environment variable
                    let command_str = "brew services restart spotifyd";
                    let mut command_vec = command_str.split(' ').collect::<Vec<& str>>();

                    let shell = command_vec[0];
                    let args = command_vec.drain(1..command_vec.len());

                    println!("{} {:?}", shell, args);
                    let cmd = Command::new(shell)
                        .args(args)
                        .output();

                    match cmd {
                        Err(_) => {
                            println!("Should've started spotifyd.");
                            return Err(SpottyBarError::NoActiveDevice)
                        },
                        Ok(x) => {
                            println!("Started spotifyd.\n{:?}", x.stdout);
                            match wait() {
                                Ok(_) => return devices(auth, tries - 1).await,
                                Err(_) => return Err(SpottyBarError::NoActiveDevice)
                            }
                        }
                    }
                }

                // TODO: make this an environment variable
                let name = "r2MBPd";
                let coll: Vec<Device> = x.devices.into_iter().filter(|y| y.name == name).collect();
                println!("FILTERED DEVICES");

                match coll.to_vec().first() {
                    Some(spotifyd) => return Ok(spotifyd.to_owned()),
                    None => return Err(SpottyBarError::NoActiveDevice)
                }
            },
            Err(x) => {
                println!("{}", x);
                return Err(SpottyBarError::RequestError)
            }
        }
    }
    .boxed()
    
}
