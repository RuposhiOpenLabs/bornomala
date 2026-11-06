use crate::localization::get_strings;
use crate::state::{ActiveView, UiState};
use crate::theme::ThemePalette;
use bornomala_engine::InputScheme;

pub struct SvgRenderer;

impl SvgRenderer {
    pub fn render_view(state: &UiState) -> String {
        let palette = ThemePalette::for_mode(state.theme);
        let strings = get_strings(state.language);

        match state.active_view {
            ActiveView::StartupScreen => Self::render_startup_screen(state, &palette, &strings),
            ActiveView::InstallerSetup => Self::render_installer_setup(state, &palette, &strings),
            ActiveView::FloatingBar => Self::render_floating_bar(state, &palette, &strings),
            ActiveView::OnScreenKeyboard => Self::render_osk(state, &palette, &strings),
            ActiveView::Converter => Self::render_converter(state, &palette, &strings),
            ActiveView::Settings => Self::render_settings(state, &palette, &strings),
            ActiveView::Welcome => Self::render_welcome(state, &palette, &strings),
            ActiveView::About => Self::render_about(state, &palette, &strings),
        }
    }

    fn xml_escape(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn render_floating_bar(state: &UiState, pal: &ThemePalette, strings: &crate::localization::Strings) -> String {
        let width = 560;
        let height = 54;

        let unicode_bg = if state.scheme == InputScheme::ClassicUnicode { pal.primary_green } else { pal.surface };
        let unicode_fg = if state.scheme == InputScheme::ClassicUnicode { "#FFFFFF" } else { pal.text_primary };

        let ansi_bg = if state.scheme == InputScheme::ClassicAnsi { pal.primary_green } else { pal.surface };
        let ansi_fg = if state.scheme == InputScheme::ClassicAnsi { "#FFFFFF" } else { pal.text_primary };

        let phonetic_bg = if state.scheme == InputScheme::Phonetic { pal.primary_green } else { pal.surface };
        let phonetic_fg = if state.scheme == InputScheme::Phonetic { "#FFFFFF" } else { pal.text_primary };

        let english_bg = if state.scheme == InputScheme::PassThrough { pal.primary_green } else { pal.surface };
        let english_fg = if state.scheme == InputScheme::PassThrough { "#FFFFFF" } else { pal.text_primary };

        let status_color = if state.is_pass_through { pal.accessible_red } else { "#28A745" };
        let status_title = if state.is_pass_through { strings.status_passthrough } else { strings.status_active };

        format!(
            r###"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <defs>
    <filter id="shadow" x="-5%" y="-10%" width="110%" height="130%">
      <feDropShadow dx="0" dy="3" stdDeviation="4" flood-color="#000000" flood-opacity="0.25"/>
    </filter>
  </defs>
  <rect width="{width}" height="{height}" rx="12" fill="{bg}" stroke="{border}" stroke-width="1.5" filter="url(#shadow)"/>
  
  <!-- Authentic Bornomala Logo Emblem -->
  <g transform="translate(14, 11)">
    <rect width="32" height="32" rx="16" fill="{green}"/>
    <image href="{logo_icon}" x="3" y="3" width="26" height="26" preserveAspectRatio="xMidYMid meet"/>
  </g>

  <!-- Title & Status -->
  <text x="56" y="25" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{text_primary}">Bornomala</text>
  <circle cx="58" cy="37" r="4" fill="{status_color}"/>
  <text x="68" y="41" font-family="sans-serif, 'SolaimanLipi'" font-size="10" fill="{text_secondary}">{status_title}</text>

  <!-- Scheme Buttons -->
  <g transform="translate(160, 11)">
    <!-- Unicode -->
    <rect x="0" y="0" width="70" height="32" rx="6" fill="{unicode_bg}" stroke="{border}" stroke-width="1"/>
    <text x="35" y="21" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{unicode_fg}" text-anchor="middle">{lbl_unicode}</text>
    
    <!-- ANSI -->
    <rect x="76" y="0" width="70" height="32" rx="6" fill="{ansi_bg}" stroke="{border}" stroke-width="1"/>
    <text x="111" y="21" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{ansi_fg}" text-anchor="middle">ANSI</text>

    <!-- Phonetic -->
    <rect x="152" y="0" width="70" height="32" rx="6" fill="{phonetic_bg}" stroke="{border}" stroke-width="1"/>
    <text x="187" y="21" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{phonetic_fg}" text-anchor="middle">{lbl_phonetic}</text>

    <!-- English -->
    <rect x="228" y="0" width="60" height="32" rx="6" fill="{english_bg}" stroke="{border}" stroke-width="1"/>
    <text x="258" y="21" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{english_fg}" text-anchor="middle">EN</text>
  </g>

  <!-- Tools Quick Buttons -->
  <g transform="translate(460, 11)">
    <rect x="0" y="0" width="28" height="32" rx="6" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="14" y="21" font-family="sans-serif" font-size="13" fill="{text_primary}" text-anchor="middle">⌨</text>

    <rect x="32" y="0" width="28" height="32" rx="6" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="46" y="21" font-family="sans-serif" font-size="13" fill="{text_primary}" text-anchor="middle">⇄</text>

    <rect x="64" y="0" width="28" height="32" rx="6" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="78" y="21" font-family="sans-serif" font-size="13" fill="{text_primary}" text-anchor="middle">⚙</text>
  </g>
</svg>"###,
            width = width,
            height = height,
            bg = pal.background,
            border = pal.border,
            surface = pal.surface,
            green = pal.primary_green,
            logo_icon = crate::branding_assets::LOGO_ICON_DATA_URI,
            text_primary = pal.text_primary,
            text_secondary = pal.text_secondary,
            status_color = status_color,
            status_title = Self::xml_escape(status_title),
            unicode_bg = unicode_bg,
            unicode_fg = unicode_fg,
            lbl_unicode = Self::xml_escape(strings.scheme_unicode),
            ansi_bg = ansi_bg,
            ansi_fg = ansi_fg,
            phonetic_bg = phonetic_bg,
            phonetic_fg = phonetic_fg,
            lbl_phonetic = Self::xml_escape(strings.scheme_phonetic),
            english_bg = english_bg,
            english_fg = english_fg,
        )
    }

    fn render_osk(state: &UiState, pal: &ThemePalette, strings: &crate::localization::Strings) -> String {
        let width = 840;
        let height = 340;

        // Sample key definitions for 4 rows
        let row1 = [
            ("`", "~", "1"), ("1", "!", "১"), ("2", "৥", "২"), ("3", "#", "৩"), ("4", "৳", "৪"),
            ("5", "%", "৫"), ("6", "৲", "৬"), ("7", "ঁ", "৭"), ("8", "*", "৮"), ("9", "(", "৯"),
            ("0", ")", "০"), ("-", "_", "-"), ("=", "+", "="), ("BS", "←", "←")
        ];
        let row2 = [
            ("Tab", "⇥", "⇥"), ("Q", "ং", "ঙ"), ("W", "য়", "য"), ("E", "ঢ", "ড"), ("R", "ফ", "প"),
            ("T", "ঠ", "ট"), ("Y", "ছ", "চ"), ("U", "ঝ", "জ"), ("I", "ঞ", "হ"), ("O", "ঘ", "গ"),
            ("P", "ঢ়", "ড়"), ("[", "{", "["), ("]", "}", "]"), ("\\", "|", "\\")
        ];
        let row3 = [
            ("Caps", "⇪", "⇪"), ("A", "র্", "ৃ"), ("S", "ূ", "ু"), ("D", "ী", "ি"), ("F", "অ", "া"),
            ("G", "।", "্"), ("H", "ভ", "ব"), ("J", "খ", "ক"), ("K", "থ", "ত"), ("L", "ধ", "দ"),
            (";", ":", ";"), ("'", "\"", "'"), ("Enter", "↵", "↵")
        ];
        let row4 = [
            ("Shift", "⇧", "⇧"), ("Z", "্র", "্য"), ("X", "ঔ", "ও"), ("C", "ৈ", "ে"), ("V", "ল", "র"),
            ("B", "ণ", "ন"), ("N", "ষ", "শ"), ("M", "স", "ণ"), (",", "<", ","), (".", ">", "."),
            ("/", "?", "/"), ("Shift", "⇧", "⇧")
        ];

        let mut svg = format!(
            r###"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <rect width="{width}" height="{height}" rx="12" fill="{bg}" stroke="{border}" stroke-width="2"/>
  <rect width="{width}" height="44" rx="12" fill="{surface}"/>
  <text x="20" y="28" font-family="sans-serif, 'SolaimanLipi'" font-size="16" font-weight="bold" fill="{text_primary}">{title}</text>
  <rect x="700" y="8" width="120" height="28" rx="6" fill="{green}"/>
  <text x="760" y="27" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{scheme}</text>
  <g transform="translate(20, 56)">
"###,
            width = width,
            height = height,
            bg = pal.background,
            surface = pal.surface,
            border = pal.border,
            green = pal.primary_green,
            text_primary = pal.text_primary,
            title = Self::xml_escape(strings.osk_title),
            scheme = Self::xml_escape(strings.scheme_unicode),
        );

        // Render Row 1
        let mut x = 0.0;
        for (latin, shift, norm) in row1 {
            let kw = if latin == "BS" { 76.0 } else { 52.0 };
            svg.push_str(&Self::render_key(x, 0.0, kw, 48.0, latin, shift, norm, pal, state.osk_shift));
            x += kw + 6.0;
        }

        // Render Row 2
        let mut x = 0.0;
        for (latin, shift, norm) in row2 {
            let kw = if latin == "Tab" { 70.0 } else { 52.0 };
            svg.push_str(&Self::render_key(x, 54.0, kw, 48.0, latin, shift, norm, pal, state.osk_shift));
            x += kw + 6.0;
        }

        // Render Row 3
        let mut x = 0.0;
        for (latin, shift, norm) in row3 {
            let kw = if latin == "Caps" { 80.0 } else if latin == "Enter" { 84.0 } else { 52.0 };
            svg.push_str(&Self::render_key(x, 108.0, kw, 48.0, latin, shift, norm, pal, state.osk_shift));
            x += kw + 6.0;
        }

        // Render Row 4
        let mut x = 0.0;
        for (latin, shift, norm) in row4 {
            let kw = if latin == "Shift" { 96.0 } else { 52.0 };
            svg.push_str(&Self::render_key(x, 162.0, kw, 48.0, latin, shift, norm, pal, state.osk_shift));
            x += kw + 6.0;
        }

        // Bottom Spacebar row
        let space_row = format!(
            r###"    <!-- Bottom control row -->
    <rect x="0" y="216" width="80" height="48" rx="6" fill="{surf}" stroke="{border}" stroke-width="1"/>
    <text x="40" y="246" font-family="sans-serif" font-size="12" fill="{text}" text-anchor="middle">Ctrl</text>

    <rect x="86" y="216" width="80" height="48" rx="6" fill="{surf}" stroke="{border}" stroke-width="1"/>
    <text x="126" y="246" font-family="sans-serif" font-size="12" fill="{text}" text-anchor="middle">Alt</text>

    <rect x="172" y="216" width="360" height="48" rx="6" fill="{surf}" stroke="{border}" stroke-width="1"/>
    <text x="352" y="246" font-family="sans-serif, 'SolaimanLipi'" font-size="14" fill="{text}" text-anchor="middle">স্পেসবার (Spacebar)</text>

    <rect x="538" y="216" width="80" height="48" rx="6" fill="{surf}" stroke="{border}" stroke-width="1"/>
    <text x="578" y="246" font-family="sans-serif" font-size="12" fill="{text}" text-anchor="middle">AltGr</text>

    <rect x="624" y="216" width="80" height="48" rx="6" fill="{green}" stroke="{border}" stroke-width="1"/>
    <text x="664" y="246" font-family="sans-serif, 'SolaimanLipi'" font-size="14" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{hasanta}</text>

    <rect x="710" y="216" width="90" height="48" rx="6" fill="{surf}" stroke="{border}" stroke-width="1"/>
    <text x="755" y="246" font-family="sans-serif, 'SolaimanLipi'" font-size="14" fill="{text}" text-anchor="middle">{danda}</text>
  </g>
</svg>"###,
            surf = pal.surface,
            border = pal.border,
            text = pal.text_primary,
            green = pal.primary_green,
            hasanta = Self::xml_escape(strings.osk_hasanta),
            danda = Self::xml_escape(strings.osk_danda),
        );

        svg.push_str(&space_row);
        svg
    }

    fn render_key(
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        latin: &str,
        shift_char: &str,
        normal_char: &str,
        pal: &ThemePalette,
        shift_active: bool,
    ) -> String {
        let (active_char, inactive_char) = if shift_active {
            (shift_char, normal_char)
        } else {
            (normal_char, shift_char)
        };

        format!(
            r###"    <g transform="translate({x}, {y})">
      <rect width="{w}" height="{h}" rx="6" fill="{surf}" stroke="{border}" stroke-width="1"/>
      <text x="6" y="14" font-family="sans-serif" font-size="9" fill="{muted}">{latin}</text>
      <text x="{rx}" y="16" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{muted}" text-anchor="end">{inactive}</text>
      <text x="{cx}" y="36" font-family="sans-serif, 'SolaimanLipi'" font-size="16" font-weight="bold" fill="{text}" text-anchor="middle">{active}</text>
    </g>
"###,
            x = x,
            y = y,
            w = w,
            h = h,
            surf = pal.surface,
            border = pal.border,
            muted = pal.text_secondary,
            text = pal.text_primary,
            latin = Self::xml_escape(latin),
            rx = w - 6.0,
            inactive = Self::xml_escape(inactive_char),
            cx = w / 2.0,
            active = Self::xml_escape(active_char),
        )
    }

    fn render_converter(state: &UiState, pal: &ThemePalette, strings: &crate::localization::Strings) -> String {
        let width = 760;
        let height = 480;

        let dir_text = if state.conv_direction_unicode_to_ansi {
            "ইউনিকোড ➔ এএনএসআই (সুতোন্নীএমজে)"
        } else {
            "এএনএসআই (সুতোন্নীএমজে) ➔ ইউনিকোড"
        };

        format!(
            r###"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <rect width="{width}" height="{height}" rx="12" fill="{bg}" stroke="{border}" stroke-width="2"/>
  
  <!-- Header Bar -->
  <rect width="{width}" height="56" rx="12" fill="{green}"/>
  <text x="24" y="35" font-family="sans-serif, 'SolaimanLipi'" font-size="18" font-weight="bold" fill="#FFFFFF">{title}</text>
  
  <!-- Direction Toggle Indicator -->
  <g transform="translate(440, 12)">
    <rect width="296" height="32" rx="6" fill="rgba(255,255,255,0.15)"/>
    <text x="148" y="21" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{dir_text}</text>
  </g>

  <!-- Left Box: Unicode Text Area -->
  <g transform="translate(24, 76)">
    <text x="0" y="16" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{text_primary}">{lbl_unicode}</text>
    <rect y="24" width="336" height="260" rx="8" fill="{surface}" stroke="{border}" stroke-width="1.5"/>
    <text x="14" y="56" font-family="sans-serif, 'SolaimanLipi'" font-size="14" fill="{text_primary}">{txt_unicode}</text>
  </g>

  <!-- Center Action Button -->
  <g transform="translate(368, 180)">
    <circle cx="12" cy="12" r="22" fill="{green}" stroke="{border}" stroke-width="1"/>
    <text x="12" y="19" font-family="sans-serif" font-size="18" font-weight="bold" fill="#FFFFFF" text-anchor="middle">⇄</text>
  </g>

  <!-- Right Box: ANSI Text Area -->
  <g transform="translate(400, 76)">
    <text x="0" y="16" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{text_primary}">{lbl_ansi}</text>
    <rect y="24" width="336" height="260" rx="8" fill="{surface}" stroke="{border}" stroke-width="1.5"/>
    <text x="14" y="56" font-family="sans-serif, monospace" font-size="14" fill="{text_primary}">{txt_ansi}</text>
  </g>

  <!-- Status & Message -->
  <rect x="24" y="372" width="712" height="36" rx="6" fill="{surface}" stroke="{border}" stroke-width="1"/>
  <circle cx="40" cy="390" r="5" fill="#28A745"/>
  <text x="54" y="395" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_primary}">{status_msg}</text>

  <!-- Bottom Buttons -->
  <g transform="translate(24, 422)">
    <rect x="0" y="0" width="140" height="38" rx="8" fill="{green}"/>
    <text x="70" y="24" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{btn_convert}</text>

    <rect x="152" y="0" width="140" height="38" rx="8" fill="{surface}" stroke="{border}" stroke-width="1.5"/>
    <text x="222" y="24" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{text_primary}" text-anchor="middle">{btn_switch}</text>

    <rect x="304" y="0" width="140" height="38" rx="8" fill="{surface}" stroke="{border}" stroke-width="1.5"/>
    <text x="374" y="24" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}" text-anchor="middle">{btn_copy}</text>

    <rect x="456" y="0" width="120" height="38" rx="8" fill="{surface}" stroke="{border}" stroke-width="1.5"/>
    <text x="516" y="24" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{accent_red}" text-anchor="middle">{btn_clear}</text>
  </g>
</svg>"###,
            width = width,
            height = height,
            bg = pal.background,
            surface = pal.surface,
            border = pal.border,
            green = pal.primary_green,
            accent_red = pal.accessible_red,
            text_primary = pal.text_primary,
            title = Self::xml_escape(strings.conv_title),
            dir_text = Self::xml_escape(dir_text),
            lbl_unicode = Self::xml_escape(strings.conv_unicode_label),
            lbl_ansi = Self::xml_escape(strings.conv_ansi_label),
            txt_unicode = Self::xml_escape(&state.conv_unicode_text),
            txt_ansi = Self::xml_escape(&state.conv_ansi_text),
            status_msg = Self::xml_escape(if state.conv_last_message.is_empty() {
                strings.conv_status_ready
            } else {
                &state.conv_last_message
            }),
            btn_convert = Self::xml_escape(strings.conv_btn_convert),
            btn_switch = Self::xml_escape(strings.conv_btn_switch),
            btn_copy = Self::xml_escape(strings.conv_btn_copy),
            btn_clear = Self::xml_escape(strings.conv_btn_clear),
        )
    }

    fn render_settings(state: &UiState, pal: &ThemePalette, strings: &crate::localization::Strings) -> String {
        let width = 640;
        let height = 580;

        let autostart_knob_x = if state.auto_startup_enabled { 43 } else { 25 };
        let autostart_color = if state.auto_startup_enabled { pal.primary_green } else { pal.border };
        let splash_knob_x = if state.show_startup_screen { 43 } else { 25 };
        let splash_color = if state.show_startup_screen { pal.primary_green } else { pal.border };

        format!(
            r###"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <rect width="{width}" height="{height}" rx="12" fill="{bg}" stroke="{border}" stroke-width="2"/>
  
  <rect width="{width}" height="56" rx="12" fill="{surface}"/>
  <text x="24" y="35" font-family="sans-serif, 'SolaimanLipi'" font-size="18" font-weight="bold" fill="{text_primary}">{title}</text>

  <g transform="translate(24, 76)">
    <!-- Setting 1: Default Mode -->
    <text x="0" y="18" font-family="sans-serif, 'SolaimanLipi'" font-size="14" font-weight="bold" fill="{text_primary}">{s_default_mode}</text>
    <rect x="0" y="28" width="592" height="42" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <circle cx="24" cy="49" r="6" fill="{green}"/>
    <text x="40" y="54" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">ইউনিকোড ফিক্সড লেআউট (Classic Unicode) [F12 বা Ctrl+Alt+V]</text>

    <!-- Setting 2: Backspace Behavior -->
    <text x="0" y="96" font-family="sans-serif, 'SolaimanLipi'" font-size="14" font-weight="bold" fill="{text_primary}">{s_backspace}</text>
    <rect x="0" y="106" width="592" height="42" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <circle cx="24" cy="127" r="6" fill="{green}"/>
    <text x="40" y="132" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">{s_backspace_atomic} (প্রস্তাবিত ও নির্ভুল)</text>

    <!-- Setting 3: Numeral switch -->
    <text x="0" y="174" font-family="sans-serif, 'SolaimanLipi'" font-size="14" font-weight="bold" fill="{text_primary}">{s_digits}</text>
    <rect x="0" y="184" width="592" height="42" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <circle cx="24" cy="205" r="6" fill="{green}"/>
    <text x="40" y="210" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">বাংলা সংখ্যা (০, ১, ২, ৩...) ডিফল্ট হিসেবে চালু</text>

    <!-- Setting 4: Per App memory -->
    <text x="0" y="252" font-family="sans-serif, 'SolaimanLipi'" font-size="14" font-weight="bold" fill="{text_primary}">{s_per_app}</text>
    <rect x="0" y="262" width="592" height="42" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <rect x="18" y="275" width="34" height="18" rx="9" fill="{green}"/>
    <circle cx="43" cy="284" r="7" fill="#FFFFFF"/>
    <text x="64" y="289" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">প্রতিটি অ্যাপ্লিকেশনের টাইপিং মোড আলাদাভাবে মনে রাখুন</text>

    <!-- Setting 5: Auto Startup -->
    <text x="0" y="330" font-family="sans-serif, 'SolaimanLipi'" font-size="14" font-weight="bold" fill="{text_primary}">{s_autostart}</text>
    <rect x="0" y="340" width="592" height="42" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <rect x="18" y="353" width="34" height="18" rx="9" fill="{autostart_color}"/>
    <circle cx="{autostart_knob_x}" cy="362" r="7" fill="#FFFFFF"/>
    <text x="64" y="367" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">{s_autostart_desc}</text>

    <!-- Setting 6: Startup Splash Screen -->
    <text x="0" y="408" font-family="sans-serif, 'SolaimanLipi'" font-size="14" font-weight="bold" fill="{text_primary}">{s_splash}</text>
    <rect x="0" y="418" width="592" height="42" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <rect x="18" y="431" width="34" height="18" rx="9" fill="{splash_color}"/>
    <circle cx="{splash_knob_x}" cy="440" r="7" fill="#FFFFFF"/>
    <text x="64" y="445" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">{s_splash_desc}</text>
  </g>

  <!-- Bottom Action -->
  <rect x="476" y="520" width="140" height="38" rx="8" fill="{green}"/>
  <text x="546" y="544" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="#FFFFFF" text-anchor="middle">সংরক্ষণ করুন</text>
</svg>"###,
            width = width,
            height = height,
            bg = pal.background,
            surface = pal.surface,
            border = pal.border,
            green = pal.primary_green,
            text_primary = pal.text_primary,
            autostart_color = autostart_color,
            autostart_knob_x = autostart_knob_x,
            splash_color = splash_color,
            splash_knob_x = splash_knob_x,
            title = Self::xml_escape(strings.settings_title),
            s_default_mode = Self::xml_escape(strings.settings_default_mode),
            s_backspace = Self::xml_escape(strings.settings_backspace),
            s_backspace_atomic = Self::xml_escape(strings.settings_backspace_atomic),
            s_digits = Self::xml_escape(strings.settings_digits),
            s_per_app = Self::xml_escape(strings.settings_per_app),
            s_autostart = Self::xml_escape(strings.settings_autostart),
            s_autostart_desc = Self::xml_escape(strings.settings_autostart_desc),
            s_splash = Self::xml_escape(strings.settings_startup_screen),
            s_splash_desc = Self::xml_escape(strings.settings_startup_screen_desc),
        )
    }

    fn render_welcome(_state: &UiState, pal: &ThemePalette, strings: &crate::localization::Strings) -> String {
        let width = 680;
        let height = 460;

        format!(
            r###"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <rect width="{width}" height="{height}" rx="16" fill="{bg}" stroke="{border}" stroke-width="2"/>
  
  <!-- Banner with Bangladesh Flag Colors & Authentic Logo -->
  <rect width="{width}" height="130" rx="16" fill="{green}"/>
  <image href="{logo_mark}" x="240" y="15" width="200" height="100" preserveAspectRatio="xMidYMid meet"/>

  <!-- Welcome Heading -->
  <text x="340" y="172" font-family="sans-serif, 'SolaimanLipi'" font-size="22" font-weight="bold" fill="{text_primary}" text-anchor="middle">{title}</text>
  <text x="340" y="198" font-family="sans-serif, 'SolaimanLipi'" font-size="14" fill="{text_secondary}" text-anchor="middle">{tagline}</text>
  <text x="340" y="218" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{accent_red}" text-anchor="middle">{sub_tagline}</text>

  <!-- 3 Steps Cards -->
  <g transform="translate(40, 240)">
    <rect width="600" height="38" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="18" y="24" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">{step1}</text>

    <rect y="46" width="600" height="38" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="18" y="70" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">{step2}</text>

    <rect y="92" width="600" height="38" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="18" y="116" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">{step3}</text>
  </g>

  <!-- Privacy guarantee and start button -->
  <text x="40" y="415" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_secondary}">🛡️ {privacy}</text>
  <rect x="480" y="394" width="160" height="42" rx="8" fill="{green}"/>
  <text x="560" y="420" font-family="sans-serif, 'SolaimanLipi'" font-size="14" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{btn_start}</text>
</svg>"###,
            width = width,
            height = height,
            bg = pal.background,
            surface = pal.surface,
            border = pal.border,
            green = pal.primary_green,
            accent_red = pal.accessible_red,
            text_primary = pal.text_primary,
            text_secondary = pal.text_secondary,
            title = Self::xml_escape(strings.welcome_title),
            logo_mark = crate::branding_assets::LOGO_MARK_DATA_URI,
            tagline = Self::xml_escape(strings.tagline),
            sub_tagline = Self::xml_escape(strings.sub_tagline),
            step1 = Self::xml_escape(strings.welcome_step1),
            step2 = Self::xml_escape(strings.welcome_step2),
            step3 = Self::xml_escape(strings.welcome_step3),
            privacy = Self::xml_escape(strings.welcome_privacy),
            btn_start = Self::xml_escape(strings.welcome_btn_start),
        )
    }

    fn render_about(_state: &UiState, pal: &ThemePalette, strings: &crate::localization::Strings) -> String {
        let width = 580;
        let height = 500;

        format!(
            r###"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <rect width="{width}" height="{height}" rx="14" fill="{bg}" stroke="{border}" stroke-width="2"/>
  
  <rect width="{width}" height="64" rx="14" fill="{green}"/>
  <image href="{logo_icon}" x="24" y="12" width="40" height="40" preserveAspectRatio="xMidYMid meet"/>
  <text x="74" y="38" font-family="sans-serif, 'SolaimanLipi'" font-size="18" font-weight="bold" fill="#FFFFFF">{title}</text>

  <g transform="translate(32, 88)">
    <text x="0" y="20" font-family="sans-serif, 'SolaimanLipi'" font-size="15" font-weight="bold" fill="{text_primary}">{version}</text>
    <text x="0" y="46" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">{publisher}</text>
    <text x="0" y="70" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">{developer}</text>
    <text x="0" y="94" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">{license}</text>

    <!-- Links & Contact Cards -->
    <rect y="112" width="516" height="34" rx="6" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="14" y="134" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_primary}">🏢 {link_org_label}:</text>
    <a href="{link_org}" target="_blank">
      <text x="502" y="134" font-family="sans-serif" font-size="12" fill="{green}" text-anchor="end" font-weight="bold">github.com/RuposhiOpenLabs</text>
    </a>

    <rect y="152" width="516" height="34" rx="6" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="14" y="174" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_primary}">👨‍💻 {link_dev_label}:</text>
    <a href="{link_dev}" target="_blank">
      <text x="502" y="174" font-family="sans-serif" font-size="12" fill="{green}" text-anchor="end" font-weight="bold">github.com/MdSagorMunshi</text>
    </a>

    <rect y="192" width="516" height="34" rx="6" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="14" y="214" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_primary}">✉️ {support_email_label}</text>

    <!-- Tagline Card -->
    <rect y="234" width="516" height="42" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="16" y="260" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{green}">{tagline}</text>

    <!-- Integrity Badges -->
    <rect y="286" width="250" height="34" rx="6" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="14" y="308" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{text_secondary}">🛡️ {offline_badge}</text>

    <rect x="260" y="286" width="256" height="34" rx="6" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="274" y="308" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{text_secondary}">🔑 {verified_badge}</text>
  </g>

  <!-- Update Checker Button -->
  <rect x="180" y="440" width="220" height="38" rx="8" fill="{green}"/>
  <text x="290" y="464" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{btn_update}</text>
