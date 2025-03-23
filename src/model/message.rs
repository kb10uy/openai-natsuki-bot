use serde::{Deserialize, Serialize};

/// `Conversation` 中の単一メッセージ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    System(SystemMessage),
    User(UserMessage),
    Function(FunctionMessage),
    Assistant(AssistantMessage),
}

#[allow(dead_code)]
impl Message {
    pub fn new_system(text: impl Into<String>) -> Message {
        Message::System(SystemMessage(text.into()))
    }

    pub fn new_user(text: impl Into<String>) -> Message {
        Message::User(UserMessage(text.into()))
    }

    pub fn new_function(function_name: impl Into<String>, text: impl Into<String>) -> Message {
        Message::Function(FunctionMessage(function_name.into(), text.into()))
    }

    pub fn new_assistant(text: impl Into<String>, is_sensitive: bool) -> Message {
        Message::Assistant(AssistantMessage {
            text: text.into(),
            is_sensitive,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserMessage(pub String);

impl From<UserMessage> for Message {
    fn from(value: UserMessage) -> Message {
        Message::User(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SystemMessage(pub String);

impl From<SystemMessage> for Message {
    fn from(value: SystemMessage) -> Message {
        Message::System(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionMessage(pub String, pub String);

impl From<FunctionMessage> for Message {
    fn from(value: FunctionMessage) -> Message {
        Message::Function(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub text: String,
    pub is_sensitive: bool,
}

impl From<AssistantMessage> for Message {
    fn from(value: AssistantMessage) -> Message {
        Message::Assistant(value)
    }
}
