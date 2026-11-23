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

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, UpdateError> {
    let clean = hex.trim();
    if clean.len() % 2 != 0 {
        return Err(UpdateError::InvalidPublicKey(
            "Odd hex string length".into(),
        ));
    }
    (0..clean.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&clean[i..i + 2], 16)
                .map_err(|e| UpdateError::InvalidPublicKey(e.to_string()))
        })
        .collect()
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Official public verification keys.
/// Supports a primary key and secondary backup key for key rotation.
pub const OFFICIAL_PRIMARY_PUBKEY_HEX: &str =
    "a2a14a7da567c9f3b98cbeec403867953c8e9d7875ba3fdc04f51685262b99c0";
pub const OFFICIAL_BACKUP_PUBKEY_HEX: &str =
    "942cf675615a88c781ea26cee0852ce8ca928ff4ee9d5a4d2dfe6ed7f359e219";

pub struct IntegrityVerifier {
    primary_key: VerifyingKey,
    backup_key: Option<VerifyingKey>,
}

impl IntegrityVerifier {
    pub fn official() -> Result<Self, UpdateError> {
        let prim_bytes = hex_to_bytes(OFFICIAL_PRIMARY_PUBKEY_HEX)?;
        let back_bytes = hex_to_bytes(OFFICIAL_BACKUP_PUBKEY_HEX)?;
        let mut p_arr = [0u8; 32];
        let mut b_arr = [0u8; 32];
        p_arr.copy_from_slice(&prim_bytes);
        b_arr.copy_from_slice(&back_bytes);
        Self::new(&p_arr, Some(&b_arr))
    }

    pub fn new(
        primary_key_bytes: &[u8; 32],
        backup_key_bytes: Option<&[u8; 32]>,
    ) -> Result<Self, UpdateError> {
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
    pub fn verify_manifest_signature(
        &self,
        manifest_bytes: &[u8],
        signature_bytes: &[u8],
    ) -> Result<(), UpdateError> {
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
            v.trim_start_matches(|c| c == 'v' || c == 'V')
                .split('.')
                .map(|p| {
                    p.chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                })
                .filter_map(|s| s.parse::<u64>().ok())
                .collect()
        };

        let inst = parse_version(installed);
        let rem = parse_version(remote);

        rem > inst
    }

