//! # Bornomala Phonetic Engine
//!
//! High-performance, 100% Avro-compatible phonetic transliteration library
//! with finite state transducer (FST) dictionary, autocorrect, morphological suffix expansion,
//! local encrypted candidate learning, and contextual beyond-Avro features.

pub mod autocorrect;
pub mod beyond_avro;
pub mod dictionary;
pub mod engine;
pub mod fst_tree;
pub mod memory;
pub mod rules;
pub mod suffix;

pub use autocorrect::{AutocorrectMatch, AutocorrectTable};
pub use beyond_avro::{BeyondAvroConfig, ContextRanker};
pub use dictionary::PhoneticDictionary;
pub use engine::{
    levenshtein_distance, separate_padding, PhoneticEngine, PhoneticSuggestion, SuggestionSource,
};
pub use memory::CandidateMemory;
pub use rules::{AvroRules, MatchRule, PatternRule, PhoneticPattern};
pub use suffix::SuffixTable;
