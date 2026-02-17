use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub node_name: String,
    pub interval_seconds: u64,
}

pub fn load_config(path: &str) -> Config {
    let contents = fs::read_to_string(path)
        .expect("Failed to read config file");

    serde_yaml::from_str(&contents)
        .expect("Failed to parse YAML config")
}
