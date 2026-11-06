# Bornomala (বর্ণমালা)

<div align="center">

![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011%20%7C%20Linux-green.svg)
![Offline](https://img.shields.io/badge/privacy-100%25%20Offline-brightgreen.svg)
![Language](https://img.shields.io/badge/rust-2021-orange.svg)
![Security](https://img.shields.io/badge/signing-Ed25519%20Verified-blueviolet.svg)

**Your mother tongue doesn’t need a license key.**  
*আপনার মাতৃভাষার জন্য কোনো লাইসেন্স কি-র প্রয়োজন নেই।*

**No crack. No virus. No invoice.**  
*কোনো ক্র্যাক নেই। কোনো ভাইরাস নেই। কোনো ইনভয়েস নেই।*

[English](#english) | [বাংলা](#বাংলা) | [Architecture](ARCHITECTURE.md) | [User Manual](MANUAL.md) | [Security](SECURITY.md)

**Publisher:** [Ruposhi Open Labs](https://github.com/RuposhiOpenLabs) &nbsp;|&nbsp; **Developer:** [Ryan Shelby](https://github.com/MdSagorMunshi/) &nbsp;|&nbsp; **Support:** [ryn@disr.it](mailto:ryn@disr.it)

</div>

---

<a name="english"></a>
## English

Bornomala is a clean, modern, signed, and completely free Bangla typing suite and input method written from scratch in pure Rust. It was created by **[Ruposhi Open Labs](https://github.com/RuposhiOpenLabs)** (developed by **[Ryan Shelby](https://github.com/MdSagorMunshi/)**) to free millions of users in Bangladesh—from government clerks and court stenographers to journalists, teachers, and students—from dependence on cracked, malware-ridden legacy software.

### Why Bornomala?
* **Classic Fixed Layout (Unicode Output):** Exactly matches the 20-year typing muscle memory of government and commercial typists (`Ctrl+Alt+V`), producing canonical Unicode Bangla (`U+0980..U+09FF`).
* **Classic Fixed Layout (Legacy ANSI Output):** Produces legacy SutonnyMJ glyphs (`Ctrl+Alt+B`) for compatibility with existing government forms, land registry documents, and publishing workflows without needing cracked software.
* **Avro-Compatible Phonetic Typing:** Full Banglish-to-Bangla phonetic transliteration (`F12`) validated 100% against official phonetic rules with intelligent offline suggestions.
* **Bijoy/Ekushe-Style Startup Splash Screen & Auto-Startup:** Displays an elegant, familiar centered splash screen on system startup or app launch, then automatically hides itself after 2.5 seconds to reveal the floating bar and system tray.
* **Lossless Bidirectional Converter:** High-fidelity conversion between modern Unicode and legacy ANSI (SutonnyMJ) text.
* **Government Office Conveniences:**
  * **Amount in Words:** Bidirectional English & Bangla conversion of currency amounts (Taka and Paisa).
  * **Revised Bangla Calendar:** Official 2019 Bangladesh revision (Baishakh–Ashwin: 31 days).
  * **Numeral Style Toggle:** Instant toggle between English (`0-9`) and Bangla (`০-৯`) numerals.
  * **Official Text Expander:** Rapid expansion of official memo headers, signatures, and legal disclaimers.
* **100% Offline & Private:** Zero keystroke logging, zero telemetry, zero accounts, zero background network calls.
* **Fail-Safe Robustness:** Text Services Framework (TSF) COM architecture with panic catch barriers and an automatic 5-error watchdog that degrades to pass-through rather than crashing host applications.

---

<a name="বাংলা"></a>
## বাংলা

বর্ণমালা (Bornomala) হলো সম্পূর্ণ বিনামূল্যে, বিজ্ঞাপনমুক্ত, উন্মুক্ত এবং শতভাগ নিরাপদ একটি আধুনিক বাংলা টাইপিং স্যুট। এটি তৈরি করেছে **[রূপসী ওপেন ল্যাবস (Ruposhi Open Labs)](https://github.com/RuposhiOpenLabs)** (প্রধান ডেভেলপার: **[রায়ান শেলবি / Ryan Shelby](https://github.com/MdSagorMunshi/)**)।

বাংলাদেশে সরকারি দপ্তর, আদালত, প্রেস ও প্রকাশনা প্রতিষ্ঠান, এবং শিক্ষা প্রতিষ্ঠানে এখনও অনেকে পুরোনো লাইসেন্সড সফটওয়্যারের ক্র্যাকড কপি ব্যবহার করতে বাধ্য হন, যা প্রায়ই ক্ষতিকর ম্যালওয়্যার ও ভাইরাসে আক্রান্ত থাকে। বর্ণমালা এই সমস্যার স্থায়ী ও আধুনিক সমাধান।

### প্রধান সুবিধাসমূহ
1. **ক্লাসিক ফিক্সড লেআউট (ইউনিকোড):** ২০ বছরের পরিচিত ফিক্সড কিবোর্ড লেআউট হুবহু অপরিবর্তিত রেখে আধুনিক ইউনিকোড বাংলা টাইপিং (`Ctrl+Alt+V`)।
2. **ক্লাসিক এএনএসআই লেআউট (সুতোন্নীএমজে):** পুরনো সরকারি ফর্ম ও দলিলের জন্য সুতোন্নী ফন্ট উপযোগী লেগ্যাসি এএনএসআই টাইপিং (`Ctrl+Alt+B`)।
3. **ফোনেটিক টাইপিং (অভ্র সামঞ্জস্যপূর্ণ):** ইংরেজি হরফে লিখে খাঁটি বাংলা উচ্চারণভিত্তিক টাইপিং (`F12`), অফলাইন শব্দকোষ ও স্বয়ংক্রিয় সাজেশন সহ।
4. **স্বয়ংক্রিয় স্টার্টআপ ও ক্লাসিক স্প্ল্যাশ স্ক্রিন:** কম্পিউটার চালু হলে বা অ্যাপ ওপেন করলে বিজয় বা একুশের ন্যায় স্ক্রিনের মাঝখানে পরিচিত স্টার্টআপ স্প্ল্যাশ স্ক্রিন দেখাবে এবং ২.৫ সেকেন্ড পর নিজে নিজেই লুকিয়ে যাবে।
5. **দ্বিমুখী রূপান্তরক (Unicode <-> ANSI):** কোনো তথ্যের বিকৃতি ছাড়াই ইউনিকোড থেকে সুতোন্নী এবং সুতোন্নী থেকে ইউনিকোডে টেক্সট কনভার্সন।
6. **অফিসিয়াল সহায়তা টুলস:**
   * চেকে বা নথিতে টাকার পরিমাণ কথায় লেখা (টাকা ও পয়সা — বাংলা ও ইংরেজিতে)।
   * বাংলাদেশ সরকারের ২০১৯ সালের সংশোধিত বাংলা ক্যালেন্ডার অনুযায়ী তারিখ রূপান্তর।
   * ইংরেজি ও বাংলা সংখ্যার তাৎক্ষণিক পরিবর্তন।
   * সরকারি মেমো ও স্মারক দ্রুত টাইপ করার টেক্সট এক্সপ্যান্ডার।
7. **১০০% অফলাইন ও নিরাপদ:** কোনো কি-লগিং নেই, কোনো ডেটা সংগ্রহ বা ইন্টারনেটে পাঠানো হয় না। কোনো বিজ্ঞাপন বা ক্র্যাকের ঝামেলা নেই।

---

## Workspace Architecture

| Crate | Purpose |
|---|---|
| [`bornomala-engine`](crates/bornomala-engine) | Pure Rust platform-agnostic core: Unicode engine, ANSI engine, Avro Phonetic engine, Converter, and Office Conveniences. |
| [`bornomala-tsf`](crates/bornomala-tsf) | Windows Text Services Framework (TSF) in-process COM DLL with panic catch boundaries and watchdog protection. |
| [`bornomala-ui`](crates/bornomala-ui) | Design system, bilingual localization (BN/EN), themes (Light/Dark/High-Contrast/Large-Text), and vector renderer. |
| [`bornomala-update`](crates/bornomala-update) | Ed25519 cryptographic signature & SHA-256 hash verifier, anti-tamper checking, and rollback protection. |
| [`bornomala-cli`](crates/bornomala-cli) | Terminal utility for text conversion, amount in words, calendar calculation, and typing inspection. |
| [`bornomala-lab`](crates/bornomala-lab) | Interactive full-screen terminal typing lab with side-by-side Unicode and ANSI visualizers. |
| [`bornomala-spec-tools`](crates/bornomala-spec-tools) | Automated specification and test vector validation runner. |

---

## Quick Start & Usage

### 1. Build and Run Tests (Linux / Windows)
```bash
# Clone the repository
git clone https://github.com/RuposhiOpenLabs/bornomala.git
cd bornomala

# Run all test suites across all crates
cargo test --workspace

# Validate all ground-truth test vectors
cargo run -p bornomala-spec-tools
```

### 2. Interactive Terminal Lab
```bash
cargo run -p bornomala-lab
```

### 3. CLI Conversion & Conveniences
```bash
# Convert Unicode to SutonnyMJ ANSI
cargo run -p bornomala-cli -- convert u2a -t "আমার সোনার বাংলা"

# Convert ANSI to Unicode
cargo run -p bornomala-cli -- convert a2u -t "Avgvi ‡mvbvi evsjv"

# Amount in words
cargo run -p bornomala-cli -- amount 125000.75 -l bn

# Revised Bangla Date
cargo run -p bornomala-cli -- date 2026 12 16
```

### 4. Windows Installable Setup & Preloaded Fonts
Download the Windows installer package:
* **Installer Path:** `target/installer/Bornomala-Setup-v0.1.0.exe` (7.6 MB compressed)
* **Custom Animated Setup Wizard:** Modern UI with smooth lazy animations, step-by-step guidance, and professional Terms & Conditions ([`TERMS.md`](TERMS.md)).
* **8 Preloaded Government & Classic Fonts:**
  1. `Nikosh.ttf` (Government Unicode Standard)
  2. `NikoshBAN.ttf` (Bengali Typography Variant)
  3. `NikoshGrameen.ttf` (Grameen Display Variant)
  4. `NikoshLight.ttf` (Light Weight Variant)
  5. `NikoshLightBan.ttf` (Light Print Variant)
  6. `SutonnyMJ Regular.ttf` (Classic ANSI Font for Land Registry & Publishing)
  7. `AdorshoLipi_20-07-2007.ttf` (Literary & Educational Standard)
  8. `Bangla.ttf` (Universal OpenType Font)
* **Interactive Font Conflict Resolution Popup:**
  If any of these fonts already exist on your system, the setup shows an interactive prompt asking whether you want to **Replace with Preloaded Font** or **Keep Existing Version**.
* **Automatic System Refresh:** The setup automatically broadcasts `WM_FONTCHANGE` upon completion so MS Word, Photoshop, Excel, and Illustrator immediately recognize the fonts without requiring a reboot.
* **Auto-Startup & Splash Screen:** Optionally registers Bornomala in Windows startup to display the Bijoy/Ekushe-style splash screen on launch.

```bash
# Preview animated setup UI in your browser:
cargo run -p bornomala-cli -- setup --html assets/installer_preview.html

# List preloaded fonts and inspect encoding support:
cargo run -p bornomala-cli -- fonts list

# Cross-compile the Windows Installer directly on Linux:
cd installer && makensis bornomala_setup.nsi
```

### 5. Manual Registration Scripts (Portable / Dev Mode)
If running portable binaries without the NSIS installer:
```cmd
REM 1. Register TSF input method service (per-user, no admin required)
scripts\register_user.bat

REM 2. Enable Windows auto-startup on logon (with Bijoy/Ekushe style splash screen)
scripts\enable_autostart.bat
```
*(No administrator rights required. Registers safely into `HKCU`.)*

To disable auto-startup at any time, run:
```cmd
scripts\disable_autostart.bat
```

---

## Support & Community
* **Organization:** [Ruposhi Open Labs](https://github.com/RuposhiOpenLabs)
* **Lead Developer:** [Ryan Shelby](https://github.com/MdSagorMunshi/)
* **Direct Support Email:** [ryn@disr.it](mailto:ryn@disr.it)
* **Bug Reports & Issues:** [GitHub Issues](https://github.com/RuposhiOpenLabs/bornomala/issues)

---

## Documentation
* [Architecture Overview](ARCHITECTURE.md)
* [User Manual & Shortcuts](MANUAL.md)
* [Reverse Engineering Specification & Evidence](SPECIFICATION.md)
* [Architectural Decisions Log](DECISIONS.md)
* [Windows Verification Register](WINDOWS_VERIFICATION.md)
* [Security & Disclosure Policy](SECURITY.md)

---

## License
Released under the [GNU General Public License v3.0 (GPLv3)](LICENSE).  
Copyright © 2026 Ruposhi Open Labs & Ryan Shelby. <ryn@disr.it>
