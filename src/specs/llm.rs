use crate::model::conversation::Conversation;

use std::fmt::Debug;

use futures::future::BoxFuture;
use thiserror::Error as ThisError;

#[allow(dead_code)]
pub trait LlmBackend: Send + Sync + Debug {
    /// `Conversation` を送信する。
    fn send_conversation<'a>(&'a self, conversation: &'a Conversation) -> BoxFuture<'a, Result<LlmUpdate, Error>>;
}

/// Conversation を送信した結果 OpenAI API によって生成された内容。
#[derive(Debug, Clone)]
pub struct LlmUpdate {
    pub text: Option<String>,
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(
        #[source]
        #[from]
        reqwest::Error,
    ),

    #[error("OpenAI error: {0}")]
    OpenAI(
        #[source]
        #[from]
        async_openai::error::OpenAIError,
    ),

    /// LLM が有効なレスポンスを生成しなかった。
    #[error("LLM returns no choice to show")]
    NoChoice,
}
