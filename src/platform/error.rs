use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("chat interface error: {0}")]
    Chat(
        #[source]
        #[from]
        crate::chat::error::Error,
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

    #[error("requirement(s) not met: {0}")]
    ExpectationMismatch(String),
}
