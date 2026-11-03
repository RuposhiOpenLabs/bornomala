# Architecture & Project Decisions Log (DECISIONS.md)

**Project:** Bornomala (বর্ণমালা)  
**Publisher:** Ruposhi Open Labs  
**Developer:** Ryan Shelby  
**Repository:** github.com/RuposhiOpenLabs/bornomala  
**License:** Mozilla Public License 2.0 (MPL-2.0)  

---

## 1. Reference Executable Path, Hashes, and Analysis Workspace

* **Status:** Decided & Executed (Phase 0 & 1).
* **Reference Executable Archive:**
  * Path: `bornomala-analysis-workspace/ref/Bijoy-Bayanno.rar`
  * SHA-256: `e1e09c67d6e96e2962c18c76095ddeb39191d2a0bcee23e8cc356c688baf5050`
* **Reference Phonetic Archive:**
  * Path: `bornomala-analysis-workspace/ref/setup_avrokeyboard_5.6.0.exe`
  * SHA-256: `e617b987f1e23dedd1514fad7b5ca81e464727418b86366f0d4808e19fd47b48`
* **Analysis Workspace Isolation:**
  * In strict compliance with security and copyright rules, the private analysis workspace was established at `/home/ryan/Desktop/PROJECT_2026/bornomala-analysis-workspace`, completely outside the public Git repository (`/home/ryan/Desktop/PROJECT_2026/bornomala`).
  * The reference executable was never executed on the Linux host machine.
  * Static extraction of the installer payload was performed using `unrar` and `7z`/`cabextract` on the MSI Cabinet archive (`_B8DA7F486B83B4A2D4D10028E8357DC3`).
  * Analysis confirmed:
    * `Bayanno2011.exe` is a native PE32 Visual Studio setup bootstrapper.
    * `BijoyBayanno.msi` installs `BijoyBayanno.exe` (a .NET assembly, MVID `5a55b96c-39ff-4eb7-9d4a-3f6aaeef0c8e`), `Converter.exe` (.NET assembly), XML layout files (`Unicode.xml`, `Bijoy Classic.xml`), and TrueType fonts.
  * No reference binary, proprietary font, artwork, icon, or decompiler transliteration is included in the public repository. All implementation code is original Rust.

---

## 2. Project Licensing

* **Status:** Decided.
* **License:** Mozilla Public License 2.0 (MPL-2.0).
* **Rationale:**
  * Bornomala integrates phonetic transliteration rules and dictionary assets derived from the open-source Avro Keyboard project (originally released by OmicronLab under MPL-2.0).
  * MPL-2.0 allows free personal, commercial, and governmental usage, permits integration with proprietary host applications (such as Microsoft Office, Adobe InDesign, web browsers), and guarantees that improvements to file-level components remain open source without imposing viral copyleft on the entire operating system or host applications.

---

## 3. Code-Signing & Release Integrity Strategy

* **Status:** Decided.
* **Mechanism:**
  * All official releases are accompanied by an Ed25519 cryptographic signature and a signed `manifest.json` containing SHA-256 hashes of all release binaries.
  * The release verification key is embedded into the client application, with a fallback secondary key and key rotation procedure documented in `SECURITY.md`.
  * Rollback protection is enforced: client updaters strictly reject manifests with version numbers `<= installed_version`.
  * Until an EV / Authenticode code-signing certificate is issued to Ruposhi Open Labs (e.g. through SignPath Foundation for open-source projects or Certum), releases are distributed as hash-verified unsigned binaries.

---

## 4. Release Distribution & Hosting

* **Status:** Decided.
* **Distribution Point:** Exclusively via GitHub Releases under `github.com/RuposhiOpenLabs/bornomala`.
* **Telemetry Policy:** Absolutely zero telemetry, analytics, tracking, or network calls, except an explicit user-initiated "Check for Updates" action.

---

## 5. Text Input Architecture (TSF vs Hooks)

* **Status:** Decided.
* **Architecture:**
  * Text Services Framework (TSF) in-process COM server DLL (`bornomala-tsf`).
  * Implements `ITfTextInputProcessor`, `ITfThreadMgrEventSink`, `ITfKeyEventSink`, `ITfCompositionSink`.
  * Avoids global low-level keyboard hooks (`WH_KEYBOARD_LL`), which are flagged by antivirus software, create input lag, and break across UAC elevation boundaries.
  * Per-user registration (`HKCU\Software\Classes\CLSID` + `ITfInputProcessorProfiles`) is attempted first to allow unprivileged installation on locked-down office machines; administrator installation fallback is provided.
  * Marked as **Unverified on Windows** in `WINDOWS_VERIFICATION.md` until tested on physical Windows 10/11 machines.

---

## 6. Bundled Word List & Dictionary

* **Status:** Decided.
* **Source:** Official 159,426-word Bangla corpus extracted from `ibus-avro` (`avrodict.js`, MPL-2.0).
* **Storage:** Clean text list in `data/dictionaries/bangla_words.txt`.
* **Engine Optimization:** An embedded high-frequency prefix trie in `bornomala-engine` delivers sub-millisecond offline suggestions with minimal memory consumption (< 15 MB).

---

## 7. Emulation, Test Oracles, and Confidence Discipline

* **Status:** Decided & Implemented.
* **Phonetic Differential Oracle:**
  * Node.js test harness executing official `avrolib.js` from `ibus-avro`.
  * Yields **Confidence Level A** test vectors (confirmed by external oracle execution).
* **Fixed Layout & Converter:**
  * Dual-source independent static extraction:
    1. Reference layout XML definitions (`Unicode.xml`, `Bijoy Classic.xml`).
    2. CIL bytecode disassembly of `BijoyBayanno.exe` (`modKeyboard.IsHooked`) and `Converter.exe` (`Form1.findNext`, `findNextJukta`).
  * Yields **Confidence Level B** test vectors (concordance between data tables and executing bytecode).
* **Level C Items:**
  * Single decompiler readings or untested Windows-specific APIs (TSF registration, topmost floating bar behavior) are cataloged in `WINDOWS_VERIFICATION.md`.

---

## 8. Windows Auto-Startup & Startup Splash Screen Experience

* **Status:** Decided & Implemented.
* **Familiarity Rationale:**
  * Decades of typists in Bangladesh using Bijoy Bayanno or Ekushe expect visual feedback upon system logon or app opening to know the typing suite is active and ready.
  * A centered splash popup displays the Bornomala logo, version, taglines, developer & publisher attribution, and an initialization progress bar.
  * The splash popup automatically closes after 2.5 seconds (2500 ms), leaving the TSF input method active and the floating toolbar/tray icon ready.
* **Registry Architecture:**
  * User-level registration: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, value `Bornomala`, command `"path\to\bornomala.exe" --startup`.
  * Requires no elevated administrator permissions.
  * Can be toggled from the Settings UI, CLI (`bornomala-cli autostart`), or batch scripts (`scripts\enable_autostart.bat`, `scripts\disable_autostart.bat`).
* **Support & Community Attribution:**
  * Organization: [Ruposhi Open Labs](https://github.com/RuposhiOpenLabs)
  * Developer: [Ryan Shelby](https://github.com/MdSagorMunshi/)
  * Support Email: `ryn@disr.it`
