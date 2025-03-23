use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("chat interface error: {0}")]
    Chat(
        #[source]
        #[from]
        crate::chat::error::Error,
    ),
}
