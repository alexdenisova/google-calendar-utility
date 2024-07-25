use email_address::EmailAddress;
use serde::{Deserialize, Serialize};

use crate::models::UtcDateTime;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventListResponse {
    pub next_page_token: Option<String>,
    pub items: Vec<EventResponse>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventResponse {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub start: TimeResponse,
    pub end: TimeResponse,
    pub creator: CreatorResponse,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeResponse {
    pub date_time: Option<UtcDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatorResponse {
    pub email: EmailAddress,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventPost {
    pub summary: String,
    pub description: Option<String>,
    pub start: TimePost,
    pub end: TimePost,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<TimePost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<TimePost>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimePost {
    pub date_time: UtcDateTime,
}
