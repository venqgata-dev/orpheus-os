use log::info;
use std::thread;
use std::time::Duration;

use crate::config::Config;

pub fn run(cfg: &Config) {
    info!("Runtime core initialized for node: {}", cfg.node_name);

    loop {
        info!(
            "Agent heartbeat — node: {}, interval: {} seconds",
            cfg.node_name,
            cfg.interval_seconds
        );

        thread::sleep(Duration::from_secs(cfg.interval_seconds));
    }
}
