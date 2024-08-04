#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
mod api_clients;
mod settings;

use std::sync::Arc;

use clap::Parser;
use color_eyre::Result as AnyResult;
use dotenvy::dotenv;
use email_address::EmailAddress;
use futures::future::join_all;
use tokio::{sync::Mutex, task};

use api_clients::ClassCRUD;
use google_api::GoogleClient;
use settings::Cli;

type GC = Arc<Mutex<GoogleClient>>;

#[tokio::main]
async fn main() -> AnyResult<()> {
    dotenv().ok();
    let cli = Cli::parse();
    cli.setup_logging()?;

    let google_client = cli.google_client().await?;
    let holi_client = cli.holi_client().await?;
    let plastilin_client = cli.plastilin_client()?;

    let tasks = vec![
        task::spawn(sync_google_calendar(
            google_client.clone(),
            cli.google.sa_email.clone(),
            holi_client,
        )),
        task::spawn(sync_google_calendar(
            google_client.clone(),
            cli.google.sa_email.clone(),
            plastilin_client,
        )),
    ];

    join_all(tasks).await;
    Ok(())
}

async fn sync_google_calendar<T>(
    google_client: GC,
    sa_email: EmailAddress,
    client: T,
) -> AnyResult<()>
where
    T: ClassCRUD,
{
    let mut google_client = google_client.lock().await;
    let classes = client.list_user_classes().await?;
    for class in classes {
        let event_matches = google_client
            .list_events(class.to_google_list_params(&sa_email))
            .await?;
        if event_matches.is_empty() {
            let response = google_client.create_event(&class.to_google_post()).await?;
            log::info!(
                "Added {} at {} to calendar",
                response.summary.unwrap(),
                response.start.unwrap()
            );
        } else {
            log::debug!("{} already in calendar", class);
        }
    }
    Ok(())
}
