use crate::collectors::{Metrics, MetricsCollector};
use crate::Error;
use clap::Parser;
use rcon_client::{AuthRequest, AuthResponse, RCONClient, RCONConfig, RCONRequest};

#[derive(Parser)]
pub struct FactorioOpts {
    /// Port to be used to connect to gameserver via RCON
    #[arg(long, default_value = "27015")]
    rcon_port: u16,

    /// Password used for RCON connection. Consider providing this argument via environment
    /// variable.
    #[arg(long)]
    rcon_password: String,
}

pub struct FactorioCollector {
    client: RCONClient,
}

impl FactorioCollector {
    pub fn new(opts: FactorioOpts) -> Result<Self, Error> {
        let retry_seconds = std::time::Duration::from_secs(5);

        let client: RCONClient;

        loop {
            let mut attempted_client = match RCONClient::new(RCONConfig {
                url: format!("localhost:{}", opts.rcon_port),
                read_timeout: Some(13),
                write_timeout: Some(37),
            }) {
                Ok(c) => c,
                Err(e) => {
                    println!(
                        "Failed to create RCON client: {}, retrying in {:?} seconds...",
                        e, retry_seconds
                    );
                    std::thread::sleep(retry_seconds);
                    continue;
                }
            };

            match attempted_client.auth(AuthRequest::new(opts.rcon_password.clone())) {
                Ok(result) => {
                    if result.is_success() {
                        client = attempted_client;
                        break;
                    }
                    println!("Factorio RCON authentication failed -- is the password correct?");
                }
                Err(e) => {
                    println!(
                        "Couldn't attempt RCON auth to server: {}, retrying in {:?} seconds...",
                        e, retry_seconds
                    );
                    std::thread::sleep(retry_seconds);
                }
            };
        }

        println!("RCON successfully connected!");

        Ok(FactorioCollector { client })
    }
}

impl MetricsCollector for FactorioCollector {
    fn metrics(&mut self) -> Result<Metrics, Error> {
        let response = self
            .client
            .execute(RCONRequest::new("/players online count".to_string()))?
            .body;

        // Response returned looks like 'Online player (0):', so parse out that int
        let count = &response
            .split('(')
            .nth(1)
            .and_then(|s| s.split(')').next())
            .and_then(|s| s.parse::<u16>().ok());

        match count {
            Some(count) => Ok(Metrics {
                players_online_count: *count,
            }),
            None => Err(Error::MetricsCollectError(format!(
                "Failed to parse any Factorio player count from response: {}",
                response,
            ))),
        }
    }
}
