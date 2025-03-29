use std::io::Error as IoError;

use lnb_core::error::{ClientError, ServerError};
use mastodon_async::Error as MastodonError;
use reqwest::Error as ReqwestError;
use thiserror::Error as ThisError;

pub struct WrappedClientError(ClientError);

impl From<WrappedClientError> for ClientError {
    fn from(value: WrappedClientError) -> Self {
        value.0
    }
}

impl From<ClientError> for WrappedClientError {
    fn from(value: ClientError) -> Self {
        Self(value)
    }
}

impl From<ServerError> for WrappedClientError {
    fn from(value: ServerError) -> Self {
        Self(ClientError::Server(value))
    }
}

impl From<ReqwestError> for WrappedClientError {
    fn from(value: ReqwestError) -> Self {
        Self(ClientError::Communication(value.into()))
    }
}

impl From<MastodonError> for WrappedClientError {
    fn from(value: MastodonError) -> Self {
        Self(ClientError::External(value.into()))
    }
}

impl From<IoError> for WrappedClientError {
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

impl From<MastodonClientError> for WrappedClientError {
    fn from(value: MastodonClientError) -> Self {
        Self(ClientError::External(value.into()))
    }
}
