use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Url;
use reqwest::{Client, ClientBuilder};

use crate::api_clients::models::Class;
use crate::api_clients::ClassCRUD;

use super::api_models::ScheduleListResponse;
use crate::api_clients::errors::{ClientError, ToClientError};

const PLASTILIN_API_URL: &str = "https://mobifitness.ru/api/v8/";

#[derive(Debug)]
pub struct PlastilinClient {
    client: Client,
    base_url: Url,
}

impl PlastilinClient {
    pub fn new(token: &str) -> Result<Self, ClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let mut basic_auth = HeaderValue::from_str(&format!("Bearer {token}")).unwrap();
        basic_auth.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, basic_auth);

        let client = ClientBuilder::new().default_headers(headers).build()?;
        Ok(Self {
            client,
            base_url: Url::parse(PLASTILIN_API_URL)?,
        })
    }
}

impl ClassCRUD for PlastilinClient {
    async fn list_user_classes(&self) -> Result<Vec<Class>, ClientError> {
        let response = self
            .client
            .get(self.base_url.join("account/schedule.json")?)
            .send()
            .await?
            .map_error()?
            .json::<ScheduleListResponse>()
            .await?;
        if let Some(schedules) = response.schedules.first() {
            return Ok(schedules.schedule.iter().map(Into::into).collect());
        }
        Ok(Vec::new())
    }
}
