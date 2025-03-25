use crate::{
    application::config::AppConfigOpenai,
    llm::{LlmChatUpdate, backend::Backend, error::Error, openai::create_openai_client},
    model::{conversation::Conversation, message::Message},
};

use std::sync::Arc;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestFunctionMessage, ChatCompletionRequestMessage, CreateChatCompletionRequest},
};
use futures::{FutureExt, future::BoxFuture};

/// OpenAI Chat Completion API を利用したバックエンド。
#[derive(Debug, Clone)]
pub struct ChatCompletionBackend(Arc<ChatCompletionBackendInner>);

impl ChatCompletionBackend {
    pub async fn new(openai_config: &AppConfigOpenai) -> Result<ChatCompletionBackend, Error> {
        let client = create_openai_client(openai_config).await?;
        let model = openai_config.model.clone();

        Ok(ChatCompletionBackend(Arc::new(ChatCompletionBackendInner {
            client,
            model,
            max_token: openai_config.max_token,
        })))
    }
}

impl Backend for ChatCompletionBackend {
    fn send_conversation<'a>(&'a self, conversation: &'a Conversation) -> BoxFuture<'a, Result<LlmChatUpdate, Error>> {
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
    async fn send_conversation(&self, conversation: &Conversation) -> Result<LlmChatUpdate, Error> {
        let messages = conversation.messages().iter().map(transform_message).collect();
        let request = CreateChatCompletionRequest {
            messages,
            model: self.model.clone(),
            max_completion_tokens: Some(self.max_token as u32),
            ..Default::default()
        };

        let response = self.client.chat().create(request).await?;
        let Some(first_choice) = response.choices.first() else {
            return Err(Error::NoChoice);
        };

        let update = LlmChatUpdate {
            text: first_choice.message.content.clone(),
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
