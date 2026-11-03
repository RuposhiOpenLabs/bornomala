use proptest::prelude::*;
use bornomala_engine::{convert_digits, BornomalaEngine, InputScheme, KeyEvent};

proptest! {
    #[test]
    fn test_digit_conversion_bijective(s in "[0-9]+") {
        let bn = convert_digits(&s, true);
        let back = convert_digits(&bn, false);
        prop_assert_eq!(s, back);
    }

    #[test]
    fn test_engine_never_panics_on_random_keys(
        key_codes in proptest::collection::vec(0u32..255u32, 1..50),
        shifts in proptest::collection::vec(proptest::bool::ANY, 1..50)
    ) {
        let mut engine = BornomalaEngine::new(InputScheme::ClassicUnicode);
        for (kc, shift) in key_codes.iter().zip(shifts.iter()) {
            let ev = KeyEvent::new_down(*kc, char::from_u32(*kc), *shift, false, false);
            let _ = engine.process_key(&ev);
        }
    }

    #[test]
    fn test_phonetic_engine_never_panics(
        s in "[a-zA-Z0-9`^ ,.\\-!]+"
    ) {
        let mut engine = BornomalaEngine::new(InputScheme::Phonetic);
        for ch in s.chars() {
            let ev = KeyEvent::new_down(ch as u32, Some(ch), ch.is_uppercase(), false, false);
            let _ = engine.process_key(&ev);
        }
    }
}
