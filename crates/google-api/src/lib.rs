use api_models::{EventPatch, EventPost};
use color_eyre::Result as AnyResult;

use errors::GoogleClientError;
use events_client::EventsClient;
use jwt::JsonWebToken;
use models::{AccessToken, GoogleEvent};
use oauth2_client::Oauth2Client;
use reqwest::{
    blocking::Response,
    header::{self, HeaderMap, HeaderValue},
};

pub mod api_models;
pub mod errors;
pub mod events_client;
pub mod jwt;
pub mod models;
pub mod oauth2_client;

pub struct GoogleAPI {
    events_client: EventsClient,
    jwt: JsonWebToken,
    access_token: AccessToken,
}

impl GoogleAPI {
    pub fn new(events_client: EventsClient, jwt: JsonWebToken) -> AnyResult<Self> {
        let access_token = Oauth2Client::new(&jwt)?.get_token()?;
        Ok(GoogleAPI {
            events_client,
            jwt,
            access_token,
        })
    }

    fn auth_headers(token: &AccessToken) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let mut basic_auth = HeaderValue::from_str(&format!("Bearer {}", token.token)).unwrap();
        basic_auth.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, basic_auth);
        headers
    }
    fn refresh_token(&mut self) -> Result<(), GoogleClientError> {
        let jwt = self.jwt.clone().refresh();
        let token = Oauth2Client::new(&jwt)?.get_token()?;
        self.jwt = jwt;
        let auth_headers = GoogleAPI::auth_headers(&token);
        self.events_client.auth_headers = auth_headers.clone();
        self.access_token = token;
        Ok(())
    }
    fn check_token(&mut self) -> Result<(), GoogleClientError> {
        if !self.access_token.is_valid() {
            self.refresh_token()?;
        }
        Ok(())
    }

    pub fn list_events(
        &mut self,
        search_param: Option<String>,
    ) -> Result<Vec<GoogleEvent>, GoogleClientError> {
        self.check_token()?;
        let response = self.events_client.list_events(search_param.clone());
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token()?;
            return self.list_events(search_param);
        }
        return response;
    }
    pub fn get_event(&mut self, event_id: &str) -> Result<GoogleEvent, GoogleClientError> {
        self.check_token()?;
        let response = self.events_client.get_event(event_id);
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token()?;
            return self.get_event(event_id);
        }
        return response;
    }
    pub fn create_event(&mut self, event: &EventPost) -> Result<GoogleEvent, GoogleClientError> {
        self.check_token()?;
        let response = self.events_client.create_event(event);
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token()?;
            return self.create_event(event);
        }
        return response;
    }
    pub fn update_event(
        &mut self,
        event_id: &str,
        event: &EventPatch,
    ) -> Result<GoogleEvent, GoogleClientError> {
        self.check_token()?;
        let response = self.events_client.update_event(event_id, event);
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token()?;
            return self.update_event(event_id, event);
        }
        return response;
    }
    pub fn delete_event(&mut self, event_id: &str) -> Result<Response, GoogleClientError> {
        self.check_token()?;
        let response = self.events_client.delete_event(event_id);
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token()?;
            return self.delete_event(event_id);
        }
        return response;
    }
}
