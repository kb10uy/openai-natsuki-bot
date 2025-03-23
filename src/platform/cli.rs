use super::{ConversationPlatform, error::Error};
use crate::{
    chat::{ChatBackend, ChatInterface},
    model::message::Message,
};

use std::{io::stdin, sync::Arc};

use colored::Colorize;
use thiserror::Error as ThisError;
use tokio::{
    spawn,
    sync::mpsc::{Sender, channel},
};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct CliPlatform<B> {
    chat: ChatInterface<B>,
}

impl<B: ChatBackend> ConversationPlatform<B> for CliPlatform<B> {
    async fn execute(self: Arc<Self>) -> Result<(), Error> {
        let mut conversation = self.chat.create_conversation();

        // CLI のテキスト入力を別スレッドに分ける
        let (tx, mut rx) = channel(1);
        spawn(Self::handle_user_input(tx));

        // 応答ループ
        while let Some(input) = rx.recv().await {
            info!("sending {input}");
            conversation.push_message(Message::new_user(input));
            let update = self.chat.send(&conversation).await?;

            if let Some(assistant_text) = update.text {
                println!(">> {}", assistant_text.bold().white());
                conversation.push_message(Message::new_assistant(assistant_text));
            }
        }
        println!("channel closed");

        Ok(())
    }
}

impl<B: ChatBackend> CliPlatform<B> {
    pub fn new(chat_interface: &ChatInterface<B>) -> Arc<Self> {
        Arc::new(CliPlatform {
            chat: chat_interface.clone(),
        })
    }

    /// stdin の行を Sender に流す。
    async fn handle_user_input(tx: Sender<String>) -> Result<(), CliError> {
        debug!("reading stdin in another task");
        let mut buffer = String::new();
        while stdin()
            .read_line(&mut buffer)
            .map_err(|_| CliError::Stdin)?
            != 0
        {
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
