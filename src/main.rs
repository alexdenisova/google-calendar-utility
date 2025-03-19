#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
mod api_clients;
mod models;
mod settings;

use chrono::{Duration, Utc};
use chrono_tz::Tz;
use clap::Parser;
use color_eyre::{eyre::OptionExt, Result as AnyResult};
use dotenvy::dotenv;
use email_address::EmailAddress;
use futures::future::join_all;
use models::PotentialClass;
use tokio::task;

use api_clients::{models::Class, ClassCRUD};
use google_api::{
    models::{GoogleEvent, GoogleEventListParams},
    GoogleClient,
};
use settings::{Cli, Commands};

#[tokio::main]
async fn main() -> AnyResult<()> {
    dotenv().ok();
    let cli = Cli::parse();
    cli.setup_logging()?;

    let holi_client = cli.holi_client().await?;
    let plastilin_client = cli.plastilin_client()?;

    match cli.command {
        Commands::SignUp(config) => {
            let mut tasks = Vec::new();
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
            join_all(tasks).await;
        }
        Commands::SyncCalendars(google_args) => {
            let mut google_client = google_args.client().await?;
            let now = Utc::now();
            let mut google_classes = google_client
                .list_events(&GoogleEventListParams {
                    search_param: None,
                    start: Some(now),
                    end: Some(now + Duration::weeks(3)),
                    creator_email: Some(google_args.sa_email.clone()),
                })
                .await?;
            if let Some(client) = holi_client {
                sync_google_calendar(
                    &mut google_client,
                    &google_args.sa_email,
                    client,
                    &mut google_classes,
                )
                .await;
            }
            if let Some(client) = plastilin_client {
                sync_google_calendar(
                    &mut google_client,
                    &google_args.sa_email,
                    client,
                    &mut google_classes,
                )
                .await;
            }
            for event in google_classes {
                if let Err(e) = google_client.delete_event(&event.id).await {
                    log::error!(
                        "Could not delete Google event {} at {}: {}",
                        event.summary(),
                        event.start(),
                        e
                    );
                } else {
                    log::info!(
                        "Deleted Google event {} at {}",
                        event.summary(),
                        event.start(),
                    );
                }
            }
        }
    }
    Ok(())
}

async fn sign_up<T>(client: T, classes: Vec<PotentialClass>, timezone: Tz)
where
    T: ClassCRUD,
{
    for potential_class in classes {
        match client.list_day_classes(&potential_class.start).await {
            Ok(classes) => {
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
            }
            Err(err) => {
                log::error!(
                    "Could not get {} classes for {}: {}",
                    T::name(),
                    potential_class.start,
                    err
                );
            }
        }
    }
}

async fn sync_google_calendar<T>(
    google_client: &mut GoogleClient,
    creator_email: &EmailAddress,
    client: T,
    google_classes: &mut Vec<GoogleEvent>,
) where
    T: ClassCRUD,
{
    if let Ok(classes) = client.get_user_classes().await {
        *google_classes = google_classes
            .iter()
            .filter(|&g| !classes.iter().any(|h| h == g))
            .cloned()
            .collect();
        let new_classes: Vec<&Class> = classes
            .iter()
            .filter(|&h| !google_classes.iter().any(|g| h == g))
            .collect();
        for class in new_classes {
            if let Ok(event_matches) = google_client
                .list_events(&class.to_google_list_params(creator_email))
                .await
            {
                if event_matches.is_empty() {
                    if let Ok(response) = google_client.create_event(&class.to_google_post()).await
                    {
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
    } else {
        log::error!("Could not get {} user classes", T::name(),);
    }
}
