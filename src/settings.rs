use std::sync::Arc;

use crate::api_clients::errors::ClientError;
use crate::api_clients::holi_yoga::holi_client::HoliClient;
use crate::api_clients::plastilin::plastilin_client::PlastilinClient;
use crate::GC;
use camino::Utf8PathBuf;
use chrono::prelude::Local;
use clap::{Args, Parser};
use color_eyre::Result as AnyResult;
use email_address::EmailAddress;
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;
use google_api::errors::GoogleClientError;
use google_api::events_client::GoogleEventsClient;
use google_api::jwt::JsonWebToken;
use google_api::GoogleClient;
use log::LevelFilter;
use tokio::sync::Mutex;
use uuid::Uuid;

const DEFAULT_HOLI_API_KEY: &str = "63b92ce0-3a63-4de5-8ee0-2756b62a0190";
const DEFAULT_HOLI_CLUB_ID: &str = "3dc77e1c-434c-11ea-bbc1-0050568bac14";

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(flatten)]
    pub google: GoogleArguments,
    #[command(flatten)]
    plastilin: PlastilinArguments,
    #[command(flatten)]
    holi_yoga: HoliYogaArguments,
    /// Set debug log level
    #[arg(long, short = 'd', default_value = "false", env = "GCU__DEBUG")]
    debug: bool,
}

#[derive(Debug, Args)]
pub struct GoogleArguments {
    /// Email address of service account
    #[arg(long = "email", env = "GCU__GOOGLE_EMAIL")]
    pub sa_email: EmailAddress,
    /// Service account private key id
    #[arg(long = "kid", env = "GCU__GOOGLE_KEY_ID")]
    pub key_id: String,
    /// Path to service account private key
    #[arg(long = "private-key", env = "GCU__GOOGLE_PRIVATE_KEY")]
    pub private_key: Utf8PathBuf,
    /// Google calendar id (usually your email address)
    #[arg(long = "calendar-id", env = "GCU__GOOGLE_CALENDAR_ID")]
    pub calendar_id: String,
}

#[derive(Debug, Args)]
pub struct HoliYogaArguments {
    /// Holi Yoga username (phone number like 79123456789)
    #[arg(id = "holi-user", env = "GCU__HOLI_USERNAME")]
    username: String,
    /// Holi Yoga password
    #[arg(id = "holi-password", env = "GCU__HOLI_PASSWORD")]
    password: String,
    /// Holi Yoga api key (api_key in request forms)
    #[arg(
        id = "holi-api-key",
        env = "GCU__HOLI_API_KEY",
        default_value = DEFAULT_HOLI_API_KEY,
    )]
    api_key: Uuid,
    /// Holi Yoga club id
    #[arg(
        id = "holi-club-id",
        env = "GCU__HOLI_CLUB_ID",
        default_value = DEFAULT_HOLI_CLUB_ID,
    )]
    club_id: Uuid,
}

#[derive(Debug, Args)]
pub struct PlastilinArguments {
    /// Token for authorization
    #[arg(id = "plastilin-token", env = "GCU__PLASTILIN_TOKEN")]
    token: String,
}

impl PlastilinArguments {
    pub async fn client(&self) -> Result<PlastilinClient, ClientError> {
        PlastilinClient::new(&self.token).await
    }
}

impl HoliYogaArguments {
    pub async fn client(&self) -> Result<HoliClient, ClientError> {
        HoliClient::new(
            self.api_key,
            self.club_id,
            self.username.clone(),
            self.password.clone(),
        )
        .await
    }
}

impl GoogleArguments {
    pub async fn client(&self) -> Result<GoogleClient, GoogleClientError> {
        let private_key: String =
            std::fs::read_to_string(&self.private_key).expect("Unable to read file");
        let jwt = JsonWebToken::build(self.key_id.clone(), self.sa_email.clone(), private_key)?;
        let events_client = GoogleEventsClient::new(&self.calendar_id)?;
        GoogleClient::new(events_client, jwt).await
    }
}

impl Cli {
    pub fn setup_logging(&self) -> AnyResult<()> {
        let colors = ColoredLevelConfig::new()
            .debug(Color::BrightBlack)
            .info(Color::BrightGreen)
            .warn(Color::BrightYellow)
            .error(Color::BrightRed);
        if self.debug {
            Dispatch::new()
                .level(LevelFilter::Debug)
                .level_for("html5ever", LevelFilter::Off)
                .level_for("selectors", LevelFilter::Off)
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "{} [{}] {}: {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S MSK"),
                        colors.color(record.level()),
                        record.target(),
                        message,
                    ));
                })
        } else {
            Dispatch::new()
                .level(LevelFilter::Info)
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "{} [{}] {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S MSK"),
                        colors.color(record.level()),
                        message,
                    ));
                })
        }
        .chain(std::io::stderr())
        .apply()?;
        Ok(())
    }
    pub async fn google_client(&self) -> Result<GC, GoogleClientError> {
        Ok(Arc::new(Mutex::new(self.google.client().await?)))
    }
    pub async fn holi_client(&self) -> Result<HoliClient, ClientError> {
        self.holi_yoga.client().await
    }
    pub async fn plastilin_client(&self) -> Result<PlastilinClient, ClientError> {
        self.plastilin.client().await
    }
}
