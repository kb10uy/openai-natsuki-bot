mod chat_completion;
mod responses;

pub use chat_completion::ChatCompletionBackend;
pub use responses::ResponsesBackend;

use futures::future::BoxFuture;

use crate::{
    llm_chat::{LlmChatUpdate, error::Error},
    model::conversation::Conversation,
};

use std::fmt::Debug;

#[allow(dead_code)]
pub trait Backend: Send + Sync + Debug {
    /// `Conversation` を送信する。
    fn send_conversation<'a>(&'a self, conversation: &'a Conversation) -> BoxFuture<'a, Result<LlmChatUpdate, Error>>;
}
