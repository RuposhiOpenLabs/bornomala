use bornomala_engine::BornomalaEngine;
use bornomala_ui::{
    get_strings, ActiveView, Language, SvgRenderer, ThemeMode, ThemePalette, UiState,
};

#[test]
fn test_theme_palettes_validity() {
    let modes = [
        ThemeMode::Light,
        ThemeMode::Dark,
        ThemeMode::HighContrast,
        ThemeMode::LargeText,
    ];

    for mode in modes {
        let palette = ThemePalette::for_mode(mode);
        assert!(!palette.primary_green.is_empty());
        assert!(!palette.accent_red.is_empty());
        assert!(!palette.background.is_empty());
        assert!(palette.font_size_base >= 12);
    }
}

#[test]
fn test_localization_completeness() {
    for lang in [Language::Bangla, Language::English] {
        let strings = get_strings(lang);
        assert!(!strings.app_title.is_empty());
        assert!(!strings.tagline.is_empty());
        assert!(!strings.sub_tagline.is_empty());
        assert!(!strings.conv_title.is_empty());
        assert!(!strings.osk_title.is_empty());
        assert!(!strings.settings_title.is_empty());
        assert!(!strings.about_title.is_empty());
        assert_eq!(strings.link_org, "https://github.com/RuposhiOpenLabs");
        assert_eq!(strings.link_dev, "https://github.com/MdSagorMunshi/");
        assert_eq!(strings.support_email, "ryn@disr.it");
        assert!(!strings.startup_title.is_empty());
        assert!(!strings.settings_autostart.is_empty());
        assert!(!strings.settings_startup_screen.is_empty());
        assert!(!strings.installer_title.is_empty());
        assert!(!strings.installer_terms_heading.is_empty());
        assert!(!strings.installer_font_conflict_title.is_empty());
        assert!(!strings.installer_comp_fonts.is_empty());
    }
}

#[test]
fn test_autostart_and_splash_behavior() {
    let mut state = UiState::new();
    assert!(state.auto_startup_enabled);
    assert!(state.show_startup_screen);
    assert_eq!(state.startup_screen_duration_ms, 2500);
    assert_eq!(state.active_view, ActiveView::StartupScreen);
    assert!(state.startup_screen_visible);
    assert!(state.startup_screen_bengali_by_default);
    assert_eq!(state.language, Language::Bangla);
    assert_eq!(
        state.get_startup_screen_path(),
        "assets/branding/startup_screen_bn.png"
    );

    state.toggle_autostart();
    assert!(!state.auto_startup_enabled);

    state.toggle_startup_screen();
    assert!(!state.show_startup_screen);

    state.dismiss_startup_screen();
    assert!(!state.startup_screen_visible);
    assert_eq!(state.active_view, ActiveView::FloatingBar);
}

#[test]
fn test_installer_setup_wizard_flow() {
    let mut state = UiState::new();
    state.active_view = ActiveView::InstallerSetup;
    assert_eq!(state.installer_step, bornomala_ui::InstallerStep::Welcome);

    // Progression: Welcome -> Terms -> Components -> Installing -> Finished
    state.next_installer_step();
    assert_eq!(state.installer_step, bornomala_ui::InstallerStep::Terms);

    state.next_installer_step();
    assert_eq!(
        state.installer_step,
        bornomala_ui::InstallerStep::Components
    );

    state.next_installer_step();
    assert_eq!(
        state.installer_step,
        bornomala_ui::InstallerStep::Installing
    );

    // Font conflict modal
    assert!(state.installer_font_conflict.is_none());
    state.trigger_font_conflict_dialog("SutonnyMJ", "SutonnyMJ Regular.ttf");
    assert!(state.installer_font_conflict.is_some());
    let modal = state.installer_font_conflict.as_ref().unwrap();
    assert_eq!(modal.font_name, "SutonnyMJ");
    assert_eq!(modal.filename, "SutonnyMJ Regular.ttf");

    // Render with conflict modal active
    let svg_conflict = SvgRenderer::render_view(&state);
    assert!(svg_conflict.contains("SutonnyMJ Regular.ttf"));

    // Dismiss modal and advance to Finished
    state.dismiss_font_conflict_dialog();
    assert!(state.installer_font_conflict.is_none());

    state.next_installer_step();
    assert_eq!(state.installer_step, bornomala_ui::InstallerStep::Finished);

    let svg_finished = SvgRenderer::render_view(&state);
    assert!(svg_finished.contains("F12"));
    assert!(svg_finished.contains("Ctrl + Alt + V"));
}

#[test]
fn test_ui_state_and_converter_execution() {
    let engine = BornomalaEngine::default();
    let mut state = UiState::new();

    // Direction Unicode -> ANSI
    state.conv_unicode_text = "আমার সোনার বাংলা".to_string();
    state.conv_direction_unicode_to_ansi = true;
    state.execute_conversion(&engine);

    assert_eq!(state.conv_ansi_text, "Avgvi ‡mvbvi evsjv");

    // Switch direction and convert back
    state.switch_converter_direction();
    state.execute_conversion(&engine);
    assert_eq!(state.conv_unicode_text, "আমার সোনার বাংলা");
}

#[test]
fn test_svg_rendering_all_views_and_themes() {
    let views = [
        ActiveView::StartupScreen,
        ActiveView::InstallerSetup,
        ActiveView::FloatingBar,
        ActiveView::OnScreenKeyboard,
        ActiveView::Converter,
        ActiveView::Settings,
        ActiveView::Welcome,
        ActiveView::About,
    ];

    for view in views {
        for theme in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::HighContrast] {
            let mut state = UiState::new();
            state.active_view = view;
            state.theme = theme;
            state.language = Language::Bangla;

            let svg = SvgRenderer::render_view(&state);
            assert!(svg.starts_with("<svg"));
            assert!(svg.ends_with("</svg>"));
            assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        }
    }
}
