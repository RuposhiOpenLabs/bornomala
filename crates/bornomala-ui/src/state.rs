use bornomala_engine::{BornomalaEngine, InputScheme};
use crate::localization::Language;
use crate::theme::ThemeMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallerStep {
    Welcome,
    Terms,
    Components,
    Installing,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontConflictModal {
    pub font_name: String,
    pub filename: String,
    pub apply_to_all: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveView {
    StartupScreen,
    FloatingBar,
    OnScreenKeyboard,
    Converter,
    Settings,
    Welcome,
    About,
    InstallerSetup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiState {
    pub active_view: ActiveView,
    pub scheme: InputScheme,
    pub language: Language,
    pub theme: ThemeMode,
    pub is_pass_through: bool,
    pub osk_shift: bool,
    pub osk_altgr: bool,
    pub conv_unicode_text: String,
    pub conv_ansi_text: String,
    pub conv_direction_unicode_to_ansi: bool,
    pub conv_last_message: String,
    pub default_scheme: InputScheme,
    pub atomic_backspace: bool,
    pub bangla_digits_default: bool,
    pub remember_per_app: bool,
    pub update_message: Option<String>,
    pub auto_startup_enabled: bool,
    pub show_startup_screen: bool,
    pub startup_screen_duration_ms: u32,
    pub startup_screen_visible: bool,
    pub startup_screen_bengali_by_default: bool,
    pub installer_step: InstallerStep,
    pub installer_agreed_terms: bool,
    pub installer_install_tsf: bool,
    pub installer_install_fonts: bool,
    pub installer_install_autostart: bool,
    pub installer_install_shortcuts: bool,
    pub installer_progress_percent: u32,
    pub installer_font_conflict: Option<FontConflictModal>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            active_view: ActiveView::StartupScreen,
            scheme: InputScheme::ClassicUnicode,
            language: Language::Bangla,
            theme: ThemeMode::Light,
            is_pass_through: false,
            osk_shift: false,
            osk_altgr: false,
            conv_unicode_text: "আমার সোনার বাংলা, আমি তোমায় ভালোবাসি।".to_string(),
            conv_ansi_text: "Avgvi †mvbvi evsjv, Avwg †Zvgvq fv‡jvevwm|".to_string(),
            conv_direction_unicode_to_ansi: true,
            conv_last_message: "".to_string(),
            default_scheme: InputScheme::ClassicUnicode,
            atomic_backspace: true,
            bangla_digits_default: true,
            remember_per_app: true,
            update_message: None,
            auto_startup_enabled: true,
            show_startup_screen: true,
            startup_screen_duration_ms: 2500,
            startup_screen_visible: true,
            startup_screen_bengali_by_default: true,
            installer_step: InstallerStep::Welcome,
            installer_agreed_terms: true,
            installer_install_tsf: true,
            installer_install_fonts: true,
            installer_install_autostart: true,
            installer_install_shortcuts: true,
            installer_progress_percent: 100,
            installer_font_conflict: None,
        }
    }
}

impl UiState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_scheme(&mut self, scheme: InputScheme) {
        self.scheme = scheme;
    }

    pub fn toggle_pass_through(&mut self) {
        self.is_pass_through = !self.is_pass_through;
    }

    pub fn execute_conversion(&mut self, engine: &BornomalaEngine) {
        if self.conv_direction_unicode_to_ansi {
            let (res, _unrep) = engine.converter().unicode_to_ansi(&self.conv_unicode_text);
            self.conv_ansi_text = res;
            self.conv_last_message = match self.language {
                Language::Bangla => "সফলভাবে এএনএসআই-তে রূপান্তর সম্পন্ন হয়েছে।".to_string(),
                Language::English => "Successfully converted to ANSI SutonnyMJ.".to_string(),
            };
        } else {
            let res = engine.converter().ansi_to_unicode(&self.conv_ansi_text);
            self.conv_unicode_text = res;
            self.conv_last_message = match self.language {
                Language::Bangla => "সফলভাবে ইউনিকোডে রূপান্তর সম্পন্ন হয়েছে।".to_string(),
                Language::English => "Successfully converted to Unicode.".to_string(),
            };
        }
    }

    pub fn clear_converter(&mut self) {
        self.conv_unicode_text.clear();
        self.conv_ansi_text.clear();
        self.conv_last_message.clear();
    }

    pub fn switch_converter_direction(&mut self) {
        self.conv_direction_unicode_to_ansi = !self.conv_direction_unicode_to_ansi;
    }

    pub fn dismiss_startup_screen(&mut self) {
        self.startup_screen_visible = false;
        if self.active_view == ActiveView::StartupScreen {
            self.active_view = ActiveView::FloatingBar;
        }
    }

    pub fn toggle_autostart(&mut self) {
        self.auto_startup_enabled = !self.auto_startup_enabled;
    }

    pub fn toggle_startup_screen(&mut self) {
        self.show_startup_screen = !self.show_startup_screen;
    }

    pub fn get_startup_screen_path(&self) -> &'static str {
        if self.startup_screen_bengali_by_default || self.language == Language::Bangla {
            "assets/branding/startup_screen_bn.png"
        } else {
            "assets/branding/startup_screen_en.png"
        }
    }

    pub fn next_installer_step(&mut self) {
        self.installer_step = match self.installer_step {
            InstallerStep::Welcome => InstallerStep::Terms,
            InstallerStep::Terms => InstallerStep::Components,
            InstallerStep::Components => InstallerStep::Installing,
            InstallerStep::Installing => InstallerStep::Finished,
            InstallerStep::Finished => InstallerStep::Finished,
        };
    }

    pub fn prev_installer_step(&mut self) {
        self.installer_step = match self.installer_step {
            InstallerStep::Welcome => InstallerStep::Welcome,
            InstallerStep::Terms => InstallerStep::Welcome,
            InstallerStep::Components => InstallerStep::Terms,
            InstallerStep::Installing => InstallerStep::Components,
            InstallerStep::Finished => InstallerStep::Components,
        };
    }

    pub fn trigger_font_conflict_dialog(&mut self, font_name: &str, filename: &str) {
        self.installer_font_conflict = Some(FontConflictModal {
            font_name: font_name.to_string(),
            filename: filename.to_string(),
            apply_to_all: false,
        });
    }

    pub fn dismiss_font_conflict_dialog(&mut self) {
        self.installer_font_conflict = None;
    }
}
