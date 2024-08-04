use std::collections::HashMap;

use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Url;
use reqwest::{Client, ClientBuilder};
use scraper::Html;
use uuid::Uuid;

use crate::api_clients::models::Class;
use crate::api_clients::ClassCRUD;

use super::api_models::{
    HoliLoginData, HoliLoginResponse, HoliMethods, HoliRequestData, HoliResponse, RequestForm,
};
use super::classes::parse_classes;
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
        holi_client.login().await?;
        Ok(holi_client)
    }

    pub async fn login(&mut self) -> Result<(), ClientError> {
        let response = self
            .client
            .post(self.base_url.clone())
            .form(&self.login_info.form())
            .send()
            .await?
            .map_error()?
            .json::<HoliLoginResponse>()
            .await?;
        self.base_form
            .insert("params[token]".to_owned(), response.token);
        Ok(())
    }
}

impl ClassCRUD for HoliClient {
    async fn list_user_classes(&self) -> Result<Vec<Class>, ClientError> {
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
            .send()
            .await?
            .map_error()?
            .json::<HoliResponse>()
            .await?;

        parse_classes(
            &Html::parse_document(&response.message),
            response.time_zone(&self.club_id)?,
        )
        .map_err(Into::into)
    }
}
