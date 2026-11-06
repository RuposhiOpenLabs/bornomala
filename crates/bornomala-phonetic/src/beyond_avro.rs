use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeyondAvroConfig {
    /// Enable context-aware suggestion ranking based on previous word
    pub enable_context_ranking: bool,
    /// Enable custom user abbreviations / snippets
    pub enable_abbreviations: bool,
    /// User custom phonetic replacements (checked before default rules)
    pub custom_rules: HashMap<String, String>,
    /// User custom abbreviations (e.g. "bd" -> "বাংলাদেশ")
    pub abbreviations: HashMap<String, String>,
}

impl Default for BeyondAvroConfig {
    fn default() -> Self {
        let mut abbreviations = HashMap::new();
        abbreviations.insert("bd".to_string(), "বাংলাদেশ".to_string());
        abbreviations.insert("gm".to_string(), "শুভ সকাল".to_string());
        abbreviations.insert("gn".to_string(), "শুভ রাত্রি".to_string());
        abbreviations.insert("thx".to_string(), "ধন্যবাদ".to_string());

        Self {
            enable_context_ranking: true,
            enable_abbreviations: true,
            custom_rules: HashMap::new(),
            abbreviations,
        }
    }
}

/// N-gram / Bigram Context Ranker.
/// Improves candidate ranking when the previous word provides semantic context.
#[derive(Debug, Clone)]
pub struct ContextRanker {
    bigrams: HashMap<&'static str, &'static [&'static str]>,
}

impl ContextRanker {
    pub fn new() -> Self {
        let mut bigrams = HashMap::new();
        bigrams.insert("আমার", &["সোনার", "দেশ", "বাংলা", "নাম", "ভাই", "মা", "বাবা"][..]);
        bigrams.insert("সোনার", &["বাংলা", "হরিণ", "মেয়ে", "ছেলে"][..]);
        bigrams.insert("গণপ্রজাতন্ত্রী", &["বাংলাদেশ"][..]);
        bigrams.insert("বাংলাদেশ", &["সরকার", "রেলওয়ে", "ব্যাংক", "জাতীয়", "দল"][..]);
        bigrams.insert("ঢাকা", &["বিশ্ববিদ্যালয়", "মহানগর", "শহর", "মেডিকেল"][..]);
        bigrams.insert("প্রধান", &["মন্ত্রী", "বিচারপতি", "শিক্ষক", "কার্যালয়"][..]);
        bigrams.insert("কর্ম", &["কর্তা", "চারী", "ফল", "ক্ষেত্র", "শালা"][..]);
        bigrams.insert("শিক্ষা", &["প্রতিষ্ঠান", "বোর্ড", "ব্যবস্থা", "মন্ত্রক"][..]);

        Self { bigrams }
    }

    /// Rerank candidates based on semantic compatibility with `prev_word`.
    pub fn rerank(&self, prev_word: &str, candidates: &mut Vec<String>) {
        if prev_word.is_empty() || candidates.len() <= 1 {
            return;
        }

        let clean_prev = prev_word.trim();
        if let Some(expected_followers) = self.bigrams.get(clean_prev) {
            // Find if any candidate matches an expected follower
            if let Some(pos) = candidates
                .iter()
                .position(|c| expected_followers.contains(&c.as_str()))
            {
                if pos > 0 {
                    let matched = candidates.remove(pos);
                    candidates.insert(0, matched);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_ranker() {
        let ranker = ContextRanker::new();
        let mut candidates = vec!["বাংলাদেশ".to_string(), "সোনার".to_string(), "দেশ".to_string()];

        // When preceded by "আমার", "সোনার" should rise to top
        ranker.rerank("আমার", &mut candidates);
        assert_eq!(candidates[0], "সোনার");
    }

    #[test]
    fn test_abbreviation_expansion() {
        let cfg = BeyondAvroConfig::default();
        assert_eq!(cfg.abbreviations.get("bd"), Some(&"বাংলাদেশ".to_string()));
    }
}
