use std::{process::{Command, ExitStatus}, io::Error};

pub(crate) fn wait() -> Result<ExitStatus, Error> {
    println!("WAITING");
    let mut child = Command::new("sleep").arg("5").spawn().unwrap();
    child.wait()
}
