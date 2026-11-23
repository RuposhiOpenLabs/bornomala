use crate::events::{EngineAction, KeyEvent, Suggestion, TextEdit};
use bornomala_phonetic::PhoneticEngine as CorePhoneticEngine;

/// Avro-compatible Phonetic Transliteration Engine for Bornomala.
/// Wraps `bornomala_phonetic::PhoneticEngine` and manages live composition state,
/// candidate suggestions, and differential inline text replacement.
#[derive(Clone)]
pub struct PhoneticEngine {
    core: CorePhoneticEngine,
    last_preview_char_count: usize,
    selected_candidate_idx: usize,
    current_suggestions: Vec<String>,
}

impl Default for PhoneticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PhoneticEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhoneticEngine")
            .field("buffer", &self.core.input_buffer())
            .field("last_preview_char_count", &self.last_preview_char_count)
            .field("selected_candidate_idx", &self.selected_candidate_idx)
            .finish()
    }
}

impl PhoneticEngine {
    pub fn new() -> Self {
        Self {
            core: CorePhoneticEngine::new(),
            last_preview_char_count: 0,
            selected_candidate_idx: 0,
            current_suggestions: Vec::new(),
        }
    }

    pub fn new_in_memory() -> Self {
        Self {
            core: CorePhoneticEngine::new_in_memory(),
            last_preview_char_count: 0,
            selected_candidate_idx: 0,
            current_suggestions: Vec::new(),
        }
    }

    pub fn reset_state(&mut self) {
        self.core.clear_buffer();
        self.last_preview_char_count = 0;
        self.selected_candidate_idx = 0;
        self.current_suggestions.clear();
    }

    pub fn roman_buffer(&self) -> &str {
        self.core.input_buffer()
    }

    pub fn last_preview_char_count(&self) -> usize {
        self.last_preview_char_count
    }

    pub fn selected_candidate_index(&self) -> usize {
        self.selected_candidate_idx
    }

    pub fn current_candidates(&self) -> &[String] {
        &self.current_suggestions
    }

    pub fn transliterate(&self, input: &str) -> String {
        self.core.transliterate_classic(input)
    }

    pub fn record_word_selection(&mut self, roman: &str, bangla: &str) {
        self.core.record_selection(roman, bangla);
    }

    /// Select candidate at specific index (0-based) and commit it.
    pub fn select_candidate_and_commit(&mut self, idx: usize) -> Option<EngineAction> {
        if self.current_suggestions.is_empty() {
            return None;
        }

        let chosen = if idx < self.current_suggestions.len() {
            self.current_suggestions[idx].clone()
        } else {
            self.current_suggestions[0].clone()
        };

        let roman = self.core.input_buffer().to_string();
        self.core.record_selection(&roman, &chosen);

        let edit = TextEdit::ReplaceLast {
            count: self.last_preview_char_count,
            text: chosen,
        };

        self.reset_state();

        Some(EngineAction {
            edits: vec![edit],
            composition: None,
            suggestions: Vec::new(),
            status_message: None,
            consumed: true,
        })
    }

