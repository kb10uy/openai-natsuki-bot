use crate::{
    assistant::Assistant,
    error::PlatformError,
    model::{
        config::AppConfigPlatformDiscord,
        message::{UserMessage, UserMessageContent},
    },
    specs::platform::ConversationPlatform,
    text::markdown::sanitize_markdown_mastodon,
};

use std::sync::{Arc, LazyLock};

use futures::{future::BoxFuture, prelude::*};
use regex::Regex;
use serenity::{
    Client as SerenityClient, Error as SerenityError,
    all::{Context, CreateMessage, EventHandler, GatewayIntents, Message as SerenityMessage, Ready, User},
};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info};

const PLATFORM_KEY: &str = "discord";

static RE_HEAD_MENTION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^\s*<@\d+?>\s*"#).expect("invalid regex"));

pub struct DiscordPlatform(Arc<DiscordPlatformInner>);

impl DiscordPlatform {
    pub async fn new(
        config_discord: &AppConfigPlatformDiscord,
        assistant: Assistant,
    ) -> Result<DiscordPlatform, PlatformError> {
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
        Ok(DiscordPlatform(Arc::new(DiscordPlatformInner { outer_discord })))
    }
}

impl ConversationPlatform for DiscordPlatform {
    fn execute(&self) -> BoxFuture<'static, Result<(), PlatformError>> {
        let cloned_inner = self.0.clone();
        cloned_inner.execute().boxed()
    }
}

struct DiscordPlatformInner {
    outer_discord: Mutex<SerenityClient>,
}

impl DiscordPlatformInner {
    async fn execute(self: Arc<Self>) -> Result<(), PlatformError> {
        let mut locked = self.outer_discord.lock().await;
        locked.start().await?;
        Ok(())
    }
}

struct SerenityMessageHandler {
    bot_user: RwLock<Option<User>>,
    max_length: usize,
    assistant: Assistant,
}

impl EventHandler for SerenityMessageHandler {
    fn ready<'a, 't>(&'a self, ctx: Context, ready: Ready) -> BoxFuture<'t, ()>
    where
        'a: 't,
        Self: 't,
    {
        do_event(self.on_ready(ctx, ready))
    }

    fn message<'a, 't>(&'a self, ctx: Context, new_message: SerenityMessage) -> BoxFuture<'t, ()>
    where
        'a: 't,
        Self: 't,
    {
        do_event(self.on_message(ctx, new_message))
    }
}

impl SerenityMessageHandler {
    async fn on_ready(&self, _ctx: Context, ready: Ready) -> Result<(), PlatformError> {
        info!("Discord platform got ready: [{}] {}", ready.user.id, ready.user.name);

        let mut bot_user = self.bot_user.write().await;
        *bot_user = Some(ready.user.into());
        Ok(())
    }

    async fn on_message(&self, ctx: Context, message: SerenityMessage) -> Result<(), PlatformError> {
        let bot_user = self.bot_user.read().await;
        let Some(bot_user) = bot_user.as_ref() else {
            return Ok(());
        };

        // (自分含む) bot のメッセージと非メンションを除外
        if message.author.bot || !message.mentions_user(bot_user) {
            return Ok(());
        }

        self.on_mentioned_message(ctx, message).await?;
        Ok(())
    }

    async fn on_mentioned_message(&self, ctx: Context, message: SerenityMessage) -> Result<(), PlatformError> {
        // Conversation の検索
        let context_key = message.referenced_message.as_ref().map(|rm| rm.id.to_string());
        let conversation = match context_key {
            None => {
                info!("creating new conversation");
                self.assistant.new_conversation()
            }
            Some(context) => {
                info!("restoring conversation with last referenced message ID {context}");
                match self.assistant.restore_conversation(PLATFORM_KEY, &context).await? {
                    Some(c) => c,
                    None => {
                        info!("conversation has been lost, creating new one");
                        self.assistant.new_conversation()
                    }
                }
            }
        };

        // TODO: attachments
        let stripped_content = RE_HEAD_MENTION.replace(&message.content, "");
        info!("[{}] {}: {}", message.id, message.author.id, stripped_content);

        let contents = vec![UserMessageContent::Text(stripped_content.to_string())];
        // contents.extend(images);

        // Conversation の更新・呼出し
        let user_message = UserMessage {
            contents,
            language: message.author.locale.clone(),
            ..Default::default()
        };
        let conversation_update = self.assistant.process_conversation(conversation, user_message).await?;
        let assistant_message = conversation_update.assistant_message();
        let attachments = conversation_update.attachments();
        info!(
            "夏稀[{}]: {:?} ({} attachment(s))",
            assistant_message.is_sensitive,
            assistant_message.text,
            attachments.len()
        );
        // TODO: attachments

        // リプライ
        // TODO: sanitize_markdown_discord
        let mut sanitized_text = sanitize_markdown_mastodon(&assistant_message.text);
        if sanitized_text.chars().count() > self.max_length {
            sanitized_text = sanitized_text.chars().take(self.max_length).collect();
            sanitized_text.push_str("...(omitted)");
        }

        let replied_message = message
            .channel_id
            .send_message(
                &ctx.http,
                CreateMessage::new().reference_message(&message).content(sanitized_text),
            )
            .await?;

        // Conversation/history の更新
        let updated_conversation = conversation_update.finish();
        let new_history_id = replied_message.id.to_string();
        self.assistant
            .save_conversation(&updated_conversation, PLATFORM_KEY, &new_history_id)
            .await?;

        Ok(())
    }
}

fn do_event<'t>(event_future: impl Future<Output = Result<(), PlatformError>> + Send + 't) -> BoxFuture<'t, ()> {
    async {
        match event_future.await {
            Ok(()) => (),
            Err(err) => {
                error!("Discord event process reported error: {err}");
            }
        }
    }
    .boxed()
}

impl From<SerenityError> for PlatformError {
    fn from(value: SerenityError) -> Self {
        PlatformError::External(value.into())
    }
}
