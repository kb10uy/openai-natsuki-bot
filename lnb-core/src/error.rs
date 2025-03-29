use std::error::Error as StdError;

use thiserror::Error as ThisError;

/// LnbClient のエラー。
#[derive(Debug, ThisError)]
pub enum ClientError {
    /// Server からのエラー。
    #[error("assistant error: {0}")]
    Server(
        #[source]
        #[from]
        ServerError,
    ),

    /// 通信関連のエラー。
    #[error("communication failed: {0}")]
    Communication(#[source] Box<dyn StdError + Send + Sync + 'static>),

    /// 外部 API などのエラー。
    #[error("external error: {0}")]
    External(#[source] Box<dyn StdError + Send + Sync + 'static>),
}

/// LnbServer のエラー。
#[derive(Debug, ThisError)]
pub enum ServerError {
    #[error("LLM error: {0}")]
    Llm(
        #[source]
        #[from]
        LlmError,
    ),

    #[error("storage error: {0}")]
    Storage(
        #[source]
        #[from]
        StorageError,
    ),

    #[error("function error: {0}")]
    Function(
        #[source]
        #[from]
        FunctionError,
    ),

    /// 期待されていた応答が存在しなかった。
    #[error("expected chat resnpose not found")]
    ChatResponseExpected,
}

/// LLM 層のエラー。
#[derive(Debug, ThisError)]
pub enum LlmError {
    #[error("communication failed: {0}")]
    Communication(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("backend error: {0}")]
    Backend(#[source] Box<dyn StdError + Send + Sync + 'static>),

    /// LLM が有効なレスポンスを生成しなかった。
    #[error("no choice returned")]
    NoChoice,

    /// JSON の復元ができない。
    #[error("invalid response format: {0}")]
    ResponseFormat(#[source] Box<dyn StdError + Send + Sync + 'static>),
}

/// Storage 層のエラー。
#[derive(Debug, ThisError)]
pub enum StorageError {
    #[error("backend error: {0}")]
    Backend(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("serialization error: {0}")]
    Serialization(#[source] Box<dyn StdError + Send + Sync + 'static>),
}

/// Function 層のエラー。
#[derive(Debug, ThisError)]
pub enum FunctionError {
    #[error("serialization error: {0}")]
    Serialization(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("external error: {0}")]
    External(#[source] Box<dyn StdError + Send + Sync + 'static>),
}
