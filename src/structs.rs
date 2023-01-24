use rocket::{
    data::{FromData, Outcome},
    http::{ext::IntoOwned, uri::Reference, Status},
    request::{self, FromRequest, Request},
    Data,
};
use snafu::prelude::*;
use std::convert::{Infallible, TryFrom};

pub(crate) struct CurrentRes {
    pub artists: String,
    pub name: String,
    pub remaining: u64,
    pub state: bool,
}

impl CurrentRes {
    pub(crate) fn to_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.artists, self.name, self.remaining, self.state
        )
    }
}

pub(crate) enum Operation {
    Prev,
    Next,
    Current,
    Pause,
    Play,
    Auth,
}

impl Operation {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Prev => "previous",
            Self::Next => "next",
            Self::Pause => "pause",
            Self::Play => "play",
            Self::Current => "currently-playing",
            Self::Auth => "auth",
        }
    }
}

impl<'a> TryFrom<&'a str> for Operation {
    type Error = SpottyBarError;
    fn try_from(v: &'a str) -> Result<Self, Self::Error> {
        match v {
            "play" => Ok(Self::Play),
            "pause" => Ok(Self::Pause),
            "next" => Ok(Self::Next),
            "prev" | "previous" => Ok(Self::Prev),
            "curr" | "current" | "currently-playing" => Ok(Self::Current),
            "auth" => Ok(Self::Auth),

            _ => Err(SpottyBarError::InvalidConversion),
        }
    }
}

#[derive(Debug, Snafu)]
pub(crate) enum SpottyBarError {
    #[snafu(display("Invalid direction"))]
    InvalidDirection,

    #[snafu(display("Invalid conversion"))]
    InvalidConversion,

    #[snafu(display("Request Error"))]
    RequestError,

    #[snafu(display("Post Request Error"))]
    PostRequestError,
}