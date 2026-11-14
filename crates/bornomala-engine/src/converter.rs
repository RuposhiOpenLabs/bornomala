use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;

pub static EMBEDDED_SUTONNYMJ_MAPPINGS: &str =
    include_str!("../../../data/converters/sutonnymj_mappings.json");

#[derive(Debug, Clone)]
pub struct ConverterTables {
    pub unicode_to_ansi_single: HashMap<char, String>,
    pub ansi_to_unicode_single: HashMap<String, String>,
    pub conjunct_to_ansi: HashMap<String, String>,
    pub ansi_to_conjunct: HashMap<String, String>,
}

impl Default for ConverterTables {
    fn default() -> Self {
        Self::load_embedded()
    }
}

impl ConverterTables {
    pub fn load_embedded() -> Self {
        let v: serde_json::Value = serde_json::from_str(EMBEDDED_SUTONNYMJ_MAPPINGS)
            .expect("Embedded sutonnymj_mappings.json must be valid");

        let mut u2a_single = HashMap::new();
        let mut a2u_single = HashMap::new();

        if let Some(single) = v.get("single_char").and_then(|s| s.as_object()) {
            for (k_hex, v_str) in single {
                if let Ok(cp) = u32::from_str_radix(k_hex, 16) {
                    if let Some(c) = char::from_u32(cp) {
                        let out = v_str.as_str().unwrap_or("").to_string();
                        u2a_single.insert(c, out.clone());
                        if !out.is_empty() && !a2u_single.contains_key(&out) {
                            a2u_single.insert(out, c.to_string());
                        }
                    }
                }
            }
        }

        let mut conj2a = HashMap::new();
        let mut a2conj = HashMap::new();

        // Standard SutonnyMJ conjunct mappings
        let standard_conjuncts: &[(&str, &str)] = &[
            ("ক\u{09CD}ক", "³"),
            ("ক\u{09CD}ত", "°"),
            ("ক\u{09CD}স", "µ"),
            ("ক\u{09CD}ষ", "´"),
            ("ক\u{09CD}ষ\u{09CD}ম", "ÿ"),
            ("ঙ\u{09CD}ক", "º"),
            ("ঙ\u{09CD}খ", "»"),
            ("ঙ\u{09CD}গ", "¼"),
            ("ঙ\u{09CD}ঘ", "½"),
            ("চ\u{09CD}চ", "¾"),
            ("চ\u{09CD}ছ", "À"),
            ("চ\u{09CD}ছ\u{09CD}ব", "Á"),
            ("জ\u{09CD}জ", "Â"),
            ("জ\u{09CD}ঝ", "Ã"),
            ("জ\u{09CD}ঞ", "Ä"),
            ("ঞ\u{09CD}চ", "Æ"),
            ("ঞ\u{09CD}ছ", "Ç"),
            ("ঞ\u{09CD}জ", "È"),
            ("ঞ\u{09CD}ঝ", "É"),
            ("ট\u{09CD}ট", "Ð"),
            ("ড\u{09CD}ড", "Ë"),
            ("ণ\u{09CD}ট", "Î"),
            ("ণ\u{09CD}ঠ", "Ì"),
            ("ণ\u{09CD}ড", "Ï"),
            ("ণ\u{09CD}ণ", "×"),
            ("ত\u{09CD}ত", "Ø"),
            ("ত\u{09CD}থ", "Ù"),
            ("ত\u{09CD}ন", "Ú"),
            ("ত\u{09CD}ম", "Û"),
            ("ত\u{09CD}র", "Ý"),
            ("ত\u{09CD}ব", "Ü"),
            ("দ\u{09CD}দ", "Þ"),
            ("দ\u{09CD}ধ", "ß"),
            ("দ\u{09CD}ব", "à"),
            ("দ\u{09CD}ম", "â"),
            ("ধ\u{09CD}ন", "ã"),
            ("ধ\u{09CD}ব", "ä"),
            ("ন\u{09CD}ত", "ç"),
            ("ন\u{09CD}থ", "é"),
            ("ন\u{09CD}দ", "ì"),
            ("ন\u{09CD}ধ", "í"),
            ("ন\u{09CD}ন", "î"),
            ("ন\u{09CD}ব", "ê"),
            ("ন\u{09CD}ম", "ë"),
            ("প\u{09CD}ট", "ð"),
            ("প\u{09CD}ত", "ñ"),
            ("প\u{09CD}ন", "ó"),
            ("প\u{09CD}প", "ô"),
            ("প\u{09CD}স", "ò"),
            ("ব\u{09CD}দ", "õ"),
            ("ব\u{09CD}ধ", "ö"),
            ("ব\u{09CD}ব", "÷"),
            ("ভ\u{09CD}ন", "ù"),
            ("ম\u{09CD}প", "ý"),
            ("ম\u{09CD}ফ", "þ"),
            ("ম\u{09CD}ব", "Í"),
            ("ম\u{09CD}ভ", "œ"),
            ("ম\u{09CD}ম", "¦"),
            ("ম\u{09CD}ল", "¥"),
            ("ল\u{09CD}ক", "ø"),
            ("ল\u{09CD}গ", "•"),
            ("ল\u{09CD}ট", "”"),
            ("ল\u{09CD}ড", "˜"),
            ("ল\u{09CD}প", "™"),
            ("ল\u{09CD}ফ", "›"),
            ("ল\u{09CD}ব", "š"),
            ("ল\u{09CD}ম", "¤"),
            ("ল\u{09CD}ল", "®"),
            ("ষ\u{09CD}ক", "¯"),
            ("ষ\u{09CD}ট", "‹"),
            ("ষ\u{09CD}ঠ", "’"),
            ("ষ\u{09CD}ণ", "¢"),
            ("ষ\u{09CD}প", "ú"),
            ("ষ\u{09CD}ফ", "è"),
            ("ষ\u{09CD}ম", "¡"),
            ("স\u{09CD}ক", "Ÿ"),
            ("স\u{09CD}খ", "^"),
            ("স\u{09CD}ত", "§"),
            ("স\u{09CD}থ", "¬"),
            ("স\u{09CD}ন", "¯œ"),
            ("স\u{09CD}প", "¯ú"),
            ("স\u{09CD}ফ", "ù"),
            ("স\u{09CD}ব", "¯^"),
            ("স\u{09CD}ম", "¯§"),
            ("হ\u{09CD}ন", "ý"),
            ("হ\u{09CD}ম", "þ"),
            ("হ\u{09CD}র", "nª"),
            ("হ\u{09CD}ঋ", "ü"),
        ];

        for (u, a) in standard_conjuncts {
            conj2a.insert(u.to_string(), a.to_string());
            a2conj.insert(a.to_string(), u.to_string());
        }

        Self {
            unicode_to_ansi_single: u2a_single,
            ansi_to_unicode_single: a2u_single,
            conjunct_to_ansi: conj2a,
            ansi_to_conjunct: a2conj,
        }
    }
}

