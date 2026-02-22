mod identity;
mod config;

use std::error::Error;
use log::{info, warn};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    info!("Starting Orpheus Agent v0.1...");

    let node = identity::load_or_create_identity()?;
    info!("Node ID: {}", node.id());

    let config = config::load_config()?;
    info!("Node Name: {}", config.node_name);
    info!("Environment: {}", config.environment);
    info!("Log Level: {}", config.log_level);

    info!("Orpheus Agent ready.");

    Ok(())
}
