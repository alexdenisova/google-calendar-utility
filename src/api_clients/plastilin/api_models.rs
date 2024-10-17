use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::api_clients::models::{Class, UtcDateTime};

#[derive(Debug, Clone, Deserialize)]
pub struct UserClassListResponse {
    pub schedules: Vec<ScheduleResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleResponse {
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
    pub trainers: Vec<TrainerResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityResponse {
    pub id: u64,
    #[serde(rename = "title")]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrainerResponse {
    pub id: String,
    #[serde(rename = "title")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostUserClass {
    #[serde(rename = "scheduleId")]
    pub class_id: String,
    pub trainer_id: String,
}

impl From<ClassResponse> for Class {
    fn from(value: ClassResponse) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.activity.name.clone(),
            instructor: {
                if let Some(trainer) = value.trainers.first() {
                    trainer.id.clone()
                } else {
                    String::new()
                }
            },
            start: value.start_datetime,
            end: value.start_datetime + Duration::minutes(i64::from(value.duration_min)),
        }
    }
}

impl From<&Class> for PostUserClass {
    fn from(value: &Class) -> Self {
        PostUserClass {
            class_id: value.id.clone(),
            trainer_id: value.instructor.clone(),
        }
    }
}
