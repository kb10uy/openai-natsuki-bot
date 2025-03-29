mod error;
mod inner;
mod text;

use std::sync::Arc;

use futures::{future::BoxFuture, prelude::*};
use serenity::{Client as SerenityClient, all::GatewayIntents};
use tokio::sync::{Mutex, RwLock};

pub struct DiscordLnbClient(Arc<DiscordLnbClientInner>);

impl DiscordLnbClient {
    pub async fn new(
        config_discord: &AppConfigPlatformDiscord,
        assistant: Assistant,
    ) -> Result<DiscordLnbClient, PlatformError> {
        let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

        let handler = SerenityMessageHandler {
            bot_user: RwLock::new(None),
            max_length: config_discord.max_length,
            assistant,
        };

        // handler itself
        let outer_discord = Mutex::new(
            SerenityClient::builder(&config_discord.token, intents)
                .event_handler(handler)
                .await?,
        );
        Ok(DiscordLnbClient(Arc::new(DiscordLnbClientInner { outer_discord })))
    }
}

impl ConversationPlatform for DiscordLnbClient {
    fn execute(&self) -> BoxFuture<'static, Result<(), PlatformError>> {
        let cloned_inner = self.0.clone();
        cloned_inner.execute().boxed()
    }
}

struct DiscordLnbClientInner {
    outer_discord: Mutex<SerenityClient>,
}

impl DiscordLnbClientInner {
    async fn execute(self: Arc<Self>) -> Result<(), PlatformError> {
        let mut locked = self.outer_discord.lock().await;
        locked.start().await?;
        Ok(())
    }
}
