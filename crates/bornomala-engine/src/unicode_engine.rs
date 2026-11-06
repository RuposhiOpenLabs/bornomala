use crate::events::{BackspaceMode, EngineAction, KeyEvent, TextEdit};
use crate::layout::{load_default_unicode_layout, LayoutDefinition};
use unicode_normalization::UnicodeNormalization;

/// Pre-base vowel signs in Bangla Unicode.
pub const I_KAR: char = '\u{09BF}'; // ি
pub const E_KAR: char = '\u{09C7}'; // ে
pub const OI_KAR: char = '\u{09C8}'; // ৈ

/// Post-base vowel signs that can combine with e-kar into split vowel signs.
pub const AA_KAR: char = '\u{09BE}'; // া
pub const O_KAR: char = '\u{09CB}'; // ো
pub const OU_KAR: char = '\u{09CC}'; // ৌ
pub const AU_LENGTH_MARK: char = '\u{09D7}'; // ৗ

/// Hasanta (Virama) and modifiers.
pub const HASANTA: char = '\u{09CD}'; // ্
pub const RA: char = '\u{09B0}'; // র
pub const YA: char = '\u{09AF}'; // য
pub const ZWJ: char = '\u{200D}';
pub const ZWNJ: char = '\u{200C}';
pub const NUKTA: char = '\u{09BC}'; // ়

/// Checks if a character is a Bangla consonant.
pub fn is_bangla_consonant(c: char) -> bool {
    matches!(c, '\u{0995}'..='\u{09B9}' | '\u{09DC}'..='\u{09DF}' | '\u{09CE}')
}

/// Checks if a character is a pre-base vowel sign.
pub fn is_prebase_kar(c: char) -> bool {
    c == I_KAR || c == E_KAR || c == OI_KAR
}

/// Checks if a character is a Bangla vowel sign (Kar).
pub fn is_bangla_vowel_sign(c: char) -> bool {
    matches!(c, '\u{09BE}'..='\u{09CC}' | '\u{09D7}')
}

/// Checks if a character is a Bangla modifier (Chandrabindu, Anusvara, Visarga).
pub fn is_bangla_modifier(c: char) -> bool {
    matches!(c, '\u{0981}'..='\u{0983}')
}

/// Result of attempting to apply Reph retroactively to the preceding cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetroactiveRephResult {
    Applied { count: usize, text: String },
    AlreadyHasReph,
    NoPrecedingConsonant,
}

/// Classic Fixed Layout Engine producing canonical Unicode.
#[derive(Debug, Clone)]
pub struct UnicodeClassicEngine {
    layout: LayoutDefinition,
    backspace_mode: BackspaceMode,
    pending_prebase_kar: Option<char>,
    pending_reph: bool,
    last_inserted: Vec<char>,
}

impl Default for UnicodeClassicEngine {
    fn default() -> Self {
        Self::new(load_default_unicode_layout(), BackspaceMode::Component)
    }
}

impl UnicodeClassicEngine {
    pub fn new(layout: LayoutDefinition, backspace_mode: BackspaceMode) -> Self {
        Self {
            layout,
            backspace_mode,
            pending_prebase_kar: None,
            pending_reph: false,
            last_inserted: Vec::new(),
        }
    }

    pub fn set_layout(&mut self, layout: LayoutDefinition) {
        self.layout = layout;
    }

    pub fn set_backspace_mode(&mut self, mode: BackspaceMode) {
        self.backspace_mode = mode;
    }

    pub fn reset_state(&mut self) {
        self.pending_prebase_kar = None;
        self.pending_reph = false;
        self.last_inserted.clear();
    }

