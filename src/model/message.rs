use serde::{Deserialize, Serialize};

/// `Conversation` 中の単一メッセージ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    System(SystemMessage),
    User(UserMessage),
    Function(FunctionMessage),
    Assistant(AssistantMessage),
}

impl Message {
    pub fn new_system(text: impl Into<String>) -> Message {
        Message::System(SystemMessage(text.into()))
    }

    pub fn new_user(text: impl Into<String>) -> Message {
        Message::User(UserMessage(text.into()))
    }

    pub fn new_function(text: impl Into<String>) -> Message {
        Message::Function(FunctionMessage(text.into()))
    }

    pub fn new_assistant(text: impl Into<String>) -> Message {
        Message::Assistant(AssistantMessage(text.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserMessage(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SystemMessage(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionMessage(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssistantMessage(pub String);
