use crate::model::message::Message;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    id: Uuid,
    messages: Vec<Message>,
}

impl Conversation {
    pub fn new_now(system: Option<Message>) -> Conversation {
        Conversation {
            id: Uuid::now_v7(),
            messages: system.into_iter().collect(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn push_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// 先頭から `taking_messages` の数だけ `messages` を複製した `Conversation` を作成する。
    #[allow(dead_code)]
    pub fn create_branch_now(&self, taking_messages: usize) -> Conversation {
        Conversation {
            id: Uuid::now_v7(),
            messages: self.messages[..taking_messages].to_vec(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredResponse {
    pub text: String,
    pub language: String,
    pub sensitive: bool,
}
