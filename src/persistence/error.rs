use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("database error: {0}")]
    Database(
        #[source]
        #[from]
        sqlx::Error,
    ),

    #[error("serialization error: {0}")]
    Serialization(String),
}
