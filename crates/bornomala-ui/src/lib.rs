//! User interface, theme system, localization, and on-screen keyboard for Bornomala.
//!
//! This crate provides:
//! 1. `theme`: Color palettes (Flag Green `#006A4E`, Flag Red `#F42A41`, accessible dark red `#C21F35`),
//!    Light, Dark, High-Contrast, and Large-Text themes.
//! 2. `localization`: Complete, polished bilingual support (Bangla and English) for all views and dialogs.
//! 3. `state`: UI models for active view, scheme selection, converter state, and settings.
//! 4. `renderer`: High-resolution vector renderer generating pixel-perfect SVG mockups for all views
//!    without requiring an active X11 display server.

pub mod branding_assets;
pub mod localization;
pub mod renderer;
pub mod state;
pub mod theme;

pub use localization::{get_strings, Language, Strings};
pub use renderer::SvgRenderer;
pub use state::{ActiveView, FontConflictModal, InstallerStep, UiState};
pub use theme::{ThemeMode, ThemePalette};
