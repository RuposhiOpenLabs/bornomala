use bornomala_engine::{
    amount_to_bangla_words, amount_to_english_words, convert_digits, get_preloaded_fonts,
    gregorian_to_bangla, AutostartManager, BanglaConverter, BornomalaEngine, FontInstallScope,
    InputScheme, KeyEvent, StartupOptions, TextEdit, REGISTRY_RUN_KEY, REGISTRY_VALUE_NAME,
};

#[test]
fn test_unicode_typing_vectors() {
    let mut engine = BornomalaEngine::new(InputScheme::ClassicUnicode);

    // 1. Single consonant ক (key 'j', KeyCode 74)
    let ev = KeyEvent::new_down(74, Some('j'), false, false, false);
    let act = engine.process_key(&ev);
    assert_eq!(act.edits, vec![TextEdit::Insert("ক".to_string())]);

    // Reset
    engine.reset_all_states();

    // 2. Pre-base i-kar 'd' (KeyCode 68) followed by 'j' (KeyCode 74) -> কি
    let ev1 = KeyEvent::new_down(68, Some('d'), false, false, false);
    let act1 = engine.process_key(&ev1);
    assert_eq!(act1.composition, Some("ি".to_string()));

    let ev2 = KeyEvent::new_down(74, Some('j'), false, false, false);
    let act2 = engine.process_key(&ev2);
    assert_eq!(act2.edits, vec![TextEdit::Insert("কি".to_string())]);

    // Reset
    engine.reset_all_states();

    // 3. Pre-base e-kar 'c' (KeyCode 67) followed by 'j' (KeyCode 74) -> কে
    let ev1 = KeyEvent::new_down(67, Some('c'), false, false, false);
    let _ = engine.process_key(&ev1);
    let ev2 = KeyEvent::new_down(74, Some('j'), false, false, false);
    let act2 = engine.process_key(&ev2);
    assert_eq!(act2.edits, vec![TextEdit::Insert("কে".to_string())]);

    // 4. Split o-kar: 'c' + 'j' -> 'কে', then 'f' (a-kar) -> 'কো'
    let ev3 = KeyEvent::new_down(70, Some('f'), false, false, false);
    let act3 = engine.process_key(&ev3);
    assert_eq!(
        act3.edits,
        vec![TextEdit::ReplaceLast {
            count: 1,
            text: "কো".chars().last().unwrap().to_string()
        }]
    );
}

#[test]
fn test_ansi_typing_vectors() {
    let mut engine = BornomalaEngine::new(InputScheme::ClassicAnsi);

    // 1. Single key 'j' -> 'K'
    let ev = KeyEvent::new_down(74, Some('j'), false, false, false);
    let act = engine.process_key(&ev);
    assert_eq!(act.edits, vec![TextEdit::Insert("K".to_string())]);

    // Reset
    engine.reset_all_states();

    // 2. Conjunct sequence: 'j' (K) + 'g' (&) + 'j' (K) -> '°' (SutonnyMJ kka glyph)
    let ev1 = KeyEvent::new_down(74, Some('j'), false, false, false);
    let _ = engine.process_key(&ev1);
    let ev2 = KeyEvent::new_down(71, Some('g'), false, false, false);
    let _ = engine.process_key(&ev2);
    let ev3 = KeyEvent::new_down(74, Some('j'), false, false, false);
    let act3 = engine.process_key(&ev3);
    assert_eq!(
        act3.edits,
        vec![TextEdit::ReplaceLast {
            count: 2,
            text: "°".to_string()
        }]
    );
}

