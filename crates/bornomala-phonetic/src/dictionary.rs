use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use crate::fst_tree::FstTree;

pub static WORDS_FST_BYTES: &[u8] = include_bytes!("../data/words.fst");
pub static PATTERNS_FST_BYTES: &[u8] = include_bytes!("../data/patterns.fst");
pub static PREPROCESSED_PATTERNS_JSON: &[u8] = include_bytes!("../data/preprocessed-patterns.json");
pub static SOURCE_COMMON_PATTERNS: &str = include_str!("../data/source-common-patterns.txt");

static WORDS_FST: Lazy<FstTree<&'static [u8]>> = Lazy::new(|| FstTree::from_fst(WORDS_FST_BYTES));
static PATTERNS_FST: Lazy<FstTree<&'static [u8]>> =
    Lazy::new(|| FstTree::from_fst(PATTERNS_FST_BYTES));

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternBlock {
    pub transliterate: Vec<String>,
    pub entire_block_optional: Option<bool>,
}

#[derive(Clone)]
pub struct PhoneticDictionary {
    patterns: HashMap<String, PatternBlock>,
    common_suffixes: Vec<&'static str>,
}

static GLOBAL_DICT: Lazy<PhoneticDictionary> = Lazy::new(PhoneticDictionary::new);

impl PhoneticDictionary {
    pub fn global() -> &'static Self {
        &GLOBAL_DICT
    }

    pub fn new() -> Self {
        let patterns: HashMap<String, PatternBlock> =
            serde_json::from_slice(PREPROCESSED_PATTERNS_JSON)
                .expect("Failed to deserialize preprocessed-patterns.json");
        let common_suffixes: Vec<&'static str> = SOURCE_COMMON_PATTERNS.lines().collect();

        Self {
            patterns,
            common_suffixes,
        }
    }

    pub fn is_valid_word(&self, word: &str) -> bool {
        let words = Lazy::force(&WORDS_FST);
        words
            .matching_node(word)
            .and_then(|n| n.get_word())
            .is_some()
    }

    /// Query dictionary suggestions for a Roman phonetic input string.
    pub fn suggest(&self, input: &str) -> Vec<String> {
        let words = Lazy::force(&WORDS_FST);
        let patterns = Lazy::force(&PATTERNS_FST);
        let normalized = fix_dictionary_query(input);

        if normalized.is_empty() {
            return Vec::new();
        }

        let (matched, mut remaining, _) = patterns.match_longest_common_prefix(&normalized);

        let matched_patterns = if let Some(block) = self.patterns.get(matched) {
            &block.transliterate
        } else {
            return Vec::new();
        };

        let mut matched_nodes = matched_patterns
            .iter()
            .filter_map(|p| words.matching_node(p))
            .collect::<Vec<_>>();

        let additional_nodes = matched_nodes
            .iter()
            .flat_map(|node| {
                self.common_suffixes
                    .iter()
                    .filter_map(|suffix| node.get_matching_node(suffix))
            })
            .collect::<Vec<_>>();

        matched_nodes.extend(additional_nodes);

        while !remaining.is_empty() {
            let (mut new_matched, new_remaining, mut complete) =
                patterns.match_longest_common_prefix(remaining);

            if !complete {
                for i in (0..remaining.len()).rev() {
                    (new_matched, _, complete) =
                        patterns.match_longest_common_prefix(&remaining[..i]);

                    if complete {
                        remaining = &remaining[i..];
                        break;
                    }
                }
            } else {
                remaining = new_remaining;
            }

            let new_matched_patterns = if let Some(block) = self.patterns.get(new_matched) {
                &block.transliterate
            } else {
                break;
            };

            let new_matched_nodes = new_matched_patterns
                .iter()
                .flat_map(|p| {
                    matched_nodes
                        .iter()
                        .filter_map(|node| node.get_matching_node(p))
                })
                .collect::<Vec<_>>();

            if self
                .patterns
                .get(new_matched)
                .map_or(false, |v| v.entire_block_optional.is_some())
            {
                matched_nodes.extend(new_matched_nodes);
            } else {
                matched_nodes = new_matched_nodes;
            }

            let additional_matched_nodes = matched_nodes
                .iter()
                .flat_map(|node| {
                    self.common_suffixes
                        .iter()
                        .filter_map(|suffix| node.get_matching_node(suffix))
                })
                .collect::<Vec<_>>();
            matched_nodes.extend(additional_matched_nodes);
        }

        let suggestions: HashSet<String> = matched_nodes
            .into_iter()
            .filter_map(|n| n.get_word())
            .collect();

        suggestions.into_iter().collect()
    }
}

pub fn fix_dictionary_query(s: &str) -> String {
    let s = s.trim();
    let mut result = String::with_capacity(s.len());
    let mut prev = ' ';
    for c in s.chars() {
        if (c == 'o' || c == 'O') && !prev.is_ascii_alphabetic() {
            result.push('O');
        } else if c.is_ascii_alphanumeric() || c == '`' {
            result.push(c.to_ascii_lowercase());
        }
        prev = c;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_suggest() {
        let dict = PhoneticDictionary::global();
        let kormo = dict.suggest("kormo");
        assert!(kormo.contains(&"কর্ম".to_string()));

        let daktar = dict.suggest("daktar");
        assert!(daktar.contains(&"ডাক্তার".to_string()));

        let amra = dict.suggest("amra");
        assert!(amra.contains(&"আমরা".to_string()));

        let bangla = dict.suggest("bangla");
        assert!(bangla.contains(&"বাংলা".to_string()) || bangla.contains(&"বাঙলা".to_string()));
    }
}