</svg>"###,
            width = width,
            height = height,
            bg = pal.background,
            surface = pal.surface,
            border = pal.border,
            green = pal.primary_green,
            logo_icon = crate::branding_assets::LOGO_ICON_DATA_URI,
            text_primary = pal.text_primary,
            text_secondary = pal.text_secondary,
            title = Self::xml_escape(strings.about_title),
            version = Self::xml_escape(strings.about_version),
            publisher = Self::xml_escape(strings.about_publisher),
            developer = Self::xml_escape(strings.about_developer),
            license = Self::xml_escape(strings.about_license),
            link_org_label = Self::xml_escape(strings.link_org_label),
            link_org = Self::xml_escape(strings.link_org),
            link_dev_label = Self::xml_escape(strings.link_dev_label),
            link_dev = Self::xml_escape(strings.link_dev),
            support_email_label = Self::xml_escape(strings.support_email_label),
            tagline = Self::xml_escape(strings.tagline),
            offline_badge = Self::xml_escape(strings.about_offline_badge),
            verified_badge = Self::xml_escape(strings.about_verified_badge),
            btn_update = Self::xml_escape(strings.about_btn_check_update),
        )
    }

    fn render_startup_screen(state: &UiState, pal: &ThemePalette, strings: &crate::localization::Strings) -> String {
        let width = 520;
        let height = 560;

        // By default, startup screen displays the rich Bengali version (Bijoy / Ekushe style)
        let bn_strings = crate::localization::get_strings(crate::localization::Language::Bangla);
        let active_strings = if state.startup_screen_bengali_by_default {
            &bn_strings
        } else {
            strings
        };

        let is_bn = state.startup_screen_bengali_by_default || state.language == crate::localization::Language::Bangla;
        let img_path = if is_bn {
            "../branding/startup_screen_bn.png"
        } else {
            "../branding/startup_screen_en.png"
        };
        let countdown_suffix = if is_bn { "(২.৫ সেকেন্ড)" } else { "(2.5s)" };

        format!(
            r###"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <defs>
    <filter id="splashShadow" x="-10%" y="-10%" width="120%" height="120%">
      <feDropShadow dx="0" dy="8" stdDeviation="16" flood-color="#000000" flood-opacity="0.35"/>
    </filter>
    <linearGradient id="splashBannerGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="{green}"/>
      <stop offset="100%" stop-color="#0D4B26"/>
    </linearGradient>
    <clipPath id="heroClip">
      <rect width="{width}" height="170" rx="18"/>
    </clipPath>
  </defs>

  <!-- Modal Background Card -->
  <rect width="{width}" height="{height}" rx="18" fill="{bg}" stroke="{border}" stroke-width="2" filter="url(#splashShadow)"/>

  <!-- Top Hero Header Banner with Branded Artwork overlay -->
  <g clip-path="url(#heroClip)">
    <rect width="{width}" height="170" fill="url(#splashBannerGrad)"/>
    <image href="{img_path}" x="0" y="0" width="{width}" height="170" preserveAspectRatio="xMidYMid slice" opacity="0.85"/>
  </g>
  <rect y="152" width="{width}" height="18" fill="{bg}"/>

  <!-- Close / Dismiss button (top-right) -->
  <circle cx="492" cy="28" r="14" fill="rgba(0,0,0,0.4)"/>
  <text x="492" y="33" font-family="sans-serif" font-size="14" fill="#FFFFFF" text-anchor="middle" font-weight="bold">✕</text>

  <!-- Centered Authentic Logo Mark -->
  <circle cx="260" cy="85" r="52" fill="#FFFFFF" opacity="0.15"/>
  <image href="{logo_icon}" x="216" y="41" width="88" height="88" preserveAspectRatio="xMidYMid meet"/>

  <!-- Title & Version -->
  <text x="260" y="200" font-family="sans-serif, 'SolaimanLipi'" font-size="24" font-weight="bold" fill="{text_primary}" text-anchor="middle">{title}</text>
  <text x="260" y="222" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{green}" text-anchor="middle">{version}</text>

  <!-- Taglines -->
  <text x="260" y="250" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_secondary}" text-anchor="middle">{tagline}</text>
  <text x="260" y="270" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{red}" text-anchor="middle">{sub_tagline}</text>

  <!-- Initialization Progress Box (Ekushe / Bijoy style auto-hiding indicator) -->
  <g transform="translate(40, 292)">
    <rect width="440" height="72" rx="10" fill="{surface}" stroke="{border}" stroke-width="1"/>
    
    <text x="20" y="26" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{text_primary}">{loading}</text>
    <text x="420" y="26" font-family="sans-serif" font-size="11" fill="{green}" text-anchor="end" font-weight="bold">100%</text>

    <!-- Progress bar -->
    <rect x="20" y="36" width="400" height="8" rx="4" fill="{border}"/>
    <rect x="20" y="36" width="400" height="8" rx="4" fill="{green}"/>

    <!-- Auto-hide countdown hint -->
    <text x="20" y="58" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{text_secondary}">⏱️ {autoclose} {countdown_suffix}</text>
  </g>

  <!-- Publisher, Developer & Official Links -->
  <g transform="translate(40, 380)">
    <!-- Ruposhi Open Labs -->
    <rect width="440" height="36" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="14" y="23" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_primary}">🏢 {publisher}</text>
    <a href="{link_org}" target="_blank">
      <text x="426" y="23" font-family="sans-serif" font-size="11" fill="{green}" text-anchor="end" font-weight="bold">github.com/RuposhiOpenLabs</text>
    </a>

    <!-- Ryan Shelby Developer -->
    <rect y="44" width="440" height="36" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="14" y="67" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_primary}">👨‍💻 {developer}</text>
    <a href="{link_dev}" target="_blank">
      <text x="426" y="67" font-family="sans-serif" font-size="11" fill="{green}" text-anchor="end" font-weight="bold">github.com/MdSagorMunshi</text>
    </a>

    <!-- Support Email -->
    <rect y="88" width="440" height="36" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="14" y="111" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_primary}">✉️ {support_email_label}</text>
  </g>

  <!-- Footer security guarantee -->
  <text x="260" y="542" font-family="sans-serif, 'SolaimanLipi'" font-size="10" fill="{text_secondary}" text-anchor="middle">🛡️ 100% Offline • Zero Telemetry • Ad-Free • Ed25519 Verified</text>
