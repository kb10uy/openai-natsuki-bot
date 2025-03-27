mod chat_completion;
mod responses;

pub use chat_completion::ChatCompletionBackend;
pub use responses::ResponsesBackend;

use crate::{USER_AGENT, error::LlmError, model::config::AppConfigLlmOpenai};

use async_openai::{Client, config::OpenAIConfig};

async fn create_openai_client(openai_config: &AppConfigLlmOpenai) -> Result<Client<OpenAIConfig>, LlmError> {
    let config = OpenAIConfig::new()
        .with_api_key(&openai_config.token)
        .with_api_base(&openai_config.endpoint);
    let http_client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).build()?;

    let client = Client::with_config(config).with_http_client(http_client);
    Ok(client)
}
