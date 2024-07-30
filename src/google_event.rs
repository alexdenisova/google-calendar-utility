use google_api::models::GoogleEventPost;
use holi_yoga_api::models::Class as HoliClass;

pub trait IntoGoogleEvent {
    fn to_google_post(&self) -> GoogleEventPost;
}

impl IntoGoogleEvent for HoliClass {
    fn to_google_post(&self) -> GoogleEventPost {
        GoogleEventPost {
            summary: self.name.to_owned(),
            description: None,
            start: self.start,
            end: self.end,
        }
    }
}
