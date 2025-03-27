mod cli;
mod mastodon;

pub use cli::CliPlatform;
pub use mastodon::MastodonPlatform;

use crate::error::PlatformError;

use reqwest::Error as ReqwestError;

impl From<ReqwestError> for PlatformError {
    fn from(value: ReqwestError) -> Self {
        PlatformError::Communication(value.into())
    }
}
