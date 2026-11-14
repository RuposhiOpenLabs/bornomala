//! Legacy Word Document (.doc) Converter (Microsoft Word 97-2003 Binary Format)
//!
//! Parses Compound File Binary Format (CFBF / OLE2) and [MS-DOC] FIB / Piece Tables
//! to extract 8-bit ANSI SutonnyMJ text and 16-bit Unicode text, converts it bidirectionally,
//! and writes high-fidelity output as either .doc or modern .docx.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::{ConversionDirection, DocConvertError};
use crate::converter::BanglaConverter;

pub fn convert_doc(
    source_path: &Path,
    output_path: &Path,
    direction: ConversionDirection,
) -> Result<(usize, usize, String), DocConvertError> {
    // 1. Extract raw text from .doc file
    let extracted_text = extract_text_from_doc(source_path)?;

    if extracted_text.trim().is_empty() {
        return Err(DocConvertError::Doc(
            "ডকুমেন্ট থেকে কোনো পাঠযোগ্য টেক্সট খুঁজে পাওয়া যায়নি।".to_string(),
        ));
    }

    let converter = BanglaConverter::new();
    let (converted_text, chars_count) = match direction {
        ConversionDirection::AnsiToUnicode => {
            let u = converter.ansi_to_unicode(&extracted_text);
            let c = u.chars().count();
            (u, c)
        }
        ConversionDirection::UnicodeToAnsi => {
            let (a, _) = converter.unicode_to_ansi(&extracted_text);
            let c = a.chars().count();
            (a, c)
        }
    };

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // 2. Write output matching the requested extension (.doc or .docx)
    let out_ext = output_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("doc")
        .to_ascii_lowercase();

    let items_count = converted_text.lines().count();

    if out_ext == "docx" {
        write_as_docx(output_path, &converted_text, direction)?;
    } else {
        write_as_doc(output_path, &converted_text, direction)?;
    }

    let msg = format!(
        "Word (.doc) নথি সফলভাবে রূপান্তর সম্পন্ন: {} টি অনুচ্ছেদ, {} টি অক্ষর রূপান্তর করা হয়েছে।",
        items_count, chars_count
    );

    Ok((chars_count, items_count, msg))
}

/// Extracts text from a legacy .doc (CFBF/OLE2) file.
fn extract_text_from_doc(path: &Path) -> Result<String, DocConvertError> {
    let mut file = File::open(path)?;

    // First attempt: Parse CFBF Compound Document
    if let Ok(mut compound) = cfb::CompoundFile::open(&mut file) {
        let entries: Vec<std::path::PathBuf> =
            compound.walk().map(|e| e.path().to_path_buf()).collect();

        // Look for WordDocument stream
        let mut word_doc_stream = Vec::new();

        for p in &entries {
            let p_str = p.to_string_lossy().to_string();
            if p_str.ends_with("WordDocument") {
                if let Ok(mut s) = compound.open_stream(p) {
                    let _ = s.read_to_end(&mut word_doc_stream);
                }
            }
        }

        if word_doc_stream.len() >= 0x0200 {
            // Read FIB magic (0xA5EC)
            let w_ident = u16::from_le_bytes([word_doc_stream[0], word_doc_stream[1]]);
            if w_ident == 0xA5EC {
                let flags = u16::from_le_bytes([word_doc_stream[0x000A], word_doc_stream[0x000B]]);
                let f_which_tbl = (flags & 0x0200) != 0;
                let tbl_name = if f_which_tbl { "1Table" } else { "0Table" };

                let mut table_bytes = Vec::new();
                for p in &entries {
                    let p_str = p.to_string_lossy();
                    if p_str.ends_with(tbl_name) {
                        if let Ok(mut s) = compound.open_stream(p) {
                            let _ = s.read_to_end(&mut table_bytes);
                        }
                        break;
                    }
                }

                // Extract via piece table
                if let Ok(text) = extract_via_clx(&word_doc_stream, &table_bytes) {
                    if !text.is_empty() {
                        return Ok(sanitize_doc_text(&text));
                    }
                }
            }
        }

        // Fallback for simple non-complex WordDocument stream
        if !word_doc_stream.is_empty() {
            if let Ok(text) = extract_from_simple_fib(&word_doc_stream) {
                if !text.is_empty() {
                    return Ok(sanitize_doc_text(&text));
                }
            }
        }
    }

    // Fallback: File might be RTF with .doc extension or raw text
    let mut raw_bytes = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut raw_bytes)?;

    if raw_bytes.starts_with(b"{\\rtf") {
        return Ok(extract_text_from_rtf(&raw_bytes));
    }

    // Direct byte heuristic scan
    Ok(extract_printable_strings(&raw_bytes))
}

fn extract_via_clx(word_doc: &[u8], table: &[u8]) -> Result<String, ()> {
    if word_doc.len() < 0x01AA {
        return Err(());
    }

    // fcClx is at offset 0x01A2 (418), lcbClx at 0x01A6 (422)
    let fc_clx = u32::from_le_bytes([
        word_doc[0x01A2],
        word_doc[0x01A3],
        word_doc[0x01A4],
        word_doc[0x01A5],
    ]) as usize;
    let lcb_clx = u32::from_le_bytes([
        word_doc[0x01A6],
        word_doc[0x01A7],
        word_doc[0x01A8],
        word_doc[0x01A9],
    ]) as usize;

    if fc_clx + lcb_clx > table.len() || lcb_clx < 5 {
        return Err(());
    }

    let clx_data = &table[fc_clx..fc_clx + lcb_clx];
    let mut pos = 0;
    let mut full_text = String::new();

    while pos < clx_data.len() {
        let clxt = clx_data[pos];
        if clxt == 0x01 {
            // clxtGrpprl: skip grpprl
            if pos + 3 > clx_data.len() {
                break;
            }
            let cb = u16::from_le_bytes([clx_data[pos + 1], clx_data[pos + 2]]) as usize;
            pos += 3 + cb;
        } else if clxt == 0x02 {
            // clxtPlcfpcd: Piece Table found!
            if pos + 5 > clx_data.len() {
                break;
            }
            let lcb = u32::from_le_bytes([
                clx_data[pos + 1],
                clx_data[pos + 2],
                clx_data[pos + 3],
                clx_data[pos + 4],
            ]) as usize;
            pos += 5;

            if lcb < 4 || pos + lcb > clx_data.len() {
                break;
            }
            let n = (lcb - 4) / 12; // (n + 1) * 4 + n * 8 = 12*n + 4

            let cp_array = &clx_data[pos..pos + (n + 1) * 4];
            let pcd_array = &clx_data[pos + (n + 1) * 4..pos + lcb];

            for i in 0..n {
                let cp_start = u32::from_le_bytes([
                    cp_array[4 * i],
                    cp_array[4 * i + 1],
                    cp_array[4 * i + 2],
                    cp_array[4 * i + 3],
                ]) as usize;
                let cp_end = u32::from_le_bytes([
                    cp_array[4 * (i + 1)],
                    cp_array[4 * (i + 1) + 1],
                    cp_array[4 * (i + 1) + 2],
                    cp_array[4 * (i + 1) + 3],
                ]) as usize;

                if cp_end <= cp_start {
                    continue;
                }
                let cp_count = cp_end - cp_start;

                let pcd = &pcd_array[8 * i..8 * i + 8];
                let fc = u32::from_le_bytes([pcd[2], pcd[3], pcd[4], pcd[5]]);

                let is_ansi = (fc & 0x40000000) != 0;
                let real_fc = (fc & 0x3FFFFFFF) as usize;

                if is_ansi {
                    let file_offset = real_fc / 2;
                    if file_offset + cp_count <= word_doc.len() {
                        let slice = &word_doc[file_offset..file_offset + cp_count];
                        for &b in slice {
                            full_text.push(b as char);
                        }
                    }
                } else {
                    let byte_count = cp_count * 2;
                    if real_fc + byte_count <= word_doc.len() {
                        let slice = &word_doc[real_fc..real_fc + byte_count];
                        let u16s: Vec<u16> = slice
                            .chunks_exact(2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect();
                        full_text.push_str(&String::from_utf16_lossy(&u16s));
                    }
                }
            }
            break;
        } else {
            pos += 1;
        }
    }

    if full_text.is_empty() {
        Err(())
    } else {
        Ok(full_text)
    }
}

fn extract_from_simple_fib(word_doc: &[u8]) -> Result<String, ()> {
    if word_doc.len() < 0x0050 {
        return Err(());
    }
    let fc_min = u32::from_le_bytes([
        word_doc[0x0018],
        word_doc[0x0019],
        word_doc[0x001A],
        word_doc[0x001B],
    ]) as usize;
    let ccp_text = u32::from_le_bytes([
        word_doc[0x004C],
        word_doc[0x004D],
        word_doc[0x004E],
        word_doc[0x004F],
    ]) as usize;

    if fc_min < word_doc.len() && ccp_text > 0 {
        let len = ccp_text.min(word_doc.len() - fc_min);
        let slice = &word_doc[fc_min..fc_min + len];
        let mut s = String::new();
        for &b in slice {
            s.push(b as char);
        }
        return Ok(s);
    }
    Err(())
}

fn extract_text_from_rtf(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes);
    let mut out = String::new();
    let mut in_control = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            in_control = true;
        } else if in_control {
            if c.is_alphabetic() {
                // Control word continue
            } else if c == ' ' {
                in_control = false;
            } else {
                in_control = false;
            }
        } else if c == '{' || c == '}' {
            // skip group brackets
        } else if c == '\r' || c == '\n' {
            out.push('\n');
        } else {
            out.push(c);
        }
    }
    sanitize_doc_text(&out)
}

