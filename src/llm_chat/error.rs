use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(
        #[source]
        #[from]
        reqwest::Error,
    ),

    #[error("OpenAI error: {0}")]
    OpenAI(
        #[source]
        #[from]
        async_openai::error::OpenAIError,
    ),

    /// OpenAI API が有効なレスポンスを生成しなかった。
    #[error("OpenAI API returns no choice to show")]
    NoChoice,
}
