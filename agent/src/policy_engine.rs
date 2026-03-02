use crate::config::PolicyConfig;

pub struct PolicyEngine {
    policy: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(policy: PolicyConfig) -> Self {
        Self { policy }
    }

    pub fn validate_environment(&self, env: &str) -> Result<(), String> {
        if self.policy.allow_environments.contains(&env.to_string()) {
            Ok(())
        } else {
            Err(format!("Environment '{}' is not allowed", env))
        }
    }

    pub fn validate_payload_size(&self, size: usize) -> Result<(), String> {
        if size <= self.policy.max_payload_size {
            Ok(())
        } else {
            Err(format!(
                "Payload size {} exceeds maximum allowed {}",
                size, self.policy.max_payload_size
            ))
        }
    }
}
