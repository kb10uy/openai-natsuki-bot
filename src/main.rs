mod application;
mod assistant;
mod llm;
mod model;
mod persistence;
mod platform;

use crate::{
    application::{cli::Arguments, config::load_config},
    assistant::Assistant,
    llm::openai::create_openai_llm,
    persistence::create_storage,
    platform::{ConversationPlatform, cli::CliPlatform, mastodon::MastodonPlatform},
};

use anyhow::{Result, bail};
use clap::Parser;
use futures::future::join_all;
use tokio::spawn;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Arguments::parse();
    let config = load_config(args.config).await?;

    let Some(assistant_identity) = config.assistant.identities.get(&config.assistant.identity) else {
        bail!("assistant identity {} not defined", config.assistant.identity);
    };

    let llm = create_openai_llm(&config.openai).await?;
    let storage = create_storage(&config.persistence).await?;
    let assistant = Assistant::new(assistant_identity, llm, storage);

    let mut platform_tasks = vec![];

    // CLI
    if config.platform.cli.enabled {
        info!("starting CLI platform");
        let cli_platform = CliPlatform::new(assistant.clone());
        let cli_task = spawn(cli_platform.execute());
        platform_tasks.push(Box::new(cli_task));
    }

    // Mastodon
    if config.platform.mastodon.enabled {
        info!("starting Mastodon platform");
        let mastodon_platform = MastodonPlatform::new(&config.platform.mastodon, assistant.clone()).await?;
        let mastodon_future = mastodon_platform.execute();
        let mastodon_task = spawn(mastodon_future);
        platform_tasks.push(Box::new(mastodon_task));
    }

    join_all(platform_tasks).await;
    Ok(())
}
