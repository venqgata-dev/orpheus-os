use serde::Serialize;
use tokio::sync::broadcast;

use crate::{config::AgentConfig, identity::NodeIdentity};

#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub node_id: String,
    pub executed: bool,
    pub message: String,
}

pub struct AppState {
    pub identity: NodeIdentity,
    pub config: AgentConfig,
    pub shutdown: broadcast::Sender<()>,
}
