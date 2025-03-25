pub mod error;
mod memory;

pub use memory::MemoryConversationStorage;

use crate::{model::conversation::Conversation, persistence::error::Error};

use std::fmt::Debug;

use futures::future::BoxFuture;
use uuid::Uuid;

/// `Conversation` の永続化層の抽象化。
/// 本当は Repository と Service に分けたりした方がいいんだろうけど、面倒なのでこれで……。
#[allow(dead_code)]
pub trait ConversationStorage: Send + Sync + Debug {
    /// `Conversation` の ID で検索する。
    fn find_by_id<'a>(&'a self, id: &'a Uuid) -> BoxFuture<'a, Result<Option<Conversation>, Error>>;

    /// `Conversation` を platform-context から検索する。
    fn find_by_platform_context<'a>(
        &'a self,
        platform: &'a str,
        context: &'a str,
    ) -> BoxFuture<'a, Result<Option<Conversation>, Error>>;

    /// `Conversation` を登録・更新する。
    fn upsert<'a>(
        &'a self,
        conversation: &'a Conversation,
        platform: &'a str,
        context: &'a str,
    ) -> BoxFuture<'a, Result<(), Error>>;
}
