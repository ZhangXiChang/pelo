use std::{error, fmt, io, num::ParseIntError};

use log::SetLoggerError;

#[derive(Debug)]
pub struct Error {
    message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl error::Error for Error {}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
impl From<SetLoggerError> for Error {
    fn from(value: SetLoggerError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
impl From<log4rs::config::runtime::ConfigErrors> for Error {
    fn from(value: log4rs::config::runtime::ConfigErrors) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
impl From<regex::Error> for Error {
    fn from(value: regex::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
