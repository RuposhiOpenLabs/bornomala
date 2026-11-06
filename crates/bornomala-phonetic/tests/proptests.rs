use bornomala_phonetic::{AvroRules, PhoneticEngine};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn test_rules_parser_never_crashes(input in "\\PC*") {
        let rules = AvroRules::global();
        let _ = rules.transliterate(&input);
    }

    #[test]
    fn test_engine_never_crashes(input in "[a-zA-Z0-9`$., ]{0,30}") {
        let engine = PhoneticEngine::new();
        let _ = engine.get_candidates(&input);
    }

    #[test]
    fn test_levenshtein_properties(a in "[a-z]{0,10}", b in "[a-z]{0,10}") {
        let d = bornomala_phonetic::levenshtein_distance(&a, &b);
        if a == b {
            prop_assert_eq!(d, 0);
        } else {
            prop_assert!(d > 0);
        }
        prop_assert_eq!(d, bornomala_phonetic::levenshtein_distance(&b, &a));
    }
}

#[test]
fn test_per_keystroke_latency_benchmark() {
    let engine = PhoneticEngine::new();
    let word = "shadhinota";

    let start = std::time::Instant::now();
    let iterations = 10_000;
    for _ in 0..iterations {
        let _ = engine.get_candidates(word);
    }
    let elapsed = start.elapsed();
    let per_keystroke = elapsed / iterations;

    println!(
        "Benchmark: {} iterations took {:?}, average {:?} per lookup",
        iterations, elapsed, per_keystroke
    );

    // Keystroke latency must be well under 1 millisecond (1000 microseconds)
    assert!(
        per_keystroke < std::time::Duration::from_millis(1),
        "Per-keystroke latency exceeded 1ms: {:?}",
        per_keystroke
    );
}
