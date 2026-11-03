use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub const REGISTRY_RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
pub const REGISTRY_VALUE_NAME: &str = "Bornomala";
pub const DEFAULT_SPLASH_DURATION_MS: u64 = 2500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupOptions {
    pub is_system_startup: bool,
    pub show_splash: bool,
    pub minimized_to_tray: bool,
    pub splash_duration_ms: u64,
}

impl Default for StartupOptions {
    fn default() -> Self {
        Self {
            is_system_startup: false,
            show_splash: true,
            minimized_to_tray: false,
            splash_duration_ms: DEFAULT_SPLASH_DURATION_MS,
        }
    }
}

impl StartupOptions {
    pub fn from_args<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut opts = Self::default();
        for arg in args {
            match arg.as_ref() {
                "--startup" | "-startup" => {
                    opts.is_system_startup = true;
                }
                "--splash" => {
                    opts.show_splash = true;
                }
                "--no-splash" | "--silent" => {
                    opts.show_splash = false;
                }
                "--minimized" | "--tray" => {
                    opts.minimized_to_tray = true;
                }
                _ => {}
            }
        }
        opts
    }

    pub fn should_show_splash_screen(&self) -> bool {
        self.show_splash && !self.minimized_to_tray
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutostartManager {
    pub app_name: String,
    pub exe_path: PathBuf,
    pub enabled: bool,
    pub show_splash_on_startup: bool,
}

impl Default for AutostartManager {
    fn default() -> Self {
        Self {
            app_name: REGISTRY_VALUE_NAME.to_string(),
            exe_path: PathBuf::from(r"C:\Program Files\Bornomala\bornomala.exe"),
            enabled: true,
            show_splash_on_startup: true,
        }
    }
}

impl AutostartManager {
    pub fn new(exe_path: impl Into<PathBuf>) -> Self {
        Self {
            app_name: REGISTRY_VALUE_NAME.to_string(),
            exe_path: exe_path.into(),
            enabled: true,
            show_splash_on_startup: true,
        }
    }

    pub fn generate_enable_command(&self) -> String {
        let path_str = self.exe_path.to_string_lossy();
        let cmd_payload = if self.show_splash_on_startup {
            format!("\"{}\" --startup", path_str)
        } else {
            format!("\"{}\" --startup --no-splash", path_str)
        };
        format!(
            "reg add \"{}\" /v \"{}\" /t REG_SZ /d \"{}\" /f",
            REGISTRY_RUN_KEY, self.app_name, cmd_payload
        )
    }

    pub fn generate_disable_command(&self) -> String {
        format!(
            "reg delete \"{}\" /v \"{}\" /f",
            REGISTRY_RUN_KEY, self.app_name
        )
    }

    pub fn generate_query_command(&self) -> String {
        format!(
            "reg query \"{}\" /v \"{}\"",
            REGISTRY_RUN_KEY, self.app_name
        )
    }
}

pub fn generate_enable_bat_script(exe_path_or_default: Option<&Path>) -> String {
    let path = exe_path_or_default
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| r"%LOCALAPPDATA%\Bornomala\bornomala.exe".to_string());

    format!(
        r#"@echo off
REM Bornomala (বর্ণমালা) — Enable Auto Startup
REM Registers Bornomala in HKCU\Software\Microsoft\Windows\CurrentVersion\Run
echo Registering Bornomala auto-startup...
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "Bornomala" /t REG_SZ /d "\"{}\" --startup" /f
if %ERRORLEVEL% EQU 0 (
    echo [SUCCESS] Bornomala will now automatically start on Windows logon with startup splash.
) else (
    echo [ERROR] Failed to register auto-startup registry key.
)
"#,
        path
    )
}

pub fn generate_disable_bat_script() -> String {
    r#"@echo off
REM Bornomala (বর্ণমালা) — Disable Auto Startup
REM Removes Bornomala from HKCU\Software\Microsoft\Windows\CurrentVersion\Run
echo Removing Bornomala auto-startup...
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "Bornomala" /f >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [SUCCESS] Bornomala auto-startup successfully disabled.
) else (
    echo [INFO] Bornomala was not registered in startup run list.
)
"#.to_string()
}
