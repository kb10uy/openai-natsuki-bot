use crate::{
    USER_AGENT,
    assistant::Assistant,
    error::PlatformError,
    model::{config::AppConfigPlatformMastodon, message::UserMessage},
    specs::platform::ConversationPlatform,
    text::markdown::sanitize_markdown_mastodon,
};

use std::sync::{Arc, LazyLock};

use futures::{future::BoxFuture, prelude::*};
use html2md::parse_html;
use mastodon_async::{
    Error as MastodonError, Mastodon, NewStatus, Visibility,
    entities::{account::Account, event::Event, notification::Type as NotificationType, status::Status},
};
use regex::Regex;
use tokio::spawn;
use tracing::{error, info};

const PLATFORM_KEY: &str = "mastodon";

static RE_HEAD_MENTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*\[@.+?\]\(.+?\)\s*"#).expect("invalid regex"));

#[derive(Debug)]
pub struct MastodonPlatform(Arc<MastodonPlatformInner>);

impl MastodonPlatform {
    pub async fn new(
        config_mastodon: &AppConfigPlatformMastodon,
        assistant: Assistant,
    ) -> Result<MastodonPlatform, PlatformError> {
        // Mastodon クライアントと自己アカウント情報
        let http_client = reqwest::ClientBuilder::new().user_agent(USER_AGENT).build()?;
        let mastodon_data = mastodon_async::Data {
            base: config_mastodon.server_url.clone().into(),
            token: config_mastodon.token.clone().into(),
            ..Default::default()
        };
        let mastodon = Mastodon::new(http_client, mastodon_data);
        let self_account = mastodon.verify_credentials().await?;

        Ok(MastodonPlatform(Arc::new(MastodonPlatformInner {
            assistant,
            mastodon,
            self_account,
            sensitive_spoiler: config_mastodon.sensitive_spoiler.clone(),
            max_length: config_mastodon.max_length,
        })))
    }
}

impl ConversationPlatform for MastodonPlatform {
    fn execute(&self) -> BoxFuture<'static, Result<(), PlatformError>> {
        let cloned_inner = self.0.clone();
        cloned_inner.execute().boxed()
    }
}

#[derive(Debug)]
struct MastodonPlatformInner {
    assistant: Assistant,
    mastodon: Mastodon,
    self_account: Account,
    sensitive_spoiler: String,
    max_length: usize,
}

impl MastodonPlatformInner {
    async fn execute(self: Arc<Self>) -> Result<(), PlatformError> {
        let user_stream = self.mastodon.stream_user().await?;
        user_stream
            .try_for_each(async |(e, _)| {
                spawn(self.clone().process_event(e));
                Ok(())
            })
            .await?;

        Ok(())
    }

    async fn process_event(self: Arc<Self>, event: Event) {
        let processed = match event {
            Event::Update(status) => self.process_status(status).await,
            Event::Notification(notification) => match notification.notification_type {
                NotificationType::Mention => match notification.status {
                    Some(status) => self.process_status(status).await,
                    None => Err(PlatformError::ExpectationMismatch("mentioned status not found".into())),
                },
                _ => Ok(()),
            },
            _ => Ok(()),
        };

        let Err(err) = processed else {
            return;
        };
        error!("mastodon event process reported error: {err}");
    }

    async fn process_status(&self, status: Status) -> Result<(), PlatformError> {
        // フィルタリング(bot flag と自分には応答しない)
        if status.account.bot || status.account.id == self.self_account.id {
            return Ok(());
        }

        // パース
        let content_markdown = parse_html(&status.content);
        let stripped = RE_HEAD_MENTION.replace_all(&content_markdown, "");
        info!("[{}] {}: {:?}", status.id, status.account.acct, stripped);

        // Conversation の検索
        let context_key = status.in_reply_to_id.map(|si| si.to_string());
        let conversation = match context_key {
            None => {
                info!("creating new conversation");
                self.assistant.new_conversation()
            }
            Some(context) => {
                info!("restoring conversation with last status ID {context}");
                match self.assistant.restore_conversation(PLATFORM_KEY, &context).await? {
                    Some(c) => c,
                    None => {
                        info!("conversation has been lost, creating new one");
                        self.assistant.new_conversation()
                    }
                }
            }
        };

        // Conversation の更新・呼出し
        let user_message = UserMessage {
            message: stripped.to_string(),
            language: status.language.and_then(|l| l.to_639_1()).map(|l| l.to_string()),
            ..Default::default()
        };
        let conversation_update = self.assistant.process_conversation(conversation, user_message).await?;
        let assistant_message = conversation_update.assistant_message();
        info!("夏稀[{}]: {:?}", assistant_message.is_sensitive, assistant_message.text);

        // リプライ構築
        // 公開範囲は最大 unlisted でリプライ元に合わせる
        // CW はリプライ元があったらそのまま、ないときは要そぎぎなら付与
        let mut sanitized_text = sanitize_markdown_mastodon(&assistant_message.text);
        if sanitized_text.chars().count() > self.max_length {
            sanitized_text = sanitized_text.chars().take(self.max_length).collect();
            sanitized_text.push_str("...(omitted)");
        }
        let reply_text = format!("@{} {sanitized_text}", status.account.acct);
        let reply_visibility = match status.visibility {
            Visibility::Public => Visibility::Unlisted,
            otherwise => otherwise,
        };
        let reply_spoiler = match &status.spoiler_text[..] {
            "" => assistant_message.is_sensitive.then(|| self.sensitive_spoiler.clone()),
            _ => Some(status.spoiler_text),
        };
        let reply_status = NewStatus {
            status: Some(reply_text),
            visibility: Some(reply_visibility),
            in_reply_to_id: Some(status.id.to_string()),
            spoiler_text: reply_spoiler,
            ..Default::default()
        };
        let replied_status = self.mastodon.new_status(reply_status).await?;

        // Conversation/history の更新
        let updated_conversation = conversation_update.finish();
        let new_history_id = replied_status.id.as_ref();
        self.assistant
            .save_conversation(&updated_conversation, PLATFORM_KEY, new_history_id)
            .await?;

        Ok(())
    }
}

impl From<MastodonError> for PlatformError {
    fn from(value: MastodonError) -> Self {
        PlatformError::External(value.into())
    }
}