    /// Performs self-protection integrity audit on installed directory.
    pub fn verify_installation_integrity(
        &self,
        base_dir: &Path,
        manifest: &ReleaseManifest,
    ) -> Result<(), UpdateError> {
        let mut tampered_files = Vec::new();

        for entry in &manifest.files {
            let file_path = base_dir.join(&entry.path);
            if !file_path.exists() {
                tampered_files.push(format!("{} (missing)", entry.path));
                continue;
            }

            match ReleaseManifest::compute_file_sha256(&file_path) {
                Ok(hash) => {
                    if !hash.eq_ignore_ascii_case(&entry.sha256) {
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

    /// Verifies downloaded file against SHA-256 and optional Ed25519 signature.
    pub fn verify_downloaded_payload(
        &self,
        file_path: &Path,
        expected_sha256: Option<&str>,
        ed25519_sig_hex: Option<&str>,
    ) -> Result<String, UpdateError> {
        let computed_hash = ReleaseManifest::compute_file_sha256(file_path)?;

        // 1. Verify SHA-256 hash match
        if let Some(expected) = expected_sha256 {
            if !computed_hash.eq_ignore_ascii_case(expected) {
                return Err(UpdateError::HashMismatch {
                    path: file_path.to_string_lossy().to_string(),
                    expected: expected.to_string(),
                    actual: computed_hash,
                });
            }
        }

        // 2. Verify Ed25519 signature over hash if provided
        if let Some(sig_hex) = ed25519_sig_hex {
            let sig_bytes = hex_to_bytes(sig_hex)?;
            self.verify_manifest_signature(computed_hash.as_bytes(), &sig_bytes)?;
        }

        Ok(computed_hash)
    }
}

/// GitHub Release API Structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubReleaseAsset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    #[serde(default)]
    pub digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateCandidate {
    pub version: String,
    pub title: String,
    pub release_notes: String,
    pub download_url: String,
    pub asset_name: String,
    pub asset_size: u64,
    pub expected_sha256: Option<String>,
    pub ed25519_signature: Option<String>,
}

pub struct UpdateChecker;

impl UpdateChecker {
    pub const GITHUB_LATEST_RELEASE_URL: &str =
        "https://api.github.com/repos/RuposhiOpenLabs/bornomala/releases/latest";

    /// Extracts release candidate from a GitHub release JSON string.
    /// Only returns Some if:
    /// 1. Not draft and not prerelease (must be stable!).
    /// 2. Version is strictly greater than currently installed version.
    /// 3. Contains a downloadable setup/update executable asset.
    pub fn parse_latest_release(
        json_str: &str,
        installed_version: &str,
    ) -> Result<Option<UpdateCandidate>, UpdateError> {
        let release: GitHubRelease = serde_json::from_str(json_str)?;

        if release.draft || release.prerelease {
            return Ok(None);
        }

        let clean_version = release
            .tag_name
            .trim_start_matches(|c| c == 'v' || c == 'V');
        if !IntegrityVerifier::is_version_strictly_greater(installed_version, clean_version) {
            return Ok(None);
        }

        // Find primary executable setup asset (e.g. Bornomala-Setup-v...exe or *.exe)
        let primary_asset = release
            .assets
            .iter()
            .find(|a| a.name.ends_with(".exe") && a.name.contains("Setup"))
            .or_else(|| release.assets.iter().find(|a| a.name.ends_with(".exe")));

        let asset = match primary_asset {
            Some(a) => a,
            None => return Ok(None),
        };

        let body = release.body.unwrap_or_default();
        let expected_sha256 =
            Self::extract_sha256_for_asset(&body, &asset.name, asset.digest.as_deref());
        let ed25519_signature = Self::extract_token(&body, "BORNOMALA_SIG:");

        Ok(Some(UpdateCandidate {
            version: clean_version.to_string(),
            title: release
                .name
                .unwrap_or_else(|| format!("Bornomala v{}", clean_version)),
            release_notes: body,
            download_url: asset.browser_download_url.clone(),
            asset_name: asset.name.clone(),
            asset_size: asset.size,
            expected_sha256,
            ed25519_signature,
        }))
    }

    pub fn extract_sha256_for_asset(
        body: &str,
        asset_name: &str,
        asset_digest: Option<&str>,
    ) -> Option<String> {
        // 1. Explicit token prefix: BORNOMALA_SHA256:
        if let Some(token) = Self::extract_token(body, "BORNOMALA_SHA256:") {
            return Some(token);
        }

        // 2. Direct digest from GitHub Release Asset (e.g. "sha256:7fb0760f0104b48cf45b6b3a09cce17683707bdad76a16de5c180d8428d20273")
        if let Some(d) = asset_digest {
            let clean = d.strip_prefix("sha256:").unwrap_or(d).trim();
            if clean.len() == 64 && clean.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(clean.to_lowercase());
            }
        }

        // 3. Scan release notes table or text lines mentioning asset_name for a 64-char hex string
        for line in body.lines() {
            if line.contains(asset_name) {
                for part in line.split(|c: char| c == '`' || c == '|' || c.is_whitespace() || c == '(' || c == ')') {
                    let trimmed = part.trim();
                    if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                        return Some(trimmed.to_lowercase());
                    }
                }
            }
        }

        None
    }

    fn extract_token(body: &str, prefix: &str) -> Option<String> {
        if let Some(idx) = body.find(prefix) {
            let rest = &body[idx + prefix.len()..];
            let token = rest
                .split_whitespace()
                .next()?
                .trim_matches(|c| c == '-' || c == '>' || c == '<' || c == '"' || c == '\'');
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
        None
    }
}

/// User Update Preferences (Skipped versions, postponement for current session, timestamp)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePreferences {
    pub skipped_version: Option<String>,
    pub postponed_version: Option<String>,
    pub last_checked_timestamp: u64,
}

impl UpdatePreferences {
    pub fn default_path() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata)
                .join("Bornomala")
                .join("update_prefs.json")
        } else {
            PathBuf::from("update_prefs.json")
        }
    }

    pub fn load_from(path: &Path) -> Self {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(prefs) = serde_json::from_str(&data) {
                return prefs;
            }
        }
        Self::default()
    }

    pub fn load() -> Self {
        Self::load_from(&Self::default_path())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)
    }

    pub fn save(&self) -> Result<(), io::Error> {
        Self::save_to(self, &Self::default_path())
    }

    /// Determines whether to display update popup to user.
    /// Returns false if:
    /// - Version matches `skipped_version`.
    /// - User clicked "Later" in this same session (`shown_in_current_session == true`).
    pub fn should_notify(&self, new_version: &str, shown_in_current_session: bool) -> bool {
        if let Some(ref skipped) = self.skipped_version {
            if skipped == new_version {
                return false;
            }
        }
        if shown_in_current_session {
            return false;
        }
        true
    }

    pub fn set_skip_version(&mut self, version: &str) {
        self.skipped_version = Some(version.to_string());
        let _ = self.save();
    }

    pub fn set_postponed(&mut self, version: &str) {
        self.postponed_version = Some(version.to_string());
        let _ = self.save();
    }
}

/// Staged Update Planner.
pub struct StagedUpdatePlanner;

impl StagedUpdatePlanner {
    /// Plans staged side-by-side extraction into versioned folder.
    pub fn prepare_stage_directory(
        app_root: &Path,
        new_version: &str,
    ) -> Result<PathBuf, io::Error> {
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
