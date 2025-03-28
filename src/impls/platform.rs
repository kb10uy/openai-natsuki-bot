mod cli;
mod mastodon;

pub use cli::CliPlatform;
pub use mastodon::MastodonPlatform;

use crate::error::PlatformError;

use std::io::Error as IoError;

use reqwest::Error as ReqwestError;

impl From<ReqwestError> for PlatformError {
    fn from(value: ReqwestError) -> Self {
        PlatformError::Communication(value.into())
    }
}

impl From<IoError> for PlatformError {
    fn from(value: IoError) -> Self {
        PlatformError::External(value.into())
    }
}
