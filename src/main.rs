mod cli;
mod config;

use crate::{
    cli::Arguments,
    config::{AppConfigOpenai, load_config},
};

use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage as Ccrm, ChatCompletionRequestSystemMessage as CcrmSystem,
        ChatCompletionRequestUserMessage as CcrmUser, CreateChatCompletionRequest,
    },
};
use clap::Parser;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Arguments::parse();
    let config = load_config(args.config).await?;
    let openai_client = create_openai_client(&config.openai).await?;

    let messages = vec![
        Ccrm::System(CcrmSystem {
            name: None,
            content: config.assistant.system_role.clone().into(),
        }),
        Ccrm::User(CcrmUser {
            name: None,
            content: args.prompt.into(),
        }),
    ];
    let request = CreateChatCompletionRequest {
        messages,
        model: config.openai.model.clone(),
        ..Default::default()
    };

    let response = openai_client
        .chat()
        .create(request)
        .await
        .context("cannot call API")?;
    let first_choice = response.choices.first().context("cannot fetch response")?;
    let message_text = first_choice
        .message
        .content
        .as_deref()
        .expect("no message found");
    println!("{message_text}");

    Ok(())
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
