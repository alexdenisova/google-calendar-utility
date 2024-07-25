use chrono::{DateTime, Duration, Utc};
use email_address::EmailAddress;

use crate::{api_models::EventResponse, oauth2_client::TokenResponse};

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
