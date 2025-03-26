use reqwest::Error as ReqwestError;
use thiserror::Error;

/// Context-based errors, plus wrapped reqwest errors
#[derive(Error, Debug)]
pub enum ECSMetadataError {
    #[error("Failed to fetch ECS metadata")]
    FetchError,
    #[error("HTTP error: {0}")]
    HttpError(#[from] ReqwestError),
    #[error("Environment variable {0} not set")]
    EnvVarNotSet(String),
}