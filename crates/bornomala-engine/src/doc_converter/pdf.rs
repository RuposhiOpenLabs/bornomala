//! PDF Document (.pdf) Converter
//!
//! Decodes PDF content streams, converts text strings in Tj and TJ operators,
//! and writes out the converted PDF while preserving graphics, images, and layout.

use std::path::Path;
use lopdf::content::Operation;
use lopdf::{Document, Object, StringFormat};

use crate::converter::BanglaConverter;
use super::{ConversionDirection, DocConvertError};

pub fn convert_pdf(
    source_path: &Path,
    output_path: &Path,
    direction: ConversionDirection,
) -> Result<(usize, usize, String), DocConvertError> {
    let mut doc = Document::load(source_path)
        .map_err(|e| DocConvertError::Pdf(format!("PDF লোড করতে ব্যর্থ: {}", e)))?;

    if doc.is_encrypted() {
        return Err(DocConvertError::Pdf(
            "PDF ফাইলটি পাসওয়ার্ড দ্বারা সুরক্ষিত (Encrypted)। অনুগ্রহ করে পাসওয়ার্ড অপসারণ করে আবার চেষ্টা করুন।".to_string(),
        ));
    }

    let pages = doc.get_pages();
    let page_count = pages.len();
    let converter = BanglaConverter::new();

    let mut total_chars = 0usize;
    let mut total_ops = 0usize;

    for (_page_num, page_id) in pages {
        // Retrieve and decode existing page content stream
        let mut content = match doc.get_and_decode_page_content(page_id) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for op in content.operations.iter_mut() {
            let (chars, changed) = transform_pdf_operation(op, &converter, direction);
            total_chars += chars;
            if changed {
                total_ops += 1;
            }
        }

        if let Ok(encoded_bytes) = content.encode() {
            let _ = doc.change_page_content(page_id, encoded_bytes);
        }
    }

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    doc.save(output_path)
        .map_err(|e| DocConvertError::Pdf(format!("রূপান্তরিত PDF সংরক্ষণ ব্যর্থ: {}", e)))?;

    let msg = format!(
        "PDF দলিল সফলভাবে রূপান্তর সম্পন্ন: {} টি পৃষ্ঠা, {} টি টেক্সট উপাদান, {} টি অক্ষর রূপান্তর করা হয়েছে।",
        page_count, total_ops, total_chars
    );

    Ok((total_chars, total_ops, msg))
}

fn transform_pdf_operation(
    op: &mut Operation,
    converter: &BanglaConverter,
    direction: ConversionDirection,
) -> (usize, bool) {
    let mut chars = 0usize;
    let mut modified = false;

    match op.operator.as_str() {
        // Tj: Show text string
        "Tj" | "'" => {
            if let Some(Object::String(ref mut bytes, ref mut format)) = op.operands.get_mut(0) {
                let (new_bytes, count, new_fmt) = convert_pdf_string(bytes, *format, converter, direction);
                if count > 0 {
                    *bytes = new_bytes;
                    *format = new_fmt;
                    chars += count;
                    modified = true;
                }
            }
        }
        // " : Set spacing, move to next line, and show text string
        "\"" => {
            if let Some(Object::String(ref mut bytes, ref mut format)) = op.operands.get_mut(2) {
                let (new_bytes, count, new_fmt) = convert_pdf_string(bytes, *format, converter, direction);
                if count > 0 {
                    *bytes = new_bytes;
                    *format = new_fmt;
                    chars += count;
                    modified = true;
                }
            }
        }
        // TJ: Show array of strings and spacing adjustments
        "TJ" => {
            if let Some(Object::Array(ref mut items)) = op.operands.get_mut(0) {
                for item in items.iter_mut() {
                    if let Object::String(ref mut bytes, ref mut format) = item {
                        let (new_bytes, count, new_fmt) = convert_pdf_string(bytes, *format, converter, direction);
                        if count > 0 {
                            *bytes = new_bytes;
                            *format = new_fmt;
                            chars += count;
                            modified = true;
                        }
                    }
                }
            }
        }
        _ => {}
    }

    (chars, modified)
}

fn convert_pdf_string(
    bytes: &[u8],
    format: StringFormat,
    converter: &BanglaConverter,
    direction: ConversionDirection,
) -> (Vec<u8>, usize, StringFormat) {
    // 1. Decode bytes into string
    let input_text = decode_pdf_bytes(bytes);
    if input_text.is_empty() {
        return (bytes.to_vec(), 0, format);
    }

    match direction {
        ConversionDirection::AnsiToUnicode => {
            let unicode = converter.ansi_to_unicode(&input_text);
            let count = unicode.chars().count();
            // Encode Unicode text as UTF-16BE with BOM for PDF standard compatibility
            let mut u16_bytes = vec![0xFE, 0xFF];
            for u in unicode.encode_utf16() {
                u16_bytes.push((u >> 8) as u8);
                u16_bytes.push((u & 0xFF) as u8);
            }
            (u16_bytes, count, StringFormat::Hexadecimal)
        }
        ConversionDirection::UnicodeToAnsi => {
            let (ansi, _) = converter.unicode_to_ansi(&input_text);
            let count = ansi.chars().count();
            (ansi.into_bytes(), count, StringFormat::Literal)
        }
    }
}

fn decode_pdf_bytes(bytes: &[u8]) -> String {
    // Check if UTF-16BE (PDF standard with FE FF BOM)
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let u16_vals: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| ((chunk[0] as u16) << 8) | (chunk[1] as u16))
            .collect();
        return String::from_utf16_lossy(&u16_vals);
    }

    // Try UTF-8
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }

    // Standard Latin1 / Windows-1252 byte-to-char mapping
    bytes.iter().map(|&b| b as char).collect()
}
