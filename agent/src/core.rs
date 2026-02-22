use crate::identity::NodeKeys;
use ed25519_dalek::Signer;
use log::info;
use std::{thread, time::Duration};

pub fn run(node_name: String, version: String, interval: u64) {
    let keys: NodeKeys = crate::identity::load_or_generate();

    info!("Orpheus OS Agent booting...");

    loop {
        let payload = format!(
            "node={},version={}",
            node_name,
            version
        );

        let signature = keys.signing_key.sign(payload.as_bytes());

        info!(
            "Heartbeat | payload: {} | signature: {}",
            payload,
            hex::encode(signature.to_bytes())
        );

        thread::sleep(Duration::from_secs(interval));
    }
}
