pub mod ansi_engine;
pub mod config;
pub mod conveniences;
pub mod converter;
pub mod doc_converter;
pub mod events;
pub mod layout;
pub mod phonetic;
pub mod unicode_engine;

pub use ansi_engine::AnsiClassicEngine;
pub use config::{AppConfig, CustomRule};
pub use conveniences::{
    amount_to_bangla_words, amount_to_english_words, convert_digits, get_preloaded_fonts,
    gregorian_to_bangla, AppProfileManager, AutostartManager, BanglaDate, FontConflictAction,
    FontInstallScope, PreloadedFont, StartupOptions, TextExpander, DEFAULT_SPLASH_DURATION_MS,
    PRELOADED_FONTS, REGISTRY_RUN_KEY, REGISTRY_VALUE_NAME,
};
pub use converter::BanglaConverter;
pub use doc_converter::{
    convert_file, default_output_path, ConversionDirection, ConversionReport, DocConvertError,
    DocumentFormat,
};
pub use events::{BackspaceMode, EngineAction, InputScheme, KeyEvent, Suggestion, TextEdit};
pub use layout::{load_default_ansi_layout, load_default_unicode_layout, LayoutDefinition};
pub use phonetic::PhoneticEngine;
pub use unicode_engine::UnicodeClassicEngine;

/// Unified Bornomala typing and text conversion engine.
#[derive(Debug, Clone)]
pub struct BornomalaEngine {
    scheme: InputScheme,
    unicode_engine: UnicodeClassicEngine,
    ansi_engine: AnsiClassicEngine,
    phonetic_engine: PhoneticEngine,
    converter: BanglaConverter,
    expander: TextExpander,
    app_profiles: AppProfileManager,
    buffer_history: String,
}

impl Default for BornomalaEngine {
    fn default() -> Self {
        Self::new(InputScheme::ClassicUnicode)
    }
}

impl BornomalaEngine {
    pub fn new(default_scheme: InputScheme) -> Self {
        Self {
            scheme: default_scheme,
            unicode_engine: UnicodeClassicEngine::default(),
            ansi_engine: AnsiClassicEngine::default(),
            phonetic_engine: PhoneticEngine::default(),
            converter: BanglaConverter::new(),
            expander: TextExpander::default(),
            app_profiles: AppProfileManager::new(default_scheme),
            buffer_history: String::new(),
        }
    }

    /// Retrieve the currently active input scheme.
    pub fn active_scheme(&self) -> InputScheme {
        self.scheme
    }

    /// Switch the active input scheme.
    pub fn set_scheme(&mut self, scheme: InputScheme) {
        self.scheme = scheme;
        self.reset_all_states();
    }

    /// Access the bidirectional Unicode <-> ANSI converter.
    pub fn converter(&self) -> &BanglaConverter {
        &self.converter
    }

    /// Mutably access the bidirectional Unicode <-> ANSI converter.
    pub fn converter_mut(&mut self) -> &mut BanglaConverter {
        &mut self.converter
    }

    /// Access the text expander.
    pub fn expander(&self) -> &TextExpander {
        &self.expander
    }

    /// Mutably access the text expander.
    pub fn expander_mut(&mut self) -> &mut TextExpander {
        &mut self.expander
    }

    /// Access the per-application profile manager.
    pub fn app_profiles(&self) -> &AppProfileManager {
        &self.app_profiles
    }

    /// Mutably access the per-application profile manager.
    pub fn app_profiles_mut(&mut self) -> &mut AppProfileManager {
        &mut self.app_profiles
    }

    /// Access the phonetic engine.
    pub fn phonetic_engine(&self) -> &PhoneticEngine {
        &self.phonetic_engine
    }

    /// Mutably access the phonetic engine.
    pub fn phonetic_engine_mut(&mut self) -> &mut PhoneticEngine {
        &mut self.phonetic_engine
    }

    /// Reset all internal composition states.
    pub fn reset_all_states(&mut self) {
        self.unicode_engine.reset_state();
        self.ansi_engine.reset_state();
        self.phonetic_engine.reset_state();
        self.buffer_history.clear();
    }

    /// Set backspace handling mode (Atomic vs Component).
    pub fn set_backspace_mode(&mut self, mode: BackspaceMode) {
        self.unicode_engine.set_backspace_mode(mode);
    }

