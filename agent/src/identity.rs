use ed25519_dalek::{SigningKey, VerifyingKey};
use rand_core::OsRng;
use std::{fs, error::Error};

pub struct NodeIdentity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl NodeIdentity {
    pub fn id(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }
}

pub fn load_or_create_identity() -> Result<NodeIdentity, Box<dyn Error>> {
    let path = "node_key";

    let signing_key = if let Ok(bytes) = fs::read(path) {
        // Existing key found
        SigningKey::from_bytes(&bytes.try_into().map_err(|_| "Invalid key length")?)
    } else {
        // Generate new key
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);
        fs::write(path, key.to_bytes())?;
        key
    };

    let verifying_key = signing_key.verifying_key();

    Ok(NodeIdentity {
        signing_key,
        verifying_key,
    })
}
