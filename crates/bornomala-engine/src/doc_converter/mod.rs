//! Bornomala Universal Document & File Converter
//!
//! Provides high-fidelity, layout-preserving bidirectional conversion
//! between Bijoy/SutonnyMJ ANSI and Unicode Bangla for:
//! - DOCX (Microsoft Word 2007-2024 / Office 365)
//! - DOC (Microsoft Word 97-2003 Binary / CFBF)
//! - PDF (Portable Document Format)
//! - TXT (Plain Text / UTF-8 / Windows-1252)

pub mod doc;
pub mod docx;
pub mod pdf;
pub mod txt;

use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionDirection {
    UnicodeToAnsi,
    AnsiToUnicode,
}

impl ConversionDirection {
    pub fn description(&self) -> &'static str {
        match self {
            Self::UnicodeToAnsi => "ইউনিকোড ➔ বিজয় ANSI (SutonnyMJ)",
            Self::AnsiToUnicode => "বিজয় ANSI (SutonnyMJ) ➔ ইউনিকোড",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    Docx,
    Doc,
    Pdf,
    Txt,
    Unknown,
}

impl DocumentFormat {
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()) {
            Some(ref ext) if ext == "docx" => DocumentFormat::Docx,
            Some(ref ext) if ext == "doc" => DocumentFormat::Doc,
            Some(ref ext) if ext == "pdf" => DocumentFormat::Pdf,
            Some(ref ext) if ext == "txt" => DocumentFormat::Txt,
            _ => DocumentFormat::Unknown,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DocumentFormat::Docx => "Word নথি (.docx)",
            DocumentFormat::Doc => "Word নথি ৯৭-২০০৩ (.doc)",
            DocumentFormat::Pdf => "PDF দলিল (.pdf)",
            DocumentFormat::Txt => "টেক্সট ফাইল (.txt)",
            DocumentFormat::Unknown => "অজানা ফরম্যাট",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversionReport {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub format: DocumentFormat,
    pub direction: ConversionDirection,
    pub characters_converted: usize,
    pub items_processed: usize,
    pub duration_ms: u128,
    pub message: String,
}

#[derive(Error, Debug)]
pub enum DocConvertError {
    #[error("ফাইল ত্রুটি (File I/O Error): {0}")]
    Io(#[from] std::io::Error),

    #[error("অসমর্থিত ফাইল ফরম্যাট: শুধুমাত্র .docx, .doc, .pdf এবং .txt ফাইল সমর্থিত।")]
    UnsupportedFormat(String),

    #[error("Word (DOCX) রূপান্তর ত্রুটি: {0}")]
    Docx(String),

    #[error("Word (DOC) রূপান্তর ত্রুটি: {0}")]
    Doc(String),

    #[error("PDF রূপান্তর ত্রুটি: {0}")]
    Pdf(String),

    #[error("টেক্সট এনকোডিং ত্রুটি: {0}")]
    Encoding(String),
}

/// Generates a default output file path with "_converted" suffix.
pub fn default_output_path(source: &Path, direction: ConversionDirection) -> PathBuf {
    let parent = source.parent().unwrap_or_else(|| Path::new(""));
    let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("document");
    let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("txt");

    let suffix = match direction {
        ConversionDirection::UnicodeToAnsi => "_ansi",
        ConversionDirection::AnsiToUnicode => "_unicode",
    };

    let new_name = format!("{}{}.{}", stem, suffix, ext);
    parent.join(new_name)
}

/// Universal file converter entry point.
pub fn convert_file(
    source_path: &Path,
    output_path: Option<&Path>,
    direction: ConversionDirection,
) -> Result<ConversionReport, DocConvertError> {
    if !source_path.exists() {
        return Err(DocConvertError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("ফাইলটি খুঁজে পাওয়া যায়নি: {}", source_path.display()),
        )));
    }

    let format = DocumentFormat::from_path(source_path);
    let out_buf = match output_path {
        Some(p) => p.to_path_buf(),
        None => default_output_path(source_path, direction),
    };

    let start_time = Instant::now();

    let (chars, items, msg) = match format {
        DocumentFormat::Txt => {
            txt::convert_txt(source_path, &out_buf, direction)?
        }
        DocumentFormat::Docx => {
            docx::convert_docx(source_path, &out_buf, direction)?
        }
        DocumentFormat::Doc => {
            doc::convert_doc(source_path, &out_buf, direction)?
        }
        DocumentFormat::Pdf => {
            pdf::convert_pdf(source_path, &out_buf, direction)?
        }
        DocumentFormat::Unknown => {
            let ext = source_path.extension().and_then(|s| s.to_str()).unwrap_or("");
            return Err(DocConvertError::UnsupportedFormat(ext.to_string()));
        }
    };

    let duration_ms = start_time.elapsed().as_millis();

    Ok(ConversionReport {
        source_path: source_path.to_path_buf(),
        output_path: out_buf,
        format,
        direction,
        characters_converted: chars,
        items_processed: items,
        duration_ms,
        message: msg,
    })
}