    /// Process incoming key event for Phonetic typing.
    pub fn process_key(&mut self, event: &KeyEvent) -> EngineAction {
        if !event.is_down {
            return EngineAction::pass_through();
        }

        // Pass through Ctrl or Alt combinations (shortcuts are handled by parent)
        if event.ctrl || event.alt {
            return EngineAction::pass_through();
        }

        // 1. Escape: Cancel composition and clear preview
        if event.key_code == 0x1B {
            if !self.core.input_buffer().is_empty() || self.last_preview_char_count > 0 {
                let count = self.last_preview_char_count;
                self.reset_state();
                return EngineAction {
                    edits: if count > 0 {
                        vec![TextEdit::DeleteBack(count)]
                    } else {
                        Vec::new()
                    },
                    composition: None,
                    suggestions: Vec::new(),
                    status_message: None,
                    consumed: true,
                };
            }
            return EngineAction::pass_through();
        }

        // 2. Tab or Down Arrow: Cycle to next suggestion
        if (event.key_code == 0x09 || event.key_code == 0x28)
            && !self.current_suggestions.is_empty()
        {
            self.selected_candidate_idx =
                (self.selected_candidate_idx + 1) % self.current_suggestions.len();
            let new_preview = self.current_suggestions[self.selected_candidate_idx].clone();
            let edit = TextEdit::ReplaceLast {
                count: self.last_preview_char_count,
                text: new_preview.clone(),
            };
            self.last_preview_char_count = new_preview.chars().count();
            return EngineAction {
                edits: vec![edit],
                composition: Some(new_preview),
                suggestions: self.build_suggestion_list(),
                status_message: None,
                consumed: true,
            };
        }

        // 3. Up Arrow: Cycle to previous suggestion
        if event.key_code == 0x26 && !self.current_suggestions.is_empty() {
            if self.selected_candidate_idx == 0 {
                self.selected_candidate_idx = self.current_suggestions.len() - 1;
            } else {
                self.selected_candidate_idx -= 1;
            }
            let new_preview = self.current_suggestions[self.selected_candidate_idx].clone();
            let edit = TextEdit::ReplaceLast {
                count: self.last_preview_char_count,
                text: new_preview.clone(),
            };
            self.last_preview_char_count = new_preview.chars().count();
            return EngineAction {
                edits: vec![edit],
                composition: Some(new_preview),
                suggestions: self.build_suggestion_list(),
                status_message: None,
                consumed: true,
            };
        }

        // 4. Number keys 1-9: Select candidate directly when candidate list is visible
        if let Some(c) = event.ch {
            if ('1'..='9').contains(&c) && !self.current_suggestions.is_empty() {
                let idx = (c as usize) - ('1' as usize);
                if idx < self.current_suggestions.len() {
                    return self.select_candidate_and_commit(idx).unwrap();
                }
            }
        }

        // 5. Backspace: Edit active composition
        if event.key_code == 0x08 {
            if !self.core.input_buffer().is_empty() {
                self.core.pop_char();
                if self.core.input_buffer().is_empty() {
                    let count = self.last_preview_char_count;
                    self.reset_state();
                    return EngineAction {
                        edits: if count > 0 {
                            vec![TextEdit::DeleteBack(count)]
                        } else {
                            Vec::new()
                        },
                        composition: None,
                        suggestions: Vec::new(),
                        status_message: None,
                        consumed: true,
                    };
                }

                self.update_suggestions();
                let top_preview = self.current_suggestions[0].clone();
                let edit = TextEdit::ReplaceLast {
                    count: self.last_preview_char_count,
                    text: top_preview.clone(),
                };
                self.last_preview_char_count = top_preview.chars().count();
                self.selected_candidate_idx = 0;

                return EngineAction {
                    edits: vec![edit],
                    composition: Some(top_preview),
                    suggestions: self.build_suggestion_list(),
                    status_message: None,
                    consumed: true,
                };
            } else {
                return EngineAction::delete_back(1);
            }
        }

        // 6. Space or Enter: Commit current composition + trailing space/newline
        if event.key_code == 0x20 || event.key_code == 0x0D {
            if !self.core.input_buffer().is_empty() {
                let chosen = if !self.current_suggestions.is_empty() {
                    self.current_suggestions[self.selected_candidate_idx].clone()
                } else {
                    self.core.transliterate_classic(self.core.input_buffer())
                };

                let roman = self.core.input_buffer().to_string();
                self.core.record_selection(&roman, &chosen);

                let trailing = if event.key_code == 0x20 { " " } else { "\n" };

                let edit = TextEdit::ReplaceLast {
                    count: self.last_preview_char_count,
                    text: format!("{}{}", chosen, trailing),
                };

                self.reset_state();

                return EngineAction {
                    edits: vec![edit],
                    composition: None,
                    suggestions: Vec::new(),
                    status_message: None,
                    consumed: true,
                };
            } else {
                return EngineAction::pass_through();
            }
        }

        // 7. Alphanumeric / backtick / dollar / period: Active Roman buffer expansion
        if let Some(c) = event.ch {
            if c.is_ascii_alphanumeric() || c == '`' || c == '$' || c == '.' {
                self.core.push_char(c);
                self.update_suggestions();

                let top_preview = self.current_suggestions[0].clone();
                let edit = TextEdit::ReplaceLast {
                    count: self.last_preview_char_count,
                    text: top_preview.clone(),
                };
                self.last_preview_char_count = top_preview.chars().count();
                self.selected_candidate_idx = 0;

                return EngineAction {
                    edits: vec![edit],
                    composition: Some(top_preview),
                    suggestions: self.build_suggestion_list(),
                    status_message: None,
                    consumed: true,
                };
            }
        }

        // 8. Other punctuation / symbols: Commit current word and pass through punctuation
        if !self.core.input_buffer().is_empty() {
            let chosen = if !self.current_suggestions.is_empty() {
                self.current_suggestions[self.selected_candidate_idx].clone()
            } else {
                self.core.transliterate_classic(self.core.input_buffer())
            };

            let roman = self.core.input_buffer().to_string();
            self.core.record_selection(&roman, &chosen);

            let mut punct_str = String::new();
            if let Some(c) = event.ch {
                // If it's standard Bengali punctuation like '.', Avro transliterates it to '।'
                let transliterated_punct = self.core.transliterate_classic(&c.to_string());
                punct_str.push_str(&transliterated_punct);
            }

            let edit = TextEdit::ReplaceLast {
                count: self.last_preview_char_count,
                text: format!("{}{}", chosen, punct_str),
            };

            self.reset_state();

            return EngineAction {
                edits: vec![edit],
                composition: None,
                suggestions: Vec::new(),
                status_message: None,
                consumed: true,
            };
        }

        // Default pass-through if buffer is empty
        if let Some(c) = event.ch {
            // Check if punctuation like '.' should produce '।' even with empty buffer
            if c == '.' || c == '$' {
                let converted = self.core.transliterate_classic(&c.to_string());
                if converted != c.to_string() {
                    return EngineAction::insert(converted);
                }
            }
        }

        EngineAction::pass_through()
    }

