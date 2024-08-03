use std::collections::HashMap;

use serde::Deserialize;
use uuid::Uuid;

use crate::api_clients::errors::ClassParseError;

pub type RequestForm = HashMap<String, String>;

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
    pub fn form(&self) -> HashMap<String, String> {
        let mut form = HashMap::new();
        form.insert("method".to_owned(), "auth".to_owned());
        form.insert("params[login]".to_owned(), self.username.clone());
        form.insert("params[pass]".to_owned(), self.password.clone());
        form.insert("api_key".to_owned(), self.api_key.to_string());
        form.insert("lang".to_owned(), "en".to_owned());
        form
    }
}

#[derive(Debug, Clone)]
pub struct HoliRequestData {
    pub method: HoliMethods,
    pub app_id: Option<Uuid>,
}

impl HoliRequestData {
    pub fn form(&self) -> HashMap<String, String> {
        let mut form = HashMap::new();
        form.insert("method".to_owned(), self.method.to_string());
        if let Some(id) = self.app_id {
            form.insert("params[AppID]".to_owned(), id.to_string());
        }
        form.insert("lang".to_owned(), "en".to_owned());
        form
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HoliResponse {
    #[serde(rename = "Message")]
    pub message: String,
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

impl HoliResponse {
    pub fn time_zone(&self, clud_id: &Uuid) -> Result<chrono_tz::Tz, ClassParseError<'static>> {
        let time_zone = self
            .clubs
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
}

#[derive(Debug, Clone)]
pub enum HoliMethods {
    GetUserClasses,
}

impl ToString for HoliMethods {
    fn to_string(&self) -> String {
        match self {
            HoliMethods::GetUserClasses => "getUserApp".to_owned(),
        }
    }
}