    /// Applies Reph (র্) retroactively to the preceding consonant or cluster in `last_inserted`.
    pub fn apply_retroactive_reph(&mut self) -> RetroactiveRephResult {
        if self.pending_prebase_kar.is_some() || self.last_inserted.is_empty() {
            return RetroactiveRephResult::NoPrecedingConsonant;
        }

        let chars = &self.last_inserted;
        let mut idx = chars.len();

        // 1. Skip trailing modifiers (Chandrabindu, Anusvara, Visarga)
        while idx > 0 && is_bangla_modifier(chars[idx - 1]) {
            idx -= 1;
        }

        // 2. Skip trailing vowel signs (Kars)
        while idx > 0 && is_bangla_vowel_sign(chars[idx - 1]) {
            idx -= 1;
        }

        // 3. Must have a consonant before any vowel signs/modifiers
        if idx == 0 {
            return RetroactiveRephResult::NoPrecedingConsonant;
        }

        // Optional Nukta right after base consonant
        if idx > 0 && chars[idx - 1] == NUKTA {
            idx -= 1;
        }

        if idx == 0 || !is_bangla_consonant(chars[idx - 1]) {
            return RetroactiveRephResult::NoPrecedingConsonant;
        }

        // Consume the final consonant of the cluster
        idx -= 1;

        // 4. Scan backwards across any conjunct links: (Consonant (+ Nukta)? (+ ZWJ)? + HASANTA (+ ZWJ)?)
        loop {
            let mut temp = idx;
            if temp > 0 && (chars[temp - 1] == ZWJ || chars[temp - 1] == ZWNJ) {
                temp -= 1;
            }
            if temp > 0 && chars[temp - 1] == HASANTA {
                temp -= 1;
            } else {
                break;
            }
            if temp > 0 && (chars[temp - 1] == ZWJ || chars[temp - 1] == ZWNJ) {
                temp -= 1;
            }
            if temp > 0 && chars[temp - 1] == NUKTA {
                temp -= 1;
            }
            if temp > 0 && is_bangla_consonant(chars[temp - 1]) {
                temp -= 1;
                idx = temp;
            } else {
                break;
            }
        }

        // 5. Check if the cluster already begins with Reph (RA + HASANTA)
        if chars[idx] == RA && chars.len() > idx + 1 && chars[idx + 1] == HASANTA {
            return RetroactiveRephResult::AlreadyHasReph;
        }

        // 6. Valid preceding cluster found: chars[idx..]
        let count = chars.len() - idx;
        let cluster: Vec<char> = chars[idx..].to_vec();

        let mut new_seq = Vec::with_capacity(cluster.len() + 2);
        new_seq.push(RA);
        new_seq.push(HASANTA);
        new_seq.extend(cluster);

        let text: String = new_seq.iter().collect::<String>().nfc().collect();

        // Update self.last_inserted
        self.last_inserted.truncate(idx);
        for c in text.chars() {
            self.last_inserted.push(c);
        }

        RetroactiveRephResult::Applied { count, text }
    }

