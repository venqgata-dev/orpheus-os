mod logger;
mod config;

use log::info;
use std::thread;
use std::time::Duration;

fn main() {
    logger::init();

    info!("Orpheus OS Agent starting...");

    let cfg = config::load_config("config.yaml");

    info!("Loaded config for node: {}", cfg.node_name);

    loop {
        info!("Agent heartbeat - interval {} seconds", cfg.interval_seconds);
        thread::sleep(Duration::from_secs(cfg.interval_seconds));
    }
}
