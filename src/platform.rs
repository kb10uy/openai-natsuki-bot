pub mod cli;
pub mod error;
pub mod mastodon;

use std::sync::Arc;

pub trait ConversationPlatform<B> {
    /// このプラットフォームに対して処理を開始する。
    async fn execute(self: Arc<Self>) -> Result<(), error::Error>;
}
