use serde::{Deserialize, Serialize};

/// Supported typing schemes in Bornomala.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputScheme {
    /// Classic fixed layout outputting canonical Unicode Bangla.
    ClassicUnicode,
    /// Classic fixed layout outputting legacy ANSI SutonnyMJ glyphs.
    ClassicAnsi,
    /// Phonetic transliteration typing (Banglish to Bangla).
    Phonetic,
    /// Direct pass-through (English / system typing).
    PassThrough,
}

impl Default for InputScheme {
    fn default() -> Self {
        Self::ClassicUnicode
    }
}

/// Backspace behavior preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackspaceMode {
    /// Deletes the entire logical cluster (e.g. conjunct + vowel sign) at once.
    Atomic,
    /// Deletes one component code point / sign at a time.
    Component,
}

impl Default for BackspaceMode {
    fn default() -> Self {
        Self::Component
    }
}

/// Incoming physical or logical key event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEvent {
    /// Virtual key code or ASCII key code.
    pub key_code: u32,
    /// Character representation if available.
    pub ch: Option<char>,
    /// Shift key is active.
    pub shift: bool,
    /// Ctrl key is active.
    pub ctrl: bool,
    /// Alt / AltGr key is active.
    pub alt: bool,
    /// Whether this is a key down (true) or key up (false) event.
    pub is_down: bool,
}

impl KeyEvent {
    pub fn new_down(key_code: u32, ch: Option<char>, shift: bool, ctrl: bool, alt: bool) -> Self {
        Self {
            key_code,
            ch,
            shift,
            ctrl,
            alt,
            is_down: true,
        }
    }

    pub fn is_printable(&self) -> bool {
        if self.ctrl || self.alt {
            return false;
        }
        if let Some(c) = self.ch {
            !c.is_control()
        } else {
            (0x20..=0x7E).contains(&self.key_code)
        }
    }
}

/// Text manipulation edit instructions emitted by the engine to the host application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextEdit {
    /// Insert text at caret.
    Insert(String),
    /// Delete `count` characters immediately before the caret and then insert `text`.
    ReplaceLast { count: usize, text: String },
    /// Delete `count` characters immediately before caret.
    DeleteBack(usize),
    /// The event is ignored by the engine and passed cleanly to the host application.
    PassThrough,
}

/// Autocomplete / phonetic suggestion entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    /// Suggested Bangla word.
    pub word: String,
    /// Transliterated Roman stem or explanation.
    pub hint: Option<String>,
    /// Frequency ranking weight.
    pub score: u32,
}

/// Comprehensive engine response following a key event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineAction {
    /// List of concrete text edits to apply in the host.
    pub edits: Vec<TextEdit>,
    /// Currently composing buffer (e.g. for IME underlining or preview).
    pub composition: Option<String>,
    /// Live suggestions for phonetic mode.
    pub suggestions: Vec<Suggestion>,
    /// Optional status or scheme toggle message.
    pub status_message: Option<String>,
    /// Consumed flag: whether the engine absorbed this key.
    pub consumed: bool,
}

impl EngineAction {
    pub fn pass_through() -> Self {
        Self {
            edits: vec![TextEdit::PassThrough],
            composition: None,
            suggestions: Vec::new(),
            status_message: None,
            consumed: false,
        }
    }

    pub fn insert(text: impl Into<String>) -> Self {
        let t = text.into();
        Self {
            edits: vec![TextEdit::Insert(t)],
            composition: None,
            suggestions: Vec::new(),
            status_message: None,
            consumed: true,
        }
    }

    pub fn replace(count: usize, text: impl Into<String>) -> Self {
        let t = text.into();
        Self {
            edits: vec![TextEdit::ReplaceLast { count, text: t }],
            composition: None,
            suggestions: Vec::new(),
            status_message: None,
            consumed: true,
        }
    }

    pub fn delete_back(count: usize) -> Self {
        Self {
            edits: vec![TextEdit::DeleteBack(count)],
            composition: None,
            suggestions: Vec::new(),
            status_message: None,
            consumed: true,
        }
    }
}