#[test]
fn test_phonetic_oracle_vectors() {
    let mut engine = BornomalaEngine::new(InputScheme::Phonetic);

    let test_cases = &[
        ("ami", "আমি"),
        ("bangla", "বাংলা"),
        ("amader", "আমাদের"),
        ("sOnar", "সোনার"),
        ("Dhaka", "ঢাকা"),
        ("bidyaloy", "বিদ্যালয়"),
        ("korrmo", "কর্ম"),
    ];

    for (roman, expected) in test_cases {
        engine.reset_all_states();
        for ch in roman.chars() {
            let ev = KeyEvent::new_down(ch as u32, Some(ch), ch.is_uppercase(), false, false);
            engine.process_key(&ev);
        }
        // Commit with Space
        let space_ev = KeyEvent::new_down(32, Some(' '), false, false, false);
        let act = engine.process_key(&space_ev);
        assert_eq!(
            act.edits,
            vec![
                TextEdit::Insert(expected.to_string()),
                TextEdit::Insert(" ".to_string())
            ],
            "Mismatch on word {}",
            roman
        );
    }
}

#[test]
fn test_converter_bidirectional() {
    let converter = BanglaConverter::new();

    let cases = &[
        ("বাংলা", "evsjv"),
        ("কি", "wK"),
        ("কে", "‡K"),
        ("কো", "‡Kv"),
        ("ক্ক", "³"),
    ];

    for (u, a) in cases {
        let (ansi_out, _) = converter.unicode_to_ansi(u);
        assert_eq!(&ansi_out, a, "Failed unicode_to_ansi for {}", u);

        let uni_out = converter.ansi_to_unicode(a);
        assert_eq!(&uni_out, u, "Failed ansi_to_unicode for {}", a);
    }
}

#[test]
fn test_amount_in_words_bangla() {
    assert_eq!(
        amount_to_bangla_words(1234.50),
        "এক হাজার দুই শত চৌত্রিশ টাকা পঞ্চাশ পয়সা মাত্র"
    );
    assert_eq!(
        amount_to_bangla_words(500.00),
        "পাঁচ শত টাকা মাত্র"
    );
    assert_eq!(
        amount_to_bangla_words(100000.00),
        "এক লাখ টাকা মাত্র"
    );
    assert_eq!(
        amount_to_bangla_words(25000000.00),
        "দুই কোটি পঞ্চাশ লাখ টাকা মাত্র"
    );
}

#[test]
fn test_amount_in_words_english() {
    assert_eq!(
        amount_to_english_words(1234.50),
        "One Thousand Two Hundred Thirty-Four Taka and Fifty Paisa Only"
    );
    assert_eq!(
        amount_to_english_words(500.00),
        "Five Hundred Taka Only"
    );
}

#[test]
fn test_bangla_date_conversion() {
    // April 14, 2024 is 1 Baishakh 1431
    let b_date = gregorian_to_bangla(2024, 4, 14);
    assert_eq!(b_date.year, 1431);
    assert_eq!(b_date.month, 1);
    assert_eq!(b_date.day, 1);
    assert_eq!(b_date.format_bangla(), "১ বৈশাখ ১৪৩১");

    // December 16, 2024 is 1 Poush 1431
    let b_date2 = gregorian_to_bangla(2024, 12, 16);
    assert_eq!(b_date2.year, 1431);
    assert_eq!(b_date2.month, 9);
    assert_eq!(b_date2.day, 1);
    assert_eq!(b_date2.format_bangla(), "১ পৌষ ১৪৩১");
}

#[test]
fn test_digit_conversion() {
    assert_eq!(convert_digits("0123456789", true), "০১২৩৪৫৬৭৮৯");
    assert_eq!(convert_digits("০১২৩৪৫৬৭৮৯", false), "0123456789");
}

