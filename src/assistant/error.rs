use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    /// `ChatInterface` 内のエラー。
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

    /// assistant role のメッセージを構築できない。
    #[error("chat interface returned no response")]
    NoAssistantResponse,
}
