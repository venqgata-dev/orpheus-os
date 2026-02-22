use ed25519_dalek::{SigningKey, VerifyingKey};
use rand_core::OsRng;
use std::fs;
use std::path::Path;

const KEY_PATH: &str = "node_key";

pub struct NodeKeys {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

pub fn load_or_generate() -> NodeKeys {
    if Path::new(KEY_PATH).exists() {
        let hex_data =
            fs::read_to_string(KEY_PATH).expect("Failed to read key file");

        let bytes =
            hex::decode(hex_data.trim()).expect("Invalid hex in key file");

        let signing_key =
            SigningKey::from_bytes(&bytes.try_into().unwrap());

        let verifying_key = signing_key.verifying_key();

        println!("Loaded existing node identity.");

        NodeKeys {
            signing_key,
            verifying_key,
        }
    } else {
        println!("Generating new node identity...");

        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let hex_string = hex::encode(signing_key.to_bytes());

        fs::write(KEY_PATH, hex_string)
            .expect("Failed to write key file");

        NodeKeys {
            signing_key,
            verifying_key,
        }
    }
}