#[test]
fn test_autostart_manager_and_commands() {
    let mgr = AutostartManager::default();
    assert_eq!(mgr.app_name, REGISTRY_VALUE_NAME);
    let enable_cmd = mgr.generate_enable_command();
    assert!(enable_cmd.contains("reg add"));
    assert!(enable_cmd.contains(REGISTRY_RUN_KEY));
    assert!(enable_cmd.contains("--startup"));

    let disable_cmd = mgr.generate_disable_command();
    assert!(disable_cmd.contains("reg delete"));
    assert!(disable_cmd.contains(REGISTRY_VALUE_NAME));

    let query_cmd = mgr.generate_query_command();
    assert!(query_cmd.contains("reg query"));

    let args = vec!["--startup".to_string(), "--splash".to_string()];
    let opts = StartupOptions::from_args(&args);
    assert!(opts.is_system_startup);
    assert!(opts.should_show_splash_screen());

    let silent_args = vec!["--startup".to_string(), "--no-splash".to_string()];
    let silent_opts = StartupOptions::from_args(&silent_args);
    assert!(silent_opts.is_system_startup);
    assert!(!silent_opts.should_show_splash_screen());
}

#[test]
fn test_preloaded_fonts_catalog() {
    let fonts = get_preloaded_fonts();
    assert_eq!(fonts.len(), 8);

    // Verify SutonnyMJ is marked as ANSI
    let sutonny = fonts.iter().find(|f| f.font_name == "SutonnyMJ").expect("SutonnyMJ present");
    assert!(sutonny.is_ansi);
    assert!(!sutonny.is_unicode);

    // Verify Nikosh is marked as Unicode
    let nikosh = fonts.iter().find(|f| f.font_name == "Nikosh").expect("Nikosh present");
    assert!(nikosh.is_unicode);
    assert!(!nikosh.is_ansi);

    // Verify PowerShell script generation includes all fonts
    let ps_user = bornomala_engine::conveniences::fonts::generate_powershell_font_installer("C:\\Fonts", FontInstallScope::CurrentUser);
    assert!(ps_user.contains("Nikosh.ttf"));
    assert!(ps_user.contains("SutonnyMJ Regular.ttf"));
    assert!(ps_user.contains("HKCU:\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts"));
}

#[test]
fn test_mode_toggle_hotkeys_return_to_default_pass_through() {
    let mut engine = BornomalaEngine::new(InputScheme::PassThrough);
    assert_eq!(engine.active_scheme(), InputScheme::PassThrough);

    // 1. Press Ctrl+Alt+V from default English -> switches to Classic Unicode
    let key_ctrl_alt_v = KeyEvent::new_down(86, Some('v'), false, true, true);
    let action1 = engine.process_key(&key_ctrl_alt_v);
    assert!(action1.consumed);
    assert_eq!(engine.active_scheme(), InputScheme::ClassicUnicode);

    // 2. Press Ctrl+Alt+V again -> switches back to default English (PassThrough)
    let action2 = engine.process_key(&key_ctrl_alt_v);
    assert!(action2.consumed);
    assert_eq!(engine.active_scheme(), InputScheme::PassThrough);

    // 3. Press Ctrl+Alt+B from default English -> switches to Classic ANSI (Bijoy)
    let key_ctrl_alt_b = KeyEvent::new_down(66, Some('b'), false, true, true);
    let action3 = engine.process_key(&key_ctrl_alt_b);
    assert!(action3.consumed);
    assert_eq!(engine.active_scheme(), InputScheme::ClassicAnsi);

    // 4. Press Ctrl+Alt+B again -> switches back to default English (PassThrough)
    let action4 = engine.process_key(&key_ctrl_alt_b);
    assert!(action4.consumed);
    assert_eq!(engine.active_scheme(), InputScheme::PassThrough);

    // 5. Press F12 from default English -> switches to Phonetic
    let key_f12 = KeyEvent::new_down(123, None, false, false, false);
    let action5 = engine.process_key(&key_f12);
    assert!(action5.consumed);
    assert_eq!(engine.active_scheme(), InputScheme::Phonetic);

    // 6. Press F12 again -> switches back to default English (PassThrough)
    let action6 = engine.process_key(&key_f12);
    assert!(action6.consumed);
    assert_eq!(engine.active_scheme(), InputScheme::PassThrough);
}
