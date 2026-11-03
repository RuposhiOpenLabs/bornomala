use std::collections::HashMap;
use crate::events::{EngineAction, KeyEvent, Suggestion, TextEdit};

pub static EMBEDDED_PHONETIC_RULES: &str =
    include_str!("../../../data/layouts/phonetic_rules.json");

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MatchRule {
    #[serde(rename = "type")]
    pub match_type: String, // "prefix" or "suffix"
    pub scope: String,      // "punctuation", "vowel", "consonant", "exact", or prefixed with "!"
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PatternRule {
    #[serde(default)]
    pub matches: Vec<MatchRule>,
    pub replace: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PhoneticPattern {
    pub find: String,
    pub replace: String,
    #[serde(default)]
    pub rules: Option<Vec<PatternRule>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PhoneticData {
    pub patterns: Vec<PhoneticPattern>,
    #[serde(default)]
    pub vowel: String,
    #[serde(default)]
    pub consonant: String,
    #[serde(default)]
    pub casesensitive: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PhoneticRuleFile {
    pub data: PhoneticData,
}

/// Avro-compatible Phonetic Transliteration Engine.
#[derive(Debug, Clone)]
pub struct PhoneticEngine {
    data: PhoneticData,
    current_roman_buffer: String,
    dictionary: Vec<String>,
    user_frequency: HashMap<String, u32>,
}

impl Default for PhoneticEngine {
    fn default() -> Self {
        Self::load_embedded()
    }
}

impl PhoneticEngine {
    pub fn load_embedded() -> Self {
        let rule_file: PhoneticRuleFile = serde_json::from_str(EMBEDDED_PHONETIC_RULES)
            .expect("Embedded phonetic_rules.json must be valid");

        let mut engine = Self {
            data: rule_file.data,
            current_roman_buffer: String::new(),
            dictionary: Vec::new(),
            user_frequency: HashMap::new(),
        };

        engine.load_core_dictionary();
        engine
    }

    fn load_core_dictionary(&mut self) {
        let core_words = &[
            "আমি", "আমরা", "আমার", "আমাদের", "তুমি", "তোমরা", "তোমার", "তোমাদের",
            "তিনি", "তারা", "তার", "তাদের", "বাংলা", "বাংলাদেশ", "সোনার", "ঢাকা",
            "বিদ্যালয়", "বিশ্ববিদ্যালয়", "সরকারি", "বাংলাদেশি", "প্রশাসনিক", "মন্ত্রক",
            "অফিস", "কর্মকর্তা", "কর্মচারী", "পরীক্ষা", "শিক্ষা", "স্বাস্থ্য", "চিকিৎসা",
            "টাকা", "পয়সা", "তারিখ", "আবেদন", "অনুরোধ", "নোটিশ", "বিল", "ভাউচার",
            "খাতায়", "কাজ", "করতে", "হবে", "করেছেন", "যিনি", "যাহা", "তাহা",
        ];
        for w in core_words {
            self.dictionary.push(w.to_string());
        }
    }

    pub fn reset_state(&mut self) {
        self.current_roman_buffer.clear();
    }

    pub fn roman_buffer(&self) -> &str {
        &self.current_roman_buffer
    }

    pub fn is_vowel(&self, c: char) -> bool {
        let cl = c.to_ascii_lowercase();
        self.data.vowel.contains(cl)
    }

    pub fn is_consonant(&self, c: char) -> bool {
        let cl = c.to_ascii_lowercase();
        self.data.consonant.contains(cl)
    }

    pub fn is_punctuation(&self, c: char) -> bool {
        !self.is_vowel(c) && !self.is_consonant(c)
    }

    pub fn is_case_sensitive(&self, c: char) -> bool {
        let cl = c.to_ascii_lowercase();
        self.data.casesensitive.contains(cl)
    }

    pub fn fix_string(&self, input: &str) -> String {
        input
            .chars()
            .map(|c| {
                if self.is_case_sensitive(c) {
                    c
                } else {
                    c.to_ascii_lowercase()
                }
            })
            .collect()
    }

    fn is_exact(&self, needle: &str, haystack: &[char], start: isize, end: isize, not_neg: bool) -> bool {
        let cond = start >= 0
            && end as usize <= haystack.len()
            && start as usize <= end as usize
            && {
                let slice: String = haystack[start as usize..end as usize].iter().collect();
                slice == needle
            };
        cond != not_neg
    }

    /// Pure string transliteration matching ibus-avro avrolib.js semantics.
    pub fn transliterate(&self, input: &str) -> String {
        let fixed_str = self.fix_string(input);
        let fixed: Vec<char> = fixed_str.chars().collect();
        let mut output = String::new();
        let mut cur = 0;

        while cur < fixed.len() {
            let start = cur;
            let mut matched = false;

            for pattern in &self.data.patterns {
                let pat_chars: Vec<char> = pattern.find.chars().collect();
                let end = cur + pat_chars.len();

                if end <= fixed.len() && &fixed[start..end] == pat_chars.as_slice() {
                    let prev = start as isize - 1;

                    if let Some(ref rules) = pattern.rules {
                        for rule in rules {
                            let mut replace = true;

                            for m in &rule.matches {
                                let chk = if m.match_type == "suffix" {
                                    end as isize
                                } else {
                                    prev
                                };

                                let negative = m.scope.starts_with('!');
                                let clean_scope = if negative {
                                    &m.scope[1..]
                                } else {
                                    &m.scope[..]
                                };
                                let val = m.value.as_deref().unwrap_or("");

                                match clean_scope {
                                    "punctuation" => {
                                        let cond = (chk < 0 && m.match_type == "prefix")
                                            || (chk >= fixed.len() as isize && m.match_type == "suffix")
                                            || (chk >= 0
                                                && (chk as usize) < fixed.len()
                                                && self.is_punctuation(fixed[chk as usize]));
                                        if !(cond != negative) {
                                            replace = false;
                                            break;
                                        }
                                    }
                                    "vowel" => {
                                        let cond = ((chk >= 0 && m.match_type == "prefix")
                                            || (chk < fixed.len() as isize && m.match_type == "suffix"))
                                            && (chk >= 0
                                                && (chk as usize) < fixed.len()
                                                && self.is_vowel(fixed[chk as usize]));
                                        if !(cond != negative) {
                                            replace = false;
                                            break;
                                        }
                                    }
                                    "consonant" => {
                                        let cond = ((chk >= 0 && m.match_type == "prefix")
                                            || (chk < fixed.len() as isize && m.match_type == "suffix"))
                                            && (chk >= 0
                                                && (chk as usize) < fixed.len()
                                                && self.is_consonant(fixed[chk as usize]));
                                        if !(cond != negative) {
                                            replace = false;
                                            break;
                                        }
                                    }
                                    "exact" => {
                                        let (s, e) = if m.match_type == "suffix" {
                                            (end as isize, end as isize + val.chars().count() as isize)
                                        } else {
                                            (
                                                start as isize - val.chars().count() as isize,
                                                start as isize,
                                            )
                                        };
                                        if !self.is_exact(val, &fixed, s, e, negative) {
                                            replace = false;
                                            break;
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            if replace {
                                output.push_str(&rule.replace);
                                cur = end - 1;
                                matched = true;
                                break;
                            }
                        }
                    }

                    if matched {
                        break;
                    }

                    // Default replacement if no rule matched
                    output.push_str(&pattern.replace);
                    cur = end - 1;
                    matched = true;
                    break;
                }
            }

            if !matched {
                output.push(fixed[cur]);
            }
            cur += 1;
        }

        output
    }

    /// Process incoming key event for Phonetic typing.
    pub fn process_key(&mut self, event: &KeyEvent) -> EngineAction {
        // Backspace removes last Roman character
        if event.key_code == 8 {
            if !self.current_roman_buffer.is_empty() {
                self.current_roman_buffer.pop();
                if self.current_roman_buffer.is_empty() {
                    return EngineAction {
                        edits: Vec::new(),
                        composition: None,
                        suggestions: Vec::new(),
                        status_message: None,
                        consumed: true,
                    };
                }
                let preview = self.transliterate(&self.current_roman_buffer);
                let suggestions = self.generate_suggestions(&preview);
                return EngineAction {
                    edits: Vec::new(),
                    composition: Some(preview),
                    suggestions,
                    status_message: None,
                    consumed: true,
                };
            } else {
                return EngineAction::delete_back(1);
            }
        }

        // Commit on Space or Enter
        if event.key_code == 32 || event.key_code == 13 {
            if !self.current_roman_buffer.is_empty() {
                let committed = self.transliterate(&self.current_roman_buffer);
                self.record_word_selection(&committed);
                self.current_roman_buffer.clear();
                let space_str = if event.key_code == 32 { " " } else { "\n" };
                return EngineAction {
                    edits: vec![
                        TextEdit::Insert(committed),
                        TextEdit::Insert(space_str.to_string()),
                    ],
                    composition: None,
                    suggestions: Vec::new(),
                    status_message: None,
                    consumed: true,
                };
            } else {
                return EngineAction::pass_through();
            }
        }

        // Ignore Ctrl or Alt key combinations
        if event.ctrl || event.alt {
            return EngineAction::pass_through();
        }

        // Append valid typing characters
        if let Some(c) = event.ch {
            if c.is_ascii_alphanumeric() || c == '`' || c == '$' {
                self.current_roman_buffer.push(c);
                let preview = self.transliterate(&self.current_roman_buffer);
                let suggestions = self.generate_suggestions(&preview);
                return EngineAction {
                    edits: Vec::new(),
                    composition: Some(preview),
                    suggestions,
                    status_message: None,
                    consumed: true,
                };
            }
        }

        // Non-alphanumeric punctuation commits current buffer and passes through
        if !self.current_roman_buffer.is_empty() {
            let committed = self.transliterate(&self.current_roman_buffer);
            self.current_roman_buffer.clear();
            EngineAction {
                edits: vec![TextEdit::Insert(committed)],
                composition: None,
                suggestions: Vec::new(),
                status_message: None,
                consumed: false,
            }
        } else {
            EngineAction::pass_through()
        }
    }

    fn generate_suggestions(&self, current_translit: &str) -> Vec<Suggestion> {
        let mut results = Vec::new();

        // 1. Exact transliterated match is primary
        results.push(Suggestion {
            word: current_translit.to_string(),
            hint: None,
            score: 1000,
        });

        // 2. Dictionary completions / prefixes
        for word in &self.dictionary {
            if word.starts_with(current_translit) && word != current_translit {
                let freq = *self.user_frequency.get(word).unwrap_or(&10);
                results.push(Suggestion {
                    word: word.clone(),
                    hint: None,
                    score: freq,
                });
                if results.len() >= 5 {
                    break;
                }
            }
        }

        results
    }

    pub fn record_word_selection(&mut self, word: &str) {
        let count = self.user_frequency.entry(word.to_string()).or_insert(0);
        *count += 1;
    }
}
