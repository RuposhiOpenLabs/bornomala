use bornomala_ui::{ActiveView, Language, SvgRenderer, ThemeMode, UiState};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = Path::new("assets/screenshots");
    fs::create_dir_all(out_dir)?;

    let views = [
        (ActiveView::StartupScreen, "startup_screen"),
        (ActiveView::InstallerSetup, "installer_setup"),
        (ActiveView::FloatingBar, "floating_bar"),
        (ActiveView::OnScreenKeyboard, "on_screen_keyboard"),
        (ActiveView::Converter, "converter"),
        (ActiveView::Settings, "settings"),
        (ActiveView::Welcome, "welcome"),
        (ActiveView::About, "about"),
    ];

    let themes = [
        (ThemeMode::Light, "light"),
        (ThemeMode::Dark, "dark"),
        (ThemeMode::HighContrast, "high_contrast"),
        (ThemeMode::LargeText, "large_text"),
    ];

    let languages = [(Language::Bangla, "bn"), (Language::English, "en")];

    let mut generated_count = 0;

    for (view, view_name) in views {
        for (theme, theme_name) in themes {
            for (lang, lang_name) in languages {
                let mut state = UiState::new();
                state.active_view = view;
                state.theme = theme;
                state.language = lang;

                let svg = SvgRenderer::render_view(&state);
                let filename = format!("{}_{}_{}.svg", view_name, theme_name, lang_name);
                let file_path = out_dir.join(&filename);
                fs::write(&file_path, svg)?;
                generated_count += 1;
            }
        }
    }

    println!(
        "Successfully generated {} UI screenshots in '{}'",
        generated_count,
        out_dir.display()
    );
    Ok(())
}
