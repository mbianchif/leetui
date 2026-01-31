use std::fmt::{self, Display};

use reqwest::header::InvalidHeaderValue;

pub type Result<T> = std::result::Result<T, LeetCodeErr>;

/// The error type for the api.
#[derive(Debug, Deserialize)]
pub enum LeetCodeErr {
    Api(String),
    Auth(String),
    InvalidHeaders(String),
    Reqwest(String),
}
use LeetCodeErr::*;
use serde::Deserialize;

impl std::error::Error for LeetCodeErr {}

impl Display for LeetCodeErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Api(e) => write!(f, "api error: {e}"),
            Auth(e) => write!(f, "auth error: {e}"),
            InvalidHeaders(e) => write!(f, "invalid headers: {e}"),
            Reqwest(e) => write!(f, "request error: {e}"),
        }
    }
}

impl From<InvalidHeaderValue> for LeetCodeErr {
    fn from(value: InvalidHeaderValue) -> Self {
        Self::InvalidHeaders(value.to_string())
    }
}

impl From<reqwest::Error> for LeetCodeErr {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value.to_string())
    }
}
