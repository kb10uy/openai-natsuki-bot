pub mod cli;
pub mod error;
pub mod mastodon;

use async_trait::async_trait;

#[async_trait]
pub trait ConversationPlatform {
    /// このプラットフォームに対して処理を開始する。
    /// 基本的には返される Future は半永久的に処理が続くが、 `execute()` は複数回呼ばれる可能性を考慮しなければならない。
    async fn execute(&self) -> Result<(), error::Error>;
}
