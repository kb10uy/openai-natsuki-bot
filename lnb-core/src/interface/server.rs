use futures::future::BoxFuture;

use crate::{
    error::ServerError,
    model::{
        conversation::{Conversation, ConversationUpdate},
        message::UserMessage,
    },
};

/// 旧 Assistant
pub trait LnbServer: Send + Sync + 'static {
    /// 新しい会話ツリーを開始する。
    fn new_conversation(&self) -> Conversation;

    /// 会話ツリーを復元する。
    fn restore_conversation<'a>(
        &'a self,
        platform: &'a str,
        context: &'a str,
    ) -> BoxFuture<'a, Result<Option<Conversation>, ServerError>>;

    /// 会話ツリーを更新する。
    fn save_conversation<'a>(
        &'a self,
        conversation: &'a Conversation,
        platform: &'a str,
        context: &'a str,
    ) -> BoxFuture<'a, Result<(), ServerError>>;

    fn process_conversation(
        &self,
        conversation: Conversation,
        user_message: UserMessage,
    ) -> BoxFuture<'_, Result<ConversationUpdate, ServerError>>;
}
