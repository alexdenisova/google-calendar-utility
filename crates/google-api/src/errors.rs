use color_eyre::Report as AnyError;
use reqwest::{blocking::Response, StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GoogleClientError {
    #[error("Error creating JWT: {error}")]
    JWT { error: JWTError },
    #[error("Access token has expired.")]
    TokenExpired,
    #[error("Event with id \"{id}\" does not exist.")]
    NotFound { id: String },
    #[error("Unexpected error: {error}.")]
    Other { error: AnyError },
}

impl From<reqwest::Error> for GoogleClientError {
    fn from(value: reqwest::Error) -> Self {
        GoogleClientError::Other {
            error: value.into(),
        }
    }
}

impl From<url::ParseError> for GoogleClientError {
    fn from(value: url::ParseError) -> Self {
        GoogleClientError::Other {
            error: value.into(),
        }
    }
}

pub trait ToGoogleClientError {
    fn map_error(self) -> Result<Self, GoogleClientError>
    where
        Self: Sized;
}

impl ToGoogleClientError for Response {
    fn map_error(self) -> Result<Self, GoogleClientError> {
        if let Err(err) = self.error_for_status_ref() {
            let url = self.url().to_string();
            let id = url.split('/').last().unwrap_or_default();
            return match self.status() {
                StatusCode::UNAUTHORIZED => Err(GoogleClientError::TokenExpired),
                StatusCode::NOT_FOUND => Err(GoogleClientError::NotFound { id: id.to_string() }),
                _ => Err(GoogleClientError::Other { error: err.into() }),
            };
        }
        Ok(self)
    }
}

#[derive(Error, Debug)]
pub enum JWTError {
    #[error("Private key wrong format, must be a RSA key.")]
    InvalidKey,
    #[error("Could not create JWT: {error}.")]
    EncodingError { error: AnyError },
}

impl From<JWTError> for GoogleClientError {
    fn from(value: JWTError) -> Self {
        GoogleClientError::JWT { error: value }
    }
}
