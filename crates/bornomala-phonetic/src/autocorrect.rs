use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static EMBEDDED_AUTODICT: &str = include_str!("../data/autodict.json");

static GLOBAL_AUTOCORRECT: Lazy<AutocorrectTable> = Lazy::new(|| {
    AutocorrectTable::load_from_json(EMBEDDED_AUTODICT).expect("Failed to load autodict.json")
});

#[derive(Debug, Clone)]
pub struct AutocorrectMatch {
    pub replacement: String,
    pub is_exact_literal: bool,
}

#[derive(Debug, Clone)]
pub struct AutocorrectTable {
    map: HashMap<String, String>,
}

impl AutocorrectTable {
    pub fn global() -> &'static Self {
        &GLOBAL_AUTOCORRECT
    }

    pub fn load_from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let map: HashMap<String, String> = serde_json::from_str(json_str)?;
        Ok(Self { map })
    }

    /// Lookup word in autocorrect dictionary.
    /// Handles exact match, lowercase match, and smiley rules.
    pub fn lookup(&self, word: &str) -> Option<AutocorrectMatch> {
        if word.is_empty() {
            return None;
        }

        // Direct lookup
        if let Some(val) = self.map.get(word) {
            return Some(AutocorrectMatch {
                replacement: val.clone(),
                is_exact_literal: val == word,
            });
        }

        // Lowercase fallback
        let lower = word.to_ascii_lowercase();
        if let Some(val) = self.map.get(&lower) {
            return Some(AutocorrectMatch {
                replacement: val.clone(),
                is_exact_literal: val == &lower,
            });
        }

        None
    }

    pub fn contains_key(&self, word: &str) -> bool {
        self.map.contains_key(word) || self.map.contains_key(&word.to_ascii_lowercase())
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocorrect_lookup() {
        let ac = AutocorrectTable::global();
        assert!(ac.len() > 2000);

        let abong = ac.lookup("abong").unwrap();
        assert_eq!(abong.replacement, "ebong");
        assert!(!abong.is_exact_literal);

        let academy = ac.lookup("academy").unwrap();
        assert_eq!(academy.replacement, "oZakaDemi");

        let smiley = ac.lookup(":-D");
        if let Some(m) = smiley {
            assert!(m.is_exact_literal);
        }
    }
}
