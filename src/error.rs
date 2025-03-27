use std::error::Error as StdError;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum PlatformError {
    /// Assistant からのエラー。
    #[error("assistant error: {0}")]
    Assistant(
        #[source]
        #[from]
        AssistantError,
    ),

    #[error("communication failed: {0}")]
    Communication(#[source] Box<dyn StdError + Send + Sync + 'static>),

    #[error("external error: {0}")]
    External(#[source] Box<dyn StdError + Send + Sync + 'static>),

    /// 前提条件の不整合が発生した。
    #[error("requirement(s) not met: {0}")]
    ExpectationMismatch(String),
}

/// Assistant 層のエラー。
#[derive(Debug, ThisError)]
pub enum AssistantError {
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
}
