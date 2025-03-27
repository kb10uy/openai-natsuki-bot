use crate::{
    error::AssistantError,
    model::{
        config::AppConfigAssistantIdentity,
        conversation::Conversation,
        message::{AssistantMessage, Message},
    },
    specs::{function::simple::SimpleFunction, llm::Llm, storage::ConversationStorage},
};

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use tokio::sync::Mutex;

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
            simple_functions: Mutex::new(HashMap::new()),
            system_role: assistant_identity.system_role.clone(),
            sensitive_marker: assistant_identity.sensitive_marker.clone(),
        }))
    }

    /// `SimpleFunction` を登録する。
    pub async fn add_simple_function(&self, simple_function: impl SimpleFunction + 'static) {
        let descriptor = simple_function.get_descriptor();

        let mut locked = self.0.simple_functions.lock().await;
        locked.insert(descriptor.name.clone(), Box::new(simple_function));
        self.0.llm.add_simple_function(descriptor).await;
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

        let (text, is_sensitive) = match response.sensitive {
            Some(v) => (response.text, v),
            None if self.0.sensitive_marker.is_empty() => (response.text, false),
            _ => match response.text.strip_prefix(&self.0.sensitive_marker) {
                Some(stripped) => (stripped.to_string(), true),
                None => (response.text, false),
            },
        };

        let assistant_response = AssistantMessage {
            text,
            is_sensitive,
            language: response.language,
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
    simple_functions: Mutex<HashMap<String, Box<dyn SimpleFunction + 'static>>>,
    system_role: String,
    sensitive_marker: String,
}

#[derive(Debug, Clone)]
pub struct ConversationUpdate {
    pub assistant_response: AssistantMessage,
}
