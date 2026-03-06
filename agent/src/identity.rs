use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use std::fs;
use std::path::Path;

pub struct NodeIdentity {
    signing_key: SigningKey,
    node_id: String,
}

impl NodeIdentity {

    pub fn id(&self) -> String {
        self.node_id.clone()
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

}

pub fn load_or_create_identity() -> Result<NodeIdentity, Box<dyn std::error::Error>> {

    // Read PORT so each node has its own identity
    let port = std::env::var("PORT").unwrap_or("8080".to_string());

    let key_path = format!("src/node_key_{}", port);

    let signing_key: SigningKey;

    if Path::new(&key_path).exists() {

        let bytes = fs::read(&key_path)?;

        // Convert Vec<u8> to [u8; 32]
        let key_bytes: [u8; 32] = bytes
            .try_into()
            .expect("invalid key length");

        signing_key = SigningKey::from_bytes(&key_bytes);

    } else {

        let mut csprng = OsRng;

        signing_key = SigningKey::generate(&mut csprng);

        fs::write(&key_path, signing_key.to_bytes())?;

    }

    // Create node ID from public key hash
    let public_key = signing_key.verifying_key().to_bytes();

    let mut hasher = Sha256::new();

    hasher.update(public_key);

    let result = hasher.finalize();

    let node_id = hex::encode(result)[0..64].to_string();

    Ok(NodeIdentity {
        signing_key,
        node_id,
    })
}
