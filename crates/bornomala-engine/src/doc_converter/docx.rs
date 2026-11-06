//! Word Document (.docx) High-Fidelity Layout-Preserving Converter
//!
//! Processes OpenXML packaging, converting text runs in document, header, footer,
//! and footnotes, while switching font metadata between SutonnyMJ and Kalpurush/Nirmala UI.
//! All tables, images, margins, styles, and page layouts are preserved 100%.

use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::converter::BanglaConverter;
use super::{ConversionDirection, DocConvertError};

pub fn convert_docx(
    source_path: &Path,
    output_path: &Path,
    direction: ConversionDirection,
) -> Result<(usize, usize, String), DocConvertError> {
    let src_file = File::open(source_path)?;
    let mut archive = ZipArchive::new(src_file)
        .map_err(|e| DocConvertError::Docx(format!("ZIP আনপ্যাক করতে ব্যর্থ: {}", e)))?;

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let out_file = File::create(output_path)?;
    let mut zip_writer = ZipWriter::new(out_file);

    let converter = BanglaConverter::new();
    let mut total_chars = 0usize;
    let mut total_runs = 0usize;

    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| DocConvertError::Docx(format!("ZIP এন্ট্রি পড়তে সমস্যা: {}", e)))?;
        let name = entry.name().to_string();

        if is_target_xml(&name) {
            let mut xml_content = String::new();
            entry.read_to_string(&mut xml_content)
                .map_err(|e| DocConvertError::Docx(format!("XML পাঠে ব্যর্থতা: {}", e)))?;

            let (converted_xml, chars, runs) = transform_word_xml(&xml_content, &converter, direction)?;
            total_chars += chars;
            total_runs += runs;

            zip_writer.start_file(&name, options)
                .map_err(|e| DocConvertError::Docx(format!("ZIP ফাইল তৈরি ব্যর্থ: {}", e)))?;
            zip_writer.write_all(converted_xml.as_bytes())?;
        } else if name == "word/fontTable.xml" {
            let mut xml_content = String::new();
            entry.read_to_string(&mut xml_content)
                .map_err(|e| DocConvertError::Docx(format!("fontTable পাঠে ব্যর্থতা: {}", e)))?;

            let updated_font_table = update_font_table(&xml_content, direction);
            zip_writer.start_file(&name, options)
                .map_err(|e| DocConvertError::Docx(format!("fontTable সংরক্ষণে ব্যর্থ: {}", e)))?;
            zip_writer.write_all(updated_font_table.as_bytes())?;
        } else {
            // Copy binary assets, images, styles, rels untouched
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;

            zip_writer.start_file(&name, options)
                .map_err(|e| DocConvertError::Docx(format!("ফাইল স্থানান্তর ব্যর্থ: {}", e)))?;
            zip_writer.write_all(&buffer)?;
        }
    }

    zip_writer.finish()
        .map_err(|e| DocConvertError::Docx(format!("নতুন DOCX প্যাকেজ সম্পন্ন করতে ব্যর্থ: {}", e)))?;

    let msg = format!(
        "Word (.docx) নথি সফলভাবে রূপান্তর সম্পন্ন: {} টি টেক্সট রান, {} টি অক্ষর রূপান্তর করা হয়েছে।",
        total_runs, total_chars
    );

    Ok((total_chars, total_runs, msg))
}

fn is_target_xml(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == "word/document.xml"
        || (lower.starts_with("word/header") && lower.ends_with(".xml"))
        || (lower.starts_with("word/footer") && lower.ends_with(".xml"))
        || lower == "word/footnotes.xml"
        || lower == "word/endnotes.xml"
        || lower == "word/comments.xml"
}

