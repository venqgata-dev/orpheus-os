use std::thread;
use std::time::{Duration, SystemTime};

use log::info;

pub struct AgentIdentity {
    pub node_name: String,
    pub version: String,
    pub started_at: SystemTime,
}

pub struct AgentConfig {
    pub interval_seconds: u64,
}

pub fn run(identity: AgentIdentity, cfg: AgentConfig) {
    info!("Orpheus OS Agent booting...");

    loop {
        let uptime = identity
            .started_at
            .elapsed()
            .unwrap()
            .as_secs();

        info!(
            "Heartbeat – node: {}, version: {}, uptime: {}s, interval: {}s",
            identity.node_name,
            identity.version,
            uptime,
            cfg.interval_seconds
        );

        thread::sleep(Duration::from_secs(cfg.interval_seconds));
    }
}
