mod chat_completion;
mod responses;

use crate::{
    application::{
        config::{AppConfigOpenai, AppConfigOpenaiBackend},
        constants::USER_AGENT,
    },
    llm::{
        LlmInterface,
        error::Error,
        openai::{chat_completion::ChatCompletionBackend, responses::ResponsesBackend},
    },
};

use async_openai::{Client, config::OpenAIConfig};

/// `AppConfigOpenai` から OpenAI AI の `LlmInterface` を構築する。
pub async fn create_openai_llm(config: &AppConfigOpenai) -> Result<LlmInterface, Error> {
    match config.backend {
        AppConfigOpenaiBackend::ChatCompletion => {
            let backend = ChatCompletionBackend::new(config).await?;
            let llm = LlmInterface::new(backend).await?;
            Ok(llm)
        }
        AppConfigOpenaiBackend::Resnposes => {
            let backend = ResponsesBackend::new(config).await?;
            let llm = LlmInterface::new(backend).await?;
            Ok(llm)
        }
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
