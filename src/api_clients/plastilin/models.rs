use chrono::Duration;

use crate::api_clients::models::Class;

use super::api_models::ClassResponse;

impl From<&ClassResponse> for Class {
    fn from(value: &ClassResponse) -> Self {
        Self {
            name: value.activity.name.clone(),
            start: value.start_datetime,
            end: value.start_datetime + Duration::minutes(value.duration_min as i64),
        }
    }
}
