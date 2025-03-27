use crate::{error::LlmError, model::conversation::Conversation, specs::function::simple::SimpleFunctionDescriptor};

use std::fmt::Debug;

use futures::future::BoxFuture;
use serde::Deserialize;
use serde_json::Value;

#[allow(dead_code)]
pub trait Llm: Send + Sync + Debug {
    /// `SimpleFunction` の追加を告知する。
    fn add_simple_function(&self, descriptor: SimpleFunctionDescriptor) -> BoxFuture<'_, ()>;

    /// `Conversation` を送信する。
    fn send_conversation<'a>(&'a self, conversation: &'a Conversation) -> BoxFuture<'a, Result<LlmUpdate, LlmError>>;
}

/// Conversation を送信した結果生成された内容。
#[derive(Debug, Clone)]
pub struct LlmUpdate {
    pub response: Option<LlmAssistantResponse>,
    pub tool_callings: Option<Vec<LlmToolCalling>>,
}

/// assistant role としての応答内容。
#[derive(Debug, Clone, Deserialize)]
pub struct LlmAssistantResponse {
    pub text: String,
    pub language: Option<String>,
    pub sensitive: Option<bool>,
}

/// Conversation を送信した結果生成された内容。
#[derive(Debug, Clone)]
pub struct LlmToolCalling {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}
