use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    InvalidSchemaJson(#[from] serde_json::Error),

    #[error("GraphQL schema is invalid")]
    InvalidSchema,
}

pub type Result<T> = std::result::Result<T, Error>;
