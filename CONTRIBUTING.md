# Contributing to Bornomala

Thank you for your interest in contributing to **Bornomala (বর্ণমালা)**!

Our mission is to provide Bangladesh and Bangla-speaking communities worldwide with a free, open-source, ad-free, completely safe typing suite.

---

## 1. Clean-Room Reverse Engineering & Legal Discipline

Because Bornomala was created to replace legacy proprietary software, strict clean-room hygiene must be maintained at all times:

1. **Zero Reference Artifacts in Repository:** Never commit reference Windows executables, proprietary installers, decompiled assemblies, or copyrighted third-party fonts (such as SutonnyMJ) into this repository.
2. **Data Tables & Original Implementation:** Only clean data tables (e.g. key codes, Unicode code point mappings) and original Rust implementations may be committed.
3. **Evidence Levels:** When submitting changes to typing rules or mappings, you must document evidence according to our confidence standard:
   * **Level A:** Confirmed by an emulated or oracle execution run.
   * **Level B:** Confirmed by dual independent static sources (e.g. data table and matching algorithm).
   * **Level C:** Single decompiled observation (must be listed in `WINDOWS_VERIFICATION.md` until physically verified on Windows hardware).

---

## 2. Development Guidelines

* **Pure Rust:** The core engine (`bornomala-engine`) must remain 100% platform-independent and free of OS-specific APIs.
* **Panic Safety:** Code in `bornomala-tsf` must never allow a Rust panic to unwind across COM / C-ABI boundaries. Always wrap entry points in `std::panic::catch_unwind`.
* **UTF-8 Boundary Safety:** Any string slicing, truncation, or history trimming must check character boundaries (`is_char_boundary` or `char_indices`).
* **100% Test Coverage:** All new features or layout adjustments must include corresponding test vectors in `data/test_vectors/` and unit tests in `tests/`.

---

## 3. Submitting Pull Requests

1. Fork the repository and create your feature branch from `main`.
2. Ensure that the entire workspace builds and all tests pass cleanly:
   ```bash
   cargo test --workspace
   cargo run -p bornomala-spec-tools
   ```
3. Run `cargo fmt` and `cargo clippy`.
4. Submit your pull request with a clear description and any associated test vectors.
