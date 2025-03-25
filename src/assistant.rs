pub mod error;

use crate::{
    application::config::AppConfigAssistant,
    assistant::error::Error,
    llm_chat::LlmChatInterface,
    model::{
        conversation::Conversation,
        message::{AssistantMessage, Message},
    },
    persistence::ConversationStorage,
};

use std::{fmt::Debug, sync::Arc};

/// 各種アシスタント動作の抽象化レイヤー。
#[derive(Debug, Clone)]
pub struct Assistant(Arc<AssistantInner>);

impl Assistant {
    pub fn new(
        assistant_config: &AppConfigAssistant,
        chat_interface: LlmChatInterface,
        storage: impl ConversationStorage + 'static,
    ) -> Assistant {
        Assistant(Arc::new(AssistantInner {
            chat_interface,
            storage: Box::new(storage),
            system_role: assistant_config.system_role.clone(),
            sensitive_marker: assistant_config.sensitive_marker.clone(),
        }))
    }

    /// 指定された `Conversation` が「完了」するまで処理する。
    pub async fn process_conversation(&self, conversation: &Conversation) -> Result<ConversationUpdate, error::Error> {
        let chat_update = self.0.chat_interface.send(conversation).await?;
        let Some(response_text) = chat_update.text else {
            return Err(error::Error::NoAssistantResponse);
        };

        let (text, is_sensitive) = match response_text.strip_prefix(&self.0.sensitive_marker) {
            Some(stripped) => (stripped.to_string(), true),
            None => (response_text, false),
        };
        let assistant_response = AssistantMessage { text, is_sensitive };

        Ok(ConversationUpdate { assistant_response })
    }

    /// 新しい `Conversation` を現在時刻の ID で初期化する。
    pub fn new_conversation(&self) -> Conversation {
        let system_message = Message::new_system(self.0.system_role.clone());
        Conversation::new_now(Some(system_message))
    }

    pub async fn restore_conversation(&self, platform: &str, context: &str) -> Result<Option<Conversation>, Error> {
        let conversation = self.0.storage.find_by_platform_context(platform, context).await?;
        Ok(conversation)
    }

    pub async fn save_conversation(
        &self,
        conversation: &Conversation,
        platform: &str,
        context: &str,
    ) -> Result<(), Error> {
        self.0.storage.upsert(conversation, platform, context).await?;
        Ok(())
    }
}

#[derive(Debug)]
struct AssistantInner {
    chat_interface: LlmChatInterface,
    storage: Box<dyn ConversationStorage>,
    system_role: String,
    sensitive_marker: String,
}

#[derive(Debug, Clone)]
pub struct ConversationUpdate {
    pub assistant_response: AssistantMessage,
}
