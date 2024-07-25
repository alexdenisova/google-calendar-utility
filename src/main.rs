use std::str::FromStr;

use color_eyre::Result as AnyResult;
use email_address::EmailAddress;
use google_api::{
    api_models::{EventPost, TimePost},
    events_client::EventsClient,
    jwt::JsonWebToken,
    GoogleAPI,
};

fn main() -> AnyResult<()> {
    let kid = "".to_owned();
    let private_key = r"".to_owned();

    let jwt = JsonWebToken::build(
        kid.clone(),
        EmailAddress::from_str("serviceaccount@gmail.com")?,
        private_key.clone(),
    )?;
    let events_client = EventsClient::new("alexadenisova@gmail.com")?;
    let mut google_api = GoogleAPI::new(events_client, jwt)?;
    let now = chrono::offset::Utc::now();
    let event = EventPost {
        summary: "testing".to_owned(),
        description: None,
        start: TimePost { date_time: now },
        end: TimePost { date_time: now },
    };
    let response = google_api.create_event(&event)?;
    google_api.delete_event(&response.id)?;
    println!("{:?}", response);
    Ok(())
}
