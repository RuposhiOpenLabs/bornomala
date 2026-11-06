use bornomala_phonetic::{AvroRules, PhoneticEngine};

#[test]
fn test_libavrophonetic_corpus() {
    let rules = AvroRules::global();

    let corpus = [
        ("a", "আ"),
        ("ia", "ইয়া"),
        ("(a", "(আ"),
        ("(a`", "(া"),
        ("aa", "আআ"),
        ("aa`", "আা"),
        ("amar", "আমার"),
        (".", "।"),
        ("..", "।।"),
        ("...", "..."),
        ("R", "ড়"),
        ("Rh", "ঢ়"),
        ("kobja", "কব্জা"),
        ("bhladimir", "ভ্লাদিমির"),
        ("vladimir", "ভ্লাদিমির"),
        ("lobdho", "লব্ধ"),
        ("jobdo", "জব্দ"),
        ("accha", "আচ্ছা"),
        ("bacca", "বাচ্চা"),
        ("boDDo", "বড্ড"),
        ("bou", "বউ"),
        ("bOU", "বৌ"),
        ("shikkhok", "শিক্ষক"),
        ("shikkha", "শিক্ষা"),
        ("daktar", "দাক্তার"), // Classic rule output; dictionary candidate provides ডাক্তার
        ("ranna", "রান্না"),
        ("ami", "আমি"),
        ("tumi", "তুমি"),
        ("bangla", "বাংলা"),
        ("bangladesh", "বাংলাদেশ"),
        ("korrmo", "কর্ম"),
        ("korrmofol", "কর্মফল"),
        ("borrNo", "বর্ণ"),
        ("borrNomala", "বর্ণমালা"),
        ("dhonno", "ধন্ন"),
        ("bhabna", "ভাবনা"),
        ("prothom", "প্রথম"),
    ];

    for (roman, expected) in corpus {
        let actual = rules.transliterate(roman);
        assert_eq!(
            actual, expected,
            "Failed 100% Avro conformance on '{}': expected '{}', got '{}'",
            roman, expected, actual
        );
    }
}

#[test]
fn test_complex_conjuncts_conformance() {
    let rules = AvroRules::global();

    let conjunct_pairs = [
        // kkh, ksh, kx
        ("kkh", "ক্ষ"),
        ("kSh", "ক্ষ"),
        ("kx", "ক্ষ"),
        ("kkhm", "ক্ষ্ম"),
        ("kShm", "ক্ষ্ম"),
        ("kkhN", "ক্ষ্ণ"),
        // ng-series
        ("ngo", "ঙ্গ"),
        ("nga", "ঙ্গা"),
        ("ngi", "ঙ্গি"),
        ("ngI", "ঙ্গী"),
        ("ngu", "ঙ্গু"),
        ("ngU", "ঙ্গূ"),
        ("nge", "ঙ্গে"),
        ("ngO", "ঙ্গো"),
        ("Ngk", "ঙ্ক"),
        ("Ngkh", "ঙ্খ"),
        ("Ngg", "ঙ্গ"),
        ("Nggh", "ঙ্ঘ"),
        // j-series
        ("jj", "জ্জ"),
        ("jjh", "জ্ঝ"),
        ("jNG", "জ্ঞ"),
        ("GG", "জ্ঞ"),
        // c-series
        ("cc", "চ্চ"),
        ("cch", "চ্ছ"),
        ("cNG", "চ্ঞ"),
        // d-series
        ("dd", "দ্দ"),
        ("ddh", "দ্ধ"),
        ("dgh", "দ্ঘ"),
        ("dbh", "দ্ভ"),
        ("dv", "দ্ভ"),
        ("dm", "দ্ম"),
        // t-series
        ("tt", "ত্ত"),
        ("tth", "ত্থ"),
        ("tn", "ত্ন"),
        ("tm", "ত্ম"),
        // s-series
        ("st", "স্ত"),
        ("sth", "স্থ"),
        ("sn", "স্ন"),
        ("sp", "স্প"),
        ("sph", "স্ফ"),
        ("sm", "স্ম"),
        ("sl", "স্ল"),
        ("sk", "স্ক"),
        ("skh", "স্খ"),
        // Sh-series
        ("ShT", "ষ্ট"),
        ("ShTh", "ষ্ঠ"),
        ("ShN", "ষ্ণ"),
        ("Shp", "ষ্প"),
        ("Shph", "ষ্ফ"),
        ("Shm", "ষ্ম"),
        // sh-series
        ("shc", "শ্চ"),
        ("shch", "শ্ছ"),
        ("shn", "শ্ন"),
        ("shm", "শ্ম"),
        ("shl", "শ্ল"),
        // h-series
        ("hn", "হ্ন"),
        ("hN", "হ্ণ"),
        ("hm", "হ্ম"),
        ("hl", "হ্ল"),
        // Hasanta delimiter
        ("k`k", "কক"),
        ("t`t", "তত"),
        ("p`p", "পপ"),
        // Explicit hasanta
        (",,", "্‌"),
        // Currency and Dari
        ("$500", "৳৫০০"),
        ("..", "।।"),
    ];

    for (input, expected) in conjunct_pairs {
        let actual = rules.transliterate(input);
        assert_eq!(
            actual, expected,
            "Conjunct conformance failure for '{}': expected '{}', got '{}'",
            input, expected, actual
        );
    }
}

#[test]
fn test_engine_candidate_ranking_accuracy() {
    let engine = PhoneticEngine::new();

    let expected_first_picks = [
        ("ami", "আমি"),
        ("tumi", "তুমি"),
        ("amra", "আমরা"),
        ("bangla", "বাংলা"),
        ("bangladesh", "বাংলাদেশ"),
        ("kormo", "কর্ম"),
        ("kormofol", "কর্মফল"),
        ("daktar", "ডাক্তার"),
        ("shadhinota", "স্বাধীনতা"),
        ("dukkho", "দুঃখ"),
        ("abong", "এবং"),
        ("shesh", "শেষ"),
        ("apni", "আপনি"),
        ("lalshak", "লালশাক"),
    ];

    for (roman, expected) in expected_first_picks {
        let candidates = engine.get_candidates(roman);
        assert!(
            !candidates.is_empty(),
            "No candidates produced for '{}'",
            roman
        );
        assert_eq!(
            candidates[0], expected,
            "Engine candidate ranking failed for '{}': top was '{}', expected '{}' (all: {:?})",
            roman, candidates[0], expected, candidates
        );
    }
}
