pub mod cli;
pub mod error;
pub mod mastodon;

use crate::chat::ChatInterface;

pub trait ConversationPlatform<B> {
    /// このプラットフォームインターフェースを作成する。
    fn create(chat_interface: &ChatInterface<B>) -> Self;

    /// このプラットフォームに対して処理を開始する。
    async fn execute(self) -> Result<(), error::Error>;
}
