use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, TimeDelta};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Url;
use reqwest::{Client, ClientBuilder};

use crate::api_clients::models::{Class, UtcDateTime};
use crate::api_clients::StudioCRUD;

use super::api_models::{ClassResponse, PostUserClass, ScheduleResponse, UserClassListResponse};
use crate::api_clients::errors::{ClientError, ToClientError};

const PLASTILIN_API_URL: &str = "https://mobifitness.ru/api/v8/";

#[derive(Debug)]
pub struct PlastilinClient {
    client: Client,
    base_url: Url,
    club_id: u16,
}

impl PlastilinClient {
    pub fn new(token: &str, club_id: u16) -> Result<Self, ClientError> {
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
            club_id,
        })
    }

    async fn list_user_classes(&self) -> Result<Vec<ClassResponse>, ClientError> {
        let response = self
            .client
            .get(self.base_url.join("account/schedule.json")?)
            .send()
            .await?
            .map_error(None)?
            .json::<UserClassListResponse>()
            .await?;
        if let Some(schedules) = response.schedules.first() {
            return Ok(schedules.schedule.clone());
        }
        Ok(Vec::new())
    }

    async fn get_day_schedule(&self, day: &UtcDateTime) -> Result<Vec<ClassResponse>, ClientError> {
        let offset = day.weekday().num_days_from_monday();
        let week_number = (NaiveDate::from_ymd_opt(day.year(), day.month(), day.day()).unwrap()
            - NaiveDate::from_ymd_opt(day.year(), 1, 1).unwrap()
            + TimeDelta::days(offset.into()))
        .num_days()
            / 7;

        let response = self
            .client
            .get(
                self.base_url
                    .join(&format!("club/{}/schedule.json", self.club_id))?,
            )
            .query(&[("year", day.year().into()), ("week", week_number)])
            .send()
            .await?
            .map_error(None)?
            .json::<ScheduleResponse>()
            .await?;
        Ok(response.schedule)
    }

    async fn post_user_class(
        &self,
        class_id: &str,
        body: PostUserClass,
    ) -> Result<(), ClientError> {
        self.client
            .post(self.base_url.join("account/reserve.json")?)
            .json(&body)
            .send()
            .await?
            .map_error(Some(class_id.to_owned()))?;
        Ok(())
    }
}

#[async_trait]
impl StudioCRUD for PlastilinClient {
    fn name(&self) -> String {
        "Plastilin".to_owned()
    }

    async fn get_user_classes(&self) -> Result<Vec<Class>, ClientError> {
        Ok(self
            .list_user_classes()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn list_day_classes(&self, day: &UtcDateTime) -> Result<Vec<Class>, ClientError> {
        Ok(self
            .get_day_schedule(day)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn sign_up_for_class(&self, class: &Class) -> Result<(), ClientError> {
        self.post_user_class(&class.id, class.into()).await?;
        log::info!("Signed up for {class}");
        Ok(())
    }
}
