# Bornomala Security & Integrity Policy

**Publisher:** [Ruposhi Open Labs](https://github.com/RuposhiOpenLabs)  
**Lead Developer:** [Ryan Shelby](https://github.com/MdSagorMunshi/)  
**Direct Security Email:** [ryn@disr.it](mailto:ryn@disr.it)  

---

## 1. Zero Keystroke Logging & Offline Privacy Guarantee

Bornomala is an input method and typing suite designed to handle sensitive government documents, personal legal records, and financial entries.

* **100% Offline by Architecture:** The typing engines (`bornomala-engine`) and text service (`bornomala-tsf`) contain zero networking dependencies, zero socket code, and zero telemetry collection.
* **No Telemetry or Analytics:** Bornomala does not collect user behavior, typing speed, process names, or error telemetry.
* **Local-Only Dictionaries:** Custom dictionary additions and expansion abbreviations are stored locally on disk in plain JSON format. They are never synchronized to any cloud or remote endpoint.

---

## 2. Release Integrity & Cryptographic Signing

Every official release of Bornomala is signed using **Ed25519** public-key cryptography and hashed with **SHA-256**.

### Verification Procedure
To verify that an installer or update binary came directly from Ruposhi Open Labs and has not been tampered with:

```bash
# 1. Inspect file hash
sha256sum bornomala_tsf.dll

# 2. Verify release manifest signature using bornomala-cli
cargo run -p bornomala-cli -- verify --manifest manifest.json --key developer_pubkey.hex
```

### Rollback Protection
The `bornomala-update` subsystem enforces monotonic version sequencing:
$$\text{version}_{\text{remote}} > \text{version}_{\text{installed}}$$
Any update package bearing a version number less than or equal to the currently installed version is rejected immediately to prevent downgrade attacks.

---

## 3. Reporting a Vulnerability

If you discover a security vulnerability or safety flaw in Bornomala:

1. **Email:** Send details directly to `ryn@disr.it` (Developer: Ryan Shelby) or open a confidential security advisory at [github.com/RuposhiOpenLabs/bornomala/security/advisories](https://github.com/RuposhiOpenLabs/bornomala/security/advisories).
2. **PGP Encryption:** Sensitive disclosures may be encrypted with our public GPG key (found on keys.openpgp.org).
3. **Disclosure Timeline:** We will acknowledge receipt within 48 hours and coordinate a fix and public disclosure timeline with you.
4. **No Bug Bounties / Paid Rewards:** Ruposhi Open Labs is a non-profit open-source initiative. We provide public credit and attribution in our security advisories and release notes.
