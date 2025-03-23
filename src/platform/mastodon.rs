use super::{ConversationPlatform, error::Error};
use crate::{
    application::{config::AppConfigPlatformMastodon, constants::USER_AGENT},
    assistant::Assistant,
    model::message::Message,
};

use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use futures::prelude::*;
use html2md::parse_html;
use mastodon_async::{
    Mastodon, NewStatus, Visibility,
    entities::{
        account::Account, event::Event, notification::Type as NotificationType, status::Status,
    },
};
use regex::Regex;
use tokio::spawn;
use tracing::info;

static RE_HEAD_MENTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*\[@.+?\]\(.+?\)\s*"#).expect("invalid regex"));

#[derive(Debug, Clone)]
pub struct MastodonPlatform {
    assistant: Arc<Assistant>,
    mastodon: Mastodon,
    self_account: Account,
    sensitive_spoiler: String,
}

#[async_trait]
impl ConversationPlatform for MastodonPlatform {
    async fn execute(self: Arc<MastodonPlatform>) -> Result<(), Error> {
        let user_stream = self.mastodon.stream_user().await?;
        user_stream
            .map_err(Error::Mastodon)
            .try_for_each(async |(e, _)| {
                let cloned_self = self.clone();
                spawn(cloned_self.process_event(e));
                Ok(())
            })
            .await?;
        Ok(())
    }
}

impl MastodonPlatform {
    pub async fn new(
        config_mastodon: &AppConfigPlatformMastodon,
        assistant: Arc<Assistant>,
    ) -> Result<Arc<MastodonPlatform>, Error> {
        // Mastodon クライアントと自己アカウント情報
        let http_client = reqwest::ClientBuilder::new()
            .user_agent(USER_AGENT)
            .build()?;
        let mastodon_data = mastodon_async::Data {
            base: config_mastodon.server_url.clone().into(),
            token: config_mastodon.token.clone().into(),
            ..Default::default()
        };
        let mastodon = Mastodon::new(http_client, mastodon_data);
        let self_account = mastodon.verify_credentials().await?;

        Ok(Arc::new(MastodonPlatform {
            assistant,
            mastodon,
            self_account,
            sensitive_spoiler: config_mastodon.sensitive_spoiler.clone(),
        }))
    }

    async fn process_event(self: Arc<Self>, event: Event) -> Result<(), Error> {
        match event {
            Event::Update(status) => self.process_status(status).await,
            Event::Notification(notification) => match notification.notification_type {
                NotificationType::Mention => {
                    let status = notification.status.ok_or_else(|| {
                        Error::ExpectationMismatch("mentioned status not found".into())
                    })?;
                    self.process_status(status).await
                }
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }

    async fn process_status(self: Arc<Self>, status: Status) -> Result<(), Error> {
        // フィルタリング(bot flag と自分には応答しない)
        if status.account.bot || status.account.id == self.self_account.id {
            return Ok(());
        }

        // パース
        let content_markdown = parse_html(&status.content);
        let stripped = RE_HEAD_MENTION.replace_all(&content_markdown, "");
        info!("[{}] {}: {:?}", status.id, status.account.acct, stripped);

        // 呼出
        let mut conversation = self.assistant.new_conversation();
        conversation.push_message(Message::new_user(stripped));

        let update = self.assistant.process_conversation(&conversation).await?;
        let assistant_response = update.assistant_response;
        info!(
            "夏稀[{}]: {:?}",
            assistant_response.is_sensitive, assistant_response.text
        );

        // リプライ構築
        // 公開範囲は最大 unlisted でリプライ元に合わせる
        // CW はリプライ元があったらそのまま、ないときは要そぎぎなら付与
        let reply_text = format!("@{} {}", status.account.acct, assistant_response.text);
        let reply_visibility = match status.visibility {
            Visibility::Public => Visibility::Unlisted,
            otherwise => otherwise,
        };
        let reply_spoiler = match &status.spoiler_text[..] {
            "" => assistant_response
                .is_sensitive
                .then(|| self.sensitive_spoiler.clone()),
            _ => Some(status.spoiler_text),
        };
        let reply_status = NewStatus {
            status: Some(reply_text),
            visibility: Some(reply_visibility),
            in_reply_to_id: Some(status.id.to_string()),
            spoiler_text: reply_spoiler,
            ..Default::default()
        };
        self.mastodon.new_status(reply_status).await?;

        Ok(())
    }
}
