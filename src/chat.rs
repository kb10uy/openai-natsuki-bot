pub mod chat_completion;
pub mod error;
// pub mod responses;

use std::fmt::Debug;

use crate::{
    application::config::AppConfig,
    model::{conversation::Conversation, message::Message},
};

#[derive(Debug, Clone)]
pub struct ChatInterface<B> {
    backend: B,
    system_role: String,
}

impl<B: ChatBackend> ChatInterface<B> {
    // ChatInterface を作成する。
    pub async fn new(config: &AppConfig) -> Result<ChatInterface<B>, error::Error> {
        let backend = B::create(config).await?;
        let system_role = config.assistant.system_role.clone();

        Ok(ChatInterface {
            backend,
            system_role,
        })
    }

    /// system role を設定して新しい `Conversation` を作成する。
    pub fn create_conversation(&self) -> Conversation {
        let system_message = Message::new_system(self.system_role.clone());
        Conversation::new_now(Some(system_message))
    }

    pub async fn send(
        &self,
        conversation: &Conversation,
    ) -> Result<ConversationUpdate, error::Error> {
        let update = self.backend.send_conversation(conversation).await?;
        Ok(update)
    }
}

pub trait ChatBackend
where
    Self: 'static + Debug + Clone + Sized,
{
    /// 初期化する。
    async fn create(config: &AppConfig) -> Result<Self, error::Error>;

    /// `Conversation` を送信する。
    async fn send_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<ConversationUpdate, error::Error>;
}

/// Conversation を送信した結果 OpenAI によって生成された内容。
#[derive(Debug, Clone)]
pub struct ConversationUpdate {
    pub text: Option<String>,
}
