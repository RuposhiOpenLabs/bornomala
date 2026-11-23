# Bornomala (বর্ণমালা) — Windows Verification Register

**Document Version:** 1.0.0  
**Publisher:** [Ruposhi Open Labs](https://github.com/RuposhiOpenLabs) &nbsp;|&nbsp; **Developer:** [Ryan Shelby](https://github.com/MdSagorMunshi/) &nbsp;|&nbsp; **Support:** [ryn@disr.it](mailto:ryn@disr.it)  
**Target Platform:** Windows 10 & Windows 11 (x86_64, i686, ARM64)  
**Host Development Environment:** Kali Linux x86_64 (Linux-direct cross-compilation; no VM used)  
**Status:** Pre-Release Cross-Compiled (Clean builds, 100% Linux unit & vector test pass; awaiting physical Windows hardware test)

---

## 1. Context & Purpose

This document provides a rigorous, transparent inventory of Windows-specific behaviors that have been cross-compiled and tested against simulated host abstractions on Linux, but which require physical validation on an actual Windows 10/11 desktop.

In accordance with our strict evidence discipline:
* **Level A (Oracle Verified on Linux):** Phonetic rule sets, suggestion generation, dictionary validation.
* **Level B (Dual-Static Verified on Linux):** Fixed Unicode layout mappings, Fixed ANSI layout mappings, SutonnyMJ conjunct glyph substitutions, Converter single-character mappings.
* **Level C (Windows Verification Required):** Windows OS subsystem interactions described below.

---

## 2. Level C Verification Checklist for Windows Hardware

| Item ID | Component | Description & Expected Behavior | Verification Command / Procedure | Status |
|---|---|---|---|---|
| **WIN-01** | `bornomala-tsf` | Per-user non-elevated COM registration in `HKCU\Software\Classes\CLSID` without UAC prompt. | Run `scripts\register_user.bat` from standard user shell. | ⏳ Pending Physical Win10/11 |
| **WIN-02** | `bornomala-tsf` | System-wide COM registration in `HKLM` for all domain/office users. | Run `scripts\register_admin.bat` from elevated shell. | ⏳ Pending Physical Win10/11 |
| **WIN-03** | `ITfTextInputProcessor` | TSF Profile activation in Microsoft Word 2016/2019/365, Notepad, and Chrome. | Type in Word: `d` + `j` -> `কি`; verify caret and inline composition underline. | ⏳ Pending Physical Win10/11 |
| **WIN-04** | `WS_EX_NOACTIVATE` | Floating Bar does not steal keyboard focus from active document window when clicked. | Click Floating Bar scheme badge while typing in Notepad; verify typing continues in Notepad. | ⏳ Pending Physical Win10/11 |
| **WIN-05** | `Shell_NotifyIconW` | Tray icon updates scheme badge (EN, UNI, ANSI, PHO) and displays quiet pass-through alerts. | Trip watchdog with 5 consecutive faults; verify tray balloon shows fail-safe warning. | ⏳ Pending Physical Win10/11 |
| **WIN-06** | COM Unregister | Clean removal of all registry keys and TSF profile references upon uninstallation. | Run `scripts\unregister.bat`; verify `reg query HKCU\Software\Classes\CLSID\{7B8961D0...}` returns error. | ⏳ Pending Physical Win10/11 |
| **WIN-07** | High-DPI Scaling | Floating Bar and On-Screen Keyboard scale crisply at 125%, 150%, and 200% Windows scaling. | Test on 4K display or laptop at 150% scaling in Windows Settings. | ⏳ Pending Physical Win10/11 |
| **WIN-08** | ARM64 Emulation / Native | Native execution on Windows on Snapdragon ARM64 devices (`aarch64-pc-windows-gnullvm`). | Test on Surface Pro 9 / 11 (ARM64) running Windows 11. | ⏳ Pending Physical Win10/11 |
| **WIN-09** | Windows Auto-Startup | Auto-launch on Windows logon via `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. | Run `scripts\enable_autostart.bat`, log off and log in; verify Bornomala loads on logon. | ⏳ Pending Physical Win10/11 |
| **WIN-10** | Startup Splash Screen | Centered Bijoy/Ekushe-style splash popup appears on boot/launch and auto-hides after 2.5s. | Launch `bornomala.exe --startup`; verify centered splash shows and closes automatically after ~2.5s. | ⏳ Pending Physical Win10/11 |
| **WIN-11** | NSIS Setup Wizard | Full animated setup installer (`Bornomala-Setup-v1.0.1.exe`) installs core DLL, CLI, fonts, and shortcuts. | Run `Bornomala-Setup-v1.0.1.exe`, walk through pages, and check install directory. | ⏳ Pending Physical Win10/11 |
| **WIN-12** | Preloaded Font Install | Automatic registration of 10 preloaded fonts (Nikosh, SutonnyMJ, AdorshoLipi, Bangla, SumiBrush) into user fonts. | Inspect `%LOCALAPPDATA%\Microsoft\Windows\Fonts` and `HKCU\Software\Microsoft\Windows NT\CurrentVersion\Fonts`. | ⏳ Pending Physical Win10/11 |
| **WIN-13** | Font Conflict Popup | Conflict dialog prompt appears when a font (e.g., SutonnyMJ) already exists, allowing Keep or Replace. | Place existing `Nikosh.ttf` in font folder before running setup; verify popup triggers and respects choice. | ⏳ Pending Physical Win10/11 |
| **WIN-14** | `WM_FONTCHANGE` Broadcast | Applications (Word, Photoshop, Notepad) detect preloaded fonts immediately without Windows reboot. | Open Word, run font installer, verify `Nikosh` and `SutonnyMJ` appear in font dropdown immediately. | ⏳ Pending Physical Win10/11 |
| **WIN-15** | Clean Uninstallation | Complete removal of files, fonts, registry keys, and shortcuts via `Uninstall.exe`. | Run `Uninstall.exe` from Add/Remove Programs; verify clean cleanup and TSF unregistration. | ⏳ Pending Physical Win10/11 |

---

## 3. Physical Test Execution Steps

When deploying Bornomala to a physical Windows machine for the first time:

1. **Copy Binaries:**
   Copy `target/x86_64-pc-windows-gnu/debug/bornomala_tsf.dll` and the `scripts/` directory to a test folder (e.g. `C:\Bornomala`).
2. **Register COM Server:**
   Execute `scripts\register_user.bat`. Confirm `regsvr32` reports success.
3. **Verify Text Service in Windows Settings:**
   Open `Windows Settings -> Time & Language -> Language & Region -> Bengali (Bangladesh) -> Options -> Keyboards`.
   Confirm `Bornomala Bangla Input Profile` is listed.
4. **Interactive Application Testing:**
   * **Classic Unicode:** Press `Ctrl+Alt+V`, type `j` `g` `j` -> verify `ক্ক`.
   * **Classic ANSI:** Press `Ctrl+Alt+B`, set font to `SutonnyMJ`, type `j` `g` `j` -> verify `°` (`ক্ক`).
   * **Phonetic:** Press `F12`, type `ami banglay gan gai ` -> verify `আমি বাংলায় গান গাই `.
5. **Watchdog Failsafe Test:**
   Verify that rapid switching or process termination never hangs or crashes the host application.
