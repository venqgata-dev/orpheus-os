use serde::Deserialize;
use std::fs;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    pub node_name: String,
    pub environment: String,
    pub log_level: String,
}

pub fn load_config() -> Result<AgentConfig, Box<dyn Error>> {
    let contents = fs::read_to_string("config.yaml")?;
    let config: AgentConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}
