use crate::Error;

pub trait MetricsCollector {
    fn metrics(&mut self) -> Result<Metrics, Error>;
}

pub struct Metrics {
    pub players_online_count: u16,
}

impl Metrics {
    pub fn to_prometheus(&self) -> String {
        format!(
            "# TYPE players_online gauge\n\
            players_online {:?}",
            self.players_online_count
        )
    }
}
