mod claude;
mod openai;

use self::openai::{ChatCompletionBackend, ResponsesBackend};
use crate::{
    model::config::{AppConfigLlm, AppConfigLlmBackend, AppConfigLlmOpenaiApi},
    specs::llm::{Error, LlmBackend},
};

pub async fn create_llm(config: &AppConfigLlm) -> Result<Box<dyn LlmBackend + 'static>, Error> {
    match config.backend {
        AppConfigLlmBackend::Openai => match config.openai.api {
            AppConfigLlmOpenaiApi::ChatCompletion => Ok(Box::new(ChatCompletionBackend::new(&config.openai).await?)),
            AppConfigLlmOpenaiApi::Resnposes => Ok(Box::new(ResponsesBackend::new(&config.openai).await?)),
        },
    }
}
