use std::collections::HashMap;

use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Url;
use scraper::Html;
use uuid::Uuid;

use crate::api_models::{
    HoliLoginData, HoliLoginResponse, HoliMethods, HoliRequestData, HoliResponse, RequestForm,
};
use crate::classes::parse_classes;
use crate::errors::{HoliClientError, ToHoliClientError};
use crate::models::Class;

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
    pub fn new(
        api_key: Uuid,
        club_id: Uuid,
        username: String,
        password: String,
    ) -> Result<Self, HoliClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let mut base_form = HashMap::new();
        base_form.insert("api_key".to_owned(), api_key.to_string());

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
        holi_client.login()?;
        Ok(holi_client)
    }

    pub fn login(&mut self) -> Result<(), HoliClientError> {
        let response = self
            .client
            .post(self.base_url.clone())
            .form(&self.login_info.form())
            .send()?
            .map_error()?
            .json::<HoliLoginResponse>()?;
        self.base_form
            .insert("params[token]".to_owned(), response.token);
        Ok(())
    }

    pub fn list_user_classes(&self) -> Result<Vec<Class>, HoliClientError> {
        let mut request_form = HoliRequestData {
            method: HoliMethods::GetUserClasses,
            app_id: None,
        }
        .form();
        request_form.extend(self.base_form.clone());
        log::debug!("List user classes form: {:?}", request_form);
        let response = self
            .client
            .post(self.base_url.clone())
            .form(&request_form)
            .send()?
            .map_error()?
            .json::<HoliResponse>()?;

        parse_classes(
            Html::parse_document(&response.message),
            response.time_zone(&self.club_id)?,
        )
        .map_err(Into::into)
    }
}
