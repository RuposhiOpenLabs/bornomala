//! Bornomala Application Configuration & User Preferences
//!
//! Handles persistent user settings, startup preferences, custom keyboard shortcuts,
//! and phonetic typing rules.
//!
//! Stored in `%APPDATA%\Bornomala\settings.json`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomRule {
    pub trigger: String,
    pub replacement: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    // ------------------------------------------------------------------------
    // 1. General & Startup
    // ------------------------------------------------------------------------
    /// If true, the app starts minimized in the Windows system tray without showing the top toolbar.
    pub toolbar_minimized: bool,

    /// Whether Bornomala should automatically run on Windows logon.
    pub launch_on_startup: bool,

    /// Whether to display the authentic startup splash screen on launch.
    pub show_splash_screen: bool,

    /// Default typing mode when Bornomala launches:
    /// "english" (default), "unicode", "ansi", "phonetic"
    pub default_mode: String,

    // ------------------------------------------------------------------------
    // 2. Hotkeys & Shortcuts
    // ------------------------------------------------------------------------
    pub hotkey_phonetic: String,
    pub hotkey_unicode: String,
    pub hotkey_ansi: String,
    pub hotkey_english: String,
    pub hotkey_converter: String,

    // ------------------------------------------------------------------------
    // 3. Typing Rules & Features
    // ------------------------------------------------------------------------
    /// Automatically convert period `.` to Bangla Dari `।` in Bangla modes
    pub auto_dari: bool,

    /// Smart consonant conjunct handling (auto-juktoborno)
    pub smart_juktoborno: bool,

    /// Word suggestions
    pub enable_suggestions: bool,

    /// User-defined abbreviation expansion rules
    pub custom_rules: Vec<CustomRule>,

    // ------------------------------------------------------------------------
    // 4. Updates
    // ------------------------------------------------------------------------
    pub auto_check_updates: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            toolbar_minimized: false,
            launch_on_startup: true,
            show_splash_screen: true,
            default_mode: "english".to_string(),

            hotkey_phonetic: "F12".to_string(),
            hotkey_unicode: "Ctrl+Alt+V".to_string(),
            hotkey_ansi: "Ctrl+Alt+B".to_string(),
            hotkey_english: "Ctrl+Alt+E".to_string(),
            hotkey_converter: "Ctrl+Alt+C".to_string(),

            auto_dari: true,
            smart_juktoborno: true,
            enable_suggestions: true,
            custom_rules: vec![
                CustomRule {
                    trigger: "bng".to_string(),
                    replacement: "বাংলাদেশ".to_string(),
                    enabled: true,
                },
                CustomRule {
                    trigger: "dhaka".to_string(),
                    replacement: "ঢাকা".to_string(),
                    enabled: true,
                },
            ],

            auto_check_updates: true,
        }
    }
}

impl AppConfig {
    /// Returns the default path `%APPDATA%\Bornomala\settings.json` on Windows.
    pub fn default_path() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata)
                .join("Bornomala")
                .join(DEFAULT_CONFIG_FILE)
        } else {
            PathBuf::from(DEFAULT_CONFIG_FILE)
        }
    }

    /// Loads configuration from default file, or creates defaults if not found.
    pub fn load() -> Self {
        Self::load_from(&Self::default_path())
    }

    /// Loads configuration from a specified path.
    pub fn load_from(path: &Path) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                return cfg;
            }
        }
        Self::default()
    }

    /// Saves configuration to the default path.
    pub fn save(&self) -> Result<(), io::Error> {
        Self::save_to(self, &Self::default_path())
    }

    /// Saves configuration to a specified path.
    pub fn save_to(&self, path: &Path) -> Result<(), io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    /// Updates Windows Registry run key for auto-start.
    pub fn apply_autostart_registry(&self) {
        #[cfg(target_os = "windows")]
        {
            let exe_path = if let Ok(cur) = std::env::current_exe() {
                cur.to_string_lossy().to_string()
            } else {
                r"C:\Program Files\Bornomala\bornomala.exe".to_string()
            };

            if self.launch_on_startup {
                let cmd_arg = if self.toolbar_minimized {
                    format!("\"{}\" --startup --tray", exe_path)
                } else {
                    format!("\"{}\" --startup", exe_path)
                };
                let _ = std::process::Command::new("reg")
                    .args([
                        "add",
                        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                        "/v",
                        "Bornomala",
                        "/t",
                        "REG_SZ",
                        "/d",
                        &cmd_arg,
                        "/f",
                    ])
                    .status();
            } else {
                let _ = std::process::Command::new("reg")
                    .args([
                        "delete",
                        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                        "/v",
                        "Bornomala",
                        "/f",
                    ])
                    .status();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_app_config_defaults_and_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_settings.json");

        let mut config = AppConfig::default();
        assert_eq!(config.default_mode, "english");
        assert!(!config.toolbar_minimized);
        assert!(config.launch_on_startup);
        assert!(config.show_splash_screen);

        config.toolbar_minimized = true;
        config.default_mode = "phonetic".to_string();
        config.custom_rules.push(CustomRule {
            trigger: "joy".to_string(),
            replacement: "জয় বাংলা".to_string(),
            enabled: true,
        });

        config.save_to(&path).unwrap();

        let loaded = AppConfig::load_from(&path);
        assert!(loaded.toolbar_minimized);
        assert_eq!(loaded.default_mode, "phonetic");
        assert_eq!(loaded.custom_rules.len(), 3);
        assert_eq!(loaded.custom_rules[2].replacement, "জয় বাংলা");
    }
}
