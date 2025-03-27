use crate::{
    error::LlmError,
    impls::llm::openai::{RESPONSE_JSON_SCHEMA, create_openai_client},
    model::{
        config::AppConfigLlmOpenai,
        conversation::{Conversation, StructuredResponse},
        message::Message,
    },
    specs::llm::{Llm, LlmUpdate},
};

use std::sync::Arc;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestFunctionMessage, ChatCompletionRequestMessage, CreateChatCompletionRequest, ResponseFormat,
    },
};
use futures::{FutureExt, future::BoxFuture};

/// OpenAI Chat Completion API を利用したバックエンド。
#[derive(Debug, Clone)]
pub struct ChatCompletionBackend(Arc<ChatCompletionBackendInner>);

impl ChatCompletionBackend {
    pub async fn new(config: &AppConfigLlmOpenai) -> Result<ChatCompletionBackend, LlmError> {
        let client = create_openai_client(config).await?;
        let model = config.model.clone();

        Ok(ChatCompletionBackend(Arc::new(ChatCompletionBackendInner {
            client,
            model,
            max_token: config.max_token,
        })))
    }
}

impl Llm for ChatCompletionBackend {
    fn send_conversation<'a>(&'a self, conversation: &'a Conversation) -> BoxFuture<'a, Result<LlmUpdate, LlmError>> {
        let cloned = self.0.clone();
        async move { cloned.send_conversation(conversation).await }.boxed()
    }
}

#[derive(Debug)]
struct ChatCompletionBackendInner {
    client: Client<OpenAIConfig>,
    model: String,
    max_token: usize,
}

impl ChatCompletionBackendInner {
    async fn send_conversation(&self, conversation: &Conversation) -> Result<LlmUpdate, LlmError> {
        let messages = conversation.messages().iter().map(transform_message).collect();
        let request = CreateChatCompletionRequest {
            messages,
            model: self.model.clone(),
            response_format: Some(ResponseFormat::JsonSchema {
                json_schema: RESPONSE_JSON_SCHEMA.clone(),
            }),
            max_completion_tokens: Some(self.max_token as u32),
            ..Default::default()
        };

        let openai_response = self.client.chat().create(request).await?;
        let Some(first_choice) = openai_response.choices.into_iter().next() else {
            return Err(LlmError::NoChoice);
        };

        /*
        let response = first_choice
            .message
            .content
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| LlmError::Backend(e.into()))?;
        */
        let update = LlmUpdate {
            // TODO: de しろ
            // response,
            response: Some(StructuredResponse {
                text: first_choice.message.content.unwrap_or_default(),
                language: "ja".into(),
                sensitive: false,
            }),
        };
        Ok(update)
    }
}

fn transform_message(message: &Message) -> ChatCompletionRequestMessage {
    match message {
        Message::System(system_message) => ChatCompletionRequestMessage::System(system_message.0.clone().into()),
        Message::User(user_message) => ChatCompletionRequestMessage::User(user_message.0.clone().into()),
        Message::Function(function_message) => {
            ChatCompletionRequestMessage::Function(ChatCompletionRequestFunctionMessage {
                name: function_message.0.clone(),
                content: Some(function_message.1.clone()),
            })
        }
        Message::Assistant(assistant_message) => {
            ChatCompletionRequestMessage::Assistant(assistant_message.text.clone().into())
        }
    }
}
