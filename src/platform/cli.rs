use super::{ConversationPlatform, error::Error};
use crate::{assistant::Assistant, model::message::Message};

use std::io::stdin;

use async_trait::async_trait;
use colored::Colorize;
use thiserror::Error as ThisError;
use tokio::{
    spawn,
    sync::mpsc::{Sender, channel},
};
use tracing::{debug, info};

#[derive(Debug)]
pub struct CliPlatform {
    assistant: Assistant,
}

#[async_trait]
impl ConversationPlatform for CliPlatform {
    async fn execute(&self) -> Result<(), Error> {
        let mut conversation = self.assistant.new_conversation();

        // CLI のテキスト入力を別スレッドに分ける
        let (tx, mut rx) = channel(1);
        spawn(CliPlatform::handle_user_input(tx));

        // 応答ループ
        while let Some(input) = rx.recv().await {
            info!("sending {input}");
            conversation.push_message(Message::new_user(input));
            let conversation_update = self.assistant.process_conversation(&conversation).await?;
            println!(">> {}", conversation_update.assistant_response.text.bold().white());
            conversation.push_message(conversation_update.assistant_response.into());
        }
        println!("channel closed");

        Ok(())
    }
}

impl CliPlatform {
    pub fn new(assistant: Assistant) -> CliPlatform {
        CliPlatform { assistant }
    }

    /// stdin の行を Sender に流す。
    async fn handle_user_input(tx: Sender<String>) -> Result<(), CliError> {
        debug!("reading stdin in another task");
        let mut buffer = String::new();
        while stdin().read_line(&mut buffer).map_err(|_| CliError::Stdin)? != 0 {
            let text = buffer.trim_end().to_string();
            tx.send(text).await.map_err(|_| CliError::Communication)?;

            buffer.clear();
        }
        Ok(())
    }
}

#[derive(Debug, Clone, ThisError)]
pub enum CliError {
    #[error("something went wrong in stdin")]
    Stdin,

    #[error("something went wrong inter-thread communication")]
    Communication,
}
