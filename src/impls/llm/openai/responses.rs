use crate::{
    impls::llm::openai::create_openai_client,
    model::{config::AppConfigLlmOpenai, conversation::Conversation},
    specs::llm::{Error, LlmBackend, LlmUpdate},
};

use std::sync::Arc;

use async_openai::{Client, config::OpenAIConfig};
use futures::{FutureExt, future::BoxFuture};

/// OpenAI Responses API を利用したバックエンド。
#[derive(Debug, Clone)]
pub struct ResponsesBackend(Arc<ResponsesBackendInner>);

impl ResponsesBackend {
    pub async fn new(config: &AppConfigLlmOpenai) -> Result<ResponsesBackend, Error> {
        let client = create_openai_client(config).await?;
        let model = config.model.clone();

        Ok(ResponsesBackend(Arc::new(ResponsesBackendInner { client, model })))
    }
}

impl LlmBackend for ResponsesBackend {
    fn send_conversation<'a>(&'a self, conversation: &'a Conversation) -> BoxFuture<'a, Result<LlmUpdate, Error>> {
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
    async fn send_conversation(&self, _conversation: &Conversation) -> Result<LlmUpdate, Error> {
        todo!();
    }
}
