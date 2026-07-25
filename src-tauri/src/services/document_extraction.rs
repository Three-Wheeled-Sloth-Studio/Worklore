use std::{fs, fs::File, io::Read, path::Path};

use quick_xml::{escape::unescape, events::Event, reader::Reader};

use crate::error::{ServiceResult, WorkLoreError};

#[derive(Debug, Clone)]
pub struct ExtractedText {
    pub text: String,
    pub extractor_version: String,
    pub warnings: Vec<String>,
}

pub fn extract(path: &Path, extension: &str) -> ServiceResult<ExtractedText> {
    match extension {
        "txt" | "md" => extract_utf8(path),
        "pdf" => extract_pdf(path),
        "docx" => extract_docx(path),
        _ => Err(WorkLoreError::UnsupportedSourceType),
    }
}

fn extract_utf8(path: &Path) -> ServiceResult<ExtractedText> {
    let text = fs::read_to_string(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::InvalidData {
            WorkLoreError::DocumentExtraction(
                "The text file is not valid UTF-8.".to_string(),
            )
        } else {
            error.into()
        }
    })?;
    Ok(ExtractedText {
        text,
        extractor_version: "plain-text-v1".to_string(),
        warnings: Vec::new(),
    })
}

fn extract_pdf(path: &Path) -> ServiceResult<ExtractedText> {
    let text = pdf_extract::extract_text(path)
        .map_err(|error| WorkLoreError::DocumentExtraction(error.to_string()))?;
    let cleaned = clean_extracted_text(&text);
    let warnings = if cleaned.trim().is_empty() {
        vec![
            "No extractable text was found. This may be a scanned PDF that requires OCR."
                .to_string(),
        ]
    } else {
        Vec::new()
    };

    Ok(ExtractedText {
        text: cleaned,
        extractor_version: "pdf-extract-0.12-v1".to_string(),
        warnings,
    })
}

fn extract_docx(path: &Path) -> ServiceResult<ExtractedText> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|error| WorkLoreError::DocumentExtraction(error.to_string()))?;
    let mut document = archive
        .by_name("word/document.xml")
        .map_err(|error| WorkLoreError::DocumentExtraction(error.to_string()))?;
    let mut xml = String::new();
    document
        .read_to_string(&mut xml)
        .map_err(|error| WorkLoreError::DocumentExtraction(error.to_string()))?;

    let text = extract_wordprocessing_xml(&xml)?;
    let cleaned = clean_extracted_text(&text);
    let warnings = if cleaned.trim().is_empty() {
        vec!["The DOCX file did not contain readable document text.".to_string()]
    } else {
        Vec::new()
    };

    Ok(ExtractedText {
        text: cleaned,
        extractor_version: "docx-ooxml-v1".to_string(),
        warnings,
    })
}

fn extract_wordprocessing_xml(xml: &str) -> ServiceResult<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut output = String::new();
    let mut inside_text = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                inside_text = has_local_name(event.name().as_ref(), b"t");
            }
            Ok(Event::Empty(event)) => {
                let name = event.name();
                if has_local_name(name.as_ref(), b"tab") {
                    output.push('\t');
                } else if has_local_name(name.as_ref(), b"br")
                    || has_local_name(name.as_ref(), b"cr")
                {
                    output.push('\n');
                }
            }
            Ok(Event::Text(event)) if inside_text => {
                let decoded = event
                    .decode()
                    .map_err(|error| WorkLoreError::DocumentExtraction(error.to_string()))?;
                let value = unescape(&decoded)
                    .map_err(|error| WorkLoreError::DocumentExtraction(error.to_string()))?;
                output.push_str(&value);
            }
            Ok(Event::End(event)) => {
                let name = event.name();
                if has_local_name(name.as_ref(), b"t") {
                    inside_text = false;
                } else if has_local_name(name.as_ref(), b"p") {
                    push_separator(&mut output, '\n');
                } else if has_local_name(name.as_ref(), b"tc") {
                    push_separator(&mut output, '\t');
                } else if has_local_name(name.as_ref(), b"tr") {
                    push_separator(&mut output, '\n');
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                return Err(WorkLoreError::DocumentExtraction(error.to_string()));
            }
        }
    }

    Ok(output)
}

fn has_local_name(qualified_name: &[u8], local_name: &[u8]) -> bool {
    qualified_name == local_name
        || qualified_name
            .strip_suffix(local_name)
            .is_some_and(|prefix| prefix.ends_with(b":"))
}

fn push_separator(output: &mut String, separator: char) {
    if !output.is_empty() && !output.ends_with(separator) {
        output.push(separator);
    }
}

fn clean_extracted_text(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut cleaned = String::with_capacity(normalized.len());
    let mut blank_lines = 0_u8;

    for line in normalized.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank_lines = blank_lines.saturating_add(1);
            if blank_lines <= 1 && !cleaned.is_empty() {
                cleaned.push('\n');
            }
            continue;
        }

        if !cleaned.is_empty() && !cleaned.ends_with('\n') {
            cleaned.push('\n');
        }
        cleaned.push_str(line);
        cleaned.push('\n');
        blank_lines = 0;
    }

    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_paragraphs_tabs_and_escaped_text_from_docx_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="urn:test">
              <w:body>
                <w:p><w:r><w:t>First &amp; second</w:t></w:r></w:p>
                <w:p><w:r><w:t>Next</w:t><w:tab/><w:t>Column</w:t></w:r></w:p>
              </w:body>
            </w:document>"#;
        let text = extract_wordprocessing_xml(xml).expect("docx XML should parse");
        assert_eq!(clean_extracted_text(&text), "First & second\nNext\tColumn");
    }

    #[test]
    fn collapses_repeated_blank_lines() {
        assert_eq!(clean_extracted_text("One\n\n\nTwo\n"), "One\n\nTwo");
    }
}
