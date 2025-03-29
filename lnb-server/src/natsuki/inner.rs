use std::collections::HashMap;

use lnb_core::{
    config::AppConfigAssistantIdentity,
    error::ServerError,
    interface::{function::simple::SimpleFunction, llm::Llm, storage::ConversationStorage},
};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct NatsukiInner {
    llm: Box<dyn Llm + 'static>,
    storage: Box<dyn ConversationStorage + 'static>,
    simple_functions: Mutex<HashMap<String, Box<dyn SimpleFunction + 'static>>>,
    system_role: String,
    sensitive_marker: String,
}

impl NatsukiInner {
    pub fn new(
        assistant_identity: &AppConfigAssistantIdentity,
        llm: Box<dyn Llm + 'static>,
        storage: Box<dyn ConversationStorage + 'static>,
    ) -> Result<NatsukiInner, ServerError> {
        Ok(NatsukiInner {
            llm,
            storage,
            simple_functions: Mutex::new(HashMap::new()),
            system_role: assistant_identity.system_role.clone(),
            sensitive_marker: assistant_identity.sensitive_marker.clone(),
        })
    }
}
