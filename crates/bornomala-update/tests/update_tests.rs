use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;
use rand_core::OsRng;
use std::fs;
use tempfile::tempdir;
use bornomala_update::{
    IntegrityVerifier, ManifestFileEntry, ReleaseManifest, StagedUpdatePlanner, UpdateError,
};

#[test]
fn test_ed25519_signature_and_hash_verification() {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let verifier = IntegrityVerifier::new(verifying_key.as_bytes(), None).unwrap();

    let manifest_content = br#"{"version":"0.2.0","release_date":"2026-11-03","files":[]}"#;
    let signature = signing_key.sign(manifest_content);

    // 1. Valid signature passes
    assert!(verifier
        .verify_manifest_signature(manifest_content, &signature.to_bytes())
        .is_ok());

    // 2. Tampered content fails
    let tampered_content = br#"{"version":"0.2.1","release_date":"2026-11-03","files":[]}"#;
    assert!(matches!(
        verifier.verify_manifest_signature(tampered_content, &signature.to_bytes()),
        Err(UpdateError::BadSignature)
    ));

    // 3. Wrong key fails
    let other_key = SigningKey::generate(&mut csprng);
    let other_verifier = IntegrityVerifier::new(other_key.verifying_key().as_bytes(), None).unwrap();
    assert!(matches!(
        other_verifier.verify_manifest_signature(manifest_content, &signature.to_bytes()),
        Err(UpdateError::BadSignature)
    ));
}

#[test]
fn test_rollback_prevention() {
    assert!(IntegrityVerifier::is_version_strictly_greater("0.1.0", "0.2.0"));
    assert!(IntegrityVerifier::is_version_strictly_greater("0.1.9", "0.2.0"));
    assert!(IntegrityVerifier::is_version_strictly_greater("1.0.0", "1.0.1"));

    // Equal versions rejected
    assert!(!IntegrityVerifier::is_version_strictly_greater("0.2.0", "0.2.0"));
    // Older versions rejected
    assert!(!IntegrityVerifier::is_version_strictly_greater("0.2.0", "0.1.9"));
    assert!(!IntegrityVerifier::is_version_strictly_greater("1.0.0", "0.9.9"));
}

#[test]
fn test_installation_tamper_detection() {
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    let file1_path = base_path.join("bornomala.dll");
    fs::write(&file1_path, b"original-dll-content").unwrap();
    let hash1 = ReleaseManifest::compute_file_sha256(&file1_path).unwrap();

    let manifest = ReleaseManifest {
        version: "0.1.0".to_string(),
        release_date: "2026-11-03".to_string(),
        min_supported_version: None,
        files: vec![ManifestFileEntry {
            path: "bornomala.dll".to_string(),
            sha256: hash1,
            size: 20,
        }],
        summary: Default::default(),
    };

    let dummy_key = [0u8; 32];
    let verifier = IntegrityVerifier::new(&dummy_key, None).unwrap();

    // Clean verification
    assert!(verifier.verify_installation_integrity(base_path, &manifest).is_ok());

    // Tamper the file
    fs::write(&file1_path, b"maliciously-altered-dll-content").unwrap();

    // Verification must detect tampering
    let err = verifier.verify_installation_integrity(base_path, &manifest).unwrap_err();
    assert!(matches!(err, UpdateError::Tampered(_)));
}

#[test]
fn test_staged_update_planner() {
    let dir = tempdir().unwrap();
    let app_root = dir.path();

    let stage = StagedUpdatePlanner::prepare_stage_directory(app_root, "0.2.0").unwrap();
    assert!(stage.exists());
    assert!(stage.ends_with("app-v0.2.0"));

    StagedUpdatePlanner::mark_pending_activation(app_root, "0.2.0").unwrap();
    let receipt_path = app_root.join("pending_update.json");
    assert!(receipt_path.exists());
}
