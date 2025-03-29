use std::error::Error as StdError;

use thiserror::Error as ThisError;

pub type ErasedError = Box<dyn StdError + Send + Sync + 'static>;

/// LnbClient のエラー。
#[derive(Debug, ThisError)]
pub enum ClientError {
    /// Server からのエラー。
    #[error("assistant error: {0}")]
    Server(#[from] ServerError),

    /// 通信関連のエラー。
    #[error("communication failed: {0}")]
    Communication(#[source] ErasedError),

    /// 外部 API などのエラー。
    #[error("external error: {0}")]
    External(#[source] ErasedError),
}

impl ClientError {
    pub fn by_communication(source: impl Into<ErasedError>) -> ClientError {
        ClientError::Communication(source.into())
    }

    pub fn by_external(source: impl Into<ErasedError>) -> ClientError {
        ClientError::External(source.into())
    }
}

/// LnbServer のエラー。
#[derive(Debug, ThisError)]
pub enum ServerError {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("function error: {0}")]
    Function(#[from] FunctionError),

    /// 期待されていた応答が存在しなかった。
    #[error("expected chat resnpose not found")]
    ChatResponseExpected,
}

/// LLM 層のエラー。
#[derive(Debug, ThisError)]
pub enum LlmError {
    #[error("communication failed: {0}")]
    Communication(#[source] ErasedError),

    #[error("backend error: {0}")]
    Backend(#[source] ErasedError),

    /// LLM が有効なレスポンスを生成しなかった。
    #[error("no choice returned")]
    NoChoice,

    /// JSON の復元ができない。
    #[error("invalid response format: {0}")]
    ResponseFormat(#[source] ErasedError),
}

impl LlmError {
    pub fn by_communication(source: impl Into<ErasedError>) -> LlmError {
        LlmError::Communication(source.into())
    }

    pub fn by_backend(source: impl Into<ErasedError>) -> LlmError {
        LlmError::Backend(source.into())
    }

    pub fn by_format(source: impl Into<ErasedError>) -> LlmError {
        LlmError::ResponseFormat(source.into())
    }
}

/// Storage 層のエラー。
#[derive(Debug, ThisError)]
pub enum StorageError {
    #[error("backend error: {0}")]
    Backend(#[source] ErasedError),

    #[error("serialization error: {0}")]
    Serialization(#[source] ErasedError),
}

impl StorageError {
    pub fn by_serialization(source: impl Into<ErasedError>) -> StorageError {
        StorageError::Backend(source.into())
    }

    pub fn by_backend(source: impl Into<ErasedError>) -> StorageError {
        StorageError::Backend(source.into())
    }
}

/// Function 層のエラー。
#[derive(Debug, ThisError)]
pub enum FunctionError {
    #[error("serialization error: {0}")]
    Serialization(#[source] ErasedError),

    #[error("external error: {0}")]
    External(#[source] ErasedError),
}

impl FunctionError {
    pub fn by_serialization(source: impl Into<ErasedError>) -> FunctionError {
        FunctionError::Serialization(source.into())
    }

    pub fn by_external(source: impl Into<ErasedError>) -> FunctionError {
        FunctionError::External(source.into())
    }
}
