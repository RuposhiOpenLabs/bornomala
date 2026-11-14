use bornomala_engine::{BornomalaEngine, InputScheme, KeyEvent, TextEdit};

#[test]
fn test_e2e_full_paragraph_typing() {
    let mut engine = BornomalaEngine::new(InputScheme::Phonetic);

    // Paragraph to type in Roman Banglish:
    // "amader desher nam bangladesh. ami banglay gan gai. amar sonar bangla, ami tomay bhalobashi."
    let words_and_expected = [
        ("amader", "আমাদের"),
        ("desher", "দেশের"),
        ("nam", "নাম"),
        ("bangladesh", "বাংলাদেশ"),
        ("ami", "আমি"),
        ("banglay", "বাংলায়"),
        ("gan", "গান"),
        ("gai", "গাই"),
        ("amar", "আমার"),
        ("sonar", "সোনার"),
        ("bangla", "বাংলা"),
        ("ami", "আমি"),
        ("tOmay", "তোমায়"),
        ("bhalObashi", "ভালোবাসি"),
    ];

    let mut full_output = String::new();

    for (roman, _expected) in words_and_expected {
        for ch in roman.chars() {
            let ev = KeyEvent::new_down(ch as u32, Some(ch), ch.is_uppercase(), false, false);
            let action = engine.process_key(&ev);
            assert!(action.consumed);
            assert!(action.composition.is_some());
        }

        // Commit word with Space
        let space_ev = KeyEvent::new_down(32, Some(' '), false, false, false);
        let commit_act = engine.process_key(&space_ev);
        assert!(commit_act.consumed);

        for edit in commit_act.edits {
            if let TextEdit::ReplaceLast { text, .. } = edit {
                full_output.push_str(&text);
            }
        }
    }

    assert_eq!(
        full_output,
        "আমাদের দেশের নাম বাংলাদেশ আমি বাংলায় গান গাই আমার সোনার বাংলা আমি তোমায় ভালোবাসি "
    );
}

#[test]
fn test_e2e_mixed_bangla_english_and_punctuation() {
    let mut engine = BornomalaEngine::new(InputScheme::Phonetic);

    // 1. Currency $500 -> ৳৫০০
    for ch in "$500".chars() {
        let ev = KeyEvent::new_down(ch as u32, Some(ch), false, false, false);
        engine.process_key(&ev);
    }
    let space_ev = KeyEvent::new_down(32, Some(' '), false, false, false);
    let act = engine.process_key(&space_ev);
    assert_eq!(
        act.edits,
        vec![TextEdit::ReplaceLast {
            count: 4,
            text: "৳৫০০ ".to_string()
        }]
    );

    // 2. Dāṛi '.' -> '।' (displays live preview '।')
    let dari_ev = KeyEvent::new_down(0xBE, Some('.'), false, false, false);
    let dari_act = engine.process_key(&dari_ev);
    assert_eq!(
        dari_act.edits,
        vec![TextEdit::ReplaceLast {
            count: 0,
            text: "।".to_string()
        }]
    );

    // 3. Double Dāṛi '..' -> '।।'
    engine.reset_all_states();
    for ch in "..".chars() {
        let ev = KeyEvent::new_down(0xBE, Some(ch), false, false, false);
        engine.process_key(&ev);
    }
    let space_ev2 = KeyEvent::new_down(32, Some(' '), false, false, false);
    let act2 = engine.process_key(&space_ev2);
    assert_eq!(
        act2.edits,
        vec![TextEdit::ReplaceLast {
            count: 2,
            text: "।। ".to_string()
        }]
    );
}

#[test]
fn test_e2e_undo_backspace_middle_of_composition() {
    let mut engine = BornomalaEngine::new(InputScheme::Phonetic);

    // Type "ban"
    for ch in "ban".chars() {
        let ev = KeyEvent::new_down(ch as u32, Some(ch), false, false, false);
        engine.process_key(&ev);
    }
    assert_eq!(engine.phonetic_engine().roman_buffer(), "ban");

    // Backspace twice -> should leave "b"
    let bs_ev = KeyEvent::new_down(8, None, false, false, false);
    engine.process_key(&bs_ev);
    engine.process_key(&bs_ev);
    assert_eq!(engine.phonetic_engine().roman_buffer(), "b");

    // Now type "harot" -> "bharot" -> "ভারত"
    for ch in "harot".chars() {
        let ev = KeyEvent::new_down(ch as u32, Some(ch), false, false, false);
        engine.process_key(&ev);
    }
    assert_eq!(engine.phonetic_engine().roman_buffer(), "bharot");

    let space_ev = KeyEvent::new_down(32, Some(' '), false, false, false);
    let act = engine.process_key(&space_ev);
    assert_eq!(
        act.edits,
        vec![TextEdit::ReplaceLast {
            count: 4,
            text: "ভারত ".to_string()
        }]
    );
}

#[test]
fn test_e2e_escape_cancellation() {
    let mut engine = BornomalaEngine::new(InputScheme::Phonetic);

    for ch in "kormo".chars() {
        let ev = KeyEvent::new_down(ch as u32, Some(ch), false, false, false);
        engine.process_key(&ev);
    }
    assert_eq!(engine.phonetic_engine().roman_buffer(), "kormo");

    // Press Escape
    let esc_ev = KeyEvent::new_down(27, None, false, false, false);
    let act = engine.process_key(&esc_ev);
    assert!(act.consumed);
    assert_eq!(engine.phonetic_engine().roman_buffer(), "");
}
