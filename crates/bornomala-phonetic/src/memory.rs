use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const STORAGE_MAGIC: &[u8; 8] = b"BORNOMEM";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CandidateMemory {
    /// Maps Roman input query -> (Candidate -> Selection Count)
    pub selections: HashMap<String, HashMap<String, u32>>,
    #[serde(skip)]
    file_path: Option<PathBuf>,
}

impl CandidateMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_file(path: PathBuf) -> Self {
        if path.exists() {
            if let Ok(loaded) = Self::load_from_path(&path) {
                let mut mem = loaded;
                mem.file_path = Some(path);
                return mem;
            }
        }
        Self {
            selections: HashMap::new(),
            file_path: Some(path),
        }
    }

    /// Record that the user picked a specific Bangla candidate for a Roman input.
    pub fn record_selection(&mut self, roman: &str, candidate: &str) {
        let entry = self.selections.entry(roman.to_string()).or_default();
        let count = entry.entry(candidate.to_string()).or_insert(0);
        *count = count.saturating_add(1);

        if let Some(ref path) = self.file_path {
            let _ = self.save_to_path(path);
        }
    }

    /// Get user selection count for a candidate.
    pub fn get_count(&self, roman: &str, candidate: &str) -> u32 {
        if let Some(map) = self.selections.get(roman) {
            if let Some(&cnt) = map.get(candidate) {
                return cnt;
            }
        }
        self.selections
            .get(&roman.to_ascii_lowercase())
            .and_then(|map| map.get(candidate).copied())
            .unwrap_or(0)
    }

    /// Rerank candidate list according to learned frequency.
    pub fn rerank(&self, roman: &str, candidates: &mut Vec<String>) {
        if candidates.is_empty() {
            return;
        }

        let history = self
            .selections
            .get(roman)
            .or_else(|| self.selections.get(&roman.to_ascii_lowercase()));

        if let Some(history) = history {
            candidates.sort_by(|a, b| {
                let count_a = history.get(a).copied().unwrap_or(0);
                let count_b = history.get(b).copied().unwrap_or(0);
                count_b.cmp(&count_a)
            });
        }
    }

    /// Derive machine-local encryption key.
    fn derive_key() -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"Bornomala::PhoneticMemory::LocalMachineKey");
        if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
            hasher.update(user.as_bytes());
        }
        if let Ok(host) = std::env::var("HOSTNAME").or_else(|_| std::env::var("COMPUTERNAME")) {
            hasher.update(host.as_bytes());
        }
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    }

    /// Salted SHA-256 CTR stream cipher for local encryption.
    fn transform_stream(data: &[u8], key: &[u8; 32], salt: &[u8; 16]) -> Vec<u8> {
        let mut out = Vec::with_capacity(data.len());
        let mut counter: u64 = 0;

        for chunk in data.chunks(32) {
            let mut hasher = Sha256::new();
            hasher.update(key);
            hasher.update(salt);
            hasher.update(&counter.to_le_bytes());
            let keystream_block = hasher.finalize();

            for (b, k) in chunk.iter().zip(keystream_block.iter()) {
                out.push(b ^ k);
            }
            counter += 1;
        }

        out
    }

    pub fn save_to_path(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json_bytes = serde_json::to_vec(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let key = Self::derive_key();
        let mut salt = [0u8; 16];
        // Generate pseudo-salt from length and current state
        let mut salt_hasher = Sha256::new();
        salt_hasher.update(&key);
        salt_hasher.update(&(json_bytes.len() as u64).to_le_bytes());
        salt.copy_from_slice(&salt_hasher.finalize()[..16]);

        let ciphertext = Self::transform_stream(&json_bytes, &key, &salt);

        let mut payload = Vec::with_capacity(8 + 16 + ciphertext.len());
        payload.extend_from_slice(STORAGE_MAGIC);
        payload.extend_from_slice(&salt);
        payload.extend_from_slice(&ciphertext);

        fs::write(path, payload)
    }

    pub fn load_from_path(path: &Path) -> std::io::Result<Self> {
        let raw = fs::read(path)?;
        if raw.len() < 24 || &raw[..8] != STORAGE_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid or corrupted candidate memory file",
            ));
        }

        let mut salt = [0u8; 16];
        salt.copy_from_slice(&raw[8..24]);
        let ciphertext = &raw[24..];

        let key = Self::derive_key();
        let plaintext = Self::transform_stream(ciphertext, &key, &salt);

        serde_json::from_slice(&plaintext)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_record_and_rerank() {
        let mut mem = CandidateMemory::new();
        let roman = "kormo";
        let mut candidates = vec!["করম".to_string(), "কর্ম".to_string(), "ক্রম".to_string()];

        // Before record, standard order
        assert_eq!(candidates[0], "করম");

        // User picks "কর্ম" 3 times
        mem.record_selection(roman, "কর্ম");
        mem.record_selection(roman, "কর্ম");
        mem.record_selection(roman, "কর্ম");

        mem.rerank(roman, &mut candidates);

        // "কর্ম" is now rank 0
        assert_eq!(candidates[0], "কর্ম");
    }

    #[test]
    fn test_memory_encryption_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_mem.enc");

        let mut mem = CandidateMemory::with_file(path.clone());
        mem.record_selection("bangla", "বাংলা");
        mem.record_selection("bangladesh", "বাংলাদেশ");

        // Verify file is saved and encrypted on disk
        let disk_bytes = fs::read(&path).unwrap();
        assert!(disk_bytes.starts_with(STORAGE_MAGIC));
        assert!(!disk_bytes.windows(6).any(|w| w == b"bangla")); // Plaintext roman is not exposed!

        let loaded = CandidateMemory::load_from_path(&path).unwrap();
        assert_eq!(loaded.get_count("bangla", "বাংলা"), 1);
        assert_eq!(loaded.get_count("bangladesh", "বাংলাদেশ"), 1);
    }
}
