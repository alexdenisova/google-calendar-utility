use color_eyre::Report as AnyError;
use reqwest::{blocking::Response, StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HoliClientError {
    #[error("Access token has expired.")]
    TokenExpired,
    #[error("Could not parse: {error}.")]
    WrongFormat { error: AnyError },
    #[error("Unexpected error: {error}.")]
    Other { error: AnyError },
}

impl From<reqwest::Error> for HoliClientError {
    fn from(value: reqwest::Error) -> Self {
        HoliClientError::Other {
            error: value.into(),
        }
    }
}

impl From<url::ParseError> for HoliClientError {
    fn from(value: url::ParseError) -> Self {
        HoliClientError::Other {
            error: value.into(),
        }
    }
}

pub trait ToHoliClientError {
    fn map_error(self) -> Result<Self, HoliClientError>
    where
        Self: Sized;
}

impl ToHoliClientError for Response {
    fn map_error(self) -> Result<Self, HoliClientError> {
        if let Err(err) = self.error_for_status_ref() {
            return match self.status() {
                StatusCode::UNAUTHORIZED => Err(HoliClientError::TokenExpired),
                _ => Err(HoliClientError::Other { error: err.into() }),
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
    #[error("Could not find {field} in response json.")]
    MissingJsonField { field: String },
    #[error("Could not parse {field}: expected format {expected}.")]
    WrongFormat { field: String, expected: String },
    #[error("Unknown timezone {time_zone}.")]
    UnknownTimezone { time_zone: String },
    #[error("Unexpected error: {error}.")]
    Other { error: AnyError },
}

impl From<ClassParseError<'static>> for HoliClientError {
    fn from(value: ClassParseError<'static>) -> Self {
        HoliClientError::WrongFormat {
            error: value.into(),
        }
    }
}
