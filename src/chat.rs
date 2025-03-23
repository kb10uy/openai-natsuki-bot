pub mod backend;
pub mod error;

use std::fmt::Debug;

use crate::{chat::backend::Backend, model::conversation::Conversation};

/// Conversation を送信した結果 OpenAI によって生成された内容。
#[derive(Debug, Clone)]
pub struct ChatUpdate {
    pub text: Option<String>,
}

#[derive(Debug)]
pub struct ChatInterface {
    backend: Box<dyn Backend>,
}

impl ChatInterface {
    // ChatInterface を作成する。
    pub async fn new<B: Backend>(backend: B) -> Result<ChatInterface, error::Error> {
        Ok(ChatInterface {
            backend: Box::new(backend),
        })
    }

    pub async fn send(&self, conversation: &Conversation) -> Result<ChatUpdate, error::Error> {
        let update = self.backend.send_conversation(conversation).await?;
        Ok(update)
    }
}
