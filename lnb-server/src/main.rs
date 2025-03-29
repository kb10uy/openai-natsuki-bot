mod cli;
mod natsuki;

use crate::natsuki::{
    Natsuki,
    function::{GetIllustUrl, ImageGenerator, LocalInfo, SelfInfo},
    llm::create_llm,
    platform::{CliPlatform, DiscordPlatform, MastodonPlatform},
    storage::create_storage,
};

use std::path::Path;

use anyhow::{Context as _, Result, bail};
use clap::Parser;
use futures::future::join_all;
use lnb_core::config::AppConfig;
use tokio::{fs::read_to_string, spawn};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = cli::Arguments::parse();
    let config = load_config(args.config).await?;

    let Some(assistant_identity) = config.assistant.identities.get(&config.assistant.identity) else {
        bail!("assistant identity {} not defined", config.assistant.identity);
    };

    let llm = create_llm(&config.llm).await?;
    let storage = create_storage(&config.storage).await?;
    let natsuki = Natsuki::new(assistant_identity, llm, storage).await?;

    natsuki.add_simple_function(SelfInfo::new()).await;
    natsuki.add_simple_function(LocalInfo::new()?).await;
    if config.tool.image_generator.enabled {
        natsuki
            .add_simple_function(ImageGenerator::new(&config.tool.image_generator)?)
            .await;
    }
    if config.tool.get_illust_url.enabled {
        natsuki
            .add_simple_function(GetIllustUrl::new(&config.tool.get_illust_url).await?)
            .await;
    }

    let mut platform_tasks = vec![];

    // CLI
    if config.platform.cli.enabled {
        info!("starting CLI platform");
        let cli_platform = CliPlatform::new(natsuki.clone());
        let cli_task = spawn(cli_platform.execute());
        platform_tasks.push(Box::new(cli_task));
    }

    // Mastodon
    if config.platform.mastodon.enabled {
        info!("starting Mastodon platform");
        let mastodon_platform = MastodonPlatform::new(&config.platform.mastodon, natsuki.clone()).await?;
        let mastodon_task = spawn(mastodon_platform.execute());
        platform_tasks.push(Box::new(mastodon_task));
    }

    // Discord
    if config.platform.discord.enabled {
        info!("starting Discord platform");
        let discord_platform = DiscordPlatform::new(&config.platform.discord, natsuki.clone()).await?;
        let discord_task = spawn(discord_platform.execute());
        platform_tasks.push(Box::new(discord_task));
    }

    join_all(platform_tasks).await;
    Ok(())
}

async fn load_config(path: impl AsRef<Path>) -> Result<AppConfig> {
    let config_str = read_to_string(path).await.context("failed to read config file")?;
    toml::from_str(&config_str).context("failed to parse config")
}
