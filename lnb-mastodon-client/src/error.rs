use std::io::Error as IoError;

use lnb_core::error::{PlatformError, ServerError};
use mastodon_async::Error as MastodonError;
use reqwest::Error as ReqwestError;
use thiserror::Error as ThisError;

pub struct WrappedPlatformError(PlatformError);

impl From<WrappedPlatformError> for PlatformError {
    fn from(value: WrappedPlatformError) -> Self {
        value.0
    }
}

impl From<PlatformError> for WrappedPlatformError {
    fn from(value: PlatformError) -> Self {
        Self(value)
    }
}

impl From<ServerError> for WrappedPlatformError {
    fn from(value: ServerError) -> Self {
        Self(PlatformError::Assistant(value))
    }
}

impl From<ReqwestError> for WrappedPlatformError {
    fn from(value: ReqwestError) -> Self {
        Self(PlatformError::Communication(value.into()))
    }
}

impl From<MastodonError> for WrappedPlatformError {
    fn from(value: MastodonError) -> Self {
        Self(PlatformError::External(value.into()))
    }
}

impl From<IoError> for WrappedPlatformError {
    fn from(value: IoError) -> Self {
        Self(PlatformError::External(value.into()))
    }
}

#[derive(Debug, ThisError)]
pub enum MastodonClientError {
    #[error("invalid mention object")]
    InvalidMention,

    #[error("unsupported image type: {0}")]
    UnsupportedImageType(String),
}

impl From<MastodonClientError> for WrappedPlatformError {
    fn from(value: MastodonClientError) -> Self {
        Self(PlatformError::External(value.into()))
    }
}
