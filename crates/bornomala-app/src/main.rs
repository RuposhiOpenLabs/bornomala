#![windows_subsystem = "windows"]

//! Bornomala (বর্ণমালা) — Native Windows Graphical Application
//!
//! Provides:
//! 1. Authentic Bengali Startup Splash Screen with 2.5s auto-hide and click dismiss.
//! 2. Floating Top Toolbar (WS_EX_TOPMOST, WS_EX_NOACTIVATE, draggable, mode switcher).
//! 3. Native Graphical Converter Window (dual multiline Unicode ⇄ SutonnyMJ ANSI).
//! 4. Windows Notification Area System Tray Icon with status tooltip and context menu.
//! 5. Global Hotkeys (F12, Ctrl+Alt+V, Ctrl+Alt+B, Ctrl+Alt+E) and WH_KEYBOARD_LL typing engine.
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
