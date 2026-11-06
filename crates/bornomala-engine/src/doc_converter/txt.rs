//! Plain Text File (.txt) Converter
//! Supports UTF-8, UTF-8 BOM, and Windows-1252/ANSI text files.

use std::fs;
use std::path::Path;
use crate::converter::BanglaConverter;
use super::{ConversionDirection, DocConvertError};

pub fn convert_txt(
    source_path: &Path,
    output_path: &Path,
    direction: ConversionDirection,
) -> Result<(usize, usize, String), DocConvertError> {
    let raw_bytes = fs::read(source_path)?;

    // 1. Decode text content
    let input_text = decode_text(&raw_bytes)?;

    let converter = BanglaConverter::new();
    let (output_text, chars_count) = match direction {
        ConversionDirection::UnicodeToAnsi => {
            let (ansi, _) = converter.unicode_to_ansi(&input_text);
            let count = ansi.chars().count();
            (ansi, count)
        }
        ConversionDirection::AnsiToUnicode => {
            let unicode = converter.ansi_to_unicode(&input_text);
            let count = unicode.chars().count();
            (unicode, count)
        }
    };

    // 2. Write converted output
    // For Unicode, prepend UTF-8 BOM so legacy Windows applications (like old Notepad) detect UTF-8 cleanly
    let final_bytes = match direction {
        ConversionDirection::AnsiToUnicode => {
            let mut b = vec![0xEF, 0xBB, 0xBF];
            b.extend_from_slice(output_text.as_bytes());
            b
        }
        ConversionDirection::UnicodeToAnsi => {
            // Write as raw bytes or UTF-8 string containing ANSI glyph codes
            output_text.into_bytes()
        }
    };

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(output_path, final_bytes)?;

    let lines_count = input_text.lines().count();
    let msg = format!(
        "টেক্সট ফাইল সফলভাবে রূপান্তর সম্পন্ন: {} টি লাইন, {} টি অক্ষর।",
        lines_count, chars_count
    );

    Ok((chars_count, lines_count, msg))
}

fn decode_text(bytes: &[u8]) -> Result<String, DocConvertError> {
    // Strip UTF-8 BOM if present
    let bytes_to_check = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };

    // Attempt UTF-8 decode first
    if let Ok(s) = std::str::from_utf8(bytes_to_check) {
        return Ok(s.to_string());
    }

    // Fallback: Windows-1252 / ISO-8859-1 (interprets each byte as char)
    let s: String = bytes.iter().map(|&b| b as char).collect();
    Ok(s)
}
