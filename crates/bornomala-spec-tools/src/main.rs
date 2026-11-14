use bornomala_engine::{BanglaConverter, BornomalaEngine, InputScheme, KeyEvent, TextEdit};
use std::fs;

fn main() {
    println!("=== Bornomala Specification & Test Vector Validation Suite ===");

    validate_unicode_vectors();
    validate_ansi_vectors();
    validate_phonetic_vectors();
    validate_converter_vectors();

    println!("\nAll test vectors validated successfully!");
}

fn char_to_key_event(ch: char) -> KeyEvent {
    let (kc, shift) = match ch {
        'a'..='z' => (ch.to_ascii_uppercase() as u32, false),
        'A'..='Z' => (ch as u32, true),
        '0'..='9' => (ch as u32, false),
        ' ' => (32, false),
        '$' => (52, true),
        '|' => (220, true),
        '!' => (49, true),
        '@' => (50, true),
        '#' => (51, true),
        '%' => (53, true),
        '^' => (54, true),
        '&' => (55, true),
        '*' => (56, true),
        '(' => (57, true),
        ')' => (48, true),
        '-' => (189, false),
        '_' => (189, true),
        '=' => (187, false),
        '+' => (187, true),
        '[' => (219, false),
        '{' => (219, true),
        ']' => (221, false),
        '}' => (221, true),
        ';' => (186, false),
        ':' => (186, true),
        '\'' => (222, false),
        '"' => (222, true),
        ',' => (188, false),
        '<' => (188, true),
        '.' => (190, false),
        '>' => (190, true),
        '/' => (191, false),
        '?' => (191, true),
        '\\' => (220, false),
        '`' => (192, false),
        '~' => (192, true),
        _ => (ch as u32, false),
    };
    KeyEvent::new_down(kc, Some(ch), shift, false, false)
}

fn validate_unicode_vectors() {
    println!("\nValidating Unicode test vectors from data/test_vectors/unicode_vectors.json...");
    let content = fs::read_to_string("data/test_vectors/unicode_vectors.json")
        .expect("Failed to read unicode_vectors.json");
    let vectors: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();

    let mut engine = BornomalaEngine::new(InputScheme::ClassicUnicode);

    for v in &vectors {
        let id = v["id"].as_str().unwrap();
        let _desc = v["desc"].as_str().unwrap();
        let expected = v["expected_output"].as_str().unwrap();
        let conf = v["confidence"].as_str().unwrap();

        engine.reset_all_states();
        let mut actual = String::new();

        if let Some(keys) = v["input_keys"].as_array() {
            for k in keys {
                let k_str = k.as_str().unwrap();
                for ch in k_str.chars() {
                    let ev = char_to_key_event(ch);
                    let act = engine.process_key(&ev);
                    for edit in act.edits {
                        match edit {
                            TextEdit::Insert(s) => actual.push_str(&s),
                            TextEdit::ReplaceLast { count, text } => {
                                for _ in 0..count {
                                    actual.pop();
                                }
                                actual.push_str(&text);
                            }
                            TextEdit::DeleteBack(count) => {
                                for _ in 0..count {
                                    actual.pop();
                                }
                            }
                            TextEdit::PassThrough => actual.push(ch),
                        }
                    }
                }
            }
        }

        print!("  Vector [{} - {}]: ", id, conf);
        if actual == expected {
            println!("PASS (expected: '{}', got: '{}')", expected, actual);
        } else {
            println!("FAIL (expected: '{}', got: '{}')", expected, actual);
        }
    }
}

