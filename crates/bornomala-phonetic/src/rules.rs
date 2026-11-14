use once_cell::sync::Lazy;
use serde::Deserialize;

pub static EMBEDDED_PHONETIC_RULES: &str = include_str!("../data/phonetic_rules.json");

static GLOBAL_RULES: Lazy<AvroRules> = Lazy::new(|| {
    AvroRules::load_from_json(EMBEDDED_PHONETIC_RULES).expect("Failed to load Avro phonetic rules")
});

#[derive(Debug, Clone, Deserialize)]
pub struct MatchRule {
    #[serde(rename = "type")]
    pub match_type: String, // "prefix" or "suffix"
    pub scope: String, // "punctuation", "vowel", "consonant", "exact", or "!..."
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatternRule {
    #[serde(default)]
    pub matches: Vec<MatchRule>,
    pub replace: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhoneticPattern {
    pub find: String,
    pub replace: String,
    #[serde(default)]
    pub rules: Option<Vec<PatternRule>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhoneticRuleData {
    pub patterns: Vec<PhoneticPattern>,
    #[serde(default)]
    pub vowel: String,
    #[serde(default)]
    pub consonant: String,
    #[serde(default)]
    pub casesensitive: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhoneticRuleFile {
    pub data: PhoneticRuleData,
}

#[derive(Debug, Clone)]
pub struct AvroRules {
    patterns: Vec<PhoneticPattern>,
    vowel_chars: String,
    consonant_chars: String,
    case_sensitive_chars: String,
}

impl AvroRules {
    pub fn global() -> &'static Self {
        &GLOBAL_RULES
    }

    pub fn load_from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let file: PhoneticRuleFile = serde_json::from_str(json_str)?;
        let mut patterns = file.data.patterns;

        // Longest match first: sort patterns descending by find pattern length
        patterns.sort_by(|a, b| b.find.len().cmp(&a.find.len()));

        Ok(Self {
            patterns,
            vowel_chars: file.data.vowel.to_ascii_lowercase(),
            consonant_chars: file.data.consonant.to_ascii_lowercase(),
            case_sensitive_chars: file.data.casesensitive.to_ascii_lowercase(),
        })
    }

    #[inline]
    pub fn is_vowel(&self, c: char) -> bool {
        let cl = c.to_ascii_lowercase();
        self.vowel_chars.contains(cl)
    }

    #[inline]
    pub fn is_consonant(&self, c: char) -> bool {
        let cl = c.to_ascii_lowercase();
        self.consonant_chars.contains(cl)
    }

    #[inline]
    pub fn is_punctuation(&self, c: char) -> bool {
        !self.is_vowel(c) && !self.is_consonant(c)
    }

    #[inline]
    pub fn is_case_sensitive(&self, c: char) -> bool {
        let cl = c.to_ascii_lowercase();
        self.case_sensitive_chars.contains(cl)
    }

    pub fn fix_case(&self, input: &str) -> String {
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

    fn is_exact(
        &self,
        needle: &str,
        haystack: &[char],
        start: isize,
        end: isize,
        not_neg: bool,
    ) -> bool {
        let cond =
            start >= 0 && end as usize <= haystack.len() && start as usize <= end as usize && {
                let slice: String = haystack[start as usize..end as usize].iter().collect();
                slice == needle
            };
        cond != not_neg
    }

    /// Pure rule-based transliteration matching Avro Classic / libavrophonetic semantics 100%.
    pub fn transliterate(&self, input: &str) -> String {
        let fixed_str = self.fix_case(input);
        let fixed: Vec<char> = fixed_str.chars().collect();
        let mut output = String::with_capacity(input.len() * 3);
        let mut cur = 0;

        while cur < fixed.len() {
            let start = cur;
            let mut matched = false;

            for pattern in &self.patterns {
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
                                            || (chk >= fixed.len() as isize
                                                && m.match_type == "suffix")
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
                                            || (chk < fixed.len() as isize
                                                && m.match_type == "suffix"))
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
                                            || (chk < fixed.len() as isize
                                                && m.match_type == "suffix"))
                                            && (chk >= 0
                                                && (chk as usize) < fixed.len()
                                                && self.is_consonant(fixed[chk as usize]));
                                        if !(cond != negative) {
                                            replace = false;
                                            break;
                                        }
                                    }
                                    "exact" => {
                                        let val_char_count = val.chars().count() as isize;
                                        let (s, e) = if m.match_type == "suffix" {
                                            (end as isize, end as isize + val_char_count)
                                        } else {
                                            (start as isize - val_char_count, start as isize)
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

                    // Default replacement if no rule exception matched
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avro_rules_conformance() {
        let rules = AvroRules::global();
        assert_eq!(rules.transliterate("a"), "আ");
        assert_eq!(rules.transliterate("ia"), "ইয়া");
        assert_eq!(rules.transliterate("(a"), "(আ");
        assert_eq!(rules.transliterate("(a`"), "(া");
        assert_eq!(rules.transliterate("aa"), "আআ");
        assert_eq!(rules.transliterate("aa`"), "আা");
        assert_eq!(rules.transliterate("amar"), "আমার");
        assert_eq!(rules.transliterate("."), "।");
        assert_eq!(rules.transliterate(".."), "।।");
        assert_eq!(rules.transliterate("..."), "...");
        assert_eq!(rules.transliterate("R"), "ড়");
        assert_eq!(rules.transliterate("Rh"), "ঢ়");
        assert_eq!(rules.transliterate("kobja"), "কব্জা");
        assert_eq!(rules.transliterate("bhladimir"), "ভ্লাদিমির");
        assert_eq!(rules.transliterate("vladimir"), "ভ্লাদিমির");
        assert_eq!(rules.transliterate("lobdho"), "লব্ধ");
        assert_eq!(rules.transliterate("jobdo"), "জব্দ");
        assert_eq!(rules.transliterate("accha"), "আচ্ছা");
        assert_eq!(rules.transliterate("bacca"), "বাচ্চা");
        assert_eq!(rules.transliterate("boDDo"), "বড্ড");
        assert_eq!(rules.transliterate("bou"), "বউ");
        assert_eq!(rules.transliterate("bOU"), "বৌ");
        assert_eq!(rules.transliterate("korrmo"), "কর্ম");
        assert_eq!(rules.transliterate("korrmofol"), "কর্মফল");
        assert_eq!(rules.transliterate("borrno"), "বর্ন");
        assert_eq!(rules.transliterate("borrNo"), "বর্ণ");
        assert_eq!(rules.transliterate("borrNomala"), "বর্ণমালা");
        assert_eq!(rules.transliterate("ami"), "আমি");
        assert_eq!(rules.transliterate("tumi"), "তুমি");
        assert_eq!(rules.transliterate("bangla"), "বাংলা");
        assert_eq!(rules.transliterate("bangladesh"), "বাংলাদেশ");
        assert_eq!(rules.transliterate("shikkha"), "শিক্ষা");
        assert_eq!(rules.transliterate("ranna"), "রান্না");
        assert_eq!(rules.transliterate("$100"), "৳১০০");
    }
}
