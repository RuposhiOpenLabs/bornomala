use crate::events::{EngineAction, KeyEvent, TextEdit};
use crate::layout::{load_default_ansi_layout, LayoutDefinition};

/// Classic Fixed Layout Engine producing legacy ANSI SutonnyMJ glyph sequences.
#[derive(Debug, Clone)]
pub struct AnsiClassicEngine {
    layout: LayoutDefinition,
    last_inserted: Vec<char>,
}

impl Default for AnsiClassicEngine {
    fn default() -> Self {
        Self::new(load_default_ansi_layout())
    }
}

impl AnsiClassicEngine {
    pub fn new(layout: LayoutDefinition) -> Self {
        Self {
            layout,
            last_inserted: Vec::new(),
        }
    }

    pub fn set_layout(&mut self, layout: LayoutDefinition) {
        self.layout = layout;
    }

    pub fn reset_state(&mut self) {
        self.last_inserted.clear();
    }

    /// Process a physical key event through the ANSI layout.
    pub fn process_key(&mut self, event: &KeyEvent) -> EngineAction {
        if !event.is_down {
            return EngineAction::pass_through();
        }

        // Handle Backspace
        if event.key_code == 8 {
            if self.last_inserted.pop().is_some() {
                return EngineAction::delete_back(1);
            } else {
                return EngineAction::delete_back(1);
            }
        }

        // Pass-through Ctrl/Alt combinations that are not typing
        if event.ctrl || event.alt {
            self.reset_state();
            return EngineAction::pass_through();
        }

        // Determine mapped character from layout
        let mapped = if let Some(m) = self.layout.lookup(event.key_code, event.shift, false) {
            m
        } else if let Some(c) = event.ch {
            if c.is_control() {
                self.reset_state();
                return EngineAction::pass_through();
            }
            c.to_string()
        } else {
            return EngineAction::pass_through();
        };

        if mapped.is_empty() {
            return EngineAction::pass_through();
        }

        // Check for multi-key Juk matching against the buffer
        // Look up to 3 previous characters in last_inserted + current mapped character
        let mut best_match: Option<(usize, String)> = None;
        let buf_len = self.last_inserted.len();

        for prefix_len in (1..=3.min(buf_len)).rev() {
            let prefix: String = self.last_inserted[buf_len - prefix_len..].iter().collect();
            let candidate = format!("{}{}", prefix, mapped);

            for juk in &self.layout.juks {
                if juk.seq == candidate {
                    best_match = Some((prefix_len, juk.out.clone()));
                    break;
                }
            }
            if best_match.is_some() {
                break;
            }
        }

        if let Some((replace_count, out_str)) = best_match {
            for _ in 0..replace_count {
                self.last_inserted.pop();
            }
            for c in out_str.chars() {
                self.last_inserted.push(c);
            }
            return EngineAction {
                edits: vec![TextEdit::ReplaceLast {
                    count: replace_count,
                    text: out_str,
                }],
                composition: None,
                suggestions: Vec::new(),
                status_message: None,
                consumed: true,
            };
        }

        // Otherwise, insert the mapped text directly
        for c in mapped.chars() {
            self.last_inserted.push(c);
        }

        EngineAction {
            edits: vec![TextEdit::Insert(mapped)],
            composition: None,
            suggestions: Vec::new(),
            status_message: None,
            consumed: true,
        }
    }
}
