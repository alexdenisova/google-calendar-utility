use std::collections::HashMap;

use color_eyre::eyre::eyre;
use serde::Deserialize;
use uuid::Uuid;

use crate::api_clients::errors::{ClassParseError, ClientError, ToClientError};

#[derive(Debug)]
pub struct RequestForm(HashMap<String, String>);

impl RequestForm {
    pub fn new() -> Self {
        RequestForm(HashMap::new())
    }
    pub fn insert(&mut self, key: &str, value: &str) {
        let RequestForm(hashmap) = self;
        hashmap.insert(key.to_owned(), value.to_owned());
    }
    pub fn insert_param(&mut self, key: &str, value: &str) {
        let RequestForm(hashmap) = self;
        hashmap.insert(format!("params[{key}]"), value.to_owned());
    }
    pub fn extend(&mut self, form: &RequestForm) {
        let RequestForm(self_hashmap) = self;
        let hashmap = form.unwrap();
        self_hashmap.extend(hashmap.clone());
    }
    pub fn unwrap(&self) -> &HashMap<String, String> {
        let RequestForm(hashmap) = self;
        hashmap
    }
}

#[derive(Debug, Clone)]
pub struct HoliLoginData {
    pub username: String,
    pub password: String,
    pub api_key: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HoliLoginResponse {
    pub token: String,
}

impl HoliLoginData {
    pub fn form(&self) -> RequestForm {
        let mut form = RequestForm::new();
        form.insert("method", "auth");
        form.insert_param("login", &self.username);
        form.insert_param("pass", &self.password);
        form.insert_param("window_width", "500");
        form.insert("api_key", &self.api_key.to_string());
        form.insert("lang", "en");
        form
    }
}

#[derive(Debug, Clone)]
pub struct HoliRequestData {
    pub method: HoliMethods,
    pub app_id: Option<Uuid>,
}

impl HoliRequestData {
    pub fn form(&self) -> RequestForm {
        let mut form = RequestForm::new();
        form.insert("method", &self.method.to_string());
        if let Some(id) = self.app_id {
            form.insert_param("AppID", &id.to_string());
        }
        form.insert_param("window_width", "500");
        form.insert("lang", "en");
        form
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HoliScheduleResponse {
    #[serde(rename = "SLIDER")]
    pub slider: HoliSliderResponse,
    #[serde(rename = "arClub")]
    pub clubs: HashMap<Uuid, HoliClubInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HoliUserClassResponse {
    #[serde(rename = "isError")]
    pub is_error: bool,
    #[serde(rename = "Message")]
    pub result: String,
    #[serde(rename = "arClub")]
    pub clubs: HashMap<Uuid, HoliClubInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HoliClubInfo {
    pub id: Uuid,
    #[serde(rename = "title")]
    pub name: String,
    pub time_zone: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HoliSliderResponse {
    #[serde(rename = "BODY")]
    pub schedule_html: String,
}

#[derive(Debug, Clone)]
pub enum HoliMethods {
    GetUserClasses,
    GetSchedule,
    SignUp,
}

impl ToString for HoliMethods {
    fn to_string(&self) -> String {
        match self {
            HoliMethods::GetUserClasses => "getUserApp",
            HoliMethods::GetSchedule => "getFitCalendar",
            HoliMethods::SignUp => "setApp",
        }
        .to_owned()
    }
}

pub(crate) trait HoliResponseTrait {
    fn time_zone(&self, clud_id: &Uuid) -> Result<chrono_tz::Tz, ClassParseError<'static>> {
        let time_zone = self
            .clubs()
            .get(clud_id)
            .ok_or(ClassParseError::MissingJsonField {
                field: clud_id.to_string(),
            })?
            .time_zone
            .clone();
        time_zone
            .parse()
            .map_err(|_| ClassParseError::UnknownTimezone { time_zone })
    }
    fn clubs(&self) -> &HashMap<Uuid, HoliClubInfo>;
}

impl HoliResponseTrait for HoliUserClassResponse {
    fn clubs(&self) -> &HashMap<Uuid, HoliClubInfo> {
        &self.clubs
    }
}

impl HoliResponseTrait for HoliScheduleResponse {
    fn clubs(&self) -> &HashMap<Uuid, HoliClubInfo> {
        &self.clubs
    }
}

impl ToClientError for HoliUserClassResponse {
    fn map_error(self, id: Option<String>) -> Result<Self, ClientError>
    where
        Self: Sized,
    {
        if self.is_error {
            match self.result.as_str() {
                "Клиент уже записан на занятие" => {
                    return Err(ClientError::AlreadyExists { id: id.unwrap() })
                }
                _ => {
                    return Err(ClientError::Other {
                        error: eyre!("{}", self.result),
                    })
                }
            }
        }
        Ok(self)
    }
}
