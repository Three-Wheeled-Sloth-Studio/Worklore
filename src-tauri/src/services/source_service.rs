use std::{fs, path::Path};

use chrono::Utc;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{
    domain::models::{
        ContentHash, ExtractionState, ExtractionStatus, ImportSourceResult, PrivacyScanState,
        PrivacyScanStatus, SourceDocument, SourceProvenance, SourceSummary, SourceType,
    },
    error::{ServiceResult, WorkLoreError},
    io_utils::{
        read_json, sanitize_file_name, sha256_file, to_vault_relative, unique_destination,
        write_json_atomic,
    },
    services::{
        contextual_entity_scan::scan_named_projects, document_extraction, entity_scan::scan_text,
        performance_service::OperationSession,
    },
};

pub fn import_source(
    vault_path: &Path,
    source_path: &Path,
    source_type: SourceType,
) -> ServiceResult<ImportSourceResult> {
    if !source_path.is_file() {
        return Err(WorkLoreError::InvalidSourceFile);
    }
    if !vault_path.join("vault.json").is_file() {
        return Err(WorkLoreError::NotAVault);
    }

    let extension = normalized_extension(source_path)?;
    if !matches!(extension.as_str(), "txt" | "md" | "pdf" | "docx") {
        return Err(WorkLoreError::UnsupportedSourceType);
    }

    let original_size = fs::metadata(source_path)?.len();
    let mut operation_metadata = Map::new();
    operation_metadata.insert("extension".to_string(), Value::String(extension.clone()));
    operation_metadata.insert(
        "sourceType".to_string(),
        Value::String(format!("{source_type:?}").to_ascii_lowercase()),
    );
    operation_metadata.insert("byteSize".to_string(), Value::from(original_size));
    let mut operation = OperationSession::start(vault_path, "import_source", operation_metadata)?;

    let result = (|| {
        let hash = operation.step("hash_source", empty_metadata(), || sha256_file(source_path))?;
        let existing = operation.step("check_duplicate", empty_metadata(), || {
            find_source_by_hash(vault_path, &hash)
        })?;
        if let Some(existing) = existing {
            let mut summary = SourceSummary::from(&existing);
            summary.duplicate_of_source_id = Some(existing.source_id.clone());
            return Ok(ImportSourceResult {
                source: summary,
                created: false,
                duplicate_detected: true,
                message: format!(
                    "This file is already stored as {}. WorkLore reused the existing source.",
                    existing.display_name
                ),
            });
        }

        let source_id = format!("source_{}", Uuid::now_v7());
        let original_file_name = source_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(WorkLoreError::InvalidSourceFile)?
            .to_string();
        let safe_file_name = sanitize_file_name(&original_file_name);
        let destination_directory = vault_path.join("sources").join(source_type.folder_name());
        fs::create_dir_all(&destination_directory)?;
        let destination = unique_destination(&destination_directory, &safe_file_name);

        operation.step("copy_source", empty_metadata(), || {
            fs::copy(source_path, &destination)?;
            Ok(())
        })?;

        let metadata = fs::metadata(&destination)?;
        let now = Utc::now().to_rfc3339();
        let (extraction, privacy_scan) = extract_and_scan(
            vault_path,
            &source_id,
            &destination,
            &extension,
            &now,
            &mut operation,
        )?;

        let document = SourceDocument {
            schema_version: 1,
            source_id: source_id.clone(),
            source_type,
            display_name: source_path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(&original_file_name)
                .to_string(),
            stored_path: to_vault_relative(vault_path, &destination)?,
            original_file_name,
            original_path_hint: Some(source_path.to_string_lossy().to_string()),
            media_type: Some(media_type_for_extension(&extension).to_string()),
            byte_size: metadata.len(),
            content_hash: ContentHash {
                algorithm: "sha256".to_string(),
                value: hash,
            },
            imported_at: now.clone(),
            updated_at: now,
            extraction,
            privacy_scan,
            provenance: SourceProvenance {
                import_method: "file_copy".to_string(),
                parent_source_id: None,
                notes: String::new(),
            },
            tags: Vec::new(),
        };

        operation.step("write_source_metadata", empty_metadata(), || {
            write_json_atomic(
                &vault_path
                    .join("sources/metadata")
                    .join(format!("{source_id}.json")),
                &document,
            )
        })?;

        Ok(ImportSourceResult {
            source: SourceSummary::from(&document),
            created: true,
            duplicate_detected: false,
            message: match (document.extraction.status, document.privacy_scan.status) {
                (_, PrivacyScanStatus::NeedsReview) => {
                    "Source imported. WorkLore found private entities that need review.".to_string()
                }
                (ExtractionStatus::Unsupported, _) => {
                    "Source imported, but no extractable text was found. OCR is not enabled yet."
                        .to_string()
                }
                (ExtractionStatus::Failed, _) => {
                    "Source copied into the vault, but text extraction failed.".to_string()
                }
                _ => "Source imported and scanned.".to_string(),
            },
        })
    })();

    operation.finish(result)
}

