use std::io::{self, BufRead, Write};
use bornomala_engine::{
    BanglaConverter, BornomalaEngine, InputScheme, KeyEvent, TextEdit,
};

fn main() {
    println!("\x1b[2J\x1b[H"); // Clear screen
    println!("===============================================================================");
    println!("  Bornomala (বর্ণমালা) — Linux Terminal Typing Lab");
    println!("  \"Your mother tongue doesn't need a license key.\"");
    println!("===============================================================================");
    println!("  Modes: [1] Classic Unicode   [2] Classic ANSI   [3] Phonetic (Avro)");
    println!("  Commands: ':u' = Unicode, ':a' = ANSI, ':p' = Phonetic, ':c' = Clear, ':q' = Exit");
    println!("-------------------------------------------------------------------------------");

    let mut engine = BornomalaEngine::new(InputScheme::ClassicUnicode);
    let converter = BanglaConverter::new();
    let mut text_buffer = String::new();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    print_prompt(engine.active_scheme());
    let _ = stdout.flush();

    for line_res in stdin.lock().lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed == ":q" || trimmed == "quit" || trimmed == "exit" {
            println!("Exiting Bornomala Terminal Lab. Goodbye!");
            break;
        } else if trimmed == ":c" || trimmed == "clear" {
            text_buffer.clear();
            engine.reset_all_states();
            println!("\x1b[2J\x1b[H");
            println!("--- Buffer Cleared ---");
            print_prompt(engine.active_scheme());
            let _ = stdout.flush();
            continue;
        } else if trimmed == ":u" || trimmed == "1" {
            engine.set_scheme(InputScheme::ClassicUnicode);
            println!("[Switched to Classic Unicode Mode]");
            print_prompt(engine.active_scheme());
            let _ = stdout.flush();
            continue;
        } else if trimmed == ":a" || trimmed == "2" {
            engine.set_scheme(InputScheme::ClassicAnsi);
            println!("[Switched to Classic ANSI (SutonnyMJ) Mode]");
            print_prompt(engine.active_scheme());
            let _ = stdout.flush();
            continue;
        } else if trimmed == ":p" || trimmed == "3" {
            engine.set_scheme(InputScheme::Phonetic);
            println!("[Switched to Phonetic Mode]");
            print_prompt(engine.active_scheme());
            let _ = stdout.flush();
            continue;
        }

        // Process line characters through active engine
        for ch in line.chars() {
            let key_code = ch as u32;
            let shift = ch.is_uppercase();
            let ev = KeyEvent::new_down(key_code, Some(ch), shift, false, false);
            let action = engine.process_key(&ev);

            for edit in action.edits {
                match edit {
                    TextEdit::Insert(s) => text_buffer.push_str(&s),
                    TextEdit::ReplaceLast { count, text } => {
                        for _ in 0..count {
                            text_buffer.pop();
                        }
                        text_buffer.push_str(&text);
                    }
                    TextEdit::DeleteBack(count) => {
                        for _ in 0..count {
                            text_buffer.pop();
                        }
                    }
                    TextEdit::PassThrough => text_buffer.push(ch),
                }
            }
        }

        // In phonetic mode, commit end of line with newline
        if engine.active_scheme() == InputScheme::Phonetic {
            let space_ev = KeyEvent::new_down(32, Some(' '), false, false, false);
            let act = engine.process_key(&space_ev);
            for edit in act.edits {
                if let TextEdit::Insert(s) = edit {
                    if s != " " {
                        text_buffer.push_str(&s);
                    }
                }
            }
        }

        // Render side-by-side analysis
        println!();
        println!("+-----------------------------------------------------------------------------+");
        println!("| BUFFER CONTENT (Current Output):                                            |");
        println!("| {:<75} |", text_buffer);
        println!("+-----------------------------------------------------------------------------+");

        let (unicode_view, ansi_view) = match engine.active_scheme() {
            InputScheme::ClassicUnicode | InputScheme::Phonetic => {
                let u = text_buffer.clone();
                let (a, _) = converter.unicode_to_ansi(&u);
                (u, a)
            }
            InputScheme::ClassicAnsi => {
                let a = text_buffer.clone();
                let u = converter.ansi_to_unicode(&a);
                (u, a)
            }
            InputScheme::PassThrough => (text_buffer.clone(), text_buffer.clone()),
        };

        println!("  [Unicode Bangla View]:    {}", unicode_view);
        print!("  [Unicode Codepoints]:     ");
        for c in unicode_view.chars().take(20) {
            print!("U+{:04X} ", c as u32);
        }
        if unicode_view.chars().count() > 20 {
            print!("...");
        }
        println!();

        println!("  [SutonnyMJ ANSI View]:    {}", ansi_view);
        print!("  [ANSI Hex Bytes]:         ");
        for b in ansi_view.bytes().take(20) {
            print!("{:02X} ", b);
        }
        if ansi_view.len() > 20 {
            print!("...");
        }
        println!();
        println!("-------------------------------------------------------------------------------");

        print_prompt(engine.active_scheme());
        let _ = stdout.flush();
    }
}

fn print_prompt(scheme: InputScheme) {
    let mode_badge = match scheme {
        InputScheme::ClassicUnicode => "\x1b[32m[Unicode]\x1b[0m",
        InputScheme::ClassicAnsi => "\x1b[33m[Classic ANSI]\x1b[0m",
        InputScheme::Phonetic => "\x1b[36m[Phonetic]\x1b[0m",
        InputScheme::PassThrough => "\x1b[37m[PassThrough]\x1b[0m",
    };
    print!("{} type keys > ", mode_badge);
}
