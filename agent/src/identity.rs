use ed25519_dalek::{SigningKey, VerifyingKey};
use rand_core::OsRng;
use std::fs;
use std::path::Path;
use hex;
use std::error::Error;

pub struct NodeIdentity {
    signing_key: SigningKey,
}

impl NodeIdentity {
    pub fn id(&self) -> String {
        let verifying_key: VerifyingKey = self.signing_key.verifying_key();
        hex::encode(verifying_key.to_bytes())
    }

    pub fn signing_key(&self) -> SigningKey {
        self.signing_key.clone()
    }
}

pub fn load_or_create_identity() -> Result<NodeIdentity, Box<dyn Error>> {
    let path = "node_key";

    if Path::new(path).exists() {
        let key_bytes = fs::read(path)?;

        if key_bytes.len() != 32 {
            return Err("Invalid key length".into());
        }

        let key_array: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| "Failed to convert key bytes")?;

        let signing_key = SigningKey::from_bytes(&key_array);

        Ok(NodeIdentity { signing_key })
    } else {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);

        fs::write(path, signing_key.to_bytes())?;

        Ok(NodeIdentity { signing_key })
    }
}
