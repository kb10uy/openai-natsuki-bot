use crate::{
    error::AssistantError,
    model::{
        config::AppConfigAssistantIdentity,
        conversation::Conversation,
        message::{AssistantMessage, Message},
    },
    specs::{llm::Llm, storage::ConversationStorage},
};

use std::{fmt::Debug, sync::Arc};

/// 各種アシスタント動作の抽象化レイヤー。
#[derive(Debug, Clone)]
pub struct Assistant(Arc<AssistantInner>);

impl Assistant {
    pub fn new(
        assistant_identity: &AppConfigAssistantIdentity,
        llm: Box<dyn Llm + 'static>,
        storage: Box<dyn ConversationStorage + 'static>,
    ) -> Assistant {
        Assistant(Arc::new(AssistantInner {
            llm,
            storage,
            system_role: assistant_identity.system_role.clone(),
        }))
    }

    /// 指定された `Conversation` が「完了」するまで処理する。
    pub async fn process_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<ConversationUpdate, AssistantError> {
        let update = self.0.llm.send_conversation(conversation).await?;
        let Some(response) = update.response else {
            return Err(AssistantError::ChatResponseExpected);
        };

        let assistant_response = AssistantMessage {
            text: response.text,
            is_sensitive: response.sensitive,
        };

        Ok(ConversationUpdate { assistant_response })
    }

    /// 新しい `Conversation` を現在時刻の ID で初期化する。
    pub fn new_conversation(&self) -> Conversation {
        let system_message = Message::new_system(self.0.system_role.clone());
        Conversation::new_now(Some(system_message))
    }

    pub async fn restore_conversation(
        &self,
        platform: &str,
        context: &str,
    ) -> Result<Option<Conversation>, AssistantError> {
        let conversation = self.0.storage.find_by_platform_context(platform, context).await?;
        Ok(conversation)
    }

    pub async fn save_conversation(
        &self,
        conversation: &Conversation,
        platform: &str,
        context: &str,
    ) -> Result<(), AssistantError> {
        self.0.storage.upsert(conversation, platform, context).await?;
        Ok(())
    }
}

#[derive(Debug)]
struct AssistantInner {
    llm: Box<dyn Llm + 'static>,
    storage: Box<dyn ConversationStorage + 'static>,
    system_role: String,
}

#[derive(Debug, Clone)]
pub struct ConversationUpdate {
    pub assistant_response: AssistantMessage,
}
