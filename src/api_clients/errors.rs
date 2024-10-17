use color_eyre::Report as AnyError;
use reqwest::{Response, StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Access token has expired.")]
    TokenExpired,
    #[error("Could not parse: {error}.")]
    WrongFormat { error: AnyError },
    #[error("Item with id {id} already exists.")]
    AlreadyExists { id: String },
    #[error("Unexpected error: {error}.")]
    Other { error: AnyError },
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        ClientError::Other {
            error: value.into(),
        }
    }
}

impl From<url::ParseError> for ClientError {
    fn from(value: url::ParseError) -> Self {
        ClientError::WrongFormat {
            error: value.into(),
        }
    }
}

impl From<uuid::Error> for ClientError {
    fn from(value: uuid::Error) -> Self {
        ClientError::WrongFormat {
            error: value.into(),
        }
    }
}

pub trait ToClientError {
    fn map_error(self, id: Option<String>) -> Result<Self, ClientError>
    where
        Self: Sized;
}

impl ToClientError for Response {
    fn map_error(self, _id: Option<String>) -> Result<Self, ClientError> {
        if let Err(err) = self.error_for_status_ref() {
            return match self.status() {
                StatusCode::UNAUTHORIZED => Err(ClientError::TokenExpired),
                _ => Err(ClientError::Other { error: err.into() }),
            };
        }
        Ok(self)
    }
}

#[derive(Error, Debug)]
pub enum ClassParseError<'a> {
    #[error("Could not parse {field}: missing {css_selector} css selector.")]
    MissingCssSelector {
        field: String,
        css_selector: &'a str,
    },
    #[error("Could not parse {css_selector} element: missing {attribute} html attribute.")]
    MissingHtmlAttribute {
        css_selector: String,
        attribute: String,
    },
    #[error("Could not find {field} in response json.")]
    MissingJsonField { field: String },
    #[error("Could not parse {field}: expected format {expected}.")]
    WrongFormat { field: String, expected: String },
    #[error("Unknown timezone {time_zone}.")]
    UnknownTimezone { time_zone: String },
    #[error("Unexpected error: {error}.")]
    Other { error: AnyError },
}

impl From<ClassParseError<'static>> for ClientError {
    fn from(value: ClassParseError<'static>) -> Self {
        ClientError::WrongFormat {
            error: value.into(),
        }
    }
}

impl From<chrono::ParseError> for ClassParseError<'static> {
    fn from(value: chrono::ParseError) -> Self {
        ClassParseError::Other {
            error: value.into(),
        }
    }
}
