use async_recursion::async_recursion;
use reqwest::Response;

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
    pub async fn new(
        mut events_client: GoogleEventsClient,
        jwt: JsonWebToken,
    ) -> Result<Self, GoogleClientError> {
        let access_token = Oauth2Client::new(&jwt)?.get_token().await?;
        events_client.update_auth_headers(&access_token.token);
        Ok(GoogleClient {
            events_client,
            jwt,
            access_token,
        })
    }

    async fn refresh_token(&mut self) -> Result<(), GoogleClientError> {
        let jwt = self.jwt.clone().refresh();
        let access_token = Oauth2Client::new(&jwt)?.get_token().await?;
        self.jwt = jwt;
        self.events_client.update_auth_headers(&access_token.token);
        self.access_token = access_token;
        Ok(())
    }

    async fn check_token(&mut self) -> Result<(), GoogleClientError> {
        if !self.access_token.is_valid() {
            self.refresh_token().await?;
        }
        Ok(())
    }

    #[async_recursion]
    pub async fn list_events(
        &mut self,
        params: &GoogleEventListParams,
    ) -> Result<Vec<GoogleEvent>, GoogleClientError> {
        self.check_token().await?;
        let response = self
            .events_client
            .list_events(params.search_param.clone(), params.start, params.end)
            .await;
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token().await?;
            return self.list_events(params).await;
        } else {
            let mut events = response?;
            if let Some(email) = &params.creator_email {
                events.retain(|x| x.creator_email.eq(email));
            }
            return Ok(events);
        }
    }
    pub async fn get_event(&mut self, event_id: &str) -> Result<GoogleEvent, GoogleClientError> {
        self.check_token().await?;
        let response = self.events_client.get_event(event_id).await;
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token().await?;
            return self.events_client.get_event(event_id).await;
        }
        response
    }
    pub async fn create_event(
        &mut self,
        event: &GoogleEventPost,
    ) -> Result<GoogleEvent, GoogleClientError> {
        self.check_token().await?;
        let response = self.events_client.create_event(&event.into()).await;
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token().await?;
            return self.events_client.create_event(&event.into()).await;
        }
        response
    }
    pub async fn update_event(
        &mut self,
        event_id: &str,
        event: &GoogleEventPatch,
    ) -> Result<GoogleEvent, GoogleClientError> {
        self.check_token().await?;
        let response = self
            .events_client
            .update_event(event_id, &event.into())
            .await;
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token().await?;
            return self
                .events_client
                .update_event(event_id, &event.into())
                .await;
        }
        response
    }
    pub async fn delete_event(&mut self, event_id: &str) -> Result<Response, GoogleClientError> {
        self.check_token().await?;
        let response = self.events_client.delete_event(event_id).await;
        if let Err(GoogleClientError::TokenExpired) = response {
            self.refresh_token().await?;
            return self.events_client.delete_event(event_id).await;
        }
        response
    }
}
