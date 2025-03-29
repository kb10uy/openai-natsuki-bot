use std::io::Error as IoError;

use lnb_core::error::{ClientError, ServerError};
use mastodon_async::Error as MastodonError;
use reqwest::Error as ReqwestError;
use thiserror::Error as ThisError;

pub struct WrappedPlatformError(ClientError);

impl From<WrappedPlatformError> for ClientError {
    fn from(value: WrappedPlatformError) -> Self {
        value.0
    }
}

impl From<ClientError> for WrappedPlatformError {
    fn from(value: ClientError) -> Self {
        Self(value)
    }
}

impl From<ServerError> for WrappedPlatformError {
    fn from(value: ServerError) -> Self {
        Self(ClientError::Server(value))
    }
}

impl From<ReqwestError> for WrappedPlatformError {
    fn from(value: ReqwestError) -> Self {
        Self(ClientError::Communication(value.into()))
    }
}

impl From<MastodonError> for WrappedPlatformError {
    fn from(value: MastodonError) -> Self {
        Self(ClientError::External(value.into()))
    }
}

impl From<IoError> for WrappedPlatformError {
    fn from(value: IoError) -> Self {
        Self(ClientError::External(value.into()))
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
        Self(ClientError::External(value.into()))
    }
}
