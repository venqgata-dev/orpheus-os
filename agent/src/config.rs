use serde::Deserialize;
use std::{fs, error::Error};

#[derive(Debug, Deserialize, Clone)]
pub struct AgentConfig {
    pub node_name: String,
    pub environment: String,
    pub log_level: String,
    pub policy: PolicyConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PolicyConfig {
    pub allow_environments: Vec<String>,
    pub max_payload_size: usize,
}

pub fn load_config() -> Result<AgentConfig, Box<dyn Error>> {
    let contents = fs::read_to_string("config.yaml")?;
    let config: AgentConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}
