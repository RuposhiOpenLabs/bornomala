use std::collections::HashMap;

/// Text Expander for frequently used government office snippets.
#[derive(Debug, Clone)]
pub struct TextExpander {
    snippets: HashMap<String, String>,
}

impl Default for TextExpander {
    fn default() -> Self {
        let mut snippets = HashMap::new();
        snippets.insert(
            ";sub".to_string(),
            "বিষয়: ".to_string(),
        );
        snippets.insert(
            ";sal".to_string(),
            "মহোদয়,\nসবিনয় নিবেদন এই যে, ".to_string(),
        );
        snippets.insert(
            ";closing".to_string(),
            "বিনীত নিবেদক,\n".to_string(),
        );
        snippets.insert(
            ";gov".to_string(),
            "গণপ্রজাতন্ত্রী বাংলাদেশ সরকার".to_string(),
        );
        snippets.insert(
            ";memo".to_string(),
            "স্মারক নং: ".to_string(),
        );
        Self { snippets }
    }
}

impl TextExpander {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_snippet(&mut self, trigger: String, replacement: String) {
        self.snippets.insert(trigger, replacement);
    }

    pub fn expand(&self, trigger: &str) -> Option<&String> {
        self.snippets.get(trigger)
    }

    pub fn check_buffer(&self, buffer: &str) -> Option<(&str, &String)> {
        for (trigger, replacement) in &self.snippets {
            if buffer.ends_with(trigger) {
                return Some((trigger.as_str(), replacement));
            }
        }
        None
    }
}
