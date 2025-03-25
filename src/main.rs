mod application;
mod assistant;
mod llm_chat;
mod model;
mod platform;

use crate::{
    application::{cli::Arguments, config::load_config},
    assistant::Assistant,
    llm_chat::{LlmChatInterface, backend::ChatCompletionBackend},
    platform::{ConversationPlatform, cli::CliPlatform, mastodon::MastodonPlatform},
};

use anyhow::Result;
use clap::Parser;
use futures::future::join_all;
use tokio::spawn;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Arguments::parse();
    let config = load_config(args.config).await?;

    let backend = ChatCompletionBackend::new(&config.openai).await?;
    let llm_chat = LlmChatInterface::new(backend).await?;
    let assistant = Assistant::new(&config.assistant, llm_chat);

    let mut platform_tasks = vec![];

    // CLI
    if config.platform.cli.enabled {
        info!("starting CLI platform");
        let cli_platform = CliPlatform::new(assistant.clone());
        let cli_future = cli_platform.execute();
        let cli_task = spawn(cli_future);
        platform_tasks.push(cli_task);
    }

    // Mastodon
    if config.platform.mastodon.enabled {
        info!("starting Mastodon platform");
        let mastodon_platform = MastodonPlatform::new(&config.platform.mastodon, assistant.clone()).await?;
        let mastodon_future = mastodon_platform.execute();
        let mastodon_task = spawn(mastodon_future);
        platform_tasks.push(mastodon_task);
    }

    join_all(platform_tasks).await;
    Ok(())
}
