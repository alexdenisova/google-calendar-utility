use email_address::EmailAddress;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::errors::JWTError;

const JWT_ASSERTION_EXPIRATION_MINS: u8 = 60;

#[derive(Clone)]
struct HeaderWrapper(Header);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    iss: EmailAddress,
    scope: String,
    aud: String,
    exp: u64,
    iat: u64,
}

#[derive(Clone)]
pub struct JsonWebToken {
    header: HeaderWrapper,
    claims: Claims,
    encoding_key: EncodingKey,
}

impl HeaderWrapper {
    pub fn build(key_id: String) -> Self {
        let mut header = Header::new(Algorithm::RS256);
        header.typ = Some("JWT".to_owned());
        header.kid = Some(key_id);
        HeaderWrapper(header)
    }
}

impl<'a> From<&'a HeaderWrapper> for &'a Header {
    fn from(value: &'a HeaderWrapper) -> Self {
        let HeaderWrapper(inside) = value;
        inside
    }
}

impl JsonWebToken {
    pub fn build(
        key_id: String,
        sa_email: EmailAddress,
        private_key: String,
    ) -> Result<Self, JWTError> {
        let iat = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let exp = iat + Duration::from_secs(60 * JWT_ASSERTION_EXPIRATION_MINS as u64);
        Ok(JsonWebToken {
        header: HeaderWrapper::build(key_id),
        claims: Claims{
          iss: sa_email,
          scope: "https://www.googleapis.com/auth/calendar https://www.googleapis.com/auth/calendar.events".to_owned(),
          aud: "https://oauth2.googleapis.com/token".to_owned(),
          exp: exp.as_secs(),
          iat: iat.as_secs()
        },
        encoding_key: EncodingKey::from_rsa_pem(private_key.as_bytes()).map_err(|_| JWTError::InvalidKey)?,
    })
    }
    pub fn encode(&self) -> Result<String, JWTError> {
        encode((&self.header).into(), &self.claims, &self.encoding_key)
            .map_err(|err| JWTError::EncodingError { error: err.into() })
    }
    pub fn refresh(mut self) -> Self {
        let iat = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let exp = iat + Duration::from_secs(60 * JWT_ASSERTION_EXPIRATION_MINS as u64);
        self.claims.iat = iat.as_secs();
        self.claims.exp = exp.as_secs();
        self
    }
}
