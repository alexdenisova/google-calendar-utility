use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Tz;
use convert_case::{Case, Casing};
use serde::Deserialize;

use crate::api_clients::models::Class;

pub type UtcDateTime = DateTime<Utc>;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignUpConfig {
    #[serde(default = "default_tz")]
    pub timezone: Tz,
    #[serde(default)]
    pub offset_weeks: u8, // sign up to classes that will be in <offset_weeks> weeks
    #[serde(default)]
    classes: HashMap<String, Vec<WeeklyClass>>, // map of <Studio, List of Classes>
}

fn default_tz() -> Tz {
    Tz::Europe__Moscow
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyClass {
    pub name: String,
    pub weekday: Weekday,
    pub start_time: NaiveTime,
}

#[derive(Debug, Clone)]
pub struct PotentialClass {
    pub name: String,
    pub start: UtcDateTime,
}

impl PotentialClass {
    pub fn eq(&self, class: &Class) -> bool {
        self.name.to_uppercase() == class.name.to_uppercase() && self.start == class.start
    }
}

impl SignUpConfig {
    pub fn classes(&self, studio: &str) -> Vec<PotentialClass> {
        let mut result = Vec::new();
        if let Some(classes) = self.classes.get(&studio.to_case(Case::Camel)) {
            result = classes
                .iter()
                .map(|class| {
                    let today = Local::now();
                    let classdate = (today
                        + Duration::days(
                            (7 * u32::from(self.offset_weeks) + 6
                                - today.weekday().pred().days_since(class.weekday))
                            .into(),
                        ))
                    .date_naive();
                    let classdatetime: DateTime<Utc> = self
                        .timezone
                        .from_local_datetime(&classdate.and_time(class.start_time))
                        .unwrap()
                        .with_timezone(&Utc);
                    PotentialClass {
                        name: class.name.clone(),
                        start: classdatetime,
                    }
                })
                .collect()
        }
        return result;
    }
}
