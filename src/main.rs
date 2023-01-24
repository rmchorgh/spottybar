#![feature(decl_macro)]
#[macro_use]
extern crate rocket;

pub mod auth;
pub mod constants;
pub mod controls;
pub mod structs;

use auth::{auth_link, key, main as start};
use controls::{current, track};

use std::env::args;
use std::thread::spawn;
use structs::Operation;

#[tokio::main]
async fn main() {
    let auth = key().unwrap();

    let a: Vec<String> = args().collect();
    if a.len() < 2 {
        panic!("No verb included.")
    } else if a.len() > 2 {
        panic!("Too many verbs included.")
    }

    let verb: Operation = a[1].as_str().try_into().unwrap();
    if let Operation::Auth = verb {
        println!("Starting auth server.");
        spawn(|| {
            auth_link();
            start();
        })
        .join()
        .expect("Server thread panicked.")
    } else {
        let v = match verb {
            Operation::Current => current(auth).await,
            _ => match track(verb, auth.clone()).await {
                Ok(_) => current(auth).await,
                Err(x) => Err(x),
            },
        };

        println!("{}", v.expect("Should have changed something."));
    }
}
