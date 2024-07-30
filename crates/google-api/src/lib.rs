use reqwest::{
    blocking::Response,
    header::{self, HeaderMap, HeaderValue},
};

use errors::GoogleClientError;
use events_client::GoogleEventsClient;
use jwt::JsonWebToken;
use models::{AccessToken, GoogleEvent, GoogleEventListParams, GoogleEventPatch, GoogleEventPost};
use oauth2_client::Oauth2Client;

pub mod api_models;
pub mod errors;
pub mod events_client;
pub mod jwt;
pub mod models;
pub mod oauth2_client;

pub struct GoogleClient {
    events_client: GoogleEventsClient,
    jwt: JsonWebToken,
    access_token: AccessToken,
}

impl GoogleClient {
    pub fn new(
        mut events_client: GoogleEventsClient,
        jwt: JsonWebToken,
    ) -> Result<Self, GoogleClientError> {
        let access_token = Oauth2Client::new(&jwt)?.get_token()?;
        let auth_headers = GoogleClient::auth_headers(&access_token);
        events_client.auth_headers = auth_headers;
        Ok(GoogleClient {
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
        let auth_headers = GoogleClient::auth_headers(&token);
        self.events_client.auth_headers = auth_headers;
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
        params: GoogleEventListParams,
    ) -> Result<Vec<GoogleEvent>, GoogleClientError> {
        self.check_token()?;
        let response =
            self.events_client
                .list_events(params.search_param.clone(), params.start, params.end);
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token()?;
            let mut events = self.list_events(params.clone())?;
            if let Some(email) = params.creator_email {
                events.retain(|x| x.creator_email == email);
            }
            return Ok(events);
        }
        response
    }
    pub fn get_event(&mut self, event_id: &str) -> Result<GoogleEvent, GoogleClientError> {
        self.check_token()?;
        let response = self.events_client.get_event(event_id);
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token()?;
            return self.get_event(event_id);
        }
        response
    }
    pub fn create_event(
        &mut self,
        event: &GoogleEventPost,
    ) -> Result<GoogleEvent, GoogleClientError> {
        self.check_token()?;
        let response = self.events_client.create_event(&event.into());
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token()?;
            return self.create_event(event);
        }
        response
    }
    pub fn update_event(
        &mut self,
        event_id: &str,
        event: &GoogleEventPatch,
    ) -> Result<GoogleEvent, GoogleClientError> {
        self.check_token()?;
        let response = self.events_client.update_event(event_id, &event.into());
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token()?;
            return self.update_event(event_id, event);
        }
        response
    }
    pub fn delete_event(&mut self, event_id: &str) -> Result<Response, GoogleClientError> {
        self.check_token()?;
        let response = self.events_client.delete_event(event_id);
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token()?;
            return self.delete_event(event_id);
        }
        response
    }
}
