mod application;
mod chat_interface;
mod model;

use crate::{
    application::{cli::Arguments, config::load_config},
    chat_interface::ChatInterface,
};

use anyhow::{Context, Result};
use async_openai::types::{
    ChatCompletionRequestMessage as Ccrm, ChatCompletionRequestSystemMessage as CcrmSystem,
    ChatCompletionRequestUserMessage as CcrmUser, CreateChatCompletionRequest,
};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Arguments::parse();
    let config = load_config(args.config).await?;
    let chat_interface = ChatInterface::new(&config).await?;

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