</svg>"###,
            width = width,
            height = height,
            bg = pal.background,
            surface = pal.surface,
            border = pal.border,
            green = pal.primary_green,
            red = pal.accent_red,
            logo_icon = crate::branding_assets::LOGO_ICON_DATA_URI,
            text_primary = pal.text_primary,
            text_secondary = pal.text_secondary,
            img_path = img_path,
            countdown_suffix = countdown_suffix,
            title = Self::xml_escape(active_strings.app_title),
            version = Self::xml_escape(active_strings.about_version),
            tagline = Self::xml_escape(active_strings.tagline),
            sub_tagline = Self::xml_escape(active_strings.sub_tagline),
            loading = Self::xml_escape(active_strings.startup_loading),
            autoclose = Self::xml_escape(active_strings.startup_autoclose),
            publisher = Self::xml_escape(active_strings.about_publisher),
            developer = Self::xml_escape(active_strings.about_developer),
            link_org = Self::xml_escape(active_strings.link_org),
            link_dev = Self::xml_escape(active_strings.link_dev),
            support_email_label = Self::xml_escape(active_strings.support_email_label),
        )
    }

    fn render_installer_setup(state: &UiState, pal: &ThemePalette, strings: &crate::localization::Strings) -> String {
        let width = 680;
        let height = 560;

        let step_idx = match state.installer_step {
            crate::state::InstallerStep::Welcome => 1,
            crate::state::InstallerStep::Terms => 2,
            crate::state::InstallerStep::Components => 3,
            crate::state::InstallerStep::Installing => 4,
            crate::state::InstallerStep::Finished => 5,
        };

        let progress_width = (step_idx * 136).min(680);

        let step_body = match state.installer_step {
            crate::state::InstallerStep::Welcome => format!(
                r###"<!-- Step 1: Welcome -->
    <g transform="translate(48, 105)">
      <!-- Authentic Animated Bornomala Logo Mark -->
      <g class="pulse-glow" transform="translate(182, 0)">
        <image href="{logo_mark}" x="0" y="0" width="220" height="130" preserveAspectRatio="xMidYMid meet"/>
      </g>

      <text x="292" y="152" font-family="sans-serif, 'SolaimanLipi'" font-size="24" font-weight="bold" fill="{text_primary}" text-anchor="middle">{welcome_heading}</text>
      <text x="292" y="178" font-family="sans-serif, 'SolaimanLipi'" font-size="14" fill="{text_secondary}" text-anchor="middle">{welcome_sub}</text>

      <!-- Feature Pill Badges with lazy float animation -->
      <g transform="translate(16, 210)">
        <rect width="170" height="42" rx="8" fill="{surface}" stroke="{border}" stroke-width="1" class="float-badge"/>
        <text x="85" y="26" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_primary}" text-anchor="middle">⌨️ ফিক্সড ও ফোনেটিক</text>

        <rect x="185" width="180" height="42" rx="8" fill="{surface}" stroke="{border}" stroke-width="1" class="float-badge" style="animation-delay: 0.2s;"/>
        <text x="275" y="26" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_primary}" text-anchor="middle">🔤 ৮টি প্রিলোডেড ফন্ট</text>

        <rect x="380" width="174" height="42" rx="8" fill="{surface}" stroke="{border}" stroke-width="1" class="float-badge" style="animation-delay: 0.4s;"/>
        <text x="467" y="26" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_primary}" text-anchor="middle">🛡️ ১০০% অফলাইন ও নিরাপদ</text>
      </g>

      <!-- Actions -->
      <rect x="424" y="340" width="160" height="42" rx="8" fill="{green}"/>
      <text x="504" y="366" font-family="sans-serif, 'SolaimanLipi'" font-size="14" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{btn_next}</text>
    </g>"###,
                green = pal.primary_green,
                logo_mark = crate::branding_assets::LOGO_MARK_DATA_URI,
                surface = pal.surface,
                border = pal.border,
                text_primary = pal.text_primary,
                text_secondary = pal.text_secondary,
                welcome_heading = Self::xml_escape(strings.installer_welcome_heading),
                welcome_sub = Self::xml_escape(strings.installer_welcome_sub),
                btn_next = Self::xml_escape(strings.installer_btn_next),
            ),
            crate::state::InstallerStep::Terms => format!(
                r###"<!-- Step 2: Terms & Conditions -->
    <g transform="translate(48, 110)">
      <text x="0" y="20" font-family="sans-serif, 'SolaimanLipi'" font-size="18" font-weight="bold" fill="{text_primary}">{terms_heading}</text>

      <!-- Scrollable Terms Box Card -->
      <rect y="36" width="584" height="250" rx="10" fill="{surface}" stroke="{border}" stroke-width="1.5"/>
      <g transform="translate(18, 56)">
        <text x="0" y="16" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{green}">১. মাতৃভাষার স্বাধীনতার নীতি (Freedom of Mother Tongue):</text>
        <text x="0" y="36" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{text_primary}">   আপনার মাতৃভাষার জন্য কোনো লাইসেন্স কি-র প্রয়োজন নেই। বর্ণমালা চিরকাল ১০০% বিনামূল্যে উন্মুক্ত।</text>

        <text x="0" y="66" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{green}">২. ১০০% অফলাইন ও সম্পূর্ণ গোপনীয়তা (Privacy Guarantee):</text>
        <text x="0" y="86" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{text_primary}">   কোনো কি-লগিং নেই, কোনো ট্র্যাকিং নেই, কোনো ইন্টারনেটে ডেটা পাঠানো হয় না।</text>

        <text x="0" y="116" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{green}">৩. প্রিলোডেড ফন্ট নীতিমালা (Preloaded Fonts Notice):</text>
        <text x="0" y="136" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{text_primary}">   সরকারি নথির সুবিধার্থে ৮টি প্রমিত বাংলা ফন্ট (নিকশ, সুতোন্নীএমজে, আদর্শলিপি ইত্যাদি) সরবরাহ করা হয়েছে।</text>

        <text x="0" y="166" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{green}">৪. নিরাপত্তা ও লাইসেন্স (GPLv3 License):</text>
        <text x="0" y="186" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{text_primary}">   কোনো ক্র্যাক বা ভাইরাস নেই। ক্রিপ্টোগ্রাফিক Ed25519 স্বাক্ষর দ্বারা যাচাইকৃত।</text>
      </g>

      <!-- Agreement Checkbox -->
      <rect y="306" width="22" height="22" rx="6" fill="{green}"/>
      <text x="11" y="322" font-family="sans-serif" font-size="14" fill="#FFFFFF" text-anchor="middle" font-weight="bold">✓</text>
      <text x="32" y="322" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{text_primary}">{terms_agree}</text>

      <!-- Actions -->
      <rect y="350" width="130" height="40" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
      <text x="65" y="375" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}" text-anchor="middle">{btn_back}</text>

      <rect x="444" y="350" width="140" height="40" rx="8" fill="{green}"/>
      <text x="514" y="375" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{btn_next}</text>
    </g>"###,
                green = pal.primary_green,
                surface = pal.surface,
                border = pal.border,
                text_primary = pal.text_primary,
                terms_heading = Self::xml_escape(strings.installer_terms_heading),
                terms_agree = Self::xml_escape(strings.installer_terms_agree),
                btn_back = Self::xml_escape(strings.installer_btn_back),
                btn_next = Self::xml_escape(strings.installer_btn_next),
            ),
            crate::state::InstallerStep::Components => format!(
                r###"<!-- Step 3: Components & Preloaded Fonts -->
    <g transform="translate(48, 110)">
      <text x="0" y="20" font-family="sans-serif, 'SolaimanLipi'" font-size="18" font-weight="bold" fill="{text_primary}">ইনস্টলেশন কম্পোনেন্ট ও প্রিলোডেড ফন্ট নির্বাচন</text>

      <g transform="translate(0, 36)">
        <!-- Comp 1: TSF Input Method -->
        <rect width="584" height="50" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
        <rect x="16" y="15" width="20" height="20" rx="5" fill="{green}"/>
        <text x="26" y="30" font-family="sans-serif" font-size="13" fill="#FFFFFF" text-anchor="middle" font-weight="bold">✓</text>
        <text x="48" y="32" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{text_primary}">{comp_tsf}</text>

        <!-- Comp 2: Preloaded Fonts -->
        <rect y="58" width="584" height="66" rx="8" fill="{surface}" stroke="{border}" stroke-width="1.5"/>
        <rect x="16" y="73" width="20" height="20" rx="5" fill="{green}"/>
        <text x="26" y="88" font-family="sans-serif" font-size="13" fill="#FFFFFF" text-anchor="middle" font-weight="bold">✓</text>
        <text x="48" y="85" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{text_primary}">{comp_fonts}</text>
        <text x="48" y="106" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{green}">🔤 Nikosh, NikoshBAN, NikoshLight, SutonnyMJ Regular, AdorshoLipi, Bangla (৮টি ফন্ট)</text>

        <!-- Comp 3: Auto-Startup -->
        <rect y="132" width="584" height="50" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
        <rect x="16" y="147" width="20" height="20" rx="5" fill="{green}"/>
        <text x="26" y="162" font-family="sans-serif" font-size="13" fill="#FFFFFF" text-anchor="middle" font-weight="bold">✓</text>
        <text x="48" y="164" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{text_primary}">{comp_autostart}</text>

        <!-- Comp 4: Shortcuts -->
        <rect y="190" width="584" height="50" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
        <rect x="16" y="205" width="20" height="20" rx="5" fill="{green}"/>
        <text x="26" y="220" font-family="sans-serif" font-size="13" fill="#FFFFFF" text-anchor="middle" font-weight="bold">✓</text>
        <text x="48" y="222" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{text_primary}">{comp_shortcuts}</text>
      </g>

      <!-- Actions -->
      <rect y="350" width="130" height="40" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
      <text x="65" y="375" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}" text-anchor="middle">{btn_back}</text>

      <rect x="424" y="350" width="160" height="40" rx="8" fill="{green}"/>
      <text x="504" y="375" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{btn_install}</text>
    </g>"###,
                green = pal.primary_green,
                surface = pal.surface,
                border = pal.border,
                text_primary = pal.text_primary,
                comp_tsf = Self::xml_escape(strings.installer_comp_tsf),
                comp_fonts = Self::xml_escape(strings.installer_comp_fonts),
                comp_autostart = Self::xml_escape(strings.installer_comp_autostart),
                comp_shortcuts = Self::xml_escape(strings.installer_comp_shortcuts),
                btn_back = Self::xml_escape(strings.installer_btn_back),
                btn_install = Self::xml_escape(strings.installer_btn_install),
            ),
            crate::state::InstallerStep::Installing => format!(
                r###"<!-- Step 4: Installing with Animated Shimmer Progress -->
    <g transform="translate(48, 140)">
      <text x="0" y="20" font-family="sans-serif, 'SolaimanLipi'" font-size="18" font-weight="bold" fill="{text_primary}">{progress_title}</text>
      <text x="584" y="20" font-family="sans-serif" font-size="16" font-weight="bold" fill="{green}" text-anchor="end">{percent}%</text>

      <!-- Animated Progress Track & Bar with Shimmer -->
      <rect y="36" width="584" height="14" rx="7" fill="{border}"/>
      <rect y="36" width="{bar_fill}" height="14" rx="7" fill="{green}"/>
      
      <!-- Ticker Log Box -->
      <rect y="70" width="584" height="200" rx="10" fill="{surface}" stroke="{border}" stroke-width="1"/>
      <g transform="translate(20, 96)">
        <text x="0" y="16" font-family="monospace" font-size="12" fill="{green}">[1/4] Extracting TSF COM module: bornomala_tsf.dll ... OK</text>
        <text x="0" y="44" font-family="monospace" font-size="12" fill="{green}">[2/4] Preloading Bangla Fonts into Windows Font Library:</text>
        <text x="20" y="66" font-family="monospace" font-size="11" fill="{text_secondary}">- Nikosh.ttf, NikoshBAN.ttf, NikoshGrameen.ttf</text>
        <text x="20" y="86" font-family="monospace" font-size="11" fill="{text_secondary}">- SutonnyMJ Regular.ttf (Classic ANSI support)</text>
        <text x="20" y="106" font-family="monospace" font-size="11" fill="{text_secondary}">- AdorshoLipi.ttf, Bangla.ttf ... OK</text>
        <text x="0" y="132" font-family="monospace" font-size="12" fill="{green}">[3/4] Registering COM Server &amp; TSF Profile in HKCU ... OK</text>
        <text x="0" y="156" font-family="monospace" font-size="12" fill="{green}">[4/4] Configuring Windows Auto-Startup &amp; Splash Screen ... OK</text>
      </g>
    </g>"###,
                green = pal.primary_green,
                surface = pal.surface,
                border = pal.border,
                text_primary = pal.text_primary,
                text_secondary = pal.text_secondary,
                percent = state.installer_progress_percent,
                bar_fill = (state.installer_progress_percent * 584) / 100,
                progress_title = Self::xml_escape(strings.installer_progress_title),
            ),
            crate::state::InstallerStep::Finished => format!(
                r###"<!-- Step 5: Finished -->
    <g transform="translate(48, 120)">
      <!-- Animated Success Checkmark Ring -->
      <g class="pulse-glow" transform="translate(242, 10)">
        <circle cx="50" cy="50" r="48" fill="{green}" opacity="0.15"/>
        <circle cx="50" cy="50" r="38" fill="{green}"/>
        <text x="50" y="62" font-family="sans-serif" font-size="34" font-weight="bold" fill="#FFFFFF" text-anchor="middle">✓</text>
      </g>

      <text x="292" y="145" font-family="sans-serif, 'SolaimanLipi'" font-size="22" font-weight="bold" fill="{text_primary}" text-anchor="middle">{finished_heading}</text>
      <text x="292" y="172" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_secondary}" text-anchor="middle">{finished_desc}</text>

      <!-- Shortcut Reminder Cards -->
      <g transform="translate(32, 205)">
        <rect width="160" height="50" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
        <text x="80" y="24" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{text_secondary}" text-anchor="middle">ফোনেটিক বাংলা</text>
        <text x="80" y="42" font-family="sans-serif" font-size="13" font-weight="bold" fill="{green}" text-anchor="middle">F12</text>

        <rect x="180" width="160" height="50" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
        <text x="260" y="24" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{text_secondary}" text-anchor="middle">ক্লাসিক ইউনিকোড</text>
        <text x="260" y="42" font-family="sans-serif" font-size="13" font-weight="bold" fill="{green}" text-anchor="middle">Ctrl + Alt + V</text>

        <rect x="360" width="160" height="50" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
        <text x="440" y="24" font-family="sans-serif, 'SolaimanLipi'" font-size="11" fill="{text_secondary}" text-anchor="middle">সুতোন্নী এএনএসআই</text>
        <text x="440" y="42" font-family="sans-serif" font-size="13" font-weight="bold" fill="{green}" text-anchor="middle">Ctrl + Alt + B</text>
      </g>

      <!-- Actions -->
      <rect x="202" y="340" width="180" height="44" rx="8" fill="{green}"/>
      <text x="292" y="367" font-family="sans-serif, 'SolaimanLipi'" font-size="14" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{btn_finish}</text>
    </g>"###,
                green = pal.primary_green,
                surface = pal.surface,
                border = pal.border,
                text_primary = pal.text_primary,
                text_secondary = pal.text_secondary,
                finished_heading = Self::xml_escape(strings.installer_finished_heading),
                finished_desc = Self::xml_escape(strings.installer_finished_desc),
                btn_finish = Self::xml_escape(strings.installer_btn_finish),
            ),
        };

        // Font Conflict Modal Popup if triggered
        let conflict_modal = if let Some(ref conflict) = state.installer_font_conflict {
            format!(
                r###"<!-- Font Conflict Dialog Modal Popup -->
  <rect width="{width}" height="{height}" fill="rgba(0,0,0,0.65)" rx="16"/>
  <g transform="translate(100, 140)">
    <rect width="480" height="280" rx="14" fill="{bg}" stroke="{accent_red}" stroke-width="2" filter="url(#installerShadow)"/>
    
    <!-- Modal Header -->
    <rect width="480" height="48" rx="14" fill="{surface}"/>
    <text x="24" y="31" font-family="sans-serif, 'SolaimanLipi'" font-size="15" font-weight="bold" fill="{accent_red}">⚠️ {c_title}</text>
    
    <!-- Message -->
    <text x="24" y="80" font-family="sans-serif, 'SolaimanLipi'" font-size="13" fill="{text_primary}">{c_desc}</text>
    <rect x="24" y="96" width="432" height="38" rx="6" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="36" y="120" font-family="sans-serif, 'SolaimanLipi'" font-size="13" font-weight="bold" fill="{green}">🔤 ফন্ট: {c_font_name} ({c_filename})</text>

    <!-- Apply to all checkbox -->
    <rect x="24" y="152" width="16" height="16" rx="4" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="48" y="165" font-family="sans-serif, 'SolaimanLipi'" font-size="12" fill="{text_secondary}">{c_apply_all}</text>

    <!-- Choice Action Buttons -->
    <rect x="24" y="196" width="205" height="44" rx="8" fill="{surface}" stroke="{border}" stroke-width="1"/>
    <text x="126" y="223" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="{text_primary}" text-anchor="middle">{c_btn_keep}</text>

    <rect x="250" y="196" width="206" height="44" rx="8" fill="{green}"/>
    <text x="353" y="223" font-family="sans-serif, 'SolaimanLipi'" font-size="12" font-weight="bold" fill="#FFFFFF" text-anchor="middle">{c_btn_replace}</text>
  </g>"###,
                width = width,
                height = height,
                bg = pal.background,
                surface = pal.surface,
                border = pal.border,
                green = pal.primary_green,
                accent_red = pal.accessible_red,
                text_primary = pal.text_primary,
                text_secondary = pal.text_secondary,
                c_title = Self::xml_escape(strings.installer_font_conflict_title),
                c_desc = Self::xml_escape(strings.installer_font_conflict_desc),
                c_font_name = Self::xml_escape(&conflict.font_name),
                c_filename = Self::xml_escape(&conflict.filename),
                c_apply_all = Self::xml_escape(strings.installer_btn_apply_all),
                c_btn_keep = Self::xml_escape(strings.installer_btn_keep_font),
                c_btn_replace = Self::xml_escape(strings.installer_btn_replace_font),
            )
        } else {
            String::new()
        };

        format!(
            r###"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <defs>
    <filter id="installerShadow" x="-10%" y="-10%" width="120%" height="120%">
      <feDropShadow dx="0" dy="10" stdDeviation="20" flood-color="#000000" flood-opacity="0.3"/>
    </filter>
    <linearGradient id="installerHeaderGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="{green}"/>
      <stop offset="100%" stop-color="#093C20"/>
    </linearGradient>
    <style>
      @keyframes pulseGlow {{
        0% {{ transform: scale(1); opacity: 0.9; }}
        50% {{ transform: scale(1.05); opacity: 1; }}
        100% {{ transform: scale(1); opacity: 0.9; }}
      }}
      @keyframes floatBadge {{
        0%, 100% {{ transform: translateY(0px); }}
        50% {{ transform: translateY(-4px); }}
      }}
      .pulse-glow {{
        animation: pulseGlow 2.8s ease-in-out infinite;
        transform-origin: center;
      }}
      .float-badge {{
        animation: floatBadge 3.2s ease-in-out infinite;
      }}
    </style>
  </defs>

  <!-- Setup Window Frame -->
  <rect width="{width}" height="{height}" rx="16" fill="{bg}" stroke="{border}" stroke-width="2" filter="url(#installerShadow)"/>

  <!-- Acrylic Top Header -->
  <rect width="{width}" height="68" rx="16" fill="url(#installerHeaderGrad)"/>
  <rect y="52" width="{width}" height="16" fill="{bg}"/>

  <!-- Authentic Bornomala Logo and Wizard Title -->
  <g transform="translate(20, 12)">
    <image href="{logo_icon}" x="0" y="2" width="40" height="40" preserveAspectRatio="xMidYMid meet"/>
    <text x="48" y="27" font-family="sans-serif, 'SolaimanLipi'" font-size="17" font-weight="bold" fill="#FFFFFF">{installer_title}</text>
  </g>

  <!-- Step Track Indicator Bar -->
  <g transform="translate(0, 68)">
    <rect width="{width}" height="6" fill="{border}"/>
    <rect width="{progress_width}" height="6" fill="{green}"/>
  </g>

  <!-- Active Step Body -->
  {step_body}

  <!-- Font Conflict Dialog (if triggered) -->
  {conflict_modal}
</svg>"###,
            width = width,
            height = height,
            bg = pal.background,
            border = pal.border,
            green = pal.primary_green,
            logo_icon = crate::branding_assets::LOGO_ICON_DATA_URI,
            installer_title = Self::xml_escape(strings.installer_title),
            progress_width = progress_width,
            step_body = step_body,
            conflict_modal = conflict_modal,
        )
    }
}
