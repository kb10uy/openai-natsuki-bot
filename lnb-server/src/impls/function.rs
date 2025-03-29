mod get_illust_url;
mod image_generator;
mod local_info;
mod self_info;

pub use self::get_illust_url::GetIllustUrl;
pub use self::image_generator::ImageGenerator;
pub use self::local_info::LocalInfo;
pub use self::self_info::SelfInfo;

use crate::error::FunctionError;

use async_openai::error::OpenAIError;
use serde_json::Error as SerdeJsonError;
use sqlx::Error as SqlxError;
use url::ParseError as UrlParseError;

impl From<SerdeJsonError> for FunctionError {
    fn from(value: SerdeJsonError) -> Self {
        FunctionError::Serialization(value.into())
    }
}

impl From<OpenAIError> for FunctionError {
    fn from(value: OpenAIError) -> Self {
        FunctionError::External(value.into())
    }
}

impl From<UrlParseError> for FunctionError {
    fn from(value: UrlParseError) -> Self {
        FunctionError::Serialization(value.into())
    }
}

impl From<SqlxError> for FunctionError {
    fn from(value: SqlxError) -> Self {
        FunctionError::Serialization(value.into())
    }
}
