mod memory;
mod sqlite;

use self::{memory::MemoryConversationStorage, sqlite::SqliteConversationStorage};
use crate::{
    error::StorageError,
    model::config::{AppConfigStorage, AppConfigStorageBackend},
    specs::storage::ConversationStorage,
};

use rmp_serde::{decode::Error as RmpDecodeError, encode::Error as RmpEncodeError};
use sqlx::Error as SqlxError;

pub async fn create_storage(config: &AppConfigStorage) -> Result<Box<dyn ConversationStorage + 'static>, StorageError> {
    let boxed_storage: Box<dyn ConversationStorage> = match config.backend {
        AppConfigStorageBackend::Memory => Box::new(MemoryConversationStorage::new()),
        AppConfigStorageBackend::Sqlite => Box::new(SqliteConversationStorage::new(&config.sqlite).await?),
    };
    Ok(boxed_storage)
}

impl From<SqlxError> for StorageError {
    fn from(value: SqlxError) -> Self {
        StorageError::Backend(value.into())
    }
}

impl From<RmpDecodeError> for StorageError {
    fn from(value: RmpDecodeError) -> Self {
        StorageError::Serialization(value.into())
    }
}

impl From<RmpEncodeError> for StorageError {
    fn from(value: RmpEncodeError) -> Self {
        StorageError::Serialization(value.into())
    }
}
