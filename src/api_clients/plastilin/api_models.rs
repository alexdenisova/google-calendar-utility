use serde::Deserialize;

use crate::api_clients::models::UtcDateTime;

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleListResponse {
    pub schedules: Vec<ClassListResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClassListResponse {
    pub schedule: Vec<ClassResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClassResponse {
    pub id: u64,
    #[serde(rename = "datetime")]
    pub start_datetime: UtcDateTime,
    #[serde(rename = "length")]
    pub duration_min: u8,
    pub activity: ActivityResponse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityResponse {
    pub id: u64,
    #[serde(rename = "title")]
    pub name: String,
}
