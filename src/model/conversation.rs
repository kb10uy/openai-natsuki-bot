use crate::model::message::Message;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    id: Uuid,
    platform: String,
    messages: Vec<Message>,
}
