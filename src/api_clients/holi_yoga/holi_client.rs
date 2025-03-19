use async_trait::async_trait;
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Url;
use reqwest::{Client, ClientBuilder};
use scraper::Html;
use uuid::Uuid;

use crate::api_clients::holi_yoga::api_models::{HoliResponseTrait, HoliScheduleResponse};
use crate::api_clients::holi_yoga::parse::parse_schedule;
use crate::api_clients::models::{Class, UtcDateTime};
use crate::api_clients::StudioCRUD;

use super::api_models::{
    HoliLoginData, HoliLoginResponse, HoliMethods, HoliRequestData, HoliUserClassResponse,
    RequestForm,
};
use super::parse::parse_user_classes;
use crate::api_clients::errors::{ClientError, ToClientError};

const HOLI_API_URL: &str = "https://reservi.ru/api-fit1c/json/v2/";

#[derive(Debug)]
pub struct HoliClient {
    client: Client,
    base_url: Url,
    base_form: RequestForm,
    login_info: HoliLoginData,
    club_id: Uuid,
}

impl HoliClient {
    pub async fn new(
        api_key: Uuid,
        club_id: Uuid,
        username: String,
        password: String,
    ) -> Result<Self, ClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let mut base_form = RequestForm::new();
        base_form.insert("api_key", &api_key.to_string());

        let client = ClientBuilder::new().default_headers(headers).build()?;
        let mut holi_client = Self {
            client,
            base_url: Url::parse(HOLI_API_URL)?,
            base_form,
            login_info: HoliLoginData {
                username,
                password,
                api_key,
            },
            club_id,
        };
        holi_client.login().await?;
        Ok(holi_client)
    }

    async fn login(&mut self) -> Result<(), ClientError> {
        let response = self
            .client
            .post(self.base_url.clone())
            .form(self.login_info.form().unwrap())
            .send()
            .await?
            .map_error(None)?
            .json::<HoliLoginResponse>()
            .await?;
        self.base_form.insert_param("token", &response.token);
        Ok(())
    }

    async fn list_user_classes(&self) -> Result<HoliUserClassResponse, ClientError> {
        let mut request_form = HoliRequestData {
            method: HoliMethods::GetUserClasses,
            app_id: None,
        }
        .form();
        request_form.extend(&self.base_form);
        log::debug!("List user classes form: {:?}", request_form);
        self.client
            .post(self.base_url.clone())
            .form(request_form.unwrap())
            .send()
            .await?
            .map_error(None)?
            .json::<HoliUserClassResponse>()
            .await?
            .map_error(None)
    }

    async fn get_day_schedule(
        &self,
        day: &UtcDateTime,
    ) -> Result<HoliScheduleResponse, ClientError> {
        let mut request_form = HoliRequestData {
            method: HoliMethods::GetSchedule,
            app_id: None,
        }
        .form();
        request_form.extend(&self.base_form);
        request_form.insert_param("show_type", "day");
        request_form.insert_param("filter_day", &day.timestamp().to_string());
        log::debug!("Get day schedule form: {:?}", request_form);
        Ok(self
            .client
            .post(self.base_url.clone())
            .form(request_form.unwrap())
            .send()
            .await?
            .map_error(None)?
            .json::<HoliScheduleResponse>()
            .await?)
    }

    async fn post_user_class(&self, class_id: Uuid) -> Result<HoliUserClassResponse, ClientError> {
        let mut request_form = HoliRequestData {
            method: HoliMethods::SignUp,
            app_id: Some(class_id),
        }
        .form();
        request_form.extend(&self.base_form);
        log::debug!("Sign up form: {:?}", request_form);
        let response = self
            .client
            .post(self.base_url.clone())
            .form(request_form.unwrap())
            .send()
            .await?
            .map_error(None)?
            .json::<HoliUserClassResponse>()
            .await?
            .map_error(Some(class_id.to_string()))?;
        log::debug!("Sign up response: {:?}", response);
        Ok(response)
    }
}

#[async_trait]
impl StudioCRUD for HoliClient {
    fn name(&self) -> String {
        "Holi Yoga".to_owned()
    }

    async fn get_user_classes(&self) -> Result<Vec<Class>, ClientError> {
        let response = self.list_user_classes().await?;
        parse_user_classes(
            &Html::parse_document(&response.result),
            response.time_zone(&self.club_id)?,
        )
        .map_err(Into::into)
    }

    async fn list_day_classes(&self, day: &UtcDateTime) -> Result<Vec<Class>, ClientError> {
        let response = self.get_day_schedule(day).await?;
        parse_schedule(
            &Html::parse_document(&response.slider.schedule_html),
            day.date_naive(),
            response.time_zone(&self.club_id)?,
        )
        .map_err(Into::into)
    }

    async fn sign_up_for_class(&self, class: &Class) -> Result<(), ClientError> {
        self.post_user_class(Uuid::parse_str(&class.id)?).await?;
        log::info!("Signed up for {class}");
        Ok(())
    }
}
