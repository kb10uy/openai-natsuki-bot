pub mod cli;
pub mod error;
pub mod mastodon;

use std::sync::Arc;

use async_trait::async_trait;

#[async_trait]
pub trait ConversationPlatform {
    /// このプラットフォームに対して処理を開始する。
    async fn execute(self: Arc<Self>) -> Result<(), error::Error>;
}
