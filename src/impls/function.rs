mod self_info;

pub use self::self_info::SelfInfo;

use crate::error::FunctionError;

use serde_json::Error as SerdeJsonError;

impl From<SerdeJsonError> for FunctionError {
    fn from(value: SerdeJsonError) -> Self {
        FunctionError::Serialization(value.into())
    }
}
