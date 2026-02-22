use ed25519_dalek::{SigningKey, VerifyingKey};
use rand_core::OsRng;
use std::fs;
use std::path::Path;
use std::error::Error;

const KEY_PATH: &str = "node_key";

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
    if Path::new(KEY_PATH).exists() {
        let hex_data = fs::read_to_string(KEY_PATH)?;
        let bytes = hex::decode(hex_data.trim())?;

        let key_bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| "Invalid key length")?;

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        println!("Loaded existing node identity.");

        Ok(NodeIdentity {
            signing_key,
            verifying_key,
        })
    } else {
        println!("Generating new node identity...");

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let hex_key = hex::encode(signing_key.to_bytes());
        fs::write(KEY_PATH, hex_key)?;

        Ok(NodeIdentity {
            signing_key,
            verifying_key,
        })
    }
}
