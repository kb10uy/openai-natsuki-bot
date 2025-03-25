pub mod backend;
pub mod error;

use std::fmt::Debug;

use crate::{llm_chat::backend::Backend, model::conversation::Conversation};

/// Conversation を送信した結果 OpenAI API によって生成された内容。
#[derive(Debug, Clone)]
pub struct LlmChatUpdate {
    pub text: Option<String>,
}

#[derive(Debug)]
pub struct LlmChatInterface {
    backend: Box<dyn Backend>,
}

impl LlmChatInterface {
    // ChatInterface を作成する。
    pub async fn new<B: Backend>(backend: B) -> Result<LlmChatInterface, error::Error> {
        Ok(LlmChatInterface {
            backend: Box::new(backend),
        })
    }

    pub async fn send(&self, conversation: &Conversation) -> Result<LlmChatUpdate, error::Error> {
        let update = self.backend.send_conversation(conversation).await?;
        Ok(update)
    }
}
