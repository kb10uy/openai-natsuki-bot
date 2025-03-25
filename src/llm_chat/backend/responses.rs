use crate::{
    application::config::AppConfigOpenai,
    llm_chat::{LlmChatUpdate, backend::Backend, error::Error, openai::create_openai_client},
    model::conversation::Conversation,
};

use std::sync::Arc;

use async_openai::{Client, config::OpenAIConfig};
use futures::{FutureExt, future::BoxFuture};

/// OpenAI Responses API を利用したバックエンド。
#[derive(Debug, Clone)]
pub struct ResponsesBackend(Arc<ResponsesBackendInner>);

impl ResponsesBackend {
    pub async fn new(openai_config: &AppConfigOpenai) -> Result<ResponsesBackend, Error> {
        let client = create_openai_client(openai_config).await?;
        let model = openai_config.model.clone();

        Ok(ResponsesBackend(Arc::new(ResponsesBackendInner { client, model })))
    }
}

impl Backend for ResponsesBackend {
    fn send_conversation<'a>(&'a self, conversation: &'a Conversation) -> BoxFuture<'a, Result<LlmChatUpdate, Error>> {
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
    async fn send_conversation(&self, _conversation: &Conversation) -> Result<LlmChatUpdate, Error> {
        todo!();
    }
}