    /// Check if a key event matches standard Bornomala shortcuts:
    /// - Ctrl + Alt + V -> Classic Unicode
    /// - Ctrl + Alt + B -> Classic ANSI
    /// - F12 -> Toggle Phonetic / PassThrough
    pub fn check_shortcut_toggle(&mut self, event: &KeyEvent) -> Option<EngineAction> {
        if !event.is_down {
            return None;
        }

        // Ctrl + Space toggle for Phonetic
        if event.ctrl && !event.alt && event.key_code == 0x20 {
            let next_scheme = if self.scheme == InputScheme::Phonetic {
                InputScheme::PassThrough
            } else {
                InputScheme::Phonetic
            };
            self.set_scheme(next_scheme);
            let msg = match next_scheme {
                InputScheme::Phonetic => "ধ্বনিভিত্তিক (Phonetic) মোড সক্রিয়",
                InputScheme::PassThrough => "ইংরেজি (English) মোড সক্রিয়",
                _ => "",
            };
            return Some(EngineAction {
                edits: Vec::new(),
                composition: None,
                suggestions: Vec::new(),
                status_message: Some(msg.to_string()),
                consumed: true,
            });
        }

        // F12 toggle for Phonetic
        if event.key_code == 0x7B {
            let next_scheme = if self.scheme == InputScheme::Phonetic {
                InputScheme::PassThrough
            } else {
                InputScheme::Phonetic
            };
            self.set_scheme(next_scheme);
            let msg = match next_scheme {
                InputScheme::Phonetic => "ধ্বনিভিত্তিক (Phonetic) মোড সক্রিয়",
                InputScheme::PassThrough => "ইংরেজি (English) মোড সক্রিয়",
                _ => "",
            };
            return Some(EngineAction {
                edits: Vec::new(),
                composition: None,
                suggestions: Vec::new(),
                status_message: Some(msg.to_string()),
                consumed: true,
            });
        }

        // Ctrl + Alt combinations
        if event.ctrl && event.alt {
            // Key 'V' (ASCII 86 / 118)
            if event.key_code == 86 || event.key_code == 118 {
                let next_scheme = if self.scheme == InputScheme::ClassicUnicode {
                    InputScheme::PassThrough
                } else {
                    InputScheme::ClassicUnicode
                };
                self.set_scheme(next_scheme);
                let msg = match next_scheme {
                    InputScheme::ClassicUnicode => "ইউনিকোড (Unicode) মোড সক্রিয়",
                    _ => "ইংরেজি (English) মোড সক্রিয়",
                };
                return Some(EngineAction {
                    edits: Vec::new(),
                    composition: None,
                    suggestions: Vec::new(),
                    status_message: Some(msg.to_string()),
                    consumed: true,
                });
            }

            // Key 'B' (ASCII 66 / 98)
            if event.key_code == 66 || event.key_code == 98 {
                let next_scheme = if self.scheme == InputScheme::ClassicAnsi {
                    InputScheme::PassThrough
                } else {
                    InputScheme::ClassicAnsi
                };
                self.set_scheme(next_scheme);
                let msg = match next_scheme {
                    InputScheme::ClassicAnsi => "ক্লাসিক এএনএসআই (Classic ANSI) মোড সক্রিয়",
                    _ => "ইংরেজি (English) মোড সক্রিয়",
                };
                return Some(EngineAction {
                    edits: Vec::new(),
                    composition: None,
                    suggestions: Vec::new(),
                    status_message: Some(msg.to_string()),
                    consumed: true,
                });
            }
        }

        None
    }

    /// Process a physical key event through the active scheme.
    pub fn process_key(&mut self, event: &KeyEvent) -> EngineAction {
        // 1. Check for global shortcut mode switching first
        if let Some(action) = self.check_shortcut_toggle(event) {
            return action;
        }

        // 2. Delegate to the active scheme
        let action = match self.scheme {
            InputScheme::ClassicUnicode => self.unicode_engine.process_key(event),
            InputScheme::ClassicAnsi => self.ansi_engine.process_key(event),
            InputScheme::Phonetic => self.phonetic_engine.process_key(event),
            InputScheme::PassThrough => EngineAction::pass_through(),
        };

        // 3. Track buffer for text expander triggers
        if let Some(ch) = event.ch {
            self.buffer_history.push(ch);
            let char_count = self.buffer_history.chars().count();
            if char_count > 30 {
                let drop_count = char_count - 20;
                if let Some((byte_idx, _)) = self.buffer_history.char_indices().nth(drop_count) {
                    self.buffer_history.drain(..byte_idx);
                }
            }

            if let Some((trigger, replacement)) = self.expander.check_buffer(&self.buffer_history) {
                let trigger_len = trigger.chars().count();
                self.buffer_history.clear();
                return EngineAction {
                    edits: vec![TextEdit::ReplaceLast {
                        count: trigger_len,
                        text: replacement.clone(),
                    }],
                    composition: None,
                    suggestions: Vec::new(),
                    status_message: Some(format!("Expanded {}", trigger)),
                    consumed: true,
                };
            }
        }

        action
    }

    /// Bidirectional Converter access.
    pub fn convert_unicode_to_ansi(&self, text: &str) -> (String, Vec<char>) {
        self.converter.unicode_to_ansi(text)
    }

    pub fn convert_ansi_to_unicode(&self, text: &str) -> String {
        self.converter.ansi_to_unicode(text)
    }

    /// Notify application switch to remember last scheme.
    pub fn on_application_focus(&mut self, app_name: &str) {
        let preferred = self.app_profiles.get_scheme_for_app(app_name);
        self.set_scheme(preferred);
    }
}
