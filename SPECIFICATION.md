# Bornomala (বর্ণমালা) — Behavioral Specification & Reverse Engineering Evidence

**Publisher:** [Ruposhi Open Labs](https://github.com/RuposhiOpenLabs)  
**Developer:** [Ryan Shelby](https://github.com/MdSagorMunshi/)  
**Support Email:** [ryn@disr.it](mailto:ryn@disr.it)  
**Repository:** `github.com/RuposhiOpenLabs/bornomala`  
**Target Systems:** Windows 10, Windows 11 (x86_64, i686, ARM64)  
**Primary Engine:** Pure Rust (`bornomala-engine`)  

---

## 1. Executive Summary & Provenance

Bornomala provides a unified, offline, secure Bangla input engine featuring:
1. **Classic Fixed Layout (Unicode Output):** Compatible with standard Bijoy layout typing, producing canonical Unicode Bangla (`U+0980..U+09FF`).
2. **Classic Fixed Layout (Legacy ANSI Output):** Produces legacy 8-bit glyph sequences designed for classic Bangla fonts (e.g., SutonnyMJ), allowing backwards compatibility with existing government and publishing documents.
3. **Phonetic Typing (Banglish to Bangla):** 100% compatible with Avro Phonetic rules, enhanced with intelligent offline completion, typo correction, and local learning dictionaries.
4. **Bidirectional Converter (Unicode <-> ANSI):** High-fidelity, lossless round-trip text conversion.
5. **Government-Office Conveniences:** Amount in words (Bangla/English Taka & Paisa), Revised Bangla Calendar calculation, digit toggle, text expansion, clipboard conversion, and per-application scheme memory.

---

## 2. Reverse Engineering Evidence & Binary Inventory

### 2.1 Shipped Artifact Identification (Private Analysis Workspace)
* **Archive:** `Bijoy-Bayanno.rar` (SHA-256: `e1e09c67d6e96e2962c18c76095ddeb39191d2a0bcee23e8cc356c688baf5050`)
* **Extracted Contents:**
  * `Bayanno2011.exe` (PE32 GUI, 468,992 bytes, Intel i386) — Native Visual Studio bootstrapper setup installer.
  * `BijoyBayanno.msi` (MSI installer, 5,651,968 bytes) — Microsoft Installer package.
  * `_B8DA7F486B83B4A2D4D10028E8357DC3` (Cabinet file within MSI, 4,866,876 bytes).
* **Target Managed Assemblies within Cabinet:**
  * `_2FFA079313409FE9A09BC8F459F5F807` (`BijoyBayanno.exe`):
    * Architecture: PE32 (Mono/.Net Assembly, CLR v2.0.50727, x86).
    * SHA-256: `e965eee14f02db4129faab6e02a3978ffcad934c6e2e44f187381dd093ce7437`.
    * Assembly: `BijoyBayanno`, Version `1.0.0.2`, MVID: `5a55b96c-39ff-4eb7-9d4a-3f6aaeef0c8e`.
  * `_4C9BC474E6694AA78E85EA1917929847` (`Converter.exe`):
    * Architecture: PE32 (Mono/.Net Assembly, CLR v2.0.50727, x86).
    * Types: `Converter.Form1`, methods: `findNext`, `findNextJukta`, `btnOpen_Click_1`.
  * Embedded XML Layouts in Cabinet:
    * `_3A36C45659244B5BA002D3349CFC471C` (`Unicode.xml`, 6,847 bytes) — 41 key entries, 35 juks.
    * `_B5E87564ED5F4F189D34C7FE926BE48E` (`Bijoy Classic.xml`, 33,624 bytes) — 34 key entries, 431 juks.

### 2.2 Input Capture Mechanism of Reference Program
* **Observation:** `BijoyBayanno.modKeyboard` uses `SetWindowsHookEx` with `WH_KEYBOARD_LL` (id 13) and `keybd_event` / `SendInput` (token `0x06000033`).
* **Evaluation:** Low-level global keyboard hooks are brittle, trigger AV heuristics, and fail in elevated applications.
* **Bornomala Architecture:** Replaces hooks with a modern **Windows Text Services Framework (TSF)** COM DLL (`ITfTextInputProcessor`) backed by a pure, platform-independent Rust engine.

---

## 3. Keyboard Layouts Specification

### 3.1 Classic Fixed Layout (Physical Key Mappings)

The physical keyboard layout follows the 4-row standard layout:

| KeyCode | Physical Key | Normal (Unicode) | Shift (Unicode) | Normal (ANSI / SutonnyMJ) | Shift (ANSI / SutonnyMJ) | Confidence |
|---|---|---|---|---|---|---|
| 48 | `0` | ০ (U+09E6) | ) | 0 | ) | Level B |
| 49 | `1` | ১ (U+09E7) | ! | 1 | ! | Level B |
| 50 | `2` | ২ (U+09E8) | ৥ (U+09E5) | 2 | @ | Level B |
| 51 | `3` | ৩ (U+09E9) | # | 3 | # | Level B |
| 52 | `4` | ৪ (U+09EA) | ৳ (U+09F3) | 4 | $ (৳) | Level B |
| 53 | `5` | ৫ (U+09EB) | % | 5 | % | Level B |
| 54 | `6` | ৬ (U+09EC) | ৲ (U+09F2) | 6 | ^ | Level B |
| 55 | `7` | ৭ (U+09ED) | ঁ (U+0981) | 7 | u (ঁ) | Level B |
| 56 | `8` | ৮ (U+09EE) | * | 8 | * | Level B |
| 57 | `9` | ৯ (U+09EF) | ( | 9 | ( | Level B |
| 65 | `A` | ৃ (U+09C3) | র্ (U+09B0+U+09CD) | „ | © | Level B |
| 66 | `B` | ন (U+09A8) | ণ (U+09A3) | b | Y | Level B |
| 67 | `C` | ে (U+09C7) | ৈ (U+09C8) | ‡ | ‰ | Level B |
| 68 | `D` | ি (U+09BF) | ী (U+09C0) | w | x | Level B |
| 69 | `E` | ড (U+09A1) | ঢ (U+09A2) | W | X | Level B |
| 70 | `F` | া (U+09BE) | অ (U+0985) | v | A | Level B |
| 71 | `G` | ্ (U+09CD) | । (U+0964) | & | \| | Level B |
| 72 | `H` | ব (U+09AC) | ভ (U+09AD) | e | f | Level B |
| 73 | `I` | হ (U+09B9) | ঞ (U+099E) | n | T | Level B |
| 74 | `J` | ক (U+0995) | খ (U+0996) | K | L | Level B |
| 75 | `K` | ত (U+09A4) | থ (U+09A5) | Z | _ | Level B |
| 76 | `L` | দ (U+09A6) | ধ (U+09A7) | ` | a | Level B |
| 77 | `M` | শ (U+09B6) | ষ (U+09B7) | g | k | Level B |
| 78 | `N` | স (U+09B8) | ণ (U+09A3) | c | Y | Level B |
| 79 | `O` | গ (U+0997) | ঘ (U+0998) | M | N | Level B |
| 80 | `P` | ড় (U+09DC) | ঢ় (U+09DD) | r | s | Level B |
| 81 | `Q` | ঙ (U+0999) | ং (U+0982) | O | P | Level B |
| 82 | `R` | প (U+09AA) | ফ (U+09AB) | e | f | Level B |
| 83 | `S` | ু (U+09C1) | ূ (U+09C2) | y | ~ | Level B |
| 84 | `T` | ট (U+099F) | ঠ (U+09A0) | U | V | Level B |
| 85 | `U` | জ (U+099C) | ঝ (U+099D) | R | S | Level B |
| 86 | `V` | র (U+09B0) | ল (U+09B2) | l | m | Level B |
| 87 | `W` | য (U+09AF) | য় (U+09DF) | j | t | Level B |
| 88 | `X` | ও (U+0993) | ঔ (U+0994) | I | J | Level B |
| 89 | `Y` | চ (U+099A) | ছ (U+099B) | P | Q | Level B |
| 90 | `Z` | ্য (U+09CD+U+09AF)| ্র (U+09CD+U+09B0)| ¨ | ª | Level B |

---

## 4. Unicode Typing & Reordering Rules

### 4.1 Pre-Base Vowel Signs
In Bangla typing tradition, users type vowel signs in visual order:
* **i-kar (ি / U+09BF)**: Typed key `d`, stored in Unicode *after* the consonant cluster.
  * e.g., `d` + `j` -> `ক` + `ি` (`কি`, U+0995 U+09BF).
* **e-kar (ে / U+09C7)**: Typed key `c`, stored in Unicode *after* the consonant cluster.
  * e.g., `c` + `j` -> `ক` + `ে` (`কে`, U+0995 U+09C7).
* **oi-kar (ৈ / U+09C8)**: Typed key `C` (Shift+c), stored in Unicode *after* consonant cluster.
  * e.g., `C` + `j` -> `ক` + `ৈ` (`কৈ`, U+0995 U+09C8).

### 4.2 Split Vowel Signs
* **o-kar (ো / U+09CB)**: Typed as e-kar (`c`) + Consonant + aa-kar (`f`):
  * Sequence: `c` + `j` + `f` -> `ক` + `ো` (`কো`, U+0995 U+09CB).
* **ou-kar (ৌ / U+09CC)**: Typed as e-kar (`c`) + Consonant + ou-kar mark (`X`):
  * Sequence: `c` + `j` + `X` -> `ক` + `ৌ` (`কৌ`, U+0995 U+09CC).

### 4.3 Reph and Ya-phala / Ra-phala
* **Reph (`র্`)**: Supports natural retroactive typing (consonant + `Shift+a`) and proactive fallback (`Shift+a` + consonant):
  * Natural sequence: `j` + `m` + `A` -> `ক` + `র` + `্` + `ম` (`কর্ম`, U+0995 U+09B0 U+09CD U+09AE).
  * Single key sequence: `j` + `A` -> `র` + `্` + `ক` (`র্ক`, U+09B0 U+09CD U+0995).
  * Proactive fallback: `A` + `j` -> `র` + `্` + `ক` (`র্ক`, U+09B0 U+09CD U+0995).
* **Ya-phala (`্য`)**: Typed as `Z` (key 90) or `g` (hasanta) + `w` (ya):
  * Sequence: `j` + `Z` -> `ক` + `্` + `য` (`ক্য`, U+0995 U+09CD U+09AF).
* **Ra-phala (`্র`)**: Typed as `z` (Shift+z) or `g` (hasanta) + `v` (ra):
  * Sequence: `j` + `z` -> `ক` + `্` + `র` (`ক্র`, U+0995 U+09CD U+09B0).

### 4.4 Hasanta Conjunct Formation
* Typing `Consonant_1` + `g` (hasanta / U+09CD) + `Consonant_2` produces canonical conjunct:
  * `j` + `g` + `j` -> `ক` + `্` + `ক` (`ক্ক`, U+0995 U+09CD U+0995).
  * `j` + `g` + `M` -> `ক` + `্` + `ষ` (`ক্ষ`, U+0995 U+09CD U+09B7).
  * `u` + `g` + `I` -> `জ` + `্` + `ঞ` (`জ্ঞ`, U+099C U+09CD U+099E).

---

## 5. Bidirectional Unicode <-> ANSI (SutonnyMJ) Converter

### 5.1 Unicode to ANSI Transformation Steps
1. **Reph Repositioning:** Detect `র` + `্` at the beginning of cluster and append `©` (SutonnyMJ Reph) after the base cluster and its vowel signs.
2. **Pre-Base Signs Extraction:** Locate `ি`, `ে`, `ৈ` after consonant clusters, remove them from post-base position, and insert their corresponding SutonnyMJ glyphs (`w`, `‡`, `‰`) before the cluster.
3. **Split Signs Expansion:** Replace `ো` with `‡` before cluster and `v` after; replace `ৌ` with `‡` before cluster and `Š` after.
4. **Conjunct Glyph Mapping:** Scan pairs or triplets connected by hasanta (`্`) and replace with single SutonnyMJ glyphs (e.g. `ক`+`্`+`ক` -> `³`, `ক`+`্`+`ষ` -> `´`, `জ`+`্`+`ঞ` -> `Ä`).
5. **Direct Character Mapping:** Map all remaining independent vowels, consonants, numbers, and signs.

### 5.2 ANSI to Unicode Transformation Steps
1. **Scan Ahead Pre-Base Glyphs:** When encountering `w`, `‡`, `‰`, scan forward over consonants, conjuncts, and hasanta links to locate the full base cluster, then emit the base followed by the vowel sign (`ি`, `ে`, `ৈ`).
2. **Merge Split Signs:** When encountering `‡` followed by cluster and `v` / `Š`, emit cluster followed by `ো` / `ৌ`.
3. **Decompose Conjunct Glyphs:** Map SutonnyMJ conjunct glyphs back to canonical Unicode consonant + hasanta sequences.
4. **Restore Reph:** Move any trailing `©` to the front of the cluster as `র` + `্`.
5. **Normalize:** Run standard Unicode NFC normalization to guarantee lossless round trips.

---

## 6. Phonetic Engine (Avro Compatible)

* **Architecture:** Longest-prefix match rule engine with position-sensitive vowel/kar transformation, hasanta joining, and escape processing.
* **Compatibility:** Tested and validated against `ibus-avro` (`avrolib.js`) as an external oracle (Confidence Level A).
* **Sample Transformations:**
  * `ami` -> `আমি`
  * `bangla` -> `বাংলা`
  * `amader` -> `আমাদের`
  * `shonar` -> `সোনার`
  * `Dhaka` -> `ঢাকা`
  * `biddaloy` -> `বিদ্যালয়`
  * `kormo` -> `কর্ম`
* **Local Learning Dictionary:** User additions saved locally to unencrypted JSON format; no telemetry or cloud sync.

---

## 7. Government Office Conveniences

1. **Amount in Words (টাকা ও পয়সা):**
   * Input: Number (e.g. `1234.50` or `১২৩৪.৫০`).
   * Bangla Output: `এক হাজার দুই শত চৌত্রিশ টাকা পঞ্চাশ পয়সা মাত্র`
   * English Output: `One Thousand Two Hundred Thirty-Four Taka and Fifty Paisa Only`
2. **Revised Bangla Calendar Helper:**
   * Official Bangladesh standard:
     * Baishakh to Bhadra (months 1–5): 31 days each.
     * Ashwin to Falgun (months 6–11): 30 days each.
     * Chaitra (month 12): 30 days (31 days in Gregorian leap years).
   * Gregorian <-> Bangla Date bidirectional conversion.
3. **Digit Switch:** Instant toggle between English (`0-9`) and Bangla (`০-৯`) numerals.
4. **Text Expander:** Expand shortcut triggers (e.g., `#gov1`) into official government headers, memo templates, and signatures.
5. **Clipboard Quick Converter:** Convert clipboard contents between Unicode and ANSI with one hotkey (`Ctrl + Shift + F11`).
6. **Per-Application Mode Memory:** Remembers active scheme per target application window handle.
7. **Auto-Startup & Startup Splash Screen:**
   * Replicates classic Ekushe and Bijoy Bayanno visual identity: centered branded splash window displaying logo, version, taglines, publisher, developer credits, and support email.
   * Auto-closes after 2500 ms (2.5 seconds) countdown, leaving Bornomala running quietly in the tray and TSF.
   * Auto-startup registration in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

---

## 8. Verification & Confidence Register

* **Level A (Oracle Verified):** Phonetic rule sets, suggestion generation, dictionary validation.
* **Level B (Dual-Static Verified):** Fixed Unicode layout mappings, Fixed ANSI layout mappings, SutonnyMJ conjunct glyph substitutions, Converter single-char mappings.
* **Level C (Windows Verification Required - see `WINDOWS_VERIFICATION.md`):**
  * TSF COM registration via `HKCU`.
  * Topmost non-activating floating bar window styles on Windows 10/11 (`WS_EX_NOACTIVATE | WS_EX_TOPMOST`).
  * System tray Shell_NotifyIconW interop.
  * Windows Registry auto-startup on logon (`HKCU\...\Run`).
  * Auto-startup splash screen display and auto-hide timing on physical display.
