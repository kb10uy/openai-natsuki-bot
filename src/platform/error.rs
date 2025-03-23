use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("assistant error: {0}")]
    Assistant(
        #[source]
        #[from]
        crate::assistant::error::Error,
    ),

    #[error("HTTP error: {0}")]
    Http(
        #[source]
        #[from]
        reqwest::Error,
    ),

    #[error("Mastodon error: {0}")]
    Mastodon(
        #[source]
        #[from]
        mastodon_async::Error,
    ),

    /// 前提条件の不整合が発生した。
    #[error("requirement(s) not met: {0}")]
    ExpectationMismatch(String),
}
