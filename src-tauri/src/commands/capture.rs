use std::path::PathBuf;

use crate::{
    domain::models::SourceType,
    error::{CommandError, CommandResult},
    services::{
        capture_catalog_service, capture_metadata_service,
        capture_service::{self, CaptureClassificationResult, CaptureRole, CaptureSourceView},
    },
};

#[tauri::command]
pub fn create_capture_source(
    vault_path: String,
    text: String,
    source_type: Option<SourceType>,
) -> CommandResult<CaptureSourceView> {
    capture_service::create_capture_source(
        &PathBuf::from(vault_path),
        &text,
        source_type.unwrap_or(SourceType::Other),
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_capture_source(
    vault_path: String,
    source_id: String,
) -> CommandResult<CaptureSourceView> {
    capture_service::get_capture_source(&PathBuf::from(vault_path), &source_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_capture_source_type(
    vault_path: String,
    source_id: String,
    source_type: SourceType,
) -> CommandResult<CaptureSourceView> {
    capture_metadata_service::update_capture_source_type(
        &PathBuf::from(vault_path),
        &source_id,
        source_type,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_captures(
    vault_path: String,
    limit: Option<u32>,
) -> CommandResult<Vec<CaptureSourceView>> {
    capture_catalog_service::list_captures(&PathBuf::from(vault_path), limit.unwrap_or(50) as usize)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_unclassified_captures(
    vault_path: String,
    limit: Option<u32>,
) -> CommandResult<Vec<CaptureSourceView>> {
    capture_service::list_unclassified_captures(
        &PathBuf::from(vault_path),
        limit.unwrap_or(8) as usize,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn classify_capture_source(
    vault_path: String,
    source_id: String,
    role: CaptureRole,
) -> CommandResult<CaptureClassificationResult> {
    capture_service::classify_capture_source(&PathBuf::from(vault_path), &source_id, role)
        .map_err(CommandError::from)
}
