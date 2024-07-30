use email_address::EmailAddress;
use google_api::models::{GoogleEventListParams, GoogleEventPost};
use holi_yoga_api::models::Class as HoliClass;

pub trait IntoGoogleEvent {
    fn to_google_post(&self) -> GoogleEventPost;
    fn to_google_list_params(&self, creator_email: &EmailAddress) -> GoogleEventListParams;
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

    fn to_google_list_params(&self, creator_email: &EmailAddress) -> GoogleEventListParams {
        GoogleEventListParams {
            search_param: Some(self.name.to_owned()),
            start: Some(self.start),
            end: Some(self.end),
            creator_email: Some(creator_email.to_owned()),
        }
    }
}
