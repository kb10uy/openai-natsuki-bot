mod application;
mod chat;
mod model;
mod platform;

use crate::{
    application::{cli::Arguments, config::load_config},
    chat::{ChatInterface, chat_completion::ChatCompletionBackend},
    platform::{ConversationPlatform, cli::CliPlatform},
};

use anyhow::Result;
use clap::Parser;
use tokio::spawn;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Arguments::parse();
    let config = load_config(args.config).await?;
    let chat_interface = ChatInterface::<ChatCompletionBackend>::new(&config).await?;

    let cli_platform = CliPlatform::create(&chat_interface);

    // join 自体の成功と ConversationPlatform の成功が必要
    spawn(cli_platform.execute()).await??;
    Ok(())
}
