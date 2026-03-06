use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use chrono::Utc;
use std::sync::Mutex;
use serde::Serialize;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use hex;

#[derive(Serialize, Clone)]
pub struct AuditRecord {
    pub id: i64,
    pub timestamp: String,
    pub environment: String,
    pub payload_size: i64,
    pub allowed: bool,
    pub message: String,
    pub previous_hash: String,
    pub hash: String,
    pub signature: String,
}

pub struct AuditLogger {
    conn: Mutex<Connection>,
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl AuditLogger {

    pub fn new(signing_key: SigningKey) -> Self {

        let verifying_key = signing_key.verifying_key();

        let conn = Connection::open("orpheus_audit.db")
            .expect("Failed to open audit database");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                environment TEXT NOT NULL,
                payload_size INTEGER NOT NULL,
                allowed INTEGER NOT NULL,
                message TEXT NOT NULL,
                previous_hash TEXT NOT NULL,
                hash TEXT NOT NULL,
                signature TEXT NOT NULL
            )",
            [],
        ).expect("Failed to create audit table");

        Self {
            conn: Mutex::new(conn),
            signing_key,
            verifying_key,
        }
    }

    pub fn log_event(
        &self,
        environment: &str,
        payload_size: usize,
        allowed: bool,
        message: &str,
    ) {

        let timestamp = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();

        let previous_hash: String = conn.query_row(
            "SELECT hash FROM audit_log ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).unwrap_or_else(|_| "GENESIS".to_string());

        let mut hasher = Sha256::new();

        hasher.update(previous_hash.as_bytes());
        hasher.update(timestamp.as_bytes());
        hasher.update(environment.as_bytes());
        hasher.update(payload_size.to_string().as_bytes());
        hasher.update(if allowed { b"1" } else { b"0" });
        hasher.update(message.as_bytes());

        let hash = format!("{:x}", hasher.finalize());

        let signature: Signature = self.signing_key.sign(hash.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        conn.execute(
            "INSERT INTO audit_log
            (timestamp, environment, payload_size, allowed, message, previous_hash, hash, signature)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                timestamp,
                environment,
                payload_size as i64,
                if allowed { 1 } else { 0 },
                message,
                previous_hash,
                hash,
                signature_hex
            ],
        ).expect("Insert failed");
    }

    pub fn get_recent(&self, limit: i64) -> Vec<AuditRecord> {

        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, timestamp, environment, payload_size, allowed, message, previous_hash, hash, signature
             FROM audit_log
             ORDER BY id DESC
             LIMIT ?1"
        ).unwrap();

        let rows = stmt.query_map(params![limit], |row| {

            Ok(AuditRecord {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                environment: row.get(2)?,
                payload_size: row.get(3)?,
                allowed: row.get::<_, i64>(4)? == 1,
                message: row.get(5)?,
                previous_hash: row.get(6)?,
                hash: row.get(7)?,
                signature: row.get(8)?,
            })

        }).unwrap();

        rows.map(|r| r.unwrap()).collect()
    }

    pub fn verify_signatures(&self) -> bool {

        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT hash, signature FROM audit_log"
        ).unwrap();

        let rows = stmt.query_map([], |row| {

            let hash: String = row.get(0)?;
            let signature: String = row.get(1)?;
            Ok((hash, signature))

        }).unwrap();

        for row in rows {

            let (hash, signature_hex) = row.unwrap();

            let sig_bytes = hex::decode(signature_hex).unwrap();
            let sig = Signature::from_slice(&sig_bytes).unwrap();

            if self.verifying_key.verify(hash.as_bytes(), &sig).is_err() {
                return false;
            }
        }

        true
    }

    pub fn chain_head(&self) -> (String, i64) {

        let conn = self.conn.lock().unwrap();

        let result: Result<(String, i64), _> = conn.query_row(
            "SELECT hash, id FROM audit_log ORDER BY id DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        match result {
            Ok(v) => v,
            Err(_) => ("GENESIS".to_string(), 0),
        }
    }
}
