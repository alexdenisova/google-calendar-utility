use std::collections::HashMap;

use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Url;
use serde::Deserialize;

use crate::errors::{GoogleClientError, JWTError, ToGoogleClientError};
use crate::jwt::JsonWebToken;
use crate::models::AccessToken;

const OAUTH2_URL: &str = "https://oauth2.googleapis.com/token";
const OUATH2_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:jwt-bearer";

#[derive(Debug)]
pub struct Oauth2Client {
    client: Client,
    base_url: Url,
    form_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u16,
}

impl Oauth2Client {
    pub fn new(jwt: &JsonWebToken) -> Result<Self, JWTError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        let mut form_params = HashMap::new();
        form_params.insert("grant_type".to_owned(), OUATH2_GRANT_TYPE.to_owned());
        form_params.insert("assertion".to_owned(), jwt.encode()?);
        let client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();
        Ok(Self {
            client,
            base_url: Url::parse(OAUTH2_URL).unwrap(),
            form_params,
        })
    }

    pub fn get_token(&self) -> Result<AccessToken, GoogleClientError> {
        let response: TokenResponse = self
            .client
            .post(self.base_url.clone())
            .form(&self.form_params)
            .send()?
            .map_error()?
            .json::<TokenResponse>()?;
        Ok(response.into())
    }
}
