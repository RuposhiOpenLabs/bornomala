#![windows_subsystem = "windows"]

//! Bornomala (বর্ণমালা) — Native Windows Graphical Application
//!
//! Provides:
//! 1. Authentic Bengali Startup Splash Screen with 2.5s auto-hide and click dismiss.
//! 2. Floating Top Toolbar (WS_EX_TOPMOST, WS_EX_NOACTIVATE, draggable, mode switcher).
//! 3. Native Graphical Converter Window (dual multiline Unicode ⇄ SutonnyMJ ANSI).
//! 4. Windows Notification Area System Tray Icon with status tooltip and context menu.
//! 5. Global Hotkeys (F12, Ctrl+Alt+V, Ctrl+Alt+B, Ctrl+Alt+E) and WH_KEYBOARD_LL typing engine.
//! 6. Secure Cryptographic Auto-Updater with live download speed, Ed25519 verification, and UAC Update Wizard.
//!
//! Publisher: Ruposhi Open Labs (https://github.com/RuposhiOpenLabs)
//! Lead Developer: Ryan Shelby (https://github.com/MdSagorMunshi/)
//! Support: ryn@disr.it

#[cfg(target_os = "windows")]
mod win32;

#[cfg(not(target_os = "windows"))]
mod stub;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // 1. Elevated Update Wizard Mode (--update-wizard <payload_path> <new_version>)
    if let Some(pos) = args.iter().position(|a| a == "--update-wizard") {
        let payload = args.get(pos + 1).cloned().unwrap_or_default();
        let target_ver = args.get(pos + 2).cloned().unwrap_or_else(|| "0.2.0".into());
        #[cfg(target_os = "windows")]
        {
            unsafe {
                win32::updater::run_update_wizard(&payload, &target_ver);
            }
            return;
        }
        #[cfg(not(target_os = "windows"))]
        {
            println!("Update wizard for {} target: {}", payload, target_ver);
            return;
        }
    }

    let converter_only = args.iter().any(|a| a == "--converter" || a == "-c");
    let startup_mode = args.iter().any(|a| a == "--startup" || a == "-s");

    #[cfg(target_os = "windows")]
    {
        win32::run_app(converter_only, startup_mode);
    }

    #[cfg(not(target_os = "windows"))]
    {
        stub::run_app(converter_only, startup_mode);
    }
}
