use crate::{
    assistant::Assistant, error::PlatformError, model::message::UserMessage, specs::platform::ConversationPlatform,
};

use std::io::stdin;

use colored::Colorize;
use futures::{FutureExt, future::BoxFuture};
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

impl ConversationPlatform for CliPlatform {
    fn execute(&self) -> BoxFuture<'static, Result<(), PlatformError>> {
        let assistant = self.assistant.clone();

        async move {
            let mut conversation = assistant.new_conversation();

            // CLI のテキスト入力を別スレッドに分ける
            let (tx, mut rx) = channel(1);
            spawn(CliPlatform::handle_user_input(tx));

            // 応答ループ
            while let Some(input) = rx.recv().await {
                info!("sending {input}");

                let user_message = UserMessage {
                    message: input,
                    ..Default::default()
                };
                let conversation_update = assistant.process_conversation(conversation, user_message).await?;

                println!(">> {}", conversation_update.assistant_message().text.bold().white());
                conversation = conversation_update.finish();
            }
            println!("channel closed");

            Ok(())
        }
        .boxed()
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
