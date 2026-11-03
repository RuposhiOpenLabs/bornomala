use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    HighContrast,
    LargeText,
}

#[derive(Debug, Clone)]
pub struct ThemePalette {
    pub mode: ThemeMode,
    pub primary_green: &'static str,
    pub accent_red: &'static str,
    pub accessible_red: &'static str,
    pub background: &'static str,
    pub surface: &'static str,
    pub surface_hover: &'static str,
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub border: &'static str,
    pub badge_active_bg: &'static str,
    pub badge_active_text: &'static str,
    pub font_size_base: u32,
    pub font_size_heading: u32,
    pub font_size_badge: u32,
}

impl ThemePalette {
    pub fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self {
                mode,
                primary_green: "#006A4E",
                accent_red: "#F42A41",
                accessible_red: "#C21F35",
                background: "#F7F8F6",
                surface: "#FFFFFF",
                surface_hover: "#EEF2EF",
                text_primary: "#1A241E",
                text_secondary: "#5A6B60",
                border: "#D1DCD5",
                badge_active_bg: "#006A4E",
                badge_active_text: "#FFFFFF",
                font_size_base: 14,
                font_size_heading: 18,
                font_size_badge: 12,
            },
            ThemeMode::Dark => Self {
                mode,
                primary_green: "#008764",
                accent_red: "#F42A41",
                accessible_red: "#E02E43",
                background: "#111713",
                surface: "#1B241E",
                surface_hover: "#243128",
                text_primary: "#E6EDE8",
                text_secondary: "#8B9E91",
                border: "#2C3A30",
                badge_active_bg: "#008764",
                badge_active_text: "#FFFFFF",
                font_size_base: 14,
                font_size_heading: 18,
                font_size_badge: 12,
            },
            ThemeMode::HighContrast => Self {
                mode,
                primary_green: "#00FF66",
                accent_red: "#FF3333",
                accessible_red: "#FF3333",
                background: "#000000",
                surface: "#0A0A0A",
                surface_hover: "#1F1F1F",
                text_primary: "#FFFFFF",
                text_secondary: "#FFFF00",
                border: "#FFFFFF",
                badge_active_bg: "#FFFF00",
                badge_active_text: "#000000",
                font_size_base: 16,
                font_size_heading: 20,
                font_size_badge: 14,
            },
            ThemeMode::LargeText => Self {
                mode,
                primary_green: "#006A4E",
                accent_red: "#C21F35",
                accessible_red: "#C21F35",
                background: "#F7F8F6",
                surface: "#FFFFFF",
                surface_hover: "#EEF2EF",
                text_primary: "#1A241E",
                text_secondary: "#445248",
                border: "#B2C2B8",
                badge_active_bg: "#006A4E",
                badge_active_text: "#FFFFFF",
                font_size_base: 18,
                font_size_heading: 24,
                font_size_badge: 16,
            },
        }
    }
}
