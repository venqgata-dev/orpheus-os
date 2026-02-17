mod logger;
mod config;
mod core;

use log::info;

fn main() {
    logger::init();

    info!("Orpheus OS Agent booting...");

    let cfg = config::load_config("config.yaml");

    core::run(&cfg);
}
