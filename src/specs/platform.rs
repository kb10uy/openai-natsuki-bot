use std::error::Error as StdError;

use futures::future::BoxFuture;
use thiserror::Error as ThisError;

pub trait ConversationPlatform {
    /// このプラットフォームに対して処理を開始する。
    /// 基本的には返される Future は半永久的に処理が続くが、`execute()` 自身は複数回呼ばれる可能性を考慮しなければならない。
    fn execute(&self) -> BoxFuture<'static, Result<(), Error>>;
}

#[derive(Debug, ThisError)]
pub enum Error {
    /// Assistant からのエラー。
    #[error("assistant error: {0}")]
    Assistant(
        #[source]
        #[from]
        crate::assistant::error::Error,
    ),

    /// Platform 自体のエラー。
    #[error("platform error: {0}")]
    PlatformSpecific(#[source] Box<dyn StdError + Send + Sync + 'static>),

    /// 前提条件の不整合が発生した。
    #[error("requirement(s) not met: {0}")]
    ExpectationMismatch(String),

    #[error("HTTP error: {0}")]
    Http(
        #[source]
        #[from]
        reqwest::Error,
    ),
}
