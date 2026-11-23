# Changelog

All notable changes to **Bornomala (বর্ণমালা)** are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.1] - 2026-11-24

### Release v1.0.1 (Production Release)

#### Fixed
* **Phonetic Memory Isolation in Automated Tests:** Isolated phonetic candidate memory during automated test runs (`cargo test`) to eliminate cross-test state leakage, preventing property test commits from polluting sequential composition test assertions.
* **Installer Font Conflict Resolution & Rendering:** Added bulk font conflict policy options ("Install All / Replace All" and "Skip All Existing Fonts") to NSIS setup wizard and resolved installation directory path label rendering.
* **Dependency Cleanups:** Removed unused dependencies from `bornomala-app` and `bornomala-phonetic` manifests, eliminating build warnings.

#### Added
* **GitHub Community Templates:** Added bilingual Issue Templates (`bug_report.md`, `feature_request.md`), Issue configuration (`config.yml`), and Pull Request Template (`pull_request_template.md`).

---

## [0.1.0] - 2026-11-03

### Initial Release (Linux-Direct Verified Edition)

#### Added
* **Pure Rust Core Engine (`bornomala-engine`):**
  * Classic Fixed Layout (Unicode Output) with visual pre-base vowel reordering (`ি`, `ে`, `ৈ`), split vowels (`ো`, `ৌ`), Reph (`র্`), hasanta conjunct formation, and selectable atomic/component backspace.
  * Classic Fixed Layout (Legacy ANSI Output) producing 8-bit glyph sequences for SutonnyMJ legacy fonts (431 Juk rules).
  * Avro-Compatible Phonetic Engine with 289 contextual rule conditions, longest-prefix matching, and local learning dictionary.
  * Bidirectional Lossless Converter (`Unicode <-> SutonnyMJ ANSI`).
  * Office Conveniences:
    * Amount in words (Bangla and English Taka & Paisa).
    * Revised Bangla Calendar helper (Bangladesh 2019 Cabinet Revision).
    * Instant numeral converter (English `0-9` ⇄ Bangla `০-৯`).
    * Official text expander for memo headers, government stamps, and signatures.
    * Per-application input scheme memory.
* **Windows Text Services Framework (TSF) COM Server (`bornomala-tsf`):**
  * In-process COM DLL (`x86_64-pc-windows-gnu`, `i686-pc-windows-gnu`, `aarch64-pc-windows-gnullvm`).
  * Panic catch barrier ensuring zero crashes cross into host applications.
  * Circuit-breaker watchdog automatically failing safe into pass-through on 5 consecutive faults.
  * Simulated Host Adapter for automated Linux integration tests.
* **User Interface & Theme System (`bornomala-ui`):**
  * Design system honoring Bangladesh flag green (`#006A4E`), flag red (`#F42A41`), and accessible dark red (`#C21F35`).
  * 4 Themes: Light, Dark, High-Contrast (Accessibility), and Large-Text.
  * 100% Bilingual localization in Bangla and English.
  * High-resolution vector SVG mockups for Floating Bar, On-Screen Keyboard, Converter, Settings, Welcome, and About dialogs.
* **Cryptographic Update & Integrity System (`bornomala-update`):**
  * Ed25519 signature verification and SHA-256 payload hashing.
  * Monotonic rollback protection (`v_remote > v_installed`).
  * Anti-tamper pre-update self-check.
* **Terminal Utilities:**
  * `bornomala-cli`: Terminal CLI tool for text conversion, amount translation, calendar calculation, and typing inspection.
  * `bornomala-lab`: Interactive full-screen terminal typing lab.
  * `bornomala-spec-tools`: Automated specification and ground-truth vector validation suite.
* **Documentation & Registration Scripts:**
  * `scripts/register_user.bat` (per-user non-admin registration).
  * `scripts/register_admin.bat` (system-wide administrator registration).
  * `scripts/unregister.bat` (clean COM uninstallation).
  * `SPECIFICATION.md`: Comprehensive reverse engineering evidence register with confidence levels (Level A, B, C).
  * `DECISIONS.md`: Formal architectural decision log.
  * `WINDOWS_VERIFICATION.md`: Verification register for physical Windows hardware testing.
