use crate::{
    application::{config::AppConfigOpenai, constants::USER_AGENT},
    llm_chat::error::Error,
};

use async_openai::{Client, config::OpenAIConfig};

pub async fn create_openai_client(openai_config: &AppConfigOpenai) -> Result<Client<OpenAIConfig>, Error> {
    let config = OpenAIConfig::new()
        .with_api_key(&openai_config.token)
        .with_api_base(&openai_config.endpoint);
    let http_client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).build()?;

    let client = Client::with_config(config).with_http_client(http_client);
    Ok(client)
}
