use std::path::PathBuf;

use crate::autocorrect::AutocorrectTable;
use crate::beyond_avro::{BeyondAvroConfig, ContextRanker};
use crate::dictionary::PhoneticDictionary;
use crate::memory::CandidateMemory;
use crate::rules::AvroRules;
use crate::suffix::SuffixTable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneticSuggestion {
    pub text: String,
    pub source: SuggestionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionSource {
    Autocorrect,
    Dictionary,
    ClassicRule,
    Suffix,
    Abbreviation,
    CustomRule,
}

#[derive(Clone)]
pub struct PhoneticEngine {
    rules: AvroRules,
    autocorrect: AutocorrectTable,
    dictionary: PhoneticDictionary,
    suffix_table: SuffixTable,
    memory: CandidateMemory,
    config: BeyondAvroConfig,
    context_ranker: ContextRanker,
    input_buffer: String,
    last_committed_word: String,
}

impl Default for PhoneticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PhoneticEngine {
    pub fn new() -> Self {
        let memory_path = dirs_fallback_memory_path();
        Self {
            rules: AvroRules::global().clone(),
            autocorrect: AutocorrectTable::global().clone(),
            dictionary: PhoneticDictionary::global().clone(),
            suffix_table: SuffixTable::global().clone(),
            memory: CandidateMemory::with_file(memory_path),
            config: BeyondAvroConfig::default(),
            context_ranker: ContextRanker::new(),
            input_buffer: String::new(),
            last_committed_word: String::new(),
        }
    }

    pub fn with_custom_memory(memory: CandidateMemory) -> Self {
        Self {
            rules: AvroRules::global().clone(),
            autocorrect: AutocorrectTable::global().clone(),
            dictionary: PhoneticDictionary::global().clone(),
            suffix_table: SuffixTable::global().clone(),
            memory,
            config: BeyondAvroConfig::default(),
            context_ranker: ContextRanker::new(),
            input_buffer: String::new(),
            last_committed_word: String::new(),
        }
    }

    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    pub fn clear_buffer(&mut self) {
        self.input_buffer.clear();
    }

    pub fn push_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn pop_char(&mut self) -> Option<char> {
        self.input_buffer.pop()
    }

    pub fn set_last_committed_word(&mut self, word: String) {
        self.last_committed_word = word;
    }

    /// Record a chosen candidate selection to train local frequency memory.
    pub fn record_selection(&mut self, roman: &str, bangla: &str) {
        self.memory.record_selection(roman, bangla);
        self.last_committed_word = bangla.to_string();
    }

    /// Pure classic rule transliteration (100% Avro compatibility).
    pub fn transliterate_classic(&self, input: &str) -> String {
        self.rules.transliterate(input)
    }

    /// Generate ranked candidates for the current or specified Roman input.
    pub fn get_candidates(&self, input: &str) -> Vec<String> {
        if input.is_empty() {
            return Vec::new();
        }

        let (begin_pad, core, end_pad) = separate_padding(input);
        if core.is_empty() {
            let direct = self.rules.transliterate(input);
            return vec![direct];
        }

        let mut candidate_list: Vec<String> = Vec::with_capacity(16);

        // 1. User Abbreviations (Beyond Avro, if enabled)
        if self.config.enable_abbreviations {
            if let Some(abbr) = self.config.abbreviations.get(&core.to_ascii_lowercase()) {
                candidate_list.push(abbr.clone());
            }
        }

        // 2. Autocorrect Dictionary (Avro 5.6.0 autodict.dct)
        if let Some(ac) = self.autocorrect.lookup(core) {
            let bn = if ac.is_exact_literal {
                ac.replacement
            } else {
                self.rules.transliterate(&ac.replacement)
            };
            if !candidate_list.contains(&bn) {
                candidate_list.push(bn);
            }
        }

        // 3. Classic Avro Phonetic rule output
        let classic_bn = self.rules.transliterate(core);

        // 4. Dictionary Suggestions (FST 159k database)
        let mut dict_words = self.dictionary.suggest(core);

        // 5. Suffix-expanded words
        let suffix_splits = self.suffix_table.split_suffixes(core);
        for (stem, _, bn_suf) in suffix_splits {
            let stem_words = self.dictionary.suggest(stem);
            for sw in stem_words {
                let full = SuffixTable::join_bengali_suffix(&sw, bn_suf);
                if !dict_words.contains(&full) {
                    dict_words.push(full);
                }
            }
        }

        // Sort dictionary suggestions:
        // Balance edit distance with conjunct affinity (e.g. 'rm' -> 'র্ম' for words like 'কর্ম')
        // and common Bengali vocabulary frequency.
        let core_has_r_consonant = has_reph_ambiguity(core);

        dict_words.sort_by(|a, b| {
            let mut score_a = (levenshtein_distance(&classic_bn, a) as i32) * 10;
            let mut score_b = (levenshtein_distance(&classic_bn, b) as i32) * 10;

            if core_has_r_consonant {
                if a.contains('্') && a.contains('র') {
                    score_a -= 15;
                }
                if b.contains('্') && b.contains('র') {
                    score_b -= 15;
                }
            }

            if is_core_bengali_word(a) {
                score_a -= 20;
            }
            if is_core_bengali_word(b) {
                score_b -= 20;
            }

            score_a.cmp(&score_b)
        });

        // Merge dictionary suggestions
        for dw in dict_words {
            if !candidate_list.contains(&dw) {
                candidate_list.push(dw);
            }
        }

        // Always ensure classic phonetic is available
        if !candidate_list.contains(&classic_bn) {
            candidate_list.push(classic_bn);
        }

        // 6. User Selection Frequency Reranking (Local Memory)
        self.memory.rerank(core, &mut candidate_list);

        // 7. Context-Aware Reranking (Beyond Avro, if enabled)
        if self.config.enable_context_ranking && !self.last_committed_word.is_empty() {
            self.context_ranker
                .rerank(&self.last_committed_word, &mut candidate_list);
        }

        // Apply padding back to all candidates
        candidate_list
            .into_iter()
            .map(|c| format!("{}{}{}", begin_pad, c, end_pad))
            .collect()
    }

    /// Primary conversion for live inline preview: returns the top-ranked candidate.
    pub fn get_live_preview(&self) -> String {
        if self.input_buffer.is_empty() {
            return String::new();
        }
        let candidates = self.get_candidates(&self.input_buffer);
        candidates.into_iter().next().unwrap_or_default()
    }
}

/// Separate leading and trailing non-alphanumeric punctuation padding.
pub fn separate_padding(word: &str) -> (&str, &str, &str) {
    let chars: Vec<(usize, char)> = word.char_indices().collect();
    if chars.is_empty() {
        return ("", "", "");
    }

    let first_alphanumeric = chars.iter().position(|(_, c)| is_core_phonetic_char(*c));
    let last_alphanumeric = chars.iter().rposition(|(_, c)| is_core_phonetic_char(*c));

    match (first_alphanumeric, last_alphanumeric) {
        (Some(start_idx), Some(end_idx)) => {
            let start_byte = chars[start_idx].0;
            let end_byte = chars[end_idx].0 + chars[end_idx].1.len_utf8();
            (&word[..start_byte], &word[start_byte..end_byte], &word[end_byte..])
        }
        _ => ("", word, ""),
    }
}

fn is_core_phonetic_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '`' || c == '$' || c == '.'
}

