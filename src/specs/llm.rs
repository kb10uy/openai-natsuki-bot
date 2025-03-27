use crate::{error::LlmError, model::conversation::Conversation};

use std::fmt::Debug;

use futures::future::BoxFuture;

#[allow(dead_code)]
pub trait Llm: Send + Sync + Debug {
    /// `Conversation` を送信する。
    fn send_conversation<'a>(&'a self, conversation: &'a Conversation) -> BoxFuture<'a, Result<LlmUpdate, LlmError>>;
}

/// Conversation を送信した結果生成された内容。
#[derive(Debug, Clone)]
pub struct LlmUpdate {
    pub text: Option<String>,
}
