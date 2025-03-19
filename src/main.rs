#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
mod api_clients;
mod models;
mod settings;

use chrono::{Duration, Utc};
use clap::Parser;
use color_eyre::Result as AnyResult;
use dotenvy::dotenv;
use email_address::EmailAddress;
use models::SignUpConfig;

use api_clients::{models::Class, StudioCRUD};
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

    let clients = cli.clients().await?;

    match cli.command {
        Commands::SignUp(config) => {
            let config = config.parse()?;
            for client in clients {
                sign_up(&*client, &config).await;
            }
        }
        Commands::SyncCalendars(google_args) => {
            let mut google_client = google_args.client().await?;
            sync_google_calendar(&mut google_client, google_args.sa_email, &clients).await?;
        }
    }
    Ok(())
}

async fn sign_up<T>(client: &T, config: &SignUpConfig)
where
    T: StudioCRUD + ?Sized,
{
    let classes = config.classes(&client.name());
    for potential_class in classes {
        match client.list_day_classes(&potential_class.start).await {
            Ok(classes) => {
                if let Some(class) = classes.iter().find(|x| potential_class.eq(x)) {
                    if let Err(e) = client.sign_up_for_class(class).await {
                        log::error!(
                            "Could not sign up for {} class {} at {}: {}",
                            client.name(),
                            class.name,
                            class.start.with_timezone(&config.timezone),
                            e
                        );
                    }
                } else {
                    log::error!(
                        "Could not find {} class {} at {}",
                        client.name(),
                        potential_class.name,
                        potential_class.start.with_timezone(&config.timezone)
                    );
                }
            }
            Err(err) => {
                log::error!(
                    "Could not get {} classes for {}: {}",
                    client.name(),
                    potential_class.start,
                    err
                );
            }
        }
    }
}

async fn sync_google_calendar(
    google_client: &mut GoogleClient,
    creator_email: EmailAddress,
    clients: &Vec<Box<dyn StudioCRUD + Send + Sync>>,
) -> AnyResult<()> {
    let now = Utc::now();
    let google_classes = google_client
        .list_events(&GoogleEventListParams {
            search_param: None,
            start: Some(now),
            end: Some(now + Duration::weeks(3)),
            creator_email: Some(creator_email),
        })
        .await?;
    let classes = get_all_classes(clients).await;
    let (to_delete, to_add) = get_class_status(&google_classes, classes);

    for class in to_add {
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
    }
    for event in to_delete {
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
    Ok(())
}

async fn get_all_classes(clients: &Vec<Box<dyn StudioCRUD + Send + Sync>>) -> Vec<Class> {
    let mut classes = Vec::new();
    for client in clients {
        if let Ok(mut value) = client.get_user_classes().await {
            if value.is_empty() {
                log::info!("Nothing to do for {}", client.name());
                continue;
            }
            classes.append(&mut value);
        } else {
            log::error!("Could not get {} user classes", client.name(),);
        }
    }
    return classes;
}

/// Seperate classes into sets of (google_classes-{studio_classes}, studio_classes-{google_classes})
/// google_classes-{studio_classes} are the classes that need to be deleted from Google Calendar
/// studio_classes-{google_classes} are the classes that need to be added to Google Calendar
fn get_class_status(
    google_classes: &Vec<GoogleEvent>,
    studio_classes: Vec<Class>,
) -> (Vec<GoogleEvent>, Vec<Class>) {
    let to_delete: Vec<GoogleEvent> = google_classes
        .iter()
        .filter(|&google| !studio_classes.iter().any(|studio| studio == google))
        .cloned()
        .collect();
    let to_add = studio_classes
        .into_iter()
        .filter(|studio| !google_classes.iter().any(|google| studio == google))
        .collect();
    return (to_delete, to_add);
}
