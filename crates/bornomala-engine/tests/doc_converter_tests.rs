use std::fs::File;
use std::io::{Read, Write};
use tempfile::tempdir;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use bornomala_engine::doc_converter::{
    convert_file, ConversionDirection, DocumentFormat,
};

#[test]
fn test_txt_conversion_bidirectional() {
    let dir = tempdir().unwrap();

    // 1. ANSI -> Unicode
    let ansi_path = dir.path().join("sample_ansi.txt");
    std::fs::write(&ansi_path, "Avgvi †mvbvi evsjv, Avwg †Zvgvq fv‡jvevwm|").unwrap();

    let unicode_out = dir.path().join("sample_unicode.txt");
    let report = convert_file(
        &ansi_path,
        Some(&unicode_out),
        ConversionDirection::AnsiToUnicode,
    ).unwrap();

    assert_eq!(report.format, DocumentFormat::Txt);
    assert!(unicode_out.exists());

    let content = std::fs::read_to_string(&unicode_out).unwrap();
    assert!(content.contains("আমার সোনার বাংলা"));
    assert!(content.contains("তোমায় ভালোবাসি"));

    // 2. Unicode -> ANSI
    let roundtrip_ansi = dir.path().join("roundtrip_ansi.txt");
    let report_rev = convert_file(
        &unicode_out,
        Some(&roundtrip_ansi),
        ConversionDirection::UnicodeToAnsi,
    ).unwrap();

    assert_eq!(report_rev.format, DocumentFormat::Txt);
    assert!(roundtrip_ansi.exists());
    let ansi_roundtrip_content = std::fs::read_to_string(&roundtrip_ansi).unwrap();
    assert!(ansi_roundtrip_content.contains("Avgvi"));
    assert!(ansi_roundtrip_content.contains("evsjv"));
}

#[test]
fn test_docx_conversion_layout_preserving() {
    let dir = tempdir().unwrap();
    let docx_in = dir.path().join("test_input.docx");

    // Build a mock DOCX ZIP archive
    {
        let file = File::create(&docx_in).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = FileOptions::default().compression_method(CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(b"<Types><Default Extension=\"xml\" ContentType=\"application/xml\"/></Types>").unwrap();

        zip.start_file("word/document.xml", opts).unwrap();
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr><w:jc w:val="center"/></w:pPr>
      <w:r>
        <w:rPr>
          <w:rFonts w:ascii="SutonnyMJ" w:hAnsi="SutonnyMJ" w:cs="SutonnyMJ"/>
          <w:b/>
          <w:color w:val="006A4E"/>
        </w:rPr>
        <w:t>Avgvi †mvbvi evsjv</w:t>
      </w:r>
    </w:p>
  </w:body>
</w:document>"#;
        zip.write_all(doc_xml.as_bytes()).unwrap();

        zip.start_file("word/header1.xml", opts).unwrap();
        let hdr_xml = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>evsjv‡`k</w:t></w:r></w:p></w:hdr>"#;
        zip.write_all(hdr_xml.as_bytes()).unwrap();

        zip.finish().unwrap();
    }

    // Convert ANSI -> Unicode
    let docx_out = dir.path().join("test_output_unicode.docx");
    let rep = convert_file(&docx_in, Some(&docx_out), ConversionDirection::AnsiToUnicode).unwrap();
    assert_eq!(rep.format, DocumentFormat::Docx);
    assert!(docx_out.exists());

    // Inspect converted DOCX archive
    {
        let file = File::open(&docx_out).unwrap();
        let mut zip = ZipArchive::new(file).unwrap();

        let doc_str = {
            let mut doc_entry = zip.by_name("word/document.xml").unwrap();
            let mut s = String::new();
            doc_entry.read_to_string(&mut s).unwrap();
            s
        };

        // 1. Text should be converted to Bengali Unicode
        assert!(doc_str.contains("আমার সোনার বাংলা"), "Output was: {}", doc_str);
        // 2. Font should be converted to Kalpurush
        assert!(doc_str.contains("Kalpurush"), "Output was: {}", doc_str);
        // 3. Layout formatting (<w:jc w:val="center"/>, <w:b/>, <w:color w:val="006A4E"/>) MUST be intact!
        assert!(doc_str.contains(r#"<w:jc w:val="center"/>"#));
        assert!(doc_str.contains("<w:b/>"));
        assert!(doc_str.contains(r#"<w:color w:val="006A4E"/>"#));

        // 4. Header text should also be converted
        let hdr_str = {
            let mut hdr_entry = zip.by_name("word/header1.xml").unwrap();
            let mut s = String::new();
            hdr_entry.read_to_string(&mut s).unwrap();
            s
        };
        assert!(hdr_str.contains("বাংলাদেশ"), "Header was: {}", hdr_str);
    }
}

#[test]
fn test_pdf_conversion() {
    let dir = tempdir().unwrap();
    let pdf_in = dir.path().join("test_input.pdf");

    // Create a simple valid PDF using lopdf
    {
        use lopdf::{Document, Object, Dictionary, Stream};
        use lopdf::content::{Content, Operation};

        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();

        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 12.into()]),
                Operation::new("Tj", vec![Object::string_literal("Avgvi †mvbvi evsjv")]),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(Stream::new(Dictionary::new(), content.encode().unwrap()));

        let page_id = doc.add_object(Dictionary::from_iter(vec![
            ("Type", "Page".into()),
            ("Parent", pages_id.into()),
            ("Contents", content_id.into()),
        ]));

        let pages = Dictionary::from_iter(vec![
            ("Type", "Pages".into()),
            ("Kids", vec![page_id.into()].into()),
            ("Count", 1.into()),
        ]);
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let catalog_id = doc.add_object(Dictionary::from_iter(vec![
            ("Type", "Catalog".into()),
            ("Pages", pages_id.into()),
        ]));
        doc.trailer.set("Root", catalog_id);

        doc.save(&pdf_in).unwrap();
    }

    let pdf_out = dir.path().join("test_output_unicode.pdf");
    let rep = convert_file(&pdf_in, Some(&pdf_out), ConversionDirection::AnsiToUnicode).unwrap();
    assert_eq!(rep.format, DocumentFormat::Pdf);
    assert!(pdf_out.exists());

    // Verify converted PDF can be loaded
    let loaded = lopdf::Document::load(&pdf_out).unwrap();
    assert_eq!(loaded.get_pages().len(), 1);
}