fn has_reph_ambiguity(word: &str) -> bool {
    let lower = word.to_ascii_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i] == 'r' && is_ascii_consonant(chars[i + 1]) && chars[i + 1] != 'r' && chars[i + 1] != 'y' && chars[i + 1] != 'w' {
            return true;
        }
    }
    false
}

fn is_ascii_consonant(c: char) -> bool {
    matches!(c, 'b' | 'c' | 'd' | 'f' | 'g' | 'h' | 'j' | 'k' | 'l' | 'm' | 'n' | 'p' | 'q' | 'r' | 's' | 't' | 'v' | 'w' | 'x' | 'y' | 'z')
}

fn is_core_bengali_word(w: &str) -> bool {
    matches!(
        w,
        "আমি" | "তুমি" | "আমরা" | "তোমরা" | "তিনি" | "তারা" | "তার" | "তাদের" |
        "বাংলা" | "বাংলাদেশ" | "কর্ম" | "কর্মফল" | "বর্ণ" | "বর্ণমালা" | "ধর্ম" |
        "ডাক্তার" | "শিক্ষক" | "শিক্ষা" | "স্বাধীনতা" | "দুঃখ" | "এবং" | "শেষ" |
        "আপনি" | "লালশাক" | "লালরং" | "সোনার" | "প্রথম" | "কাজ" | "কথা" | "সময়" |
        "জীবন" | "মানুষ" | "দেশ" | "সরকার" | "ঢাকা" | "বই" | "পানি" | "ভাত" |
        "আজ" | "কাল" | "ভালো" | "মন্দ" | "এক" | "দুই" | "তিন" | "চার" | "পাঁচ"
    )
}

/// Standard Levenshtein edit distance.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut dp = vec![vec![0; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
}

fn dirs_fallback_memory_path() -> PathBuf {
    if let Ok(app_data) = std::env::var("APPDATA") {
        PathBuf::from(app_data).join("Bornomala").join("phonetic_memory.enc")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".bornomala").join("phonetic_memory.enc")
    } else {
        PathBuf::from("phonetic_memory.enc")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_live_preview_and_candidates() {
        let engine = PhoneticEngine::new();

        // 1. Classic word test
        let ami_candidates = engine.get_candidates("ami");
        assert_eq!(ami_candidates[0], "আমি");

        // 2. Complex conjunct word test
        let kormo_candidates = engine.get_candidates("kormo");
        assert!(kormo_candidates.contains(&"কর্ম".to_string()));

        // 3. Autocorrect test
        let abong_candidates = engine.get_candidates("abong");
        assert_eq!(abong_candidates[0], "এবং");

        // 4. Suffix test
        let deshe_candidates = engine.get_candidates("deshe");
        assert!(deshe_candidates.contains(&"দেশে".to_string()));

        // 5. Abbreviation test
        let bd_candidates = engine.get_candidates("bd");
        assert_eq!(bd_candidates[0], "বাংলাদেশ");
    }

    #[test]
    fn test_separate_padding() {
        assert_eq!(separate_padding("hello"), ("", "hello", ""));
        assert_eq!(separate_padding("\"ami\""), ("\"", "ami", "\""));
        assert_eq!(separate_padding("(amar)"), ("(", "amar", ")"));
    }
}
