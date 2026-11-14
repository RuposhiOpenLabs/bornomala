use bornomala_engine::conveniences::fonts::{
    generate_powershell_font_installer, get_preloaded_fonts, FontInstallScope,
};
use bornomala_engine::{
    amount_to_bangla_words, amount_to_english_words, convert_digits, gregorian_to_bangla,
    AutostartManager, BanglaConverter, BornomalaEngine, InputScheme, KeyEvent, TextEdit,
    DEFAULT_SPLASH_DURATION_MS, REGISTRY_RUN_KEY, REGISTRY_VALUE_NAME,
};
use bornomala_ui::{ActiveView, FontConflictModal, InstallerStep, Language, SvgRenderer, UiState};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;

fn print_usage() {
    println!(
        r#"Bornomala (বর্ণমালা) CLI — Universal Bangla Text & Engine Tool
"Your mother tongue doesn't need a license key."
Ruposhi Open Labs | Developer: Ryan Shelby

USAGE:
    bornomala-cli <COMMAND> [OPTIONS]

COMMANDS:
    setup     Launch or preview the custom animated Windows Install Setup UI/UX
              Usage: bornomala-cli setup [--html [OUT.html]] [--step <1-5>] [--conflict]

    fonts     Manage and inspect preloaded Bangla fonts (Nikosh, SutonnyMJ, etc.)
              Usage: bornomala-cli fonts [list | script [--ps1|--bat]]

    splash    Display the animated startup splash screen (Bijoy/Ekushe style)
              Usage: bornomala-cli splash [--no-wait] [--duration <ms>] [--english]

    autostart Manage Windows auto-startup configuration and registry scripts
              Usage: bornomala-cli autostart [enable [PATH] | disable | status | script]

    about     Display publisher, developer credits, official links, and contact
              Usage: bornomala-cli about

    convert   Convert text between Unicode and legacy SutonnyMJ ANSI
              Options: --to-ansi <TEXT|FILE>
                       --to-unicode <TEXT|FILE>

    inspect   Inspect character codes and hex Unicode codepoints
              Usage: bornomala-cli inspect <TEXT>

    type      Run a simulated keystroke script through an input scheme
              Usage: bornomala-cli type --scheme <unicode|ansi|phonetic> <KEYS>

    amount    Convert numerical amount into formal Bangla and English words
              Usage: bornomala-cli amount <NUMBER>

    date      Show today's date or convert date to Revised Bangla Calendar
              Usage: bornomala-cli date [YYYY-MM-DD]

    digits    Toggle digits between English (0-9) and Bangla (০-৯)
              Usage: bornomala-cli digits [--to-bangla | --to-english] <TEXT>

    help      Show this help information
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "setup" => handle_setup(&args[2..]),
        "fonts" => handle_fonts(&args[2..]),
        "splash" => handle_splash(&args[2..]),
        "autostart" => handle_autostart(&args[2..]),
        "about" => handle_about(),
        "convert" => handle_convert(&args[2..]),
        "inspect" => handle_inspect(&args[2..]),
        "type" => handle_type(&args[2..]),
        "amount" => handle_amount(&args[2..]),
        "date" => handle_date(&args[2..]),
        "digits" => handle_digits(&args[2..]),
        "help" | "--help" | "-h" => print_usage(),
        other => {
            eprintln!("Unknown command: {}", other);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn get_input_text(param: Option<&String>) -> String {
    if let Some(arg) = param {
        if let Ok(content) = fs::read_to_string(arg) {
            content
        } else {
            arg.clone()
        }
    } else {
        let mut buffer = String::new();
        let _ = io::stdin().read_to_string(&mut buffer);
        buffer
    }
}

fn handle_convert(args: &[String]) {
    use bornomala_engine::doc_converter::{convert_file, ConversionDirection, DocumentFormat};
    use std::path::Path;

    let converter = BanglaConverter::new();
    if args.is_empty() {
        eprintln!("Error: specify --to-ansi, --to-unicode, or --file <DOCX|DOC|PDF|TXT>");
        return;
    }

    if args[0] == "--file" {
        if args.len() < 2 {
            eprintln!("Usage: bornomala-cli convert --file <PATH> [--to-unicode | --to-ansi] [--out <OUT>]");
            return;
        }
        let src = Path::new(&args[1]);
        let mut direction = ConversionDirection::AnsiToUnicode;
        let mut out_path = None;

        let mut i = 2;
        while i < args.len() {
            if args[i] == "--to-ansi" {
                direction = ConversionDirection::UnicodeToAnsi;
            } else if args[i] == "--to-unicode" {
                direction = ConversionDirection::AnsiToUnicode;
            } else if args[i] == "--out" && i + 1 < args.len() {
                out_path = Some(Path::new(&args[i + 1]));
                i += 1;
            }
            i += 1;
        }

        match convert_file(src, out_path, direction) {
            Ok(rep) => {
                println!("✓ Conversion Succeeded!");
                println!("Source:     {}", rep.source_path.display());
                println!("Output:     {}", rep.output_path.display());
                println!("Format:     {:?}", rep.format);
                println!("Direction:  {}", rep.direction.description());
                println!("Elements:   {} items processed", rep.items_processed);
                println!(
                    "Characters: {} characters converted",
                    rep.characters_converted
                );
                println!("Time:       {} ms", rep.duration_ms);
            }
            Err(e) => {
                eprintln!("✕ Document conversion error: {}", e);
            }
        }
        return;
    }

    match args[0].as_str() {
        "--to-ansi" => {
            let input_arg = args.get(1);
            if let Some(path_str) = input_arg {
                let p = Path::new(path_str);
                if p.exists()
                    && matches!(
                        DocumentFormat::from_path(p),
                        DocumentFormat::Docx
                            | DocumentFormat::Doc
                            | DocumentFormat::Pdf
                            | DocumentFormat::Txt
                    )
                {
                    match convert_file(p, None, ConversionDirection::UnicodeToAnsi) {
                        Ok(rep) => {
                            println!(
                                "✓ Document converted to ANSI: {}",
                                rep.output_path.display()
                            );
                            return;
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }

            let input = get_input_text(input_arg);
            let (ansi, unrep) = converter.unicode_to_ansi(&input);
            print!("{}", ansi);
            if !unrep.is_empty() {
                eprintln!(
                    "\n[Warning] Unrepresentable characters in ANSI: {:?}",
                    unrep
                );
            }
        }
        "--to-unicode" => {
            let input_arg = args.get(1);
            if let Some(path_str) = input_arg {
                let p = Path::new(path_str);
                if p.exists()
                    && matches!(
                        DocumentFormat::from_path(p),
                        DocumentFormat::Docx
                            | DocumentFormat::Doc
                            | DocumentFormat::Pdf
                            | DocumentFormat::Txt
                    )
                {
                    match convert_file(p, None, ConversionDirection::AnsiToUnicode) {
                        Ok(rep) => {
                            println!(
                                "✓ Document converted to Unicode: {}",
                                rep.output_path.display()
                            );
                            return;
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }

            let input = get_input_text(input_arg);
            let uni = converter.ansi_to_unicode(&input);
            print!("{}", uni);
        }
        other => {
            eprintln!("Unknown conversion flag: {}", other);
        }
    }
}

fn handle_inspect(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: please provide text to inspect");
        return;
    }
    let text = &args[0];
    println!("Text: {}", text);
    println!("Length: {} characters", text.chars().count());
    println!("{:-<50}", "");
    println!(
        "{:<6} {:<10} {:<15} {}",
        "Index", "Char", "Codepoint", "Hex Bytes (UTF-8)"
    );
    println!("{:-<50}", "");
    for (i, c) in text.chars().enumerate() {
        let mut b = [0u8; 4];
        let bytes = c.encode_utf8(&mut b).as_bytes();
        let hex_bytes = bytes
            .iter()
            .map(|x| format!("{:02X}", x))
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "{:<6} {:<10} U+{:04X}         {}",
            i, c, c as u32, hex_bytes
        );
    }
}

fn handle_type(args: &[String]) {
    if args.len() < 3 || args[0] != "--scheme" {
        eprintln!("Usage: bornomala-cli type --scheme <unicode|ansi|phonetic> <KEYS>");
        return;
    }

    let scheme = match args[1].to_lowercase().as_str() {
        "unicode" => InputScheme::ClassicUnicode,
        "ansi" => InputScheme::ClassicAnsi,
        "phonetic" => InputScheme::Phonetic,
        other => {
            eprintln!(
                "Invalid scheme: {}. Choose 'unicode', 'ansi', or 'phonetic'",
                other
            );
            return;
        }
    };

    let script = &args[2];
    let mut engine = BornomalaEngine::new(scheme);
    let mut output = String::new();

    for ch in script.chars() {
        let key_code = ch as u32;
        let shift = ch.is_uppercase();
        let ev = KeyEvent::new_down(key_code, Some(ch), shift, false, false);
        let action = engine.process_key(&ev);

        for edit in action.edits {
            match edit {
                TextEdit::Insert(s) => output.push_str(&s),
                TextEdit::ReplaceLast { count, text } => {
                    for _ in 0..count {
                        output.pop();
                    }
                    output.push_str(&text);
                }
                TextEdit::DeleteBack(count) => {
                    for _ in 0..count {
                        output.pop();
                    }
                }
                TextEdit::PassThrough => output.push(ch),
            }
        }
    }

    // If phonetic has pending composition, commit with space
    if scheme == InputScheme::Phonetic {
        let space_ev = KeyEvent::new_down(32, Some(' '), false, false, false);
        let act = engine.process_key(&space_ev);
        for edit in act.edits {
            if let TextEdit::Insert(s) = edit {
                if s != " " {
                    output.push_str(&s);
                }
            }
        }
    }

    println!("Input Script: {}", script);
    println!("Output Text:  {}", output);
    print!("Codepoints:   ");
    for c in output.chars() {
        print!("U+{:04X} ", c as u32);
    }
    println!();
}

fn handle_amount(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: bornomala-cli amount <NUMBER>");
        return;
    }
    let num_str = &args[0];
    let cleaned = num_str.replace(',', "");
    match cleaned.parse::<f64>() {
        Ok(val) => {
            println!("Amount: {}", val);
            println!("Bangla:  {}", amount_to_bangla_words(val));
            println!("English: {}", amount_to_english_words(val));
        }
        Err(_) => {
            eprintln!("Invalid numeric amount: {}", num_str);
        }
    }
}

fn handle_date(args: &[String]) {
    let (y, m, d) = if args.is_empty() {
        // Current date or default
        (2026, 11, 3)
    } else {
        let parts: Vec<&str> = args[0].split('-').collect();
        if parts.len() == 3 {
            let y = parts[0].parse::<i32>().unwrap_or(2026);
            let m = parts[1].parse::<u32>().unwrap_or(11);
            let d = parts[2].parse::<u32>().unwrap_or(3);
            (y, m, d)
        } else {
            eprintln!("Invalid date format. Use YYYY-MM-DD");
            return;
        }
    };

    let b_date = gregorian_to_bangla(y, m, d);
    println!("Gregorian Date: {:04}-{:02}-{:02}", y, m, d);
    println!("Bangla Date:    {}", b_date.format_bangla());
    println!("Month / Year:   {} {}", b_date.month_name, b_date.year);
}

fn handle_digits(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: bornomala-cli digits [--to-bangla | --to-english] <TEXT>");
        return;
    }

    let (to_bangla, text) = if args[0] == "--to-bangla" {
        (true, get_input_text(args.get(1)))
    } else if args[0] == "--to-english" {
        (false, get_input_text(args.get(1)))
    } else {
        (true, get_input_text(Some(&args[0])))
    };

    println!("{}", convert_digits(&text, to_bangla));
}

fn handle_splash(args: &[String]) {
    let mut duration_ms = DEFAULT_SPLASH_DURATION_MS;
    let mut no_wait = false;
    let mut english_mode = false;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--no-wait" {
            no_wait = true;
        } else if args[i] == "--english" || args[i] == "--en" {
            english_mode = true;
        } else if args[i] == "--duration" && i + 1 < args.len() {
            if let Ok(d) = args[i + 1].parse::<u64>() {
                duration_ms = d;
            }
            i += 1;
        }
        i += 1;
    }

    let splash_asset = if english_mode {
        "assets/branding/startup_screen_en.png"
    } else {
        "assets/branding/startup_screen_bn.png"
    };

    println!("\x1b[2J\x1b[1;1H");
    println!(
        "\x1b[1;32m======================================================================\x1b[0m"
    );
    println!(
        "\x1b[1;32m      ____                                               _            \x1b[0m"
    );
    println!(
        "\x1b[1;32m     |  _ \\                                             | |           \x1b[0m"
    );
    println!(
        "\x1b[1;32m     | |_) | ___  _ __ _ __   ___  _ __ ___   __ _  ___| | __ _      \x1b[0m"
    );
    println!("\x1b[1;31m     |  _ < / _ \\| '__| '_ \\ / _ \\| '_ ` _ \\ / _` |/ __| |/ _` |     \x1b[0m");
    println!(
        "\x1b[1;31m     | |_) | (_) | |  | | | | (_) | | | | | | (_| | (__| | (_| |     \x1b[0m"
    );
    println!("\x1b[1;32m     |____/ \\___/|_|  |_| |_|\\___/|_| |_| |_|\\__,_|\\___|_|\\__,_|     \x1b[0m");
    println!(
        "\x1b[1;32m                         ব  র্ণ  মা  লা                                \x1b[0m"
    );
    println!(
        "\x1b[1;32m======================================================================\x1b[0m"
    );
    if english_mode {
        println!("\x1b[1;37m Bornomala — Free & Offline Bangla Typing Suite v0.1.0\x1b[0m");
        println!("\x1b[0;36m \"Your mother tongue doesn't need a license key.\"\x1b[0m");
        println!("\x1b[1;31m No crack. No virus. No invoice.\x1b[0m");
        println!("----------------------------------------------------------------------");
        println!(" 🎨 Splash Graphic: {}\x1b[0m", splash_asset);
        println!(" 🏢 Publisher:      Ruposhi Open Labs  \x1b[1;34m(https://github.com/RuposhiOpenLabs)\x1b[0m");
        println!(" 👨‍💻 Developer:      Ryan Shelby        \x1b[1;34m(https://github.com/MdSagorMunshi/)\x1b[0m");
        println!(" ✉️  Support:        \x1b[1;33mryn@disr.it\x1b[0m");
        println!(" 🛡️  Integrity:      100% Offline | Zero Telemetry | Ed25519 Signed");
        println!("----------------------------------------------------------------------");
        print!(" Initializing typing engine & layout tables... ");
    } else {
        println!("\x1b[1;37m বর্ণমালা (Bornomala) — স্টার্টআপ স্প্ল্যাশ স্ক্রিন (বাংলা সংস্করণ)\x1b[0m");
        println!("\x1b[0;36m \"আপনার মাতৃভাষার জন্য কোনো লাইসেন্স কি-র প্রয়োজন নেই।\"\x1b[0m");
        println!("\x1b[1;31m কোনো ক্র্যাক নেই। কোনো ভাইরাস নেই। কোনো ইনভয়েস নেই।\x1b[0m");
        println!("----------------------------------------------------------------------");
        println!(" 🎨 স্প্ল্যাশ গ্রাফিক: {}\x1b[0m", splash_asset);
        println!(" 🏢 প্রকাশক:       রূপসী ওপেন ল্যাবস  \x1b[1;34m(https://github.com/RuposhiOpenLabs)\x1b[0m");
        println!(" 👨‍💻 ডেভেলপার:      রায়ান শেলবি        \x1b[1;34m(https://github.com/MdSagorMunshi/)\x1b[0m");
        println!(" ✉️  সাপোর্ট:        \x1b[1;33mryn@disr.it\x1b[0m");
        println!(" 🛡️  নিরাপত্তা:      ১০০% অফলাইন • কোনো টেলিমეტ্রি নেই • Ed25519 দ্বারা সুরক্ষিত");
        println!("----------------------------------------------------------------------");
        print!(" কিবোর্ড ইঞ্জিন ও লেআউট টেবিল লোড হচ্ছে... ");
    }
    let _ = io::stdout().flush();

    if !no_wait {
        let step = (duration_ms / 20).max(10);
        print!("[");
        for _ in 0..20 {
            print!("#");
            let _ = io::stdout().flush();
            thread::sleep(Duration::from_millis(step));
        }
        if english_mode {
            println!("] Ready!");
            println!(
                "\n Auto-hiding startup splash screen (2.5s). Background text service running..."
            );
        } else {
            println!("] প্রস্তুত!");
            println!(
                "\n ২.৫ সেকেন্ডের মধ্যে স্ক্রিনটি স্বয়ংক্রিয়ভাবে লুকিয়ে যাবে। ব্যাকগ্রাউন্ড টেক্সট সার্ভিস সক্রিয়..."
            );
        }
    } else {
        if english_mode {
            println!("[####################] Ready (immediate)!");
        } else {
            println!("[####################] প্রস্তুত (তাৎক্ষণিক)!");
        }
    }
    println!(
        "\x1b[1;32m======================================================================\x1b[0m"
    );
}

fn handle_autostart(args: &[String]) {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("status");
    let mgr = if let Some(path_arg) = args.get(1) {
        AutostartManager::new(path_arg)
    } else {
        AutostartManager::default()
    };

    match sub {
        "enable" => {
            println!("Windows Registry Auto-Startup Registration:");
            println!("Registry Key:   {}", REGISTRY_RUN_KEY);
            println!("Value Name:     {}", REGISTRY_VALUE_NAME);
            println!("Target Path:    {}", mgr.exe_path.display());
            println!("\nRun the following command on Windows (cmd/PowerShell):");
            println!("  {}", mgr.generate_enable_command());
        }
        "disable" => {
            println!("Windows Registry Auto-Startup De-registration:");
            println!("Run the following command on Windows (cmd/PowerShell):");
            println!("  {}", mgr.generate_disable_command());
        }
        "status" => {
            println!("Windows Auto-Startup Registry Query:");
            println!("Registry Key:   {}", REGISTRY_RUN_KEY);
            println!("Value Name:     {}", REGISTRY_VALUE_NAME);
            println!("Query Command:  {}", mgr.generate_query_command());
        }
        "script" => {
            println!("--- scripts/enable_autostart.bat ---");
            println!(
                "{}",
                bornomala_engine::conveniences::autostart::generate_enable_bat_script(None)
            );
            println!("--- scripts/disable_autostart.bat ---");
            println!(
                "{}",
                bornomala_engine::conveniences::autostart::generate_disable_bat_script()
            );
        }
        other => {
            eprintln!("Unknown autostart subcommand: {}", other);
            eprintln!("Usage: bornomala-cli autostart [enable [PATH] | disable | status | script]");
        }
    }
}

fn handle_about() {
    println!(
        r#"======================================================================
Bornomala (বর্ণমালা) — Free, Open Source Bangla Typing Suite
"Your mother tongue doesn't need a license key."
======================================================================
Version:    0.1.0 (v0.1.0)
Publisher:  Ruposhi Open Labs
            GitHub:  https://github.com/RuposhiOpenLabs

Developer:  Ryan Shelby
            GitHub:  https://github.com/MdSagorMunshi/

Support:    ryn@disr.it
License:    GNU General Public License v3.0 (GPLv3)

Design Principles:
  • 100% Offline: Zero internet required for typing, conversion, or layout switching.
  • Zero Telemetry: Never tracks keystrokes, applications, or user habits.
  • Zero Ads & Free: No cracked copies, no viruses, no trial limits, no payment invoices.
  • Cryptographically Verified: Ed25519 signature checks on updates.
  • High-Reliability TSF Architecture: Built in Rust with panic-safe pass-through fail-safes.
======================================================================"#
    );
}

fn handle_fonts(args: &[String]) {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
    let fonts = get_preloaded_fonts();

    match sub {
        "list" => {
            println!("==========================================================================================");
            println!(" Bornomala (বর্ণমালা) — Preloaded Government & Classic Bangla Fonts (৮টি ফন্ট)");
            println!("==========================================================================================");
            println!(
                "{:<4} {:<28} {:<18} {:<10} {}",
                "No.", "Font Name", "Type", "Scope", "Description / বিবরণ"
            );
            println!("{:-<95}", "");
            for (idx, font) in fonts.iter().enumerate() {
                let enc = if font.is_unicode && font.is_ansi {
                    "Uni + ANSI"
                } else if font.is_unicode {
                    "Unicode (বিসিসি)"
                } else {
                    "Classic ANSI"
                };
                println!(
                    "{:<4} {:<28} {:<18} {:<10} {}",
                    idx + 1,
                    font.filename,
                    enc,
                    "Preloaded",
                    font.description_bn
                );
            }
            println!("{:-<95}", "");
            println!(" Total: 8 preloaded fonts included in installer & fonts/ directory.");
            println!(
                " Auto-installed by setup wizard with conflict popup detection if already present."
            );
            println!("==========================================================================================");
        }
        "script" => {
            let use_bat = args.iter().any(|a| a == "--bat");
            if use_bat {
                if let Ok(content) = fs::read_to_string("scripts/install_fonts.bat") {
                    println!("{}", content);
                } else {
                    eprintln!("scripts/install_fonts.bat not found");
                }
            } else {
                let script =
                    generate_powershell_font_installer(r"\..onts", FontInstallScope::CurrentUser);
                println!("{}", script);
            }
        }
        other => {
            eprintln!("Unknown fonts subcommand: {}", other);
            eprintln!("Usage: bornomala-cli fonts [list | script [--ps1|--bat]]");
        }
    }
}

fn handle_setup(args: &[String]) {
    use bornomala_ui::ThemeMode;

    let mut step = InstallerStep::Welcome;
    let mut show_conflict = false;
    let mut html_out: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--step" => {
                if i + 1 < args.len() {
                    match args[i + 1].as_str() {
                        "1" | "welcome" => step = InstallerStep::Welcome,
                        "2" | "terms" => step = InstallerStep::Terms,
                        "3" | "components" => step = InstallerStep::Components,
                        "4" | "installing" => step = InstallerStep::Installing,
                        "5" | "finished" => step = InstallerStep::Finished,
                        _ => {}
                    }
                    i += 1;
                }
            }
            "--conflict" => show_conflict = true,
            "--html" => {
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    html_out = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    html_out = Some("assets/installer_preview.html".to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }

    println!("======================================================================");
    println!(" Bornomala (বর্ণমালা) — Animated Windows Install Setup UI/UX");
    println!("======================================================================");
    println!(" Active Step:     {:?}", step);
    println!(
        " Conflict Modal:  {}",
        if show_conflict {
            "Visible (SutonnyMJ conflict test)"
        } else {
            "None"
        }
    );
    println!(" Preloaded Fonts: 8 Fonts Ready (Nikosh, SutonnyMJ, AdorshoLipi, Bangla)");
    println!(" Smooth Lazy Animations: Pulse Glow, Float Badges, Shimmer Progress");
    println!("----------------------------------------------------------------------");

    if let Some(path) = html_out {
        let step_defs = [
            ("welcome", InstallerStep::Welcome, false),
            ("terms", InstallerStep::Terms, false),
            ("components", InstallerStep::Components, false),
            ("installing", InstallerStep::Installing, false),
            ("finished", InstallerStep::Finished, false),
            ("conflict", InstallerStep::Components, true),
        ];

        let mut svg_json_items = Vec::new();
        for (name, st, conf) in step_defs {
            let mut s_light = UiState::default();
            s_light.active_view = ActiveView::InstallerSetup;
            s_light.installer_step = st;
            s_light.installer_progress_percent = 78;
            s_light.language = Language::Bangla;
            s_light.theme = ThemeMode::Light;
            if conf {
                s_light.installer_font_conflict = Some(FontConflictModal {
                    font_name: "SutonnyMJ Regular.ttf".to_string(),
                    filename: "SutonnyMJ Regular.ttf".to_string(),
                    apply_to_all: false,
                });
            }
            let light_svg = SvgRenderer::render_view(&s_light);

            let mut s_dark = s_light.clone();
            s_dark.theme = ThemeMode::Dark;
            let dark_svg = SvgRenderer::render_view(&s_dark);

            let light_escaped = serde_json::to_string(&light_svg).unwrap();
            let dark_escaped = serde_json::to_string(&dark_svg).unwrap();
            svg_json_items.push(format!(
                "  \"{}\": {{ light: {}, dark: {} }}",
                name, light_escaped, dark_escaped
            ));
        }

        let json_body = svg_json_items.join(",\n");

        let html = format!(
            r###"<!DOCTYPE html>
<html lang="bn">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Bornomala (বর্ণমালা) — Animated Windows Install Setup</title>
  <style>
    :root {{
      --bg: #0d1117;
      --card-bg: rgba(22, 27, 34, 0.85);
      --accent-green: #006A4E;
      --accent-red: #F42A41;
      --text: #e6edf3;
      --border: #30363d;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "SolaimanLipi", "Kalpurush", sans-serif;
      background: radial-gradient(circle at 50% 15%, #183e30 0%, #0a1712 65%, #050a08 100%);
      color: var(--text);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 30px 20px;
    }}
    header {{
      text-align: center;
      margin-bottom: 24px;
    }}
    h1 {{
      font-size: 28px;
      font-weight: 700;
      color: #ffffff;
      margin-bottom: 6px;
      text-shadow: 0 2px 10px rgba(0,0,0,0.5);
    }}
    .tagline {{
      font-size: 14px;
      color: #8b949e;
    }}
    .badge-bar {{
      display: flex;
      gap: 12px;
      justify-content: center;
      margin-top: 12px;
      flex-wrap: wrap;
    }}
    .badge {{
      background: rgba(0, 106, 78, 0.25);
      border: 1px solid #006A4E;
      padding: 4px 12px;
      border-radius: 20px;
      font-size: 12px;
      color: #4cdbb3;
    }}
    .controls-wrapper {{
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 12px;
      margin-bottom: 24px;
      max-width: 820px;
      width: 100%;
    }}
    .btn-group {{
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
      justify-content: center;
    }}
    button {{
      background: #21262d;
      border: 1px solid var(--border);
      color: var(--text);
      padding: 8px 16px;
      border-radius: 8px;
      cursor: pointer;
      font-size: 13px;
      font-weight: 600;
      transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
    }}
    button:hover {{
      background: #30363d;
      transform: translateY(-2px);
      box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    }}
    button.active {{
      background: var(--accent-green);
      border-color: #108e68;
      color: #fff;
      box-shadow: 0 0 16px rgba(0, 106, 78, 0.6);
    }}
    .stage {{
      background: var(--card-bg);
      backdrop-filter: blur(16px);
      -webkit-backdrop-filter: blur(16px);
      border: 1px solid rgba(255,255,255,0.14);
      border-radius: 22px;
      padding: 24px;
      box-shadow: 0 25px 60px rgba(0, 0, 0, 0.7);
      display: flex;
      flex-direction: column;
      align-items: center;
      max-width: 740px;
      width: 100%;
      position: relative;
      transition: opacity 0.3s ease;
    }}
    .svg-container {{
      width: 100%;
      display: flex;
      justify-content: center;
      filter: drop-shadow(0 15px 35px rgba(0,0,0,0.6));
      transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.3s ease;
    }}
    .svg-container svg {{
      max-width: 100%;
      height: auto;
      border-radius: 16px;
    }}
    .toast {{
      position: fixed;
      bottom: 24px;
      background: #1f6feb;
      color: white;
      padding: 10px 20px;
      border-radius: 8px;
      font-size: 13px;
      font-weight: 600;
      opacity: 0;
      transform: translateY(20px);
      transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
      pointer-events: none;
      z-index: 1000;
    }}
    .toast.show {{
      opacity: 1;
      transform: translateY(0);
    }}
    footer {{
      margin-top: 30px;
      font-size: 12px;
      color: #8b949e;
      text-align: center;
      line-height: 1.6;
    }}
    footer a {{ color: #58a6ff; text-decoration: none; }}
  </style>
</head>
<body>
  <header>
    <h1>বর্ণমালা ইনস্টলেশন ও সেটআপ উইজার্ড (Bornomala Windows Setup)</h1>
    <div class="tagline">Custom Animated UI/UX • Smooth Lazy Animations • 8 Preloaded Fonts • EULA & Privacy Guarantee</div>
    <div class="badge-bar">
      <div class="badge">🛡️ ১০০% অফলাইন ও নিরাপদ</div>
      <div class="badge">🔤 ৮টি প্রিলোডেড ফন্ট (নিকশ ও সুতোন্নী)</div>
      <div class="badge">⚖️ মজিলা পাবলিক লাইসেন্স ২.০</div>
      <div class="badge">🔑 কোনো কি-লগিং নেই</div>
    </div>
  </header>

  <div class="controls-wrapper">
    <div class="btn-group">
      <button id="btn-welcome" class="active" onclick="switchStep('welcome')">১. স্বাগতম</button>
      <button id="btn-terms" onclick="switchStep('terms')">২. ব্যবহারের নীতিমালা</button>
      <button id="btn-components" onclick="switchStep('components')">৩. উপাদান ও ফন্ট</button>
      <button id="btn-installing" onclick="switchStep('installing')">৪. ইনস্টল হচ্ছে</button>
      <button id="btn-finished" onclick="switchStep('finished')">৫. সম্পন্ন</button>
      <button id="btn-conflict" style="border-color: #F42A41; color: #ff7b72;" onclick="switchStep('conflict')">⚠️ ফন্ট সংঘাত পপআপ</button>
    </div>

    <div class="btn-group" style="margin-top: 6px;">
      <button id="theme-light" class="active" onclick="setTheme('light')">☀️ Light Theme</button>
      <button id="theme-dark" onclick="setTheme('dark')">🌙 Dark Theme</button>
    </div>
  </div>

  <div class="stage">
    <div class="svg-container" id="viewport"></div>
  </div>

  <div class="toast" id="toast">Status Updated</div>

  <footer>
    <p>Bornomala Typing Suite v0.1.0 • Ruposhi Open Labs (<a href="https://github.com/RuposhiOpenLabs" target="_blank">github.com/RuposhiOpenLabs</a>)</p>
    <p>Lead Developer: Ryan Shelby (<a href="https://github.com/MdSagorMunshi/" target="_blank">github.com/MdSagorMunshi</a>) • Support: ryn@disr.it</p>
  </footer>

  <script>
    const svgData = {{
{json_body}
    }};

    let currentStep = 'welcome';
    let currentTheme = 'light';

    function render() {{
      const container = document.getElementById('viewport');
      container.style.opacity = '0';
      container.style.transform = 'translateY(8px) scale(0.99)';

      setTimeout(() => {{
        const item = svgData[currentStep];
        container.innerHTML = currentTheme === 'light' ? item.light : item.dark;
        container.style.opacity = '1';
        container.style.transform = 'translateY(0px) scale(1)';
      }}, 120);

      document.querySelectorAll('.btn-group button').forEach(b => {{
        if (b.id === 'btn-' + currentStep) b.classList.add('active');
        else if (b.id.startsWith('btn-')) b.classList.remove('active');
      }});

      document.getElementById('theme-light').classList.toggle('active', currentTheme === 'light');
      document.getElementById('theme-dark').classList.toggle('active', currentTheme === 'dark');
    }}

    function switchStep(step) {{
      currentStep = step;
      render();
      if (step === 'conflict') {{
        showToast('ফন্ট সংঘাত শনাক্ত: SutonnyMJ Regular.ttf — Keep or Replace choice');
      }}
    }}

    function setTheme(theme) {{
      currentTheme = theme;
      render();
      showToast('Theme switched to ' + theme);
    }}

    function showToast(msg) {{
      const t = document.getElementById('toast');
      t.innerText = msg;
      t.classList.add('show');
      setTimeout(() => t.classList.remove('show'), 2800);
    }}

    render();
  </script>
</body>
</html>"###,
            json_body = json_body
        );

        if let Err(e) = fs::write(&path, html) {
            eprintln!("Failed to write HTML preview to {}: {}", path, e);
        } else {
            println!(
                " Saved interactive animated HTML setup preview to: {}",
                path
            );
        }
    } else {
        println!(" (Tip: use '--html preview.html' to generate an interactive browser preview)");
    }
    println!("======================================================================");
}
