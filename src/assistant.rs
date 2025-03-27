use crate::{
    error::AssistantError,
    model::{
        config::AppConfigAssistantIdentity,
        conversation::{Conversation, ConversationUpdate, IncompleteConversation},
        message::{AssistantMessage, FunctionResponseMessage, Message, MessageFunctionCall, UserMessage},
    },
    specs::{function::simple::SimpleFunction, llm::Llm, storage::ConversationStorage},
};

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use tokio::sync::Mutex;
use tracing::{info, warn};

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
        conversation: Conversation,
        user_message: UserMessage,
    ) -> Result<ConversationUpdate, AssistantError> {
        let mut incomplete_conversation = IncompleteConversation::start(conversation, user_message);

        // 1 回目の呼出しで tool calling を判定する
        let first_update = self.0.llm.send_conversation(&incomplete_conversation).await?;
        let assistant_update = if let Some(tool_callings) = first_update.tool_callings {
            let call_message = Message::new_function_calls(tool_callings.clone());
            let response_messages = self.process_tool_callings(tool_callings).await?;

            incomplete_conversation.latest_messages.push(call_message);
            incomplete_conversation
                .latest_messages
                .extend(response_messages.into_iter().map(|m| m.into()));

            self.0.llm.send_conversation(&incomplete_conversation).await?
        } else {
            first_update
        };

        let Some(response) = assistant_update.response else {
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

        Ok(incomplete_conversation.finish(AssistantMessage {
            text,
            is_sensitive,
            language: response.language,
        }))
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

    async fn process_tool_callings(
        &self,
        tool_callings: Vec<MessageFunctionCall>,
    ) -> Result<Vec<FunctionResponseMessage>, AssistantError> {
        let locked = self.0.simple_functions.lock().await;

        let mut responses = vec![];
        for tool_calling in tool_callings {
            info!("calling tool {} (id: {})", tool_calling.name, tool_calling.id);
            // MCP と複合するのをあとで考える
            let Some(simple_function) = locked.get(&tool_calling.name) else {
                warn!("tool {} not found, skipping", tool_calling.name);
                continue;
            };
            let result = simple_function.call(&tool_calling.id, tool_calling.arguments).await?;
            responses.push(FunctionResponseMessage {
                id: tool_calling.id,
                name: tool_calling.name,
                result,
            });
        }

        Ok(responses)
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
