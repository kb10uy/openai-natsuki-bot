use crate::{
    llm::{LlmUpdate, error::Error},
    model::conversation::Conversation,
};

use std::fmt::Debug;

use futures::future::BoxFuture;

#[allow(dead_code)]
pub trait Backend: Send + Sync + Debug {
    /// `Conversation` を送信する。
    fn send_conversation<'a>(&'a self, conversation: &'a Conversation) -> BoxFuture<'a, Result<LlmUpdate, Error>>;
}
