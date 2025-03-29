mod inner;

use crate::natsuki::inner::NatsukiInner;

use std::sync::Arc;

use futures::future::BoxFuture;
use lnb_core::{
    config::AppConfigAssistantIdentity,
    error::ServerError,
    interface::{llm::Llm, server::LnbServer, storage::ConversationStorage},
    model::{
        conversation::{Conversation, ConversationUpdate},
        message::UserMessage,
    },
};

#[derive(Debug, Clone)]
pub struct Natsuki(Arc<NatsukiInner>);

impl Natsuki {
    pub fn new(
        assistant_identity: &AppConfigAssistantIdentity,
        llm: Box<dyn Llm + 'static>,
        storage: Box<dyn ConversationStorage + 'static>,
    ) -> Natsuki {
        Natsuki(Arc::new(NatsukiInner::new(assistant_identity, llm, storage)))
    }
}

impl LnbServer for Natsuki {
    fn new_conversation(&self) -> Conversation {
        todo!()
    }

    fn restore_conversation<'a>(
        &'a self,
        platform: &'a str,
        context: &'a str,
    ) -> BoxFuture<'a, Result<Option<Conversation>, ServerError>> {
        todo!()
    }

    fn save_conversation<'a>(
        &'a self,
        conversation: &'a Conversation,
        platform: &'a str,
        context: &'a str,
    ) -> BoxFuture<'a, Result<(), ServerError>> {
        todo!()
    }

    fn process_conversation(
        &self,
        conversation: Conversation,
        user_message: UserMessage,
    ) -> BoxFuture<'_, Result<ConversationUpdate, ServerError>> {
        todo!()
    }
}
