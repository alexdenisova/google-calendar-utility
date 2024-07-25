use color_eyre::Result as AnyResult;
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Url;

use crate::api_models::{EventListResponse, EventPatch, EventPost, EventResponse};
use crate::errors::{GoogleClientError, ToGoogleClientError};
use crate::models::GoogleEvent;

const PAGE_SIZE: &str = "15";
const GOOGLE_API_URL: &str = "https://www.googleapis.com";

#[derive(Debug)]
pub struct EventsClient {
    client: Client,
    base_url: Url,
    pub(crate) auth_headers: HeaderMap,
}

impl EventsClient {
    pub fn new(calendar_id: &str) -> AnyResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

        let client = ClientBuilder::new().default_headers(headers).build()?;
        Ok(Self {
            client,
            base_url: Url::parse(GOOGLE_API_URL)?
                .join(&format!("/calendar/v3/calendars/{calendar_id}/events/"))?,
            auth_headers: HeaderMap::new(),
        })
    }

    pub(crate) fn list_events(
        &self,
        search_param: Option<String>,
    ) -> Result<Vec<GoogleEvent>, GoogleClientError> {
        let mut responses: Vec<GoogleEvent> = Vec::new();
        let mut next_page_token = None;
        let now = chrono::offset::Utc::now().to_rfc3339();
        loop {
            let mut query_params = Vec::from([
                ("maxResults", PAGE_SIZE.to_owned()),
                ("timeMin", now.clone()),
            ]);
            if let Some(token) = next_page_token {
                query_params.push(("pageToken", token));
            }
            if let Some(ref q) = search_param {
                query_params.push(("q", q.clone()));
            }
            let response = self
                .client
                .get(self.base_url.clone())
                .headers(self.auth_headers.clone())
                .query(&query_params)
                .send()
                .map_err(Into::<GoogleClientError>::into)?
                .map_error()?
                .json::<EventListResponse>()?;
            next_page_token = response.next_page_token.clone();
            responses.append(&mut response.items.into_iter().map(Into::into).collect());
            if next_page_token.is_none() {
                break;
            }
        }
        Ok(responses)
    }

    pub(crate) fn get_event(&self, event_id: &str) -> Result<GoogleEvent, GoogleClientError> {
        let response = self
            .client
            .get(self.base_url.join(&format!("{event_id}"))?)
            .headers(self.auth_headers.clone())
            .send()?
            .map_error()?
            .json::<EventResponse>()?;
        Ok(response.into())
    }

    pub(crate) fn create_event(&self, event: &EventPost) -> Result<GoogleEvent, GoogleClientError> {
        let response = self
            .client
            .post(self.base_url.clone())
            .headers(self.auth_headers.clone())
            .json(&event)
            .send()?
            .map_error()?
            .json::<EventResponse>()?;
        Ok(response.into())
    }

    pub(crate) fn update_event(
        &self,
        event_id: &str,
        event: &EventPatch,
    ) -> Result<GoogleEvent, GoogleClientError> {
        let response = self
            .client
            .patch(self.base_url.join(&format!("{event_id}"))?)
            .headers(self.auth_headers.clone())
            .json(&event)
            .send()?
            .map_error()?
            .json::<EventResponse>()?;
        Ok(response.into())
    }

    pub(crate) fn delete_event(&self, event_id: &str) -> Result<Response, GoogleClientError> {
        self.client
            .delete(self.base_url.join(&format!("{event_id}"))?)
            .headers(self.auth_headers.clone())
            .send()?
            .map_error()
    }
}
