use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    /// `LlmInterface` 内のエラー。
    #[error("LLM interface error: {0}")]
    Llm(
        #[source]
        #[from]
        crate::specs::llm::Error,
    ),

    /// 永続化層のエラー。
    #[error("persistence error: {0}")]
    Persistence(
        #[source]
        #[from]
        crate::specs::storage::Error,
    ),

    #[error("HTTP error: {0}")]
    Http(
        #[source]
        #[from]
        reqwest::Error,
    ),

    /// assistant role のメッセージを構築できない。
    #[error("LLM interface returned no response")]
    NoAssistantResponse,
}
