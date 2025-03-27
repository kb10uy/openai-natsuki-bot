use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `Conversation` 中の単一メッセージ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    System(SystemMessage),
    User(UserMessage),
    FunctionCalls(FunctionCallsMessage),
    FunctionResponse(FunctionResponseMessage),
    Assistant(AssistantMessage),
}

#[allow(dead_code)]
impl Message {
    pub fn new_system(text: impl Into<String>) -> Message {
        Message::System(SystemMessage(text.into()))
    }

    pub fn new_user(text: impl Into<String>, name: Option<String>, language: Option<String>) -> Message {
        Message::User(UserMessage {
            message: text.into(),
            name,
            language,
        })
    }

    pub fn new_function_calls(calls: impl IntoIterator<Item = FunctionCall>) -> Message {
        Message::FunctionCalls(FunctionCallsMessage(calls.into_iter().collect()))
    }

    pub fn new_function_response(id: impl Into<String>, name: impl Into<String>, result: impl Into<Value>) -> Message {
        Message::FunctionResponse(FunctionResponseMessage {
            id: id.into(),
            name: name.into(),
            result: result.into(),
        })
    }

    pub fn new_assistant(text: impl Into<String>, is_sensitive: bool, language: Option<String>) -> Message {
        Message::Assistant(AssistantMessage {
            text: text.into(),
            is_sensitive,
            language,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserMessage {
    pub message: String,
    pub name: Option<String>,
    pub language: Option<String>,
}

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
pub struct FunctionCallsMessage(pub Vec<FunctionCall>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

impl From<FunctionCallsMessage> for Message {
    fn from(value: FunctionCallsMessage) -> Message {
        Message::FunctionCalls(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionResponseMessage {
    pub id: String,
    pub name: String,
    pub result: Value,
}

impl From<FunctionResponseMessage> for Message {
    fn from(value: FunctionResponseMessage) -> Message {
        Message::FunctionResponse(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub text: String,
    pub is_sensitive: bool,
    pub language: Option<String>,
}

impl From<AssistantMessage> for Message {
    fn from(value: AssistantMessage) -> Message {
        Message::Assistant(value)
    }
}