    fn update_suggestions(&mut self) {
        let buf = self.core.input_buffer();
        if buf.is_empty() {
            self.current_suggestions.clear();
            return;
        }
        self.current_suggestions = self.core.get_candidates(buf);
        if self.current_suggestions.is_empty() {
            self.current_suggestions
                .push(self.core.transliterate_classic(buf));
        }
    }

    fn build_suggestion_list(&self) -> Vec<Suggestion> {
        let roman = self.core.input_buffer();
        self.current_suggestions
            .iter()
            .enumerate()
            .map(|(i, w)| Suggestion {
                word: w.clone(),
                hint: if i == 0 {
                    Some(roman.to_string())
                } else {
                    None
                },
                score: (100usize.saturating_sub(i * 5)) as u32,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phonetic_live_composition_keystrokes() {
        let mut engine = PhoneticEngine::new();

        // 1. Type 'a'
        let act1 = engine.process_key(&KeyEvent::new_down(65, Some('a'), false, false, false));
        assert_eq!(
            act1.edits,
            vec![TextEdit::ReplaceLast {
                count: 0,
                text: "আ".to_string()
            }]
        );
        assert_eq!(act1.composition, Some("আ".to_string()));

        // 2. Type 'm'
        let act2 = engine.process_key(&KeyEvent::new_down(77, Some('m'), false, false, false));
        assert_eq!(
            act2.edits,
            vec![TextEdit::ReplaceLast {
                count: 1,
                text: "আম".to_string()
            }]
        );

        // 3. Type 'i'
        let act3 = engine.process_key(&KeyEvent::new_down(73, Some('i'), false, false, false));
        assert_eq!(
            act3.edits,
            vec![TextEdit::ReplaceLast {
                count: 2,
                text: "আমি".to_string()
            }]
        );

        // 4. Press Space
        let act4 = engine.process_key(&KeyEvent::new_down(32, Some(' '), false, false, false));
        assert_eq!(
            act4.edits,
            vec![TextEdit::ReplaceLast {
                count: 3,
                text: "আমি ".to_string()
            }]
        );
        assert_eq!(engine.roman_buffer(), "");
        assert_eq!(engine.last_preview_char_count(), 0);
    }

    #[test]
    fn test_phonetic_backspace_editing() {
        let mut engine = PhoneticEngine::new();

        // Type 'a', 'm'
        engine.process_key(&KeyEvent::new_down(65, Some('a'), false, false, false));
        engine.process_key(&KeyEvent::new_down(77, Some('m'), false, false, false));

        // Backspace 'm'
        let bs1 = engine.process_key(&KeyEvent::new_down(8, None, false, false, false));
        assert_eq!(
            bs1.edits,
            vec![TextEdit::ReplaceLast {
                count: 2,
                text: "আ".to_string()
            }]
        );
        assert_eq!(engine.roman_buffer(), "a");

        // Backspace 'a'
        let bs2 = engine.process_key(&KeyEvent::new_down(8, None, false, false, false));
        assert_eq!(bs2.edits, vec![TextEdit::DeleteBack(1)]);
        assert_eq!(engine.roman_buffer(), "");
    }

    #[test]
    fn test_phonetic_escape_cancellation() {
        let mut engine = PhoneticEngine::new();

        engine.process_key(&KeyEvent::new_down(65, Some('a'), false, false, false));
        engine.process_key(&KeyEvent::new_down(77, Some('m'), false, false, false));

        // Escape cancels composition
        let esc = engine.process_key(&KeyEvent::new_down(27, None, false, false, false));
        assert_eq!(esc.edits, vec![TextEdit::DeleteBack(2)]);
        assert_eq!(engine.roman_buffer(), "");
    }
}
