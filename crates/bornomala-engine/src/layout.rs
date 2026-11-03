use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("Failed to parse layout JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid layout metadata: missing or empty '{0}'")]
    InvalidMetadata(&'static str),
    #[error("Empty key mappings in layout")]
    EmptyKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutMetadata {
    pub id: String,
    pub name: String,
    pub name_bn: String,
    pub output_mode: String,
    #[serde(default)]
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutDefinition {
    #[serde(alias = "Alt", default)]
    pub alt: bool,
    #[serde(alias = "Ctrl", default)]
    pub ctrl: bool,
    #[serde(alias = "Shift", default)]
    pub shift: bool,
    #[serde(alias = "KeyCode")]
    pub key_code: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMapEntry {
    #[serde(alias = "Normal", default)]
    pub normal: String,
    #[serde(alias = "Shift", default)]
    pub shift: String,
    #[serde(alias = "AltGr", default)]
    pub altgr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JukEntry {
    #[serde(alias = "Seq")]
    pub seq: String,
    #[serde(alias = "Out")]
    pub out: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDefinition {
    pub metadata: LayoutMetadata,
    #[serde(default)]
    pub shortcut: Option<ShortcutDefinition>,
    pub keys: HashMap<String, KeyMapEntry>,
    #[serde(default)]
    pub juks: Vec<JukEntry>,
}

impl LayoutDefinition {
    /// Strict validation of the layout definition.
    pub fn validate(&self) -> Result<(), LayoutError> {
        if self.metadata.id.trim().is_empty() {
            return Err(LayoutError::InvalidMetadata("id"));
        }
        if self.metadata.name.trim().is_empty() {
            return Err(LayoutError::InvalidMetadata("name"));
        }
        if self.keys.is_empty() {
            return Err(LayoutError::EmptyKeys);
        }
        Ok(())
    }

    /// Load layout definition from a JSON string.
    pub fn from_json_str(json_str: &str) -> Result<Self, LayoutError> {
        let layout: Self = serde_json::from_str(json_str)?;
        layout.validate()?;
        Ok(layout)
    }

    /// Lookup character for a key code and modifier state.
    pub fn lookup(&self, key_code: u32, shift: bool, altgr: bool) -> Option<String> {
        let key_str = key_code.to_string();
        let entry = self.keys.get(&key_str)?;
        if altgr {
            if let Some(ref a) = entry.altgr {
                if !a.is_empty() {
                    return Some(a.clone());
                }
            }
        }
        if shift {
            if !entry.shift.is_empty() {
                Some(entry.shift.clone())
            } else {
                None
            }
        } else if !entry.normal.is_empty() {
            Some(entry.normal.clone())
        } else {
            None
        }
    }
}

/// Embedded default layouts.
pub static EMBEDDED_CLASSIC_UNICODE: &str = include_str!("../../../data/layouts/classic_unicode.json");
pub static EMBEDDED_CLASSIC_ANSI: &str = include_str!("../../../data/layouts/classic_ansi.json");

/// Load the default Classic Unicode layout.
pub fn load_default_unicode_layout() -> LayoutDefinition {
    LayoutDefinition::from_json_str(EMBEDDED_CLASSIC_UNICODE)
        .expect("Embedded classic_unicode.json must be valid")
}

/// Load the default Classic ANSI layout.
pub fn load_default_ansi_layout() -> LayoutDefinition {
    LayoutDefinition::from_json_str(EMBEDDED_CLASSIC_ANSI)
        .expect("Embedded classic_ansi.json must be valid")
}