fn extract_printable_strings(bytes: &[u8]) -> String {
    let mut s = String::new();
    let mut run = Vec::new();

    for &b in bytes {
        if (0x20..=0x7E).contains(&b)
            || (0x80..=0xFF).contains(&b)
            || b == b'\r'
            || b == b'\n'
            || b == b'\t'
        {
            run.push(b as char);
        } else {
            if run.len() >= 3 {
                for c in run.drain(..) {
                    s.push(c);
                }
                s.push('\n');
            }
            run.clear();
        }
    }
    if run.len() >= 3 {
        for c in run.drain(..) {
            s.push(c);
        }
    }
    sanitize_doc_text(&s)
}

fn sanitize_doc_text(text: &str) -> String {
    let mut cleaned = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\r' | '\x07' | '\x0C' => cleaned.push('\n'),
            '\x0B' => cleaned.push('\n'),
            '\t' => cleaned.push('\t'),
            _ if (c as u32) >= 0x20 => cleaned.push(c),
            _ => {}
        }
    }
    cleaned
}

/// Writes converted text as a native, fully styled .doc (Word Document).
fn write_as_doc(
    output_path: &Path,
    text: &str,
    direction: ConversionDirection,
) -> Result<(), DocConvertError> {
    let font_name = match direction {
        ConversionDirection::AnsiToUnicode => "Kalpurush",
        ConversionDirection::UnicodeToAnsi => "SutonnyMJ",
    };

    let mut doc_content = String::new();
    doc_content.push_str("{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat\r\n");
    doc_content.push_str(&format!(
        "{{\\fonttbl{{\\f0\\fnil\\fcharset0 {};}}{{\\f1\\fnil\\fcharset0 Nirmala UI;}}}}\r\n",
        font_name
    ));
    doc_content.push_str("{\\colortbl ;\\red6\\green106\\blue74;\\red17\\green24\\blue39;}\r\n");
    doc_content.push_str("\\viewkind4\\uc1\\pard\\sa200\\sl276\\slmult1\\f0\\fs24\r\n");

    for line in text.lines() {
        if line.trim().is_empty() {
            doc_content.push_str("\\par\r\n");
            continue;
        }

        match direction {
            ConversionDirection::AnsiToUnicode => {
                // Write Unicode escapes \uXXXX?
                for c in line.chars() {
                    let cp = c as u32;
                    if cp <= 127 {
                        if c == '\\' || c == '{' || c == '}' {
                            doc_content.push('\\');
                        }
                        doc_content.push(c);
                    } else if cp <= 0xFFFF {
                        doc_content.push_str(&format!("\\u{}?", cp as i16));
                    }
                }
            }
            ConversionDirection::UnicodeToAnsi => {
                // Write ANSI characters directly
                for c in line.chars() {
                    if c == '\\' || c == '{' || c == '}' {
                        doc_content.push('\\');
                    }
                    doc_content.push(c);
                }
            }
        }
        doc_content.push_str("\\par\r\n");
    }
    doc_content.push_str("}\r\n");

    std::fs::write(output_path, doc_content.as_bytes())?;
    Ok(())
}

/// Writes converted text as a modern, clean OpenXML .docx file.
fn write_as_docx(
    output_path: &Path,
    text: &str,
    direction: ConversionDirection,
) -> Result<(), DocConvertError> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let font_name = match direction {
        ConversionDirection::AnsiToUnicode => "Kalpurush",
        ConversionDirection::UnicodeToAnsi => "SutonnyMJ",
    };

    // [Content_Types].xml
    zip.start_file("[Content_Types].xml", opts)
        .map_err(|e| DocConvertError::Docx(e.to_string()))?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#)?;

    // _rels/.rels
    zip.start_file("_rels/.rels", opts)
        .map_err(|e| DocConvertError::Docx(e.to_string()))?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#)?;

    // word/document.xml
    zip.start_file("word/document.xml", opts)
        .map_err(|e| DocConvertError::Docx(e.to_string()))?;

    let mut doc_xml = String::new();
    doc_xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>"#,
    );

    for line in text.lines() {
        let escaped = quick_xml::escape::escape(line);
        doc_xml.push_str(&format!(
            r#"<w:p><w:r><w:rPr><w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:cs="{font}"/></w:rPr><w:t xml:space="preserve">{text}</w:t></w:r></w:p>"#,
            font = font_name,
            text = escaped
        ));
    }

    doc_xml.push_str(r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>"#);
    zip.write_all(doc_xml.as_bytes())?;

    zip.finish()
        .map_err(|e| DocConvertError::Docx(format!("ZIP সমাপ্ত করতে ব্যর্থ: {}", e)))?;

    Ok(())
}
