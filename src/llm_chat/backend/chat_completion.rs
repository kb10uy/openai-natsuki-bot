use crate::{
    application::{config::AppConfigOpenai, constants::USER_AGENT},
    llm_chat::{LlmChatUpdate, backend::Backend, error::Error},
    model::{conversation::Conversation, message::Message},
};

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestFunctionMessage, ChatCompletionRequestMessage, CreateChatCompletionRequest},
};
use async_trait::async_trait;

/// OpenAI の Chat Completion API を利用したバックエンド。
#[derive(Debug)]
pub struct ChatCompletionBackend {
    client: Client<OpenAIConfig>,
    model: String,
}

impl ChatCompletionBackend {
    pub async fn new(openai_config: &AppConfigOpenai) -> Result<ChatCompletionBackend, Error> {
        let client = create_openai_client(openai_config).await?;
        let model = openai_config.model.clone();

        Ok(ChatCompletionBackend { client, model })
    }
}

#[async_trait]
impl Backend for ChatCompletionBackend {
    async fn send_conversation(&self, conversation: &Conversation) -> Result<LlmChatUpdate, Error> {
        let messages = conversation.messages().iter().map(transform_message).collect();
        let request = CreateChatCompletionRequest {
            messages,
            model: self.model.clone(),
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

async fn create_openai_client(openai_config: &AppConfigOpenai) -> Result<Client<OpenAIConfig>, Error> {
    let config = OpenAIConfig::new()
        .with_api_key(&openai_config.token)
        .with_api_base(&openai_config.endpoint);
    let http_client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).build()?;

    let client = Client::with_config(config).with_http_client(http_client);
    Ok(client)
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
