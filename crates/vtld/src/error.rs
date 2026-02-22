use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(Box<redb::Error>),

    #[error("database storage error: {0}")]
    DatabaseStorage(Box<redb::StorageError>),

    #[error("database table error: {0}")]
    DatabaseTable(Box<redb::TableError>),

    #[error("database transaction error: {0}")]
    DatabaseTransaction(Box<redb::TransactionError>),

    #[error("database commit error: {0}")]
    DatabaseCommit(Box<redb::CommitError>),

    #[error("database error: {0}")]
    DatabaseCreate(Box<redb::DatabaseError>),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

impl From<redb::Error> for Error {
    fn from(e: redb::Error) -> Self {
        Error::Database(Box::new(e))
    }
}

impl From<redb::StorageError> for Error {
    fn from(e: redb::StorageError) -> Self {
        Error::DatabaseStorage(Box::new(e))
    }
}

impl From<redb::TableError> for Error {
    fn from(e: redb::TableError) -> Self {
        Error::DatabaseTable(Box::new(e))
    }
}

impl From<redb::TransactionError> for Error {
    fn from(e: redb::TransactionError) -> Self {
        Error::DatabaseTransaction(Box::new(e))
    }
}

impl From<redb::CommitError> for Error {
    fn from(e: redb::CommitError) -> Self {
        Error::DatabaseCommit(Box::new(e))
    }
}

impl From<redb::DatabaseError> for Error {
    fn from(e: redb::DatabaseError) -> Self {
        Error::DatabaseCreate(Box::new(e))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct AdminError(pub Error);

impl From<Error> for AdminError {
    fn from(err: Error) -> Self {
        AdminError(err)
    }
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            Error::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({ "error": self.0.to_string() });
        (status, Json(body)).into_response()
    }
}
