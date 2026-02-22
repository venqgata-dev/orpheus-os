mod identity;
mod core;

use std::time::SystemTime;
use identity::NodeKeys;
use core::{AgentIdentity, AgentConfig};

fn main() {
    env_logger::init();

    // Load or generate persistent keys
    let keys: NodeKeys = identity::load_or_generate();

    println!(
        "Node public key: {:?}",
        keys.verifying_key
    );

    let identity = AgentIdentity {
        node_name: "orpheus-node-1".to_string(),
        version: "0.1.0".to_string(),
        started_at: SystemTime::now(),
    };

    let cfg = AgentConfig {
        interval_seconds: 5,
    };

    core::run(identity, cfg);
}