pub fn list_sources(vault_path: &Path) -> ServiceResult<Vec<SourceSummary>> {
    let mut documents = read_source_documents(vault_path)?;
    documents.sort_by(|left, right| right.imported_at.cmp(&left.imported_at));
    Ok(documents.iter().map(SourceSummary::from).collect())
}

fn read_source_documents(vault_path: &Path) -> ServiceResult<Vec<SourceDocument>> {
    let directory = vault_path.join("sources/metadata");
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut documents = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
        {
            documents.push(read_json(&entry.path())?);
        }
    }
    Ok(documents)
}

fn find_source_by_hash(vault_path: &Path, hash: &str) -> ServiceResult<Option<SourceDocument>> {
    Ok(read_source_documents(vault_path)?
        .into_iter()
        .find(|document| document.content_hash.value == hash))
}

fn extract_and_scan(
    vault_path: &Path,
    source_id: &str,
    destination: &Path,
    extension: &str,
    now: &str,
    operation: &mut OperationSession,
) -> ServiceResult<(ExtractionState, PrivacyScanState)> {
    let extracted = match operation.step("extract_text", empty_metadata(), || {
        document_extraction::extract(destination, extension)
    }) {
        Ok(extracted) => extracted,
        Err(error) => {
            return Ok((
                ExtractionState {
                    status: ExtractionStatus::Failed,
                    extractor_version: None,
                    text_path: None,
                    character_count: 0,
                    warnings: Vec::new(),
                    error: Some(error.to_string()),
                },
                PrivacyScanState {
                    status: PrivacyScanStatus::Unavailable,
                    scan_version: None,
                    scanned_at: Some(now.to_string()),
                    review_item_ids: Vec::new(),
                },
            ));
        }
    };

    if extracted.text.trim().is_empty() {
        return Ok((
            ExtractionState {
                status: ExtractionStatus::Unsupported,
                extractor_version: Some(extracted.extractor_version),
                text_path: None,
                character_count: 0,
                warnings: extracted.warnings,
                error: None,
            },
            PrivacyScanState {
                status: PrivacyScanStatus::Unavailable,
                scan_version: None,
                scanned_at: Some(now.to_string()),
                review_item_ids: Vec::new(),
            },
        ));
    }

    let extracted_path = vault_path
        .join(".worklore/extraction-cache")
        .join(format!("{source_id}.txt"));
    let mut cache_metadata = Map::new();
    cache_metadata.insert(
        "characterCount".to_string(),
        Value::from(extracted.text.chars().count() as u64),
    );
    operation.step("write_extraction_cache", cache_metadata, || {
        fs::write(&extracted_path, &extracted.text)?;
        Ok(())
    })?;

    let base_scan = operation.step("scan_private_entities", empty_metadata(), || {
        scan_text(vault_path, "source", source_id, &extracted.text)
    })?;
    let contextual_scan = operation.step("scan_named_projects", empty_metadata(), || {
        scan_named_projects(vault_path, "source", source_id, &extracted.text)
    })?;
    let scans_complete = base_scan.status == PrivacyScanStatus::Complete
        && contextual_scan.status == PrivacyScanStatus::Complete;
    let mut review_item_ids = base_scan.review_item_ids;
    review_item_ids.extend(contextual_scan.review_item_ids);
    review_item_ids.sort();
    review_item_ids.dedup();
    let privacy_status = if review_item_ids.is_empty() && scans_complete {
        PrivacyScanStatus::Complete
    } else {
        PrivacyScanStatus::NeedsReview
    };

    Ok((
        ExtractionState {
            status: ExtractionStatus::Complete,
            extractor_version: Some(extracted.extractor_version),
            text_path: Some(to_vault_relative(vault_path, &extracted_path)?),
            character_count: extracted.text.chars().count(),
            warnings: extracted.warnings,
            error: None,
        },
        PrivacyScanState {
            status: privacy_status,
            scan_version: Some(format!(
                "{}+{}",
                base_scan.scan_version, contextual_scan.scan_version
            )),
            scanned_at: Some(contextual_scan.scanned_at),
            review_item_ids,
        },
    ))
}

fn empty_metadata() -> Map<String, Value> {
    Map::new()
}

fn normalized_extension(path: &Path) -> ServiceResult<String> {
    path.extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or(WorkLoreError::UnsupportedSourceType)
}

fn media_type_for_extension(extension: &str) -> &'static str {
    match extension {
        "txt" => "text/plain",
        "md" => "text/markdown",
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_media_types_are_stable() {
        assert_eq!(media_type_for_extension("txt"), "text/plain");
        assert_eq!(media_type_for_extension("pdf"), "application/pdf");
    }
}
