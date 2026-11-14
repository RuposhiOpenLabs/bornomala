use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static EMBEDDED_SUFFIXES: &str = include_str!("../data/suffixes.json");

static GLOBAL_SUFFIX_TABLE: Lazy<SuffixTable> = Lazy::new(|| {
    SuffixTable::load_from_json(EMBEDDED_SUFFIXES).expect("Failed to load suffixes.json")
});

#[derive(Clone, Debug)]
pub struct SuffixTable {
    suffixes: HashMap<String, String>,
}

impl SuffixTable {
    pub fn global() -> &'static Self {
        &GLOBAL_SUFFIX_TABLE
    }

    pub fn load_from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let suffixes: HashMap<String, String> = serde_json::from_str(json_str)?;
        Ok(Self { suffixes })
    }

    pub fn get_suffix(&self, roman_suffix: &str) -> Option<&str> {
        self.suffixes.get(roman_suffix).map(|s| s.as_str())
    }

    /// Split word into all possible (stem, suffix) pairs where suffix exists in suffix table.
    pub fn split_suffixes<'a>(&'a self, word: &'a str) -> Vec<(&'a str, &'a str, &'a str)> {
        let mut results = Vec::new();
        let len = word.len();
        if len < 2 {
            return results;
        }

        for i in 1..len {
            let (stem, suf) = word.split_at(i);
            if let Some(bn_suffix) = self.get_suffix(suf) {
                results.push((stem, suf, bn_suffix));
            }
        }

        results
    }

    /// Join a Bengali stem with a Bengali suffix using Avro's morphological sandhi rules.
    pub fn join_bengali_suffix(stem: &str, suffix: &str) -> String {
        if stem.is_empty() {
            return suffix.to_string();
        }
        if suffix.is_empty() {
            return stem.to_string();
        }

        let stem_last_char = stem.chars().last().unwrap();
        let suffix_first_char = suffix.chars().next().unwrap();

        // 1. If stem ends in vowel and suffix begins with kar (dependent vowel sign), insert য় (\u09df)
        if is_bengali_vowel(stem_last_char) && is_bengali_kar(suffix_first_char) {
            return format!("{}য়{}", stem, suffix);
        }

        // 2. If stem ends with ৎ (\u09ce), convert to ত (\u09a4) before vowel-based suffix
        if stem_last_char == 'ৎ' {
            let mut prefix_stem: String = stem.chars().take(stem.chars().count() - 1).collect();
            prefix_stem.push('ত');
            return format!("{}{}", prefix_stem, suffix);
        }

        // 3. If stem ends with ং (\u0982), convert to ঙ (\u0999) before suffix
        if stem_last_char == 'ং' {
            let mut prefix_stem: String = stem.chars().take(stem.chars().count() - 1).collect();
            prefix_stem.push('ঙ');
            return format!("{}{}", prefix_stem, suffix);
        }

        format!("{}{}", stem, suffix)
    }
}

pub fn is_bengali_vowel(c: char) -> bool {
    matches!(
        c,
        'অ' | 'আ'
            | 'ই'
            | 'ঈ'
            | 'উ'
            | 'ঊ'
            | 'ঋ'
            | 'এ'
            | 'ঐ'
            | 'ও'
            | 'ঔ'
            | 'া'
            | 'ি'
            | 'ী'
            | 'ু'
            | 'ূ'
            | 'ৃ'
            | 'ে'
            | 'ৈ'
            | 'ো'
            | 'ৌ'
    )
}

pub fn is_bengali_kar(c: char) -> bool {
    matches!(c, 'া' | 'ি' | 'ী' | 'ু' | 'ূ' | 'ৃ' | 'ে' | 'ৈ' | 'ো' | 'ৌ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suffix_split_and_join() {
        let table = SuffixTable::global();
        assert!(table.get_suffix("e").is_some());
        assert!(table.get_suffix("gulo").is_some());
        assert_eq!(table.get_suffix("gulo"), Some("গুলো"));

        // Join tests
        assert_eq!(SuffixTable::join_bengali_suffix("দেশ", "ে"), "দেশে");
        assert_eq!(SuffixTable::join_bengali_suffix("বই", "গুলো"), "বইগুলো");
        assert_eq!(SuffixTable::join_bengali_suffix("হঠাৎ", "ের"), "হঠাতের");
    }
}
