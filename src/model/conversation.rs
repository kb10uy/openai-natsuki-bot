use crate::model::message::{AssistantMessage, Message, UserMessage};

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
}

#[derive(Debug, Clone)]
pub struct IncompleteConversation {
    pub id: Uuid,
    pub latest_messages: Vec<Message>,
}

impl IncompleteConversation {
    pub fn start(mut conversation: Conversation, user_message: UserMessage) -> IncompleteConversation {
        conversation.messages.push(user_message.into());

        IncompleteConversation {
            id: conversation.id,
            latest_messages: conversation.messages,
        }
    }

    pub fn finish(self, last_assistant_message: AssistantMessage) -> ConversationUpdate {
        ConversationUpdate {
            conversation: Conversation {
                id: self.id,
                messages: self.latest_messages,
            },
            assistant_message: last_assistant_message,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversationUpdate {
    conversation: Conversation,
    assistant_message: AssistantMessage,
}

impl ConversationUpdate {
    pub fn assistant_message(&self) -> &AssistantMessage {
        &self.assistant_message
    }

    pub fn finish(mut self) -> Conversation {
        self.conversation.messages.push(self.assistant_message.into());
        self.conversation
    }
}