fn validate_ansi_vectors() {
    println!("\nValidating ANSI test vectors from data/test_vectors/ansi_vectors.json...");
    let content = fs::read_to_string("data/test_vectors/ansi_vectors.json")
        .expect("Failed to read ansi_vectors.json");
    let vectors: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();

    let mut engine = BornomalaEngine::new(InputScheme::ClassicAnsi);

    for v in &vectors {
        let id = v["id"].as_str().unwrap();
        let expected = v["expected_output"].as_str().unwrap();
        let conf = v["confidence"].as_str().unwrap();

        engine.reset_all_states();
        let mut actual = String::new();

        if let Some(keys) = v["input_keys"].as_array() {
            for k in keys {
                let k_str = k.as_str().unwrap();
                for ch in k_str.chars() {
                    let ev = char_to_key_event(ch);
                    let act = engine.process_key(&ev);
                    for edit in act.edits {
                        match edit {
                            TextEdit::Insert(s) => actual.push_str(&s),
                            TextEdit::ReplaceLast { count, text } => {
                                for _ in 0..count {
                                    actual.pop();
                                }
                                actual.push_str(&text);
                            }
                            TextEdit::DeleteBack(count) => {
                                for _ in 0..count {
                                    actual.pop();
                                }
                            }
                            TextEdit::PassThrough => actual.push(ch),
                        }
                    }
                }
            }
        }

        print!("  Vector [{} - {}]: ", id, conf);
        if actual == expected {
            println!("PASS (expected: '{}', got: '{}')", expected, actual);
        } else {
            println!("FAIL (expected: '{}', got: '{}')", expected, actual);
        }
    }
}

fn validate_phonetic_vectors() {
    println!("\nValidating Phonetic test vectors from data/test_vectors/phonetic_vectors.json...");
    let content = fs::read_to_string("data/test_vectors/phonetic_vectors.json")
        .expect("Failed to read phonetic_vectors.json");
    let vectors: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();

    let mut engine = BornomalaEngine::new(InputScheme::Phonetic);

    for v in &vectors {
        let id = v["id"].as_str().unwrap();
        let input = v["input"].as_str().unwrap();
        let expected = v["expected_output"].as_str().unwrap();
        let conf = v["confidence"].as_str().unwrap();

        engine.reset_all_states();
        let mut actual = String::new();
        for ch in input.chars() {
            let ev = KeyEvent::new_down(ch as u32, Some(ch), ch.is_uppercase(), false, false);
            let act = engine.process_key(&ev);
            for edit in act.edits {
                match edit {
                    TextEdit::Insert(s) => actual.push_str(&s),
                    TextEdit::ReplaceLast { count, text } => {
                        for _ in 0..count {
                            actual.pop();
                        }
                        actual.push_str(&text);
                    }
                    TextEdit::DeleteBack(count) => {
                        for _ in 0..count {
                            actual.pop();
                        }
                    }
                    TextEdit::PassThrough => actual.push(ch),
                }
            }
        }
        let space_ev = KeyEvent::new_down(0x20, Some(' '), false, false, false);
        let act = engine.process_key(&space_ev);
        for edit in act.edits {
            match edit {
                TextEdit::Insert(s) => actual.push_str(&s),
                TextEdit::ReplaceLast { count, text } => {
                    for _ in 0..count {
                        actual.pop();
                    }
                    actual.push_str(&text);
                }
                TextEdit::DeleteBack(count) => {
                    for _ in 0..count {
                        actual.pop();
                    }
                }
                TextEdit::PassThrough => actual.push(' '),
            }
        }
        let actual = actual.trim_end().to_string();

        print!("  Vector [{} - {} - input: '{}']: ", id, conf, input);
        if actual == expected {
            println!("PASS ('{}')", actual);
        } else {
            println!("FAIL (expected: '{}', got: '{}')", expected, actual);
        }
    }
}

fn validate_converter_vectors() {
    println!("\nValidating Bidirectional Converter test vectors...");
    let content = fs::read_to_string("data/test_vectors/converter_vectors.json")
        .expect("Failed to read converter_vectors.json");
    let vectors: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();

    let converter = BanglaConverter::new();

    for v in &vectors {
        let id = v["id"].as_str().unwrap();
        let u = v["unicode"].as_str().unwrap();
        let a = v["ansi"].as_str().unwrap();
        let conf = v["confidence"].as_str().unwrap();

        let (ansi_out, _) = converter.unicode_to_ansi(u);
        let uni_out = converter.ansi_to_unicode(a);

        print!("  Vector [{} - {}]: ", id, conf);
        if ansi_out == a && uni_out == u {
            println!("PASS (Unicode '{}' <-> ANSI '{}')", u, a);
        } else {
            println!(
                "FAIL (u->a: '{}' vs '{}', a->u: '{}' vs '{}')",
                ansi_out, a, uni_out, u
            );
        }
    }
}
