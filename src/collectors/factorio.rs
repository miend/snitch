use crate::collectors::{Metrics, MetricsCollector};
use crate::Error;
use clap::Parser;
use rcon_client::{AuthRequest, RCONClient, RCONConfig, RCONRequest};

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
        let mut client = RCONClient::new(RCONConfig {
            url: format!("localhost:{}", opts.rcon_port),
            read_timeout: Some(13),
            write_timeout: Some(37),
        })?;

        let auth_result = client.auth(AuthRequest::new(opts.rcon_password))?;
        assert!(auth_result.is_success());

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
