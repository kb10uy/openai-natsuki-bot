use crate::{
    error::LlmError,
    impls::llm::openai::create_openai_client,
    model::{config::AppConfigLlmOpenai, conversation::IncompleteConversation},
    specs::{
        function::simple::SimpleFunctionDescriptor,
        llm::{Llm, LlmUpdate},
    },
};

use std::sync::Arc;

use async_openai::{Client, config::OpenAIConfig};
use futures::{FutureExt, future::BoxFuture};

/// OpenAI Responses API を利用したバックエンド。
#[derive(Debug, Clone)]
pub struct ResponsesBackend(Arc<ResponsesBackendInner>);

impl ResponsesBackend {
    pub async fn new(config: &AppConfigLlmOpenai) -> Result<ResponsesBackend, LlmError> {
        let client = create_openai_client(config).await?;
        let model = config.model.clone();

        Ok(ResponsesBackend(Arc::new(ResponsesBackendInner { client, model })))
    }
}

impl Llm for ResponsesBackend {
    fn add_simple_function(&self, _descriptor: SimpleFunctionDescriptor) -> BoxFuture<'_, ()> {
        todo!()
    }

    fn send_conversation<'a>(
        &'a self,
        conversation: &'a IncompleteConversation,
    ) -> BoxFuture<'a, Result<LlmUpdate, LlmError>> {
        let cloned = self.0.clone();
        async move { cloned.send_conversation(conversation).await }.boxed()
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct ResponsesBackendInner {
    client: Client<OpenAIConfig>,
    model: String,
}

impl ResponsesBackendInner {
    async fn send_conversation(&self, _conversation: &IncompleteConversation) -> Result<LlmUpdate, LlmError> {
        todo!();
    }
}