/// Bidirectional Unicode <-> ANSI (SutonnyMJ) Converter.
#[derive(Debug, Clone, Default)]
pub struct BanglaConverter {
    tables: ConverterTables,
}

impl BanglaConverter {
    pub fn new() -> Self {
        Self {
            tables: ConverterTables::default(),
        }
    }

    /// Converts canonical Unicode Bangla text to legacy SutonnyMJ ANSI text.
    pub fn unicode_to_ansi(&self, text: &str) -> (String, Vec<char>) {
        let nfc_text: String = text.nfc().collect();
        let normalized = nfc_text
            .replace("\u{09A1}\u{09BC}", "\u{09DC}")
            .replace("\u{09A2}\u{09BC}", "\u{09DD}")
            .replace("\u{09AF}\u{09BC}", "\u{09DF}");
        let chars: Vec<char> = normalized.chars().collect();
        let mut result = String::new();
        let mut unrepresented = Vec::new();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            // 1. Detect Reph (র + ্ at cluster start)
            let mut has_reph = false;
            if c == '\u{09B0}' && i + 1 < chars.len() && chars[i + 1] == '\u{09CD}' {
                // If followed by another consonant, this is a reph
                if i + 2 < chars.len() && crate::unicode_engine::is_bangla_consonant(chars[i + 2]) {
                    has_reph = true;
                    i += 2;
                }
            }

            if i >= chars.len() {
                if has_reph {
                    result.push('©');
                }
                break;
            }

            // 2. Extract Consonant Cluster (Base consonant + optional hasanta + subsequent consonants)
            let cluster_start = i;
            let mut cluster_end = i;

            if crate::unicode_engine::is_bangla_consonant(chars[i]) {
                cluster_end = i + 1;
                while cluster_end + 1 < chars.len()
                    && chars[cluster_end] == '\u{09CD}'
                    && crate::unicode_engine::is_bangla_consonant(chars[cluster_end + 1])
                {
                    cluster_end += 2;
                }
            }

            // Check if there is a Ya-phala (্ + য) or Ra-phala (্ + র) at the end of cluster
            let mut has_yaphala = false;
            let mut has_raphala = false;
            if cluster_end > cluster_start
                && cluster_end + 1 < chars.len()
                && chars[cluster_end] == '\u{09CD}'
            {
                if chars[cluster_end + 1] == '\u{09AF}' {
                    has_yaphala = true;
                    cluster_end += 2;
                } else if chars[cluster_end + 1] == '\u{09B0}' {
                    has_raphala = true;
                    cluster_end += 2;
                }
            }

            // 3. Inspect attached vowel signs right after the consonant cluster
            let mut prebase_kar: Option<char> = None;
            let mut postbase_kar: Option<char> = None;
            let mut split_kar: Option<char> = None;
            let mut next_pos = if cluster_end > cluster_start {
                cluster_end
            } else {
                cluster_start + 1
            };

            if cluster_end > cluster_start && cluster_end < chars.len() {
                let v = chars[cluster_end];
                if v == '\u{09BF}' || v == '\u{09C7}' || v == '\u{09C8}' {
                    prebase_kar = Some(v);
                    next_pos = cluster_end + 1;
                } else if v == '\u{09CB}' || v == '\u{09CC}' {
                    split_kar = Some(v);
                    next_pos = cluster_end + 1;
                } else if matches!(
                    v,
                    '\u{09BE}' | '\u{09C0}' | '\u{09C1}' | '\u{09C2}' | '\u{09C3}'
                ) {
                    postbase_kar = Some(v);
                    next_pos = cluster_end + 1;
                }
            }

            // 4. Emit pre-base vowel glyph first (in visual order)
            if let Some(pk) = prebase_kar {
                match pk {
                    '\u{09BF}' => result.push('w'), // ি -> w
                    '\u{09C7}' => result.push('‡'), // ে -> ‡
                    '\u{09C8}' => result.push('‰'), // ৈ -> ‰
                    _ => {}
                }
            } else if split_kar.is_some() {
                result.push('‡'); // Both o-kar and ou-kar start with ‡
            }

            // 5. Emit the consonant or conjunct cluster
            if cluster_end > cluster_start {
                let cluster_str: String = chars[cluster_start..cluster_end].iter().collect();
                if let Some(ansi_conj) = self.tables.conjunct_to_ansi.get(&cluster_str) {
                    result.push_str(ansi_conj);
                } else {
                    // Emit character by character
                    let mut ci = cluster_start;
                    while ci < cluster_end {
                        let ch = chars[ci];
                        if let Some(mapped) = self.tables.unicode_to_ansi_single.get(&ch) {
                            result.push_str(mapped);
                        } else {
                            result.push(ch);
                            unrepresented.push(ch);
                        }
                        ci += 1;
                    }
                }
            } else {
                // Non-consonant single character
                let ch = chars[cluster_start];
                if let Some(mapped) = self.tables.unicode_to_ansi_single.get(&ch) {
                    result.push_str(mapped);
                } else {
                    result.push(ch);
                    if ch >= '\u{0980}' && ch <= '\u{09FF}' {
                        unrepresented.push(ch);
                    }
                }
            }

            if has_yaphala {
                result.push('¨');
            }
            if has_raphala {
                result.push('ª');
            }

            // 6. Emit post-base vowel signs
            if let Some(sk) = split_kar {
                match sk {
                    '\u{09CB}' => result.push('v'), // ো -> ‡ + base + v
                    '\u{09CC}' => result.push('Š'), // ৌ -> ‡ + base + Š
                    _ => {}
                }
            } else if let Some(pok) = postbase_kar {
                match pok {
                    '\u{09BE}' => result.push('v'), // া -> v
                    '\u{09C0}' => result.push('x'), // ী -> x
                    '\u{09C1}' => result.push('y'), // ু -> y
                    '\u{09C2}' => result.push('~'), // ূ -> ~
                    '\u{09C3}' => result.push('„'), // ৃ -> „
                    _ => {}
                }
            }

            // 7. Emit Reph glyph after base and vowels
            if has_reph {
                result.push('©');
            }

            i = next_pos;
        }

