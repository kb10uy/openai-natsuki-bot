use crate::model::message::Message;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    id: Uuid,
    messages: Vec<Message>,
}

#[allow(dead_code)]
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
}
