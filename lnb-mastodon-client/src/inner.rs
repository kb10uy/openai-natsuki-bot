use crate::text::{sanitize_markdown_for_mastodon, sanitize_mention_html_from_mastodon};

use std::sync::Arc;

use futures::prelude::*;
use lnb_core::{
    APP_USER_AGENT,
    config::AppConfigPlatformMastodon,
    error::ClientError,
    interface::server::LnbServer,
    model::{
        conversation::ConversationAttachment,
        message::{UserMessage, UserMessageContent},
    },
};
use mastodon_async::{
    Mastodon, NewStatus, Visibility,
    entities::{AttachmentId, account::Account, event::Event, notification::Type as NotificationType, status::Status},
    prelude::MediaType,
};
use reqwest::Client;
use tempfile::NamedTempFile;
use thiserror::Error as ThisError;
use tokio::{fs::File, io::AsyncWriteExt, spawn};
use tracing::{debug, error, info};
use url::Url;

const PLATFORM_KEY: &str = "mastodon";

#[derive(Debug)]
pub struct MastodonLnbClientInner<S> {
    assistant: S,
    http_client: Client,
    mastodon: Mastodon,
    self_account: Account,
    sensitive_spoiler: String,
    max_length: usize,
}

impl<S: LnbServer> MastodonLnbClientInner<S> {
    pub async fn new(
        config_mastodon: &AppConfigPlatformMastodon,
        assistant: S,
    ) -> Result<MastodonLnbClientInner<S>, ClientError> {
        // Mastodon クライアントと自己アカウント情報
        let http_client = reqwest::ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .build()
            .map_err(ClientError::by_communication)?;
        let mastodon_data = mastodon_async::Data {
            base: config_mastodon.server_url.clone().into(),
            token: config_mastodon.token.clone().into(),
            ..Default::default()
        };
        let mastodon = Mastodon::new(http_client.clone(), mastodon_data);
        let self_account = mastodon.verify_credentials().map_err(ClientError::by_external).await?;

        Ok(MastodonLnbClientInner {
            assistant,
            http_client,
            mastodon,
            self_account,
            sensitive_spoiler: config_mastodon.sensitive_spoiler.clone(),
            max_length: config_mastodon.max_length,
        })
    }

    pub async fn execute(self: Arc<Self>) -> Result<(), ClientError> {
        let user_stream = self
            .mastodon
            .stream_user()
            .map_err(ClientError::by_communication)
            .await?;
        user_stream
            .try_for_each(async |(e, _)| {
                spawn(self.clone().process_event(e));
                Ok(())
            })
            .map_err(ClientError::by_communication)
            .await?;

        Ok(())
    }

    async fn process_event(self: Arc<Self>, event: Event) {
        let processed = match event {
            Event::Update(status) => self.process_status(status).await,
            Event::Notification(notification) => match notification.notification_type {
                NotificationType::Mention => match notification.status {
                    Some(status) => self.process_status(status).await,
                    None => Err(ClientError::External(MastodonClientError::InvalidMention.into())),
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

    async fn process_status(&self, status: Status) -> Result<(), ClientError> {
        // フィルタリング(bot flag と自分には応答しない)
        if status.account.bot || status.account.id == self.self_account.id {
            return Ok(());
        }

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

        // パース
        let sanitized_mention_text = sanitize_mention_html_from_mastodon(&status.content);
        let images: Vec<_> = status
            .media_attachments
            .into_iter()
            .filter(|a| matches!(a.media_type, MediaType::Image | MediaType::Gifv))
            .map(|atch| UserMessageContent::ImageUrl(atch.preview_url))
            .collect();
        info!(
            "[{}] {}: {:?} ({} image(s))",
            status.id,
            status.account.acct,
            sanitized_mention_text,
            images.len()
        );

        let mut contents = vec![UserMessageContent::Text(sanitized_mention_text)];
        contents.extend(images);

        // Conversation の更新・呼出し
        let user_message = UserMessage {
            contents,
            language: status.language.and_then(|l| l.to_639_1()).map(|l| l.to_string()),
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

        // 添付メディア
        let mut attachment_ids = vec![];
        for attachment in attachments {
            match attachment {
                ConversationAttachment::Image { url, description } => {
                    let image_id = self.upload_image(url, description.as_deref()).await?;
                    attachment_ids.push(image_id);
                }
            }
        }

        // リプライ構築
        // 公開範囲は最大 unlisted でリプライ元に合わせる
        // CW はリプライ元があったらそのまま、ないときは要そぎぎなら付与
        let mut sanitized_text = sanitize_markdown_for_mastodon(&assistant_message.text);
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
            media_ids: Some(attachment_ids),
            ..Default::default()
        };
        let replied_status = self
            .mastodon
            .new_status(reply_status)
            .map_err(ClientError::by_external)
            .await?;

        // Conversation/history の更新
        let updated_conversation = conversation_update.finish();
        let new_history_id = replied_status.id.as_ref();
        self.assistant
            .save_conversation(&updated_conversation, PLATFORM_KEY, new_history_id)
            .await?;

        Ok(())
    }

    async fn upload_image(&self, url: &Url, description: Option<&str>) -> Result<AttachmentId, ClientError> {
        // ダウンロード
        let response = self
            .http_client
            .get(url.to_string())
            .send()
            .map_err(ClientError::by_external)
            .await?;
        let image_data = response.bytes().map_err(ClientError::by_external).await?;
        let mime_type = infer::get(&image_data).map(|ft| ft.mime_type());

        // tempfile に書き出し
        let tempfile = match mime_type {
            Some("image/jpeg") => NamedTempFile::with_suffix(".jpg").map_err(ClientError::by_external)?,
            Some("image/png") => NamedTempFile::with_suffix(".png").map_err(ClientError::by_external)?,
            Some("image/gif") => NamedTempFile::with_suffix(".gif").map_err(ClientError::by_external)?,
            Some(otherwise) => {
                return Err(ClientError::External(
                    MastodonClientError::UnsupportedImageType(otherwise.to_string()).into(),
                ));
            }
            None => {
                return Err(ClientError::External(
                    MastodonClientError::UnsupportedImageType("(unknown)".into()).into(),
                ));
            }
        };
        debug!("writing temporary image at {:?}", tempfile.path());
        // tokio File にするので分解する
        let restored_tempfile = {
            let (temp_file, temp_path) = tempfile.into_parts();
            let mut async_file = File::from_std(temp_file);
            async_file
                .write_all(&image_data)
                .await
                .map_err(ClientError::by_external)?;
            let restored_file = async_file.into_std().await;
            NamedTempFile::from_parts(restored_file, temp_path)
        };
        // アップロード
        let uploaded_attachment = self
            .mastodon
            .media(restored_tempfile.path(), description.map(|d| d.to_string()))
            .map_err(ClientError::by_external)
            .await?;
        // ここまで生き残らせる
        drop(restored_tempfile);

        Ok(uploaded_attachment.id)
    }
}

#[derive(Debug, ThisError)]
pub enum MastodonClientError {
    #[error("invalid mention object")]
    InvalidMention,

    #[error("unsupported image type: {0}")]
    UnsupportedImageType(String),
}
