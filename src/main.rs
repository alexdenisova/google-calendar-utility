mod google_event;
mod settings;

use clap::Parser;
use color_eyre::Result as AnyResult;
use dotenvy::dotenv;
use google_event::IntoGoogleEvent;
use settings::Cli;

fn main() -> AnyResult<()> {
    dotenv().ok();
    let cli = Cli::parse();
    cli.setup_logging()?;

    let mut google_client = cli.google_client()?;
    let holi_client = cli.holi_client()?;

    let classes = holi_client.list_user_classes()?;
    for class in classes {
        let event_matches =
            google_client.list_events(class.to_google_list_params(&cli.google.sa_email))?;
        if event_matches.is_empty() {
            let response = google_client.create_event(&class.to_google_post())?;
            log::info!(
                "Added {} at {} to calendar",
                response.summary.unwrap(),
                response.start.unwrap()
            );
        } else {
            log::debug!("{} at {} already in calendar", class.name, class.start);
        }
    }
    Ok(())
}
