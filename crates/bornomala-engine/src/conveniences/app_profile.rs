use std::collections::HashMap;
use crate::events::InputScheme;

/// Tracks and remembers last used typing scheme per application executable name or window title.
#[derive(Debug, Clone, Default)]
pub struct AppProfileManager {
    profiles: HashMap<String, InputScheme>,
    default_scheme: InputScheme,
}

impl AppProfileManager {
    pub fn new(default_scheme: InputScheme) -> Self {
        let mut profiles = HashMap::new();
        // Standard defaults: modern apps default to Unicode; legacy desktop publishing apps default to ANSI
        profiles.insert("notepad.exe".to_string(), InputScheme::ClassicUnicode);
        profiles.insert("winword.exe".to_string(), InputScheme::ClassicAnsi);
        profiles.insert("illustrator.exe".to_string(), InputScheme::ClassicAnsi);
        profiles.insert("photoshop.exe".to_string(), InputScheme::ClassicAnsi);
        profiles.insert("indesign.exe".to_string(), InputScheme::ClassicAnsi);

        Self {
            profiles,
            default_scheme,
        }
    }

    /// Retrieve the preferred scheme for an application.
    pub fn get_scheme_for_app(&self, app_name: &str) -> InputScheme {
        let key = app_name.to_lowercase();
        self.profiles
            .get(&key)
            .copied()
            .unwrap_or(self.default_scheme)
    }

    /// Update preferred scheme for an application.
    pub fn set_scheme_for_app(&mut self, app_name: String, scheme: InputScheme) {
        let key = app_name.to_lowercase();
        self.profiles.insert(key, scheme);
    }
}