    /// Process a physical key event through the Unicode layout.
    pub fn process_key(&mut self, event: &KeyEvent) -> EngineAction {
        if !event.is_down {
            return EngineAction::pass_through();
        }

        // Handle Backspace
        if event.key_code == 8 {
            return self.handle_backspace();
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

        let chars: Vec<char> = mapped.chars().collect();
        if chars.is_empty() {
            return EngineAction::pass_through();
        }

        // Check for Juk matching against the last inserted char + current input
        if let Some(last_ch) = self.last_inserted.last().copied() {
            let combo = format!("{}{}", last_ch, mapped);
            for juk in &self.layout.juks {
                if juk.seq == combo {
                    // Replace last character with the Juk output
                    self.last_inserted.pop();
                    let out_chars: Vec<char> = juk.out.chars().collect();
                    self.last_inserted.extend(&out_chars);
                    return EngineAction {
                        edits: vec![TextEdit::ReplaceLast {
                            count: 1,
                            text: juk.out.clone(),
                        }],
                        composition: None,
                        suggestions: Vec::new(),
                        status_message: None,
                        consumed: true,
                    };
                }
            }
        }

        let first_ch = chars[0];

        // 1. Check for Reph shortcut (Shift+A in Bijoy Unicode emits র্)
        if mapped == "র্" || (chars.len() == 2 && chars[0] == RA && chars[1] == HASANTA) {
            match self.apply_retroactive_reph() {
                RetroactiveRephResult::Applied { count, text } => {
                    return EngineAction {
                        edits: vec![TextEdit::ReplaceLast { count, text }],
                        composition: None,
                        suggestions: Vec::new(),
                        status_message: None,
                        consumed: true,
                    };
                }
                RetroactiveRephResult::AlreadyHasReph => {
                    return EngineAction {
                        edits: Vec::new(),
                        composition: None,
                        suggestions: Vec::new(),
                        status_message: None,
                        consumed: true,
                    };
                }
                RetroactiveRephResult::NoPrecedingConsonant => {
                    self.pending_reph = true;
                    return EngineAction {
                        edits: Vec::new(),
                        composition: Some("র্".to_string()),
                        suggestions: Vec::new(),
                        status_message: None,
                        consumed: true,
                    };
                }
            }
        }

        // 2. Check for pre-base vowel signs typed before consonant (ি, ে, ৈ)
        if chars.len() == 1 && is_prebase_kar(first_ch) {
            self.pending_prebase_kar = Some(first_ch);
            return EngineAction {
                edits: Vec::new(),
                composition: Some(first_ch.to_string()),
                suggestions: Vec::new(),
                status_message: None,
                consumed: true,
            };
        }

        // 3. Check for split vowel sign completion:
        // If the last inserted was Consonant + E_KAR, and user now types AA_KAR ('া'),
        // combine them into O_KAR ('ো').
        // If user now types AU_LENGTH_MARK ('ৗ'), combine them into OU_KAR ('ৌ').
        if self.last_inserted.len() >= 2 {
            let n = self.last_inserted.len();
            if self.last_inserted[n - 1] == E_KAR {
                if first_ch == AA_KAR {
                    self.last_inserted.pop();
                    self.last_inserted.push(O_KAR);
                    return EngineAction {
                        edits: vec![TextEdit::ReplaceLast {
                            count: 1,
                            text: O_KAR.to_string(),
                        }],
                        composition: None,
                        suggestions: Vec::new(),
                        status_message: None,
                        consumed: true,
                    };
                } else if first_ch == AU_LENGTH_MARK {
                    self.last_inserted.pop();
                    self.last_inserted.push(OU_KAR);
                    return EngineAction {
                        edits: vec![TextEdit::ReplaceLast {
                            count: 1,
                            text: OU_KAR.to_string(),
                        }],
                        composition: None,
                        suggestions: Vec::new(),
                        status_message: None,
                        consumed: true,
                    };
                }
            }
        }

        // 4. Normal consonant or cluster output with any pending prebase kar or reph
        let mut out = String::new();

        if self.pending_reph {
            out.push(RA);
            out.push(HASANTA);
            self.pending_reph = false;
        }

        out.push_str(&mapped);

        if let Some(kar) = self.pending_prebase_kar.take() {
            // Pre-base kar is placed AFTER the consonant
            out.push(kar);
        }

        let normalized: String = out.nfc().collect();
        for c in normalized.chars() {
            self.last_inserted.push(c);
        }

        EngineAction {
            edits: vec![TextEdit::Insert(normalized)],
            composition: None,
            suggestions: Vec::new(),
            status_message: None,
            consumed: true,
        }
    }

    /// Handles logical backspace in the Unicode buffer.
    pub fn handle_backspace(&mut self) -> EngineAction {
        if self.pending_prebase_kar.is_some() {
            self.pending_prebase_kar = None;
            return EngineAction {
                edits: Vec::new(),
                composition: None,
                suggestions: Vec::new(),
                status_message: None,
                consumed: true,
            };
        }

        if self.pending_reph {
            self.pending_reph = false;
            return EngineAction {
                edits: Vec::new(),
                composition: None,
                suggestions: Vec::new(),
                status_message: None,
                consumed: true,
            };
        }

        if self.last_inserted.is_empty() {
            return EngineAction::delete_back(1);
        }

        match self.backspace_mode {
            BackspaceMode::Component => {
                self.last_inserted.pop();
                EngineAction::delete_back(1)
            }
            BackspaceMode::Atomic => {
                // Determine the size of the cluster to remove
                let mut count = 0;
                while let Some(ch) = self.last_inserted.pop() {
                    count += 1;
                    if is_bangla_consonant(ch) {
                        // Check if previous char is hasanta
                        if self.last_inserted.last().copied() == Some(HASANTA) {
                            continue;
                        }
                        break;
                    }
                }
                if count == 0 {
                    count = 1;
                }
                EngineAction::delete_back(count)
            }
        }
    }
}
