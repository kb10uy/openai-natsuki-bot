use crate::{
    USER_AGENT,
    assistant::Assistant,
    error::PlatformError,
    model::{
        config::AppConfigPlatformMastodon,
        conversation::ConversationAttachment,
        message::{UserMessage, UserMessageContent},
    },
    specs::platform::ConversationPlatform,
    text::markdown::sanitize_markdown_mastodon,
};

use std::sync::{Arc, LazyLock};

use futures::{future::BoxFuture, prelude::*};
use html2md::parse_html;
use mastodon_async::{
    Error as MastodonError, Mastodon, NewStatus, Visibility,
    entities::{account::Account, event::Event, notification::Type as NotificationType, status::Status},
    format_err,
    prelude::MediaType,
};
use mastodon_async_entities::AttachmentId;
use regex::Regex;
use reqwest::Client;
use tempfile::NamedTempFile;
use tokio::{fs::File, io::AsyncWriteExt, spawn};
use tracing::{debug, error, info};
use url::Url;

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
        let mastodon = Mastodon::new(http_client.clone(), mastodon_data);
        let self_account = mastodon.verify_credentials().await?;

        Ok(MastodonPlatform(Arc::new(MastodonPlatformInner {
            assistant,
            http_client,
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
    http_client: Client,
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
        let content_markdown = parse_html(&status.content);
        let stripped = RE_HEAD_MENTION.replace_all(&content_markdown, "");
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
            stripped,
            images.len()
        );

        let mut contents = vec![UserMessageContent::Text(stripped.to_string())];
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
            media_ids: Some(attachment_ids),
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

    async fn upload_image(&self, url: &Url, description: Option<&str>) -> Result<AttachmentId, PlatformError> {
        // ダウンロード
        let response = self.http_client.get(url.to_string()).send().await?;
        let image_data = response.bytes().await?;
        let mime_type = infer::get(&image_data).map(|ft| ft.mime_type());

        // tempfile に書き出し
        let tempfile = match mime_type {
            Some("image/jpeg") => NamedTempFile::with_suffix(".jpg")?,
            Some("image/png") => NamedTempFile::with_suffix(".png")?,
            Some("image/gif") => NamedTempFile::with_suffix(".gif")?,
            _ => {
                return Err(PlatformError::Communication(
                    format_err!("unsupported image type: {mime_type:?}").into(),
                ));
            }
        };
        debug!("writing temporary image at {:?}", tempfile.path());
        // tokio File にするので分解する
        let restored_tempfile = {
            let (temp_file, temp_path) = tempfile.into_parts();
            let mut async_file = File::from_std(temp_file);
            async_file.write_all(&image_data).await?;
            let restored_file = async_file.into_std().await;
            NamedTempFile::from_parts(restored_file, temp_path)
        };
        // アップロード
        let uploaded_attachment = self
            .mastodon
            .media(restored_tempfile.path(), description.map(|d| d.to_string()))
            .await?;
        // ここまで生き残らせる
        drop(restored_tempfile);

        Ok(uploaded_attachment.id)
    }
}

impl From<MastodonError> for PlatformError {
    fn from(value: MastodonError) -> Self {
        PlatformError::External(value.into())
    }
}
