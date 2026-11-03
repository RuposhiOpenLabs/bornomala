pub mod amount_to_words;
pub mod bangla_date;
pub mod digit_converter;
pub mod text_expander;
pub mod app_profile;
pub mod autostart;
pub mod fonts;

pub use amount_to_words::{amount_to_bangla_words, amount_to_english_words};
pub use bangla_date::{gregorian_to_bangla, BanglaDate};
pub use digit_converter::convert_digits;
pub use text_expander::TextExpander;
pub use app_profile::AppProfileManager;
pub use autostart::{AutostartManager, StartupOptions, DEFAULT_SPLASH_DURATION_MS, REGISTRY_RUN_KEY, REGISTRY_VALUE_NAME};
pub use fonts::{get_preloaded_fonts, FontConflictAction, FontInstallScope, PreloadedFont, PRELOADED_FONTS};
