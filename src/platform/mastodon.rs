use super::{ConversationPlatform, error::Error};
use crate::{
    application::{config::AppConfigPlatformMastodon, constants::USER_AGENT},
    chat::{ChatBackend, ChatInterface},
    model::message::Message,
};

use std::sync::Arc;

use futures::prelude::*;
use mastodon_async::{Mastodon, entities::event::Event};
use tokio::spawn;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct MastodonPlatform<B> {
    chat: ChatInterface<B>,
    mastodon: Mastodon,
}

impl<B: ChatBackend> ConversationPlatform<B> for MastodonPlatform<B> {
    async fn execute(self: Arc<Self>) -> Result<(), Error> {
        let user_stream = self.mastodon.stream_user().await?;

        Ok(())
    }
}

impl<B: ChatBackend> MastodonPlatform<B> {
    pub fn new(
        config_mastodon: &AppConfigPlatformMastodon,
        chat_interface: &ChatInterface<B>,
    ) -> Result<Arc<Self>, Error> {
        let http_client = reqwest::ClientBuilder::new()
            .user_agent(USER_AGENT)
            .build()?;
        let mastodon = Mastodon::new(
            http_client,
            mastodon_async::Data {
                base: config_mastodon.server_url.clone().into(),
                token: config_mastodon.token.clone().into(),
                ..Default::default()
            },
        );

        Ok(Arc::new(MastodonPlatform {
            chat: chat_interface.clone(),
            mastodon,
        }))
    }

    async fn process_event(self: Arc<Self>, event: Event) -> Result<(), Error> {
        Ok(())
    }
}
