use crate::{
    application::{config::AppConfigPlatformMastodon, constants::USER_AGENT},
    assistant::Assistant,
    model::{conversation::Conversation, message::Message},
    platform::{ConversationPlatform, error::Error},
};

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use futures::{future::BoxFuture, prelude::*};
use html2md::parse_html;
use mastodon_async::{
    Mastodon, NewStatus, Visibility,
    entities::{account::Account, event::Event, notification::Type as NotificationType, status::Status},
};
use regex::Regex;
use tokio::{spawn, sync::Mutex};
use tracing::info;

static RE_HEAD_MENTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*\[@.+?\]\(.+?\)\s*"#).expect("invalid regex"));

#[derive(Debug)]
pub struct MastodonPlatform(Arc<MastodonPlatformInner>);

impl MastodonPlatform {
    pub async fn new(
        config_mastodon: &AppConfigPlatformMastodon,
        assistant: Assistant,
    ) -> Result<MastodonPlatform, Error> {
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

            history: Mutex::new(HashMap::new()),
        })))
    }
}

impl ConversationPlatform for MastodonPlatform {
    fn execute(&self) -> BoxFuture<'static, Result<(), Error>> {
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

    // Arc<Mutex<Conversation>> そんなわけなくない？
    /// Mastodon 側のツリー最後の in_reply_to StatudId と Conversation の組
    history: Mutex<HashMap<String, Arc<Mutex<Conversation>>>>,
}

impl MastodonPlatformInner {
    async fn execute(self: Arc<Self>) -> Result<(), Error> {
        let user_stream = self.mastodon.stream_user().await?;
        user_stream
            .map_err(Error::Mastodon)
            .try_for_each(async |(e, _)| {
                spawn(self.clone().process_event(e));
                Ok(())
            })
            .await?;

        Ok(())
    }

    async fn process_event(self: Arc<Self>, event: Event) -> Result<(), Error> {
        match event {
            Event::Update(status) => self.process_status(status).await,
            Event::Notification(notification) => match notification.notification_type {
                NotificationType::Mention => {
                    let status = notification
                        .status
                        .ok_or_else(|| Error::ExpectationMismatch("mentioned status not found".into()))?;
                    self.process_status(status).await
                }
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }

    async fn process_status(&self, status: Status) -> Result<(), Error> {
        // フィルタリング(bot flag と自分には応答しない)
        if status.account.bot || status.account.id == self.self_account.id {
            return Ok(());
        }

        // パース
        let content_markdown = parse_html(&status.content);
        let stripped = RE_HEAD_MENTION.replace_all(&content_markdown, "");
        info!("[{}] {}: {:?}", status.id, status.account.acct, stripped);

        // Conversation の検索
        let mut locked_history = self.history.lock().await;
        let in_reply_to_id_str = status.in_reply_to_id.map(|si| si.to_string());
        let (old_history_id, conversation) = match in_reply_to_id_str {
            Some(id) if locked_history.contains_key(&id) => {
                info!("found conversation with last status ID {id}");
                let conversation = locked_history[&id].clone();
                (Some(id), conversation)
            }
            _ => {
                info!("creating new conversation");
                (None, Arc::new(Mutex::new(self.assistant.new_conversation())))
            }
        };

        // Conversation の更新・呼出し
        let mut locked_conversation = conversation.lock().await;
        locked_conversation.push_message(Message::new_user(stripped));

        let update = self.assistant.process_conversation(&locked_conversation).await?;
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
            "" => assistant_response.is_sensitive.then(|| self.sensitive_spoiler.clone()),
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
        locked_conversation.push_message(assistant_response.into());
        drop(locked_conversation);

        if let Some(id) = old_history_id {
            locked_history.remove(&id);
        }
        let new_history_id = replied_status.id.to_string();
        locked_history.insert(new_history_id, conversation);

        Ok(())
    }
}
