use std::{fmt::Display, io};

#[derive(Debug)]
pub enum StorageError {
    Database(sqlx::Error),
    Serialization(serde_json::Error),
    Io(io::Error),
    NotFound(String),
    InvalidData(String),
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(f, "database error: {error}"),
            Self::Serialization(error) => write!(f, "serialization error: {error}"),
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::NotFound(message) => write!(f, "resource not found: {message}"),
            Self::InvalidData(message) => write!(f, "invalid data: {message}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<sqlx::Error> for StorageError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

impl From<io::Error> for StorageError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