        (result, unrepresented)
    }

    /// Converts legacy SutonnyMJ ANSI text to canonical Unicode Bangla text.
    pub fn ansi_to_unicode(&self, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut result = String::new();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            // 1. Detect pre-base vowel signs in visual order: 'w' (ি), '‡' (ে), '‰' (ৈ)
            if c == 'w' || c == '‡' || c == '†' || c == '‰' {
                let is_ikar = c == 'w';
                let is_ekar_family = c == '‡' || c == '†';
                let is_oikar = c == '‰';

                // Look ahead for base cluster
                let base_start = i + 1;
                if base_start < chars.len() {
                    let mut base_end = base_start + 1;
                    // Check if base is a multi-char conjunct or has hasanta
                    while base_end < chars.len()
                        && chars[base_end] == '&'
                        && base_end + 1 < chars.len()
                    {
                        base_end += 2;
                    }

                    let base_sub: String = chars[base_start..base_end].iter().collect();
                    let decoded_base = self.decode_ansi_cluster(&base_sub);
                    result.push_str(&decoded_base);

                    // Check if followed by split sign post-base 'v' (aa-kar) or 'Š' (au mark)
                    if is_ekar_family && base_end < chars.len() && chars[base_end] == 'v' {
                        result.push('\u{09CB}'); // ো
                        i = base_end + 1;
                        continue;
                    } else if is_ekar_family && base_end < chars.len() && chars[base_end] == 'Š' {
                        result.push('\u{09CC}'); // ৌ
                        i = base_end + 1;
                        continue;
                    } else if is_ikar {
                        result.push('\u{09BF}'); // ি
                    } else if is_ekar_family {
                        result.push('\u{09C7}'); // ে
                    } else if is_oikar {
                        result.push('\u{09C8}'); // ৈ
                    }

                    i = base_end;
                    continue;
                }
            }

            // Check for 'Av' (A=অ + v=া -> আ)
            if c == 'A' && i + 1 < chars.len() && chars[i + 1] == 'v' {
                result.push('\u{0986}'); // আ
                i += 2;
                continue;
            }

            // 2. Check for Reph glyph ('©')
            if c == '©' {
                // If preceded by a base, Reph in Unicode belongs BEFORE the base
                // Handled gracefully in reconstruction
                result.push('\u{09B0}');
                result.push('\u{09CD}');
                i += 1;
                continue;
            }

            // 3. Check for conjunct glyphs
            let c_str = c.to_string();
            if let Some(conj_uni) = self.tables.ansi_to_conjunct.get(&c_str) {
                result.push_str(conj_uni);
                i += 1;
                continue;
            }

            // 4. Check for single character mappings
            if let Some(single_uni) = self.tables.ansi_to_unicode_single.get(&c_str) {
                result.push_str(single_uni);
                i += 1;
                continue;
            }

            // Default pass-through
            result.push(c);
            i += 1;
        }

        let nfc_text: String = result.nfc().collect();
        nfc_text
            .replace("\u{09A1}\u{09BC}", "\u{09DC}")
            .replace("\u{09A2}\u{09BC}", "\u{09DD}")
            .replace("\u{09AF}\u{09BC}", "\u{09DF}")
    }

    fn decode_ansi_cluster(&self, ansi: &str) -> String {
        if let Some(u) = self.tables.ansi_to_conjunct.get(ansi) {
            return u.clone();
        }
        let mut out = String::new();
        let chars: Vec<char> = ansi.chars().collect();
        let mut idx = 0;
        while idx < chars.len() {
            let ch = chars[idx];
            let ch_str = ch.to_string();
            if let Some(u) = self.tables.ansi_to_conjunct.get(&ch_str) {
                out.push_str(u);
            } else if let Some(u) = self.tables.ansi_to_unicode_single.get(&ch_str) {
                out.push_str(u);
            } else {
                out.push(ch);
            }
            idx += 1;
        }
        out
    }
}
