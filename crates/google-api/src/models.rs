use chrono::{DateTime, Duration, Utc};
use email_address::EmailAddress;

use crate::{
    api_models::{EventPatch, EventPost, EventResponse, TimePost},
    oauth2_client::TokenResponse,
};

pub type UtcDateTime = DateTime<Utc>;

#[derive(Debug, Clone)]
pub struct AccessToken {
    pub token: String,
    pub expires_at: UtcDateTime,
}

impl From<TokenResponse> for AccessToken {
    fn from(value: TokenResponse) -> Self {
        let now = chrono::offset::Utc::now();
        AccessToken {
            token: value.access_token,
            expires_at: now + Duration::seconds(value.expires_in as i64),
        }
    }
}

impl AccessToken {
    pub fn is_valid(&self) -> bool {
        let now = chrono::offset::Utc::now();
        now < self.expires_at
    }
}

#[derive(Debug, Clone)]
pub struct GoogleEvent {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub start: Option<UtcDateTime>,
    pub end: Option<UtcDateTime>,
    pub creator_email: EmailAddress,
}

impl From<EventResponse> for GoogleEvent {
    fn from(value: EventResponse) -> Self {
        GoogleEvent {
            id: value.id,
            summary: value.summary,
            description: value.description,
            start: value.start.date_time,
            end: value.end.date_time,
            creator_email: value.creator.email,
        }
    }
}

impl GoogleEvent {
    pub fn summary(&self) -> String {
        self.summary.clone().unwrap_or("\"\"".to_owned())
    }
    pub fn start(&self) -> String {
        self.start.map(|s| s.to_string()).unwrap_or("-".to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct GoogleEventListParams {
    pub search_param: Option<String>,
    pub start: Option<UtcDateTime>,
    pub end: Option<UtcDateTime>,
    pub creator_email: Option<EmailAddress>,
}

#[derive(Debug, Clone)]
pub struct GoogleEventPost {
    pub summary: String,
    pub description: Option<String>,
    pub start: UtcDateTime,
    pub end: UtcDateTime,
}

impl From<&GoogleEventPost> for EventPost {
    fn from(value: &GoogleEventPost) -> Self {
        EventPost {
            summary: value.summary.clone(),
            description: value.description.clone(),
            start: TimePost {
                date_time: value.start,
            },
            end: TimePost {
                date_time: value.end,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct GoogleEventPatch {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub start: Option<UtcDateTime>,
    pub end: Option<UtcDateTime>,
}

impl From<&GoogleEventPatch> for EventPatch {
    fn from(value: &GoogleEventPatch) -> Self {
        EventPatch {
            summary: value.summary.clone(),
            description: value.description.clone(),
            start: value.start.map(|date_time| TimePost { date_time }),
            end: value.end.map(|date_time| TimePost { date_time }),
        }
    }
}
