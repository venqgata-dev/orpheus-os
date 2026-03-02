use sha2::{Digest, Sha256};
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct LedgerEntry {
    timestamp: String,
    node_id: String,
    environment: String,
    prompt_hash: String,
    allowed: bool,
    prev_hash: String,
    entry_hash: String,
}

fn sha256_hex(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

fn get_last_hash() -> String {
    if let Ok(contents) = read_to_string("audit.log") {
        if let Some(last_line) = contents.lines().last() {
            if let Ok(entry) = serde_json::from_str::<LedgerEntry>(last_line) {
                return entry.entry_hash;
            }
        }
    }
    "GENESIS".to_string()
}

pub fn log_verification(
    node_id: &str,
    environment: &str,
    prompt: &str,
    allowed: bool,
) -> Result<(), Box<dyn Error>> {

    let prompt_hash = sha256_hex(prompt);
    let prev_hash = get_last_hash();

    let entry_core = format!(
        "{}{}{}{}{}",
        node_id, environment, prompt_hash, allowed, prev_hash
    );

    let entry_hash = sha256_hex(&entry_core);

    let entry = LedgerEntry {
        timestamp: Utc::now().to_rfc3339(),
        node_id: node_id.to_string(),
        environment: environment.to_string(),
        prompt_hash,
        allowed,
        prev_hash,
        entry_hash,
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("audit.log")?;

    writeln!(file, "{}", serde_json::to_string(&entry)?)?;

    Ok(())
}

pub fn verify_ledger() -> Result<bool, Box<dyn Error>> {

    let contents = match read_to_string("audit.log") {
        Ok(c) => c,
        Err(_) => return Ok(true), // empty ledger = valid
    };

    let mut previous_hash = "GENESIS".to_string();

    for line in contents.lines() {

        let entry: LedgerEntry = serde_json::from_str(line)?;

        if entry.prev_hash != previous_hash {
            return Ok(false);
        }

        let entry_core = format!(
            "{}{}{}{}{}",
            entry.node_id,
            entry.environment,
            entry.prompt_hash,
            entry.allowed,
            entry.prev_hash
        );

        let recalculated_hash = sha256_hex(&entry_core);

        if recalculated_hash != entry.entry_hash {
            return Ok(false);
        }

        previous_hash = entry.entry_hash;
    }

    Ok(true)
}
