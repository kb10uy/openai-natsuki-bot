use crate::application::{
    config::{AppConfig, AppConfigOpenai},
    constants::USER_AGENT,
};

use anyhow::Result;
use async_openai::{Client, config::OpenAIConfig};

#[derive(Debug, Clone)]
pub struct ChatInterface {
    client: Client<OpenAIConfig>,
    model: String,
    system_role: String,
}

impl ChatInterface {
    pub async fn new(config: &AppConfig) -> Result<ChatInterface> {
        let client = ChatInterface::create_openai_client(&config.openai).await?;

        Ok(ChatInterface {
            client,
            model: config.openai.model.clone(),
            system_role: config.assistant.system_role.clone(),
        })
    }

    async fn create_openai_client(openai_config: &AppConfigOpenai) -> Result<Client<OpenAIConfig>> {
        let config = OpenAIConfig::new()
            .with_api_key(&openai_config.token)
            .with_api_base(&openai_config.endpoint);
        let http_client = reqwest::ClientBuilder::new()
            .user_agent(USER_AGENT)
            .build()?;

        let client = Client::with_config(config).with_http_client(http_client);
        Ok(client)
    }
}
