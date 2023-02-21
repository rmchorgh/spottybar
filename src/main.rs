#![feature(decl_macro)]
#[macro_use]
extern crate rocket;

pub mod auth;
pub mod constants;
pub mod controls;
pub mod structs;

use auth::{auth_link, key, main as start};
use controls::{current, track};
use structs::Operation;

use std::any::Any;
use std::env::args;
use std::thread::spawn;

#[tokio::main]
async fn main() {
    let a: Vec<String> = args().collect();
    if a.len() < 2 {
        panic!("No verb included.")
    } else if a.len() > 2 {
        panic!("Too many verbs included.")
    }

    let auth = key().unwrap();

    let verb: Operation = match a[1].as_str().try_into() {
        Ok(v) => v,
        Err(_) => panic!("Not a valid operation verb."),
    };

    if let Operation::Auth = verb {
        let _ = authorize();
    } else {
        let v = match verb {
            Operation::Current => current(auth).await,
            _ => track(verb.clone(), auth.clone(), 1)
                .await
        };

        println!("{}", v.expect("Should have changed something."));
    }
}

fn authorize() -> Result<(), Box<dyn Any + Send>> {
    println!("Starting auth server.");
    return spawn(|| {
        auth_link();
        start();
    })
    .join();
}
