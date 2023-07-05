use diesel::result::Error as dieselError;
use kube::runtime::wait::Error as kubeWaitError;
use kube::Error as kubeError;
use thiserror;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("database error: {0}")]
    Database(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

impl std::convert::From<dieselError> for Error {
    fn from(err: dieselError) -> Self {
        match err {
            dieselError::NotFound => Error::NotFound("row not found".into()),
            dieselError::DatabaseError(_, _) => Error::Database(err.to_string()),
            _ => Error::Internal(err.to_string()),
        }
    }
}

impl std::convert::From<kubeError> for Error {
    fn from(err: kubeError) -> Self {
        match err {
            _ => Error::Internal(err.to_string()),
        }
    }
}

impl std::convert::From<kubeWaitError> for Error {
    fn from(err: kubeWaitError) -> Self {
        match err {
            _ => Error::Internal(err.to_string()),
        }
    }
}
