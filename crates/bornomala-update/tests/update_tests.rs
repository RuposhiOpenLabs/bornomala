use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;
use rand_core::OsRng;
use std::fs;
use tempfile::tempdir;
use bornomala_update::{
    bytes_to_hex, IntegrityVerifier, ManifestFileEntry, ReleaseManifest,
    StagedUpdatePlanner, UpdateChecker, UpdateError, UpdatePreferences,
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
    assert!(IntegrityVerifier::is_version_strictly_greater("v0.1.0", "v0.2.0"));

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

#[test]
fn test_github_release_checker_parsing() {
    let json_release = r#"{
        "tag_name": "v0.2.0",
        "name": "Bornomala v0.2.0 (Spring Release)",
        "body": "Major improvements and security patches.\n<!-- BORNOMALA_SHA256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 -->",
        "draft": false,
        "prerelease": false,
        "assets": [
            {
                "name": "Bornomala-Setup-v0.2.0.exe",
                "size": 9450120,
                "browser_download_url": "https://github.com/RuposhiOpenLabs/bornomala/releases/download/v0.2.0/Bornomala-Setup-v0.2.0.exe"
            }
        ]
    }"#;

    // 1. When installed is older (v0.1.0), it should find candidate
    let res = UpdateChecker::parse_latest_release(json_release, "0.1.0").unwrap();
    assert!(res.is_some());
    let cand = res.unwrap();
    assert_eq!(cand.version, "0.2.0");
    assert_eq!(cand.asset_name, "Bornomala-Setup-v0.2.0.exe");
    assert_eq!(cand.expected_sha256.as_deref(), Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"));

    // 2. When installed is already v0.2.0, returns None
    let res_current = UpdateChecker::parse_latest_release(json_release, "0.2.0").unwrap();
    assert!(res_current.is_none());

    // 3. Draft or prerelease releases should be ignored
    let json_prerelease = json_release.replace("\"prerelease\": false", "\"prerelease\": true");
    let res_pre = UpdateChecker::parse_latest_release(&json_prerelease, "0.1.0").unwrap();
    assert!(res_pre.is_none());
}

#[test]
fn test_update_preferences_skip_and_later() {
    let dir = tempdir().unwrap();
    let prefs_file = dir.path().join("update_prefs.json");

    let mut prefs = UpdatePreferences::load_from(&prefs_file);

    // Initial state: not skipped, should notify
    assert!(prefs.should_notify("0.2.0", false));

    // Later chosen for this session (postponed)
    // If shown_in_current_session is true, don't notify again in this session
    assert!(!prefs.should_notify("0.2.0", true));

    // But next time computer/app boots (shown_in_current_session is reset to false):
    assert!(prefs.should_notify("0.2.0", false));

    // Now user chooses "Skip this version"
    prefs.set_skip_version("0.2.0");
    prefs.save_to(&prefs_file).unwrap();

    let reloaded = UpdatePreferences::load_from(&prefs_file);
    // Now even on fresh boot, this version is skipped!
    assert!(!reloaded.should_notify("0.2.0", false));

    // But if a newer version comes out (0.3.0), it will notify!
    assert!(reloaded.should_notify("0.3.0", false));
}

#[test]
fn test_payload_cryptographic_verification() {
    let dir = tempdir().unwrap();
    let payload_path = dir.path().join("Bornomala-Setup-v0.2.0.exe");
    let payload_data = b"BORNOMALA_TEST_SETUP_BINARY_CONTENT";
    fs::write(&payload_path, payload_data).unwrap();

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifier = IntegrityVerifier::new(signing_key.verifying_key().as_bytes(), None).unwrap();

    let sha256_hash = ReleaseManifest::compute_file_sha256(&payload_path).unwrap();
    let signature = signing_key.sign(sha256_hash.as_bytes());
    let sig_hex = bytes_to_hex(&signature.to_bytes());

    // Valid hash + signature succeeds
    assert!(verifier
        .verify_downloaded_payload(&payload_path, Some(&sha256_hash), Some(&sig_hex))
        .is_ok());

    // Tampered payload fails hash check
    fs::write(&payload_path, b"CORRUPTED_MALICIOUS_PAYLOAD").unwrap();
    let err = verifier
        .verify_downloaded_payload(&payload_path, Some(&sha256_hash), Some(&sig_hex))
        .unwrap_err();
    assert!(matches!(err, UpdateError::HashMismatch { .. }));
}
