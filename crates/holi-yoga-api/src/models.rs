use std::fmt::Display;

use chrono::{DateTime, Utc};

pub type UtcDateTime = DateTime<Utc>;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub start: UtcDateTime,
    pub end: UtcDateTime,
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}",
            self.name,
            self.start.format("%d.%m.%Y %H:%M")
        )
    }
}
