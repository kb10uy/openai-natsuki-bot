mod claude;
mod openai;

use self::openai::{ChatCompletionBackend, ResponsesBackend};
use crate::{
    error::LlmError,
    model::config::{AppConfigLlm, AppConfigLlmBackend, AppConfigLlmOpenaiApi},
    specs::llm::Llm,
};

use async_openai::error::OpenAIError;
use reqwest::Error as ReqwestError;

pub async fn create_llm(config: &AppConfigLlm) -> Result<Box<dyn Llm + 'static>, LlmError> {
    match config.backend {
        AppConfigLlmBackend::Openai => match config.openai.api {
            AppConfigLlmOpenaiApi::ChatCompletion => Ok(Box::new(ChatCompletionBackend::new(&config.openai).await?)),
            AppConfigLlmOpenaiApi::Resnposes => Ok(Box::new(ResponsesBackend::new(&config.openai).await?)),
        },
    }
}

impl From<OpenAIError> for LlmError {
    fn from(value: OpenAIError) -> Self {
        LlmError::Backend(value.into())
    }
}

impl From<ReqwestError> for LlmError {
    fn from(value: ReqwestError) -> Self {
        LlmError::Communication(value.into())
    }
}
