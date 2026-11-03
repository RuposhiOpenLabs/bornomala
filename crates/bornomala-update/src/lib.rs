use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON Deserialization Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Cryptographic signature verification failed")]
    BadSignature,
    #[error("Rollback attempt rejected: remote version {remote} <= installed version {installed}")]
    RollbackRejected { remote: String, installed: String },
    #[error("File hash mismatch for {path}: expected {expected}, got {actual}")]
    HashMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    #[error("Integrity check failed: files were tampered with: {0:?}")]
    Tampered(Vec<String>),
    #[error("Invalid public key format: {0}")]
    InvalidPublicKey(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFileEntry {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub version: String,
    pub release_date: String,
    #[serde(default)]
    pub min_supported_version: Option<String>,
    pub files: Vec<ManifestFileEntry>,
    #[serde(default)]
    pub summary: HashMap<String, String>,
}

impl ReleaseManifest {
    pub fn compute_file_sha256(path: &Path) -> Result<String, io::Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// Official public verification keys.
/// Supports a primary key and secondary backup key for key rotation.
pub const OFFICIAL_PRIMARY_PUBKEY_HEX: &str =
    "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
pub const OFFICIAL_BACKUP_PUBKEY_HEX: &str =
    "b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3";

pub struct IntegrityVerifier {
    primary_key: VerifyingKey,
    backup_key: Option<VerifyingKey>,
}

impl IntegrityVerifier {
    pub fn new(primary_key_bytes: &[u8; 32], backup_key_bytes: Option<&[u8; 32]>) -> Result<Self, UpdateError> {
        let primary_key = VerifyingKey::from_bytes(primary_key_bytes)
            .map_err(|e| UpdateError::InvalidPublicKey(e.to_string()))?;
        let backup_key = if let Some(bytes) = backup_key_bytes {
            Some(
                VerifyingKey::from_bytes(bytes)
                    .map_err(|e| UpdateError::InvalidPublicKey(e.to_string()))?,
            )
        } else {
            None
        };

        Ok(Self {
            primary_key,
            backup_key,
        })
    }

    /// Verifies manifest bytes against an Ed25519 signature.
    pub fn verify_manifest_signature(&self, manifest_bytes: &[u8], signature_bytes: &[u8]) -> Result<(), UpdateError> {
        if signature_bytes.len() != 64 {
            return Err(UpdateError::BadSignature);
        }
        let sig = Signature::from_slice(signature_bytes).map_err(|_| UpdateError::BadSignature)?;

        if self.primary_key.verify(manifest_bytes, &sig).is_ok() {
            return Ok(());
        }

        if let Some(ref bk) = self.backup_key {
            if bk.verify(manifest_bytes, &sig).is_ok() {
                return Ok(());
            }
        }

        Err(UpdateError::BadSignature)
    }

    /// Compares two semver strings to guarantee strict progression (`remote > installed`).
    pub fn is_version_strictly_greater(installed: &str, remote: &str) -> bool {
        let parse_version = |v: &str| -> Vec<u64> {
            v.split('.')
                .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
                .filter_map(|s| s.parse::<u64>().ok())
                .collect()
        };

        let inst = parse_version(installed);
        let rem = parse_version(remote);

        rem > inst
    }

    /// Performs self-protection integrity audit on installed directory.
    pub fn verify_installation_integrity(&self, base_dir: &Path, manifest: &ReleaseManifest) -> Result<(), UpdateError> {
        let mut tampered_files = Vec::new();

        for entry in &manifest.files {
            let file_path = base_dir.join(&entry.path);
            if !file_path.exists() {
                tampered_files.push(format!("{} (missing)", entry.path));
                continue;
            }

            match ReleaseManifest::compute_file_sha256(&file_path) {
                Ok(hash) => {
                    if hash != entry.sha256 {
                        tampered_files.push(format!("{} (hash mismatch)", entry.path));
                    }
                }
                Err(_) => {
                    tampered_files.push(format!("{} (unreadable)", entry.path));
                }
            }
        }

        if tampered_files.is_empty() {
            Ok(())
        } else {
            Err(UpdateError::Tampered(tampered_files))
        }
    }
}

/// Staged Update Planner.
pub struct StagedUpdatePlanner;

impl StagedUpdatePlanner {
    /// Plans staged side-by-side extraction into versioned folder.
    pub fn prepare_stage_directory(app_root: &Path, new_version: &str) -> Result<PathBuf, io::Error> {
        let stage_dir = app_root.join(format!("app-v{}", new_version));
        fs::create_dir_all(&stage_dir)?;
        Ok(stage_dir)
    }

    /// Writes pending activation receipt for next restart.
    pub fn mark_pending_activation(app_root: &Path, new_version: &str) -> Result<(), io::Error> {
        let pending_file = app_root.join("pending_update.json");
        let receipt = serde_json::json!({
            "target_version": new_version,
            "target_dir": format!("app-v{}", new_version),
            "created_at": "now"
        });
        fs::write(pending_file, receipt.to_string())
    }
}
