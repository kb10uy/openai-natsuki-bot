mod chat_completion;
mod responses;

pub use chat_completion::ChatCompletionBackend;

use crate::{
    chat::{ChatUpdate, error::Error},
    model::conversation::Conversation,
};

use std::fmt::Debug;

use async_trait::async_trait;

#[async_trait]
#[allow(dead_code)]
pub trait Backend: 'static + Send + Sync + Debug {
    /// `Conversation` を送信する。
    async fn send_conversation(&self, conversation: &Conversation) -> Result<ChatUpdate, Error>;
}
