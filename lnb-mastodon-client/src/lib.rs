mod error;
mod inner;
mod text;

use crate::inner::MastodonPlatformInner;

use std::sync::Arc;

use futures::{future::BoxFuture, prelude::*};
use lnb_core::{
    config::AppConfigPlatformMastodon,
    error::PlatformError,
    interface::{client::LnbClient, server::LnbServer},
};

#[derive(Debug)]
pub struct MastodonPlatform<S>(Arc<MastodonPlatformInner<S>>);

impl<S: LnbServer> MastodonPlatform<S> {
    pub async fn new(
        config_mastodon: &AppConfigPlatformMastodon,
        assistant: S,
    ) -> Result<MastodonPlatform<S>, PlatformError> {
        let inner = match MastodonPlatformInner::new(config_mastodon, assistant).await {
            Ok(i) => i,
            Err(e) => return Err(e.into()),
        };
        Ok(MastodonPlatform(Arc::new(inner)))
    }
}

impl<S: LnbServer> LnbClient for MastodonPlatform<S> {
    fn execute(&self) -> BoxFuture<'static, Result<(), PlatformError>> {
        let cloned_inner = self.0.clone();
        async {
            cloned_inner.execute().await?;
            Ok(())
        }
        .boxed()
    }
}
