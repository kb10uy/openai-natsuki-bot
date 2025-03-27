mod memory;
mod sqlite;

use self::{memory::MemoryConversationStorage, sqlite::SqliteConversationStorage};
use crate::{
    model::config::{AppConfigStorage, AppConfigStorageBackend},
    specs::storage::{ConversationStorage, Error},
};

use futures::TryFutureExt;

pub async fn create_storage(config: &AppConfigStorage) -> Result<Box<dyn ConversationStorage + 'static>, Error> {
    let boxed_storage: Box<dyn ConversationStorage> = match config.backend {
        AppConfigStorageBackend::Memory => Box::new(MemoryConversationStorage::new()),
        AppConfigStorageBackend::Sqlite => Box::new(
            SqliteConversationStorage::new(&config.sqlite)
                .map_err(|e| Error::Internal(e.into()))
                .await?,
        ),
    };
    Ok(boxed_storage)
}
