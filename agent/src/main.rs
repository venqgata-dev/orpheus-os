mod identity;
mod core;

use log::info;

fn main() {
    env_logger::init();

    let node_name = "orpheus-node-1".to_string();
    let version = "0.1.0".to_string();
    let interval = 5;

    core::run(node_name, version, interval);
}
