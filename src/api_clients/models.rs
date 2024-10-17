use std::fmt::Display;

use chrono::{DateTime, Local, Utc};
use email_address::EmailAddress;
use google_api::models::{GoogleEventListParams, GoogleEventPost};

pub type UtcDateTime = DateTime<Utc>;

#[derive(Debug, Clone)]
pub struct Class {
    pub id: String,
    pub name: String,
    pub instructor: String,
    pub start: UtcDateTime,
    pub end: UtcDateTime,
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.instructor == other.instructor
            && self.start == other.start
            && self.end == other.end
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}",
            self.name,
            self.start.with_timezone(&Local).format("%d.%m.%Y %H:%M")
        )
    }
}

impl Class {
    pub fn to_google_post(&self) -> GoogleEventPost {
        GoogleEventPost {
            summary: self.name.clone(),
            description: None,
            start: self.start,
            end: self.end,
        }
    }

    pub fn to_google_list_params(&self, creator_email: &EmailAddress) -> GoogleEventListParams {
        GoogleEventListParams {
            search_param: Some(self.name.clone()),
            start: Some(self.start),
            end: Some(self.end),
            creator_email: Some(creator_email.to_owned()),
        }
    }
}