fn transform_word_xml(
    xml: &str,
    converter: &BanglaConverter,
    direction: ConversionDirection,
) -> Result<(String, usize, usize), DocConvertError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut in_text_node = false;
    let mut chars_count = 0usize;
    let mut runs_count = 0usize;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,

            // Handle start tag <w:t> or <w:t xml:space="preserve">
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"w:t" || name.as_ref().ends_with(b":t") {
                    in_text_node = true;
                }
                writer.write_event(Event::Start(e.clone()))
                    .map_err(|err| DocConvertError::Docx(err.to_string()))?;
            }

            // Handle end tag </w:t>
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"w:t" || name.as_ref().ends_with(b":t") {
                    in_text_node = false;
                }
                writer.write_event(Event::End(e.clone()))
                    .map_err(|err| DocConvertError::Docx(err.to_string()))?;
            }

            // Handle empty tags like <w:rFonts ... />
            Ok(Event::Empty(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"w:rFonts" || name.as_ref().ends_with(b":rFonts") {
                    let modified_fonts = transform_font_tag(e, direction);
                    writer.write_event(Event::Empty(modified_fonts))
                        .map_err(|err| DocConvertError::Docx(err.to_string()))?;
                } else {
                    writer.write_event(Event::Empty(e.clone()))
                        .map_err(|err| DocConvertError::Docx(err.to_string()))?;
                }
            }

            // Handle text node content
            Ok(Event::Text(ref e)) => {
                if in_text_node {
                    let raw_text = e.unescape()
                        .map_err(|err| DocConvertError::Docx(err.to_string()))?;
                    let (converted, count) = match direction {
                        ConversionDirection::UnicodeToAnsi => {
                            if contains_bangla_unicode(&raw_text) {
                                let (ansi, _) = converter.unicode_to_ansi(&raw_text);
                                let c = ansi.chars().count();
                                (ansi, c)
                            } else {
                                (raw_text.to_string(), 0)
                            }
                        }
                        ConversionDirection::AnsiToUnicode => {
                            let unicode = converter.ansi_to_unicode(&raw_text);
                            let c = unicode.chars().count();
                            (unicode, c)
                        }
                    };

                    chars_count += count;
                    runs_count += 1;

                    writer.write_event(Event::Text(BytesText::new(&converted)))
                        .map_err(|err| DocConvertError::Docx(err.to_string()))?;
                } else {
                    writer.write_event(Event::Text(e.clone()))
                        .map_err(|err| DocConvertError::Docx(err.to_string()))?;
                }
            }

            Ok(other) => {
                writer.write_event(other)
                    .map_err(|err| DocConvertError::Docx(err.to_string()))?;
            }

            Err(e) => return Err(DocConvertError::Docx(format!("XML পার্সিং ত্রুটি: {:?}", e))),
        }
        buf.clear();
    }

    let result_bytes = writer.into_inner().into_inner();
    let result_str = String::from_utf8(result_bytes)
        .map_err(|e| DocConvertError::Docx(format!("UTF-8 রূপান্তর ব্যর্থ: {}", e)))?;

    Ok((result_str, chars_count, runs_count))
}

fn transform_font_tag<'a>(e: &'a BytesStart<'a>, direction: ConversionDirection) -> BytesStart<'static> {
    let mut new_elem = BytesStart::new(String::from_utf8_lossy(e.name().as_ref()).to_string());

    let target_font = match direction {
        ConversionDirection::AnsiToUnicode => "Kalpurush",
        ConversionDirection::UnicodeToAnsi => "SutonnyMJ",
    };

    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();

        let new_val = if key.ends_with("ascii") || key.ends_with("hAnsi") || key.ends_with("cs") {
            match direction {
                ConversionDirection::AnsiToUnicode => {
                    if is_bijoy_font_name(&val) {
                        target_font.to_string()
                    } else {
                        val
                    }
                }
                ConversionDirection::UnicodeToAnsi => {
                    if is_unicode_font_name(&val) {
                        target_font.to_string()
                    } else {
                        val
                    }
                }
            }
        } else {
            val
        };

        new_elem.push_attribute((key.as_str(), new_val.as_str()));
    }

    new_elem
}

fn update_font_table(xml: &str, direction: ConversionDirection) -> String {
    let (find_font, add_font) = match direction {
        ConversionDirection::AnsiToUnicode => ("SutonnyMJ", "Kalpurush"),
        ConversionDirection::UnicodeToAnsi => ("Kalpurush", "SutonnyMJ"),
    };

    if !xml.contains(add_font) && xml.contains(find_font) {
        let snippet = format!(
            r#"<w:font w:name="{}"><w:charset w:val="00"/><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>"#,
            add_font
        );
        xml.replace("</w:fonts>", &format!("{}</w:fonts>", snippet))
    } else {
        xml.to_string()
    }
}

fn contains_bangla_unicode(s: &str) -> bool {
    s.chars().any(|c| ('\u{0980}'..='\u{09FF}').contains(&c))
}

fn is_bijoy_font_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("sutonny")
        || lower.contains("sutonnymj")
        || lower.contains("adorsholipi")
        || lower.contains("bangla")
        || lower.contains("sutonny_bangla")
}

fn is_unicode_font_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("kalpurush")
        || lower.contains("solaiman")
        || lower.contains("siyam")
        || lower.contains("nikosh")
        || lower.contains("vrinda")
        || lower.contains("nirmala")
        || lower.contains("bengali")
}
