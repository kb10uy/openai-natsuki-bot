mod backend;
pub mod error;
pub mod openai;

use crate::{llm::backend::Backend, model::conversation::Conversation};

use std::fmt::Debug;

/// Conversation を送信した結果 OpenAI API によって生成された内容。
#[derive(Debug, Clone)]
pub struct LlmUpdate {
    pub text: Option<String>,
}

#[derive(Debug)]
pub struct LlmInterface {
    backend: Box<dyn Backend>,
}

impl LlmInterface {
    // `LlmInterface` を作成する。
    pub async fn new(backend: impl Backend + 'static) -> Result<LlmInterface, error::Error> {
        Ok(LlmInterface {
            backend: Box::new(backend),
        })
    }

    /// `Conversation` を送信する。
    pub async fn send(&self, conversation: &Conversation) -> Result<LlmUpdate, error::Error> {
        let update = self.backend.send_conversation(conversation).await?;
        Ok(update)
    }
}
