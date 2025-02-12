mod collectors;
// TODO: Have Factorio return its own error and don't import this here
use rcon_client::RCONError;

use clap::{Parser, Subcommand};
use thiserror::Error;
use tiny_http::{Response, Server};

#[derive(Error, Debug)]
enum Error {
    #[error("Error constructing RCON client")]
    RconClientError(#[from] RCONError),

    #[error("Error parsing or formatting metrics: {0}")]
    MetricsCollectError(String),
}

/*
    Top-level CLI argument parsing is handled here, but the specific arguments
    for each game are found in their respective files under collectors/.
*/

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Port to expose Prometheus metrics on
    #[arg(short, long, default_value = "8080")]
    metrics_port: u16,

    /// The game being targeted for metrics collection
    #[command(subcommand)]
    command: Game,
}

#[derive(Subcommand)]
enum Game {
    /// Collect metrics from Factorio
    Factorio(collectors::FactorioOpts),

    /// Collect metrics from Don't Starve Together
    DontStarveTogether(collectors::DontStarveTogetherOpts),
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let mut collector: Box<dyn collectors::MetricsCollector> = match cli.command {
        Game::Factorio(opts) => Box::new(collectors::FactorioCollector::new(opts)?),
        _ => todo!(),
    };

    let bind = format!("0.0.0.0:{}", cli.metrics_port);

    let server = Server::http(bind).unwrap();

    for request in server.incoming_requests() {
        println!(
            "received request! method: {:?}, url: {:?}, headers: {:?}",
            request.method(),
            request.url(),
            request.headers()
        );

        let metrics = collector.metrics()?.to_prometheus();
        let response = Response::from_string(metrics);
        let _ = request.respond(response);
    }

    Ok(())
}
