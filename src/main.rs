#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
mod api_clients;
mod models;
mod settings;

use std::sync::Arc;

use chrono_tz::Tz;
use clap::Parser;
use color_eyre::{eyre::OptionExt, Result as AnyResult};
use dotenvy::dotenv;
use email_address::EmailAddress;
use futures::future::join_all;
use models::PotentialClass;
use tokio::{sync::Mutex, task};

use api_clients::{models::Class, ClassCRUD};
use google_api::GoogleClient;
use settings::{Cli, Commands};

type GC = Arc<Mutex<GoogleClient>>;

#[tokio::main]
async fn main() -> AnyResult<()> {
    dotenv().ok();
    let cli = Cli::parse();
    cli.setup_logging()?;

    let holi_client = cli.holi_client().await?;
    let plastilin_client = cli.plastilin_client()?;

    let mut tasks = Vec::new();
    match cli.command {
        Commands::SignUp(config) => {
            let config = config.parse()?;
            let holi_classes = config.holi_classes();
            let plastilin_classes = config.plastilin_classes();
            if !holi_classes.is_empty() {
                let holi_client = holi_client.ok_or_eyre("Holi Yoga args missing.")?;
                tasks.push(task::spawn(sign_up(
                    holi_client,
                    holi_classes,
                    config.timezone,
                )));
            }
            if !plastilin_classes.is_empty() {
                let plastilin_client = plastilin_client.ok_or_eyre("Holi Yoga args missing.")?;
                tasks.push(task::spawn(sign_up(
                    plastilin_client,
                    plastilin_classes,
                    config.timezone,
                )));
            }
        }
        Commands::SyncCalendars(google_args) => {
            let google_client = google_args.client().await?;
            if let Some(client) = holi_client {
                let holi_classes = client.get_user_classes().await?;
                tasks.push(task::spawn(sync_google_calendar(
                    google_client.clone(),
                    google_args.sa_email.clone(),
                    holi_classes,
                )));
            }
            if let Some(client) = plastilin_client {
                let plastilin_classes = client.get_user_classes().await?;
                tasks.push(task::spawn(sync_google_calendar(
                    google_client.clone(),
                    google_args.sa_email.clone(),
                    plastilin_classes,
                )));
            }
        }
    }

    join_all(tasks).await;
    Ok(())
}

async fn sign_up<T>(client: T, classes: Vec<PotentialClass>, timezone: Tz)
where
    T: ClassCRUD,
{
    for potential_class in classes {
        if let Ok(classes) = client.list_day_classes(&potential_class.start).await {
            if let Some(class) = classes.iter().find(|x| potential_class.eq(x)) {
                if let Err(e) = client.sign_up_for_class(class).await {
                    log::error!(
                        "Could not sign up for {} class {} at {}: {}",
                        T::name(),
                        class.name,
                        class.start.with_timezone(&timezone),
                        e
                    );
                }
            } else {
                log::error!(
                    "Could not find {} class {} at {}",
                    T::name(),
                    potential_class.name,
                    potential_class.start.with_timezone(&timezone)
                );
            }
        } else {
            log::error!(
                "Could not get {} classes for {}",
                T::name(),
                potential_class.start
            );
        }
    }
}

async fn sync_google_calendar(google_client: GC, sa_email: EmailAddress, classes: Vec<Class>) {
    let mut google_client = google_client.lock().await;
    for class in classes {
        if let Ok(event_matches) = google_client
            .list_events(class.to_google_list_params(&sa_email))
            .await
        {
            if event_matches.is_empty() {
                if let Ok(response) = google_client.create_event(&class.to_google_post()).await {
                    log::info!(
                        "Added {} at {} to calendar",
                        response.summary.unwrap(),
                        response.start.unwrap()
                    );
                } else {
                    log::error!(
                        "Could not create Google event {} at {}",
                        class.name,
                        class.start
                    );
                }
            } else {
                log::debug!("{} already in calendar", class);
            }
        } else {
            log::error!("Could not list Google events");
        }
    }
}
