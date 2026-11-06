# Bornomala (বর্ণমালা) — User Manual & Keyboard Guide

**Release:** v0.1.0-alpha  
**Publisher:** [Ruposhi Open Labs](https://github.com/RuposhiOpenLabs)  
**Developer:** [Ryan Shelby](https://github.com/MdSagorMunshi/)  
**Support Email:** [ryn@disr.it](mailto:ryn@disr.it)  

---

## 1. Quick Start Guide (দ্রুত শুরু)

Bornomala provides seamless Bangla typing across all applications on Windows 10 and 11.

### Hotkey Summary (শর্টকাটসমূহ)

| Hotkey | Action / Mode | বাংলা বিবরণ |
|---|---|---|
| **F12** | Toggle Phonetic (Banglish) / English | ফোনেটিক মোড চালু বা বন্ধ |
| **Ctrl + Alt + V** | Switch to Classic Unicode Mode | ক্লাসিক ইউনিকোড ফিক্সড লেআউট |
| **Ctrl + Alt + B** | Switch to Classic ANSI (SutonnyMJ) Mode | ক্লাসিক এএনএসআই (সুতোন্নী ফন্ট) |
| **Ctrl + Shift + F11** | Quick Convert Clipboard Text | ক্লিপবোর্ডের টেক্সট রূপান্তর |
| **Ctrl + Shift + F12** | Toggle Bangla / English Numerals | সংখ্যা পরিবর্তন (০-৯ ⇄ 0-9) |

---

## 2. Typing Modes (টাইপিং মোডসমূহ)

### 2.1 Classic Fixed Layout (Unicode) — `Ctrl + Alt + V`
This mode uses the traditional Bijoy-style physical keyboard layout but outputs standard, modern Unicode Bangla.

* **Typing Basic Letters:**
  * Press `j` -> `ক`
  * Press `Shift + j` -> `খ`
  * Press `o` -> `গ`
  * Press `Shift + o` -> `ঘ`
* **Typing Vowels with Consonants (কার চিহ্ন):**
  * Type in traditional visual order:
  * `d` + `j` -> `কি` (i-kar before consonant)
  * `c` + `j` -> `কে` (e-kar before consonant)
  * `Shift + c` + `j` -> `কৈ` (oi-kar before consonant)
  * `c` + `j` + `f` -> `কো` (o-kar: e-kar + consonant + a-kar)
  * `c` + `j` + `Shift + x` -> `কৌ` (ou-kar)
* **Typing Conjuncts (যুক্তবর্ণ):**
  * Type first consonant + `g` (hasanta / ্) + second consonant:
  * `j` + `g` + `j` -> `ক্ক` (ক + ক)
  * `j` + `g` + `m` -> `ক্ষ` (ক + ষ)
  * `u` + `g` + `Shift + i` -> `জ্ঞ` (জ + ঞ)
  * `k` + `g` + `k` -> `ত্ত` (ত + ত)
* **Reph and Phala (রেফ ও ফলা):**
  * **Reph (`র্`):** Type consonant + `Shift + a` (natural retroactive flow, e.g. `কম` + `Shift + a` -> `কর্ম`, `j` + `Shift + a` -> `র্ক`), or `Shift + a` before consonant as proactive fallback.
  * **Ya-phala (`্য`):** Type consonant + `z`:
    * `j` + `z` -> `ক্য`
  * **Ra-phala (`্র`):** Type consonant + `Shift + z`:
    * `j` + `Shift + z` -> `ক্র`

---

### 2.2 Classic Fixed Layout (ANSI / SutonnyMJ) — `Ctrl + Alt + B`
Use this mode when working on legacy government forms, land records, or publishing templates where the active font is set to **SutonnyMJ**.

> [!IMPORTANT]
> When typing in Classic ANSI mode, make sure your word processor font is set to **SutonnyMJ** or a compatible legacy ANSI font. If you see English characters like `Avgvi` instead of Bangla, simply select the text and change the font to `SutonnyMJ`.

---

### 2.3 Avro Phonetic Mode — `F12`
Type Bangla using natural English letters (transliteration):

| What you type | What Bornomala outputs |
|---|---|
| `ami banglay gan gai` | `আমি বাংলায় গান গাই` |
| `amader shonar bangladesh` | `আমাদের সোনার বাংলাদেশ` |
| `Dhaka` | `ঢাকা` |
| `bidyaloy` | `বিদ্যালয়` |
| `korrmo` | `কর্ম` |
| `sOnar` | `সোনার` |

---

## 3. Printable Keyboard Cheat-Sheet

```
+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-------+
|  ~  |  !  |  ৥  |  #  |  ৳  |  %  |  ৲  |  ঁ  |  *  |  (  |  )  |  _  |  +  | BACK  |
|  `  |  ১  |  ২  |  ৩  |  ৪  |  ৫  |  ৬  |  ৭  |  ৮  |  ৯  |  ০  |  -  |  =  | SPACE |
+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-------+
| TAB   |  ং  |  য়  |  ঢ  |  ফ  |  ঠ  |  ছ  |  ঝ  |  ঞ  |  ঘ  |  ঢ়  |  {  |  }  |   |   |
|       |  ঙ  |  য  |  ড  |  প  |  ট  |  চ  |  জ  |  হ  |  গ  |  ড়  |  [  |  ]  |   \   |
+-------+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-------+
| CAPS    |  র্  |  ূ  |  ী  |  অ  |  ।  |  ভ  |  খ  |  থ  |  ধ  |  :  |  "  | ENTER   |
|         |  ৃ  |  ু  |  ি  |  া  |  ্  |  ব  |  ক  |  ত  |  দ  |  ;  |  '  |         |
+---------+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----------+
| SHIFT     |  ্র  |  ঔ  |  ৈ  |  ল  |  ণ  |  ষ  |  শ  |  <  |  >  |  ?  | SHIFT     |
|           |  ্য  |  ও  |  ে  |  র  |  ন  |  স  |  ণ  |  ,  |  .  |  /  |           |
+-----------+-----+-----+-----+-----+-----+-----+-----+-----+-----+-----+---------------+
| CTRL  | ALT   |                 SPACEBAR                 | ALTGR | ্ (71) | । (71S)|
+-------+-------+------------------------------------------+-------+--------+--------+
```

---

## 4. Government Office Conveniences

### 4.1 Amount in Words (টাকার পরিমাণ কথায় লেখা)
When drafting bank vouchers, cheques, tenders, or government budget notes:
* Type a currency number (e.g. `125000.50`) and press the converter hotkey.
* **Bangla Output:** `এক লাখ পঁচিশ হাজার টাকা পঞ্চাশ পয়সা মাত্র`
* **English Output:** `One Lakh Twenty-Five Thousand Taka and Fifty Paisa Only`

### 4.2 Revised Bangla Calendar Helper (সরকারি বাংলা ক্যালেন্ডার)
Computes exact dates according to the 2019 Bangladesh Cabinet revision:
* Baishakh to Ashwin (Months 1–6): 31 days each.
* Kartik to Magh (Months 7–10): 30 days each.
* Falgun (Month 11): 29 days (30 days in leap years).
* Chaitra (Month 12): 30 days.

### 4.3 Official Text Expander
Quickly insert official memo headers and signature blocks by typing short abbreviations:
* `#gov1` -> `গণপ্রজাতন্ত্রী বাংলাদেশ সরকার`
* `#memo` -> `স্মারক নং:`
* `#sig`  -> Signature block template with date and designation.

---

## 5. Troubleshooting & FAQ

**Q: I typed in Microsoft Word and see English symbols like `Avgvi †mvbvi` instead of Bangla.**  
**A:** You are typing in Classic ANSI mode with an English font selected. Select the text and choose `SutonnyMJ` from your font dropdown. Or, if you want modern standard Bangla, switch to **Classic Unicode** (`Ctrl+Alt+V`).

**Q: Does Bornomala need internet access?**  
**A:** No. Bornomala is 100% offline. It has zero network code and will never send your keystrokes anywhere.

**Q: What if Bornomala encounters an issue in an application?**  
**A:** Bornomala has an automated safety watchdog. If 5 consecutive errors occur, it safely bypasses itself into pass-through mode so your active document never hangs or crashes.

---

## 6. Auto Startup & Classic Splash Screen

Bornomala replicates the beloved, familiar visual experience of classic tools like **Bijoy Bayanno** and **Ekushe**:

### 6.1 Startup Splash Screen
* When Bornomala starts up (either automatically when Windows boots or when you manually open the application), an elegant branded splash screen popup appears centered on your screen.
* **Bengali Version by Default:** By default, the startup screen displays the rich Bengali version with authentic typography and artwork (`assets/branding/startup_screen_bn.png`), matching the familiar Ekushe and Bijoy Bayanno experience.
* The splash screen displays the Bornomala logo, version information, publisher & developer credits, and verifies typing engine readiness.
* **Auto-Hide:** After approximately **2.5 seconds** (2500 ms), the startup screen automatically closes itself, leaving the Bornomala floating bar and system tray icon ready in the background.
* **Instant Dismiss:** Typists in a hurry can click the `✕` close button in the top-right corner to dismiss the splash immediately.
* **Asset Location:** Official high-resolution assets are organized in `assets/branding/` (`startup_screen_bn.png`, `startup_screen_en.png`, `logo.png`).

### 6.2 Managing Windows Auto-Startup
Bornomala configures auto-startup per-user without requiring Administrator privileges:
* **Enable Auto-Startup via Script:**
  Double-click `scripts\enable_autostart.bat`. This registers Bornomala under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
* **Disable Auto-Startup via Script:**
  Double-click `scripts\disable_autostart.bat`.
* **Settings Toggle:**
  Open Bornomala Settings (`⚙`) and switch the "Start Bornomala with Windows" or "Show Startup Splash Screen" toggle.
* **CLI Management:**
  ```bash
  bornomala-cli autostart enable
  bornomala-cli autostart disable
  bornomala-cli autostart status
  ```

---

## 7. Windows Installer & Preloaded Fonts (ইনস্টলেশন ও প্রিলোডেড ফন্ট)

### 7.1 Windows Installer Package (`Bornomala-Setup-v0.1.0.exe`)
Bornomala provides a lightweight, native 64-bit Windows setup wizard built with NSIS:
* **Zero Administrator Privileges Required:** Installs directly to `%LOCALAPPDATA%\Bornomala` and registers per-user COM text services, allowing use on restricted government or school workstations.
* **Modern Animated UI / UX:** Includes lazy-animated transition cards, progress bar shimmer, and clean step-by-step guidance.
* **Professional Terms & Conditions:** Built-in EULA and offline privacy pledge guarantee that your keystrokes are never logged or transmitted.

### 7.2 Preloaded Fonts & Conflict Resolution
The installer bundles 8 official government and standard Bangla fonts:
1. `Nikosh.ttf` — Bangladesh Government standard Unicode font
2. `NikoshBAN.ttf` — Official typography variant
3. `NikoshGrameen.ttf` — Clear high-legibility display variant
4. `NikoshLight.ttf` — Ultra-clean light weight
5. `NikoshLightBan.ttf` — Clean print typography
6. `SutonnyMJ Regular.ttf` — Classic ANSI font for land records and government archives
7. `AdorshoLipi_20-07-2007.ttf` — Standard educational and literary font
8. `Bangla.ttf` — Universal traditional OpenType font

**Conflict Detection Popup:**
If any font already exists on your machine, Bornomala presents an interactive conflict dialog:
* **[Yes / হ্যাঁ]**: Replace with Bornomala's verified preloaded font.
* **[No / না]**: Keep your computer's existing font file untouched.

**Zero-Reboot Font Refresh:**
The installer broadcasts `WM_FONTCHANGE` upon completion so MS Word, Adobe Photoshop, InDesign, and browsers immediately recognize the installed fonts without needing a system restart.

---

## 8. Support, Community & Security Contact
* **GitHub Organization:** [Ruposhi Open Labs](https://github.com/RuposhiOpenLabs)
* **Lead Developer:** [Ryan Shelby](https://github.com/MdSagorMunshi/)
* **Direct Support & Inquiries:** [ryn@disr.it](mailto:ryn@disr.it)
* **Source Repository:** [github.com/RuposhiOpenLabs/bornomala](https://github.com/RuposhiOpenLabs/bornomala)
