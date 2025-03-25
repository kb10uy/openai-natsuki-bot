mod application;
mod assistant;
mod llm_chat;
mod model;
mod persistence;
mod platform;

use crate::{
    application::{
        cli::Arguments,
        config::{AppConfig, AppConfigOpenaiBackend, AppConfigPersistenceEngine, load_config},
    },
    assistant::Assistant,
    llm_chat::{
        LlmChatInterface,
        backend::{ChatCompletionBackend, ResponsesBackend},
    },
    persistence::{MemoryConversationStorage, SqliteConversationStorage},
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

    let assistant = construct_assistant(&config).await?;
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

async fn construct_assistant(config: &AppConfig) -> Result<Assistant> {
    let Some(assistant_identity) = config.assistant.identities.get(&config.assistant.identity) else {
        bail!("assistant identity {} not defined", config.assistant.identity);
    };

    let llm_chat = construct_llm_chat(config).await?;
    match config.persistence.engine {
        AppConfigPersistenceEngine::Sqlite => {
            let storage = SqliteConversationStorage::new(&config.persistence).await?;
            Ok(Assistant::new(assistant_identity, llm_chat, storage))
        }
        AppConfigPersistenceEngine::Memory => {
            let storage = MemoryConversationStorage::new();
            Ok(Assistant::new(assistant_identity, llm_chat, storage))
        }
    }
}

async fn construct_llm_chat(config: &AppConfig) -> Result<LlmChatInterface> {
    match config.openai.backend {
        AppConfigOpenaiBackend::ChatCompletion => {
            let backend = ChatCompletionBackend::new(&config.openai).await?;
            let llm_chat = LlmChatInterface::new(backend).await?;
            Ok(llm_chat)
        }
        AppConfigOpenaiBackend::Resnposes => {
            let backend = ResponsesBackend::new(&config.openai).await?;
            let llm_chat = LlmChatInterface::new(backend).await?;
            Ok(llm_chat)
        }
    }
}
