use std::path::PathBuf;

use crate::{
    domain::models::{
        CloudIdentifierMode, ImportSourceResult, SourceSummary, SourceType, VaultSummary,
    },
    error::{CommandError, CommandResult},
    services::{source_service, vault_service},
};

#[tauri::command]
pub fn create_vault(path: String, name: String) -> CommandResult<VaultSummary> {
    vault_service::create_vault(&PathBuf::from(path), &name).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_vault_in_parent(
    parent_path: String,
    name: String,
) -> CommandResult<VaultSummary> {
    vault_service::create_vault_in_parent(&PathBuf::from(parent_path), &name)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn open_vault(path: String) -> CommandResult<VaultSummary> {
    vault_service::open_vault(&PathBuf::from(path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn update_cloud_identifier_mode(
    vault_path: String,
    mode: CloudIdentifierMode,
) -> CommandResult<VaultSummary> {
    vault_service::update_cloud_identifier_mode(&PathBuf::from(vault_path), mode)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn import_source(
    vault_path: String,
    source_path: String,
    source_type: SourceType,
) -> CommandResult<ImportSourceResult> {
    tauri::async_runtime::spawn_blocking(move || {
        source_service::import_source(
            &PathBuf::from(vault_path),
            &PathBuf::from(source_path),
            source_type,
        )
        .map_err(CommandError::from)
    })
    .await
    .map_err(CommandError::background_task)?
}

#[tauri::command]
pub fn list_sources(vault_path: String) -> CommandResult<Vec<SourceSummary>> {
    source_service::list_sources(&PathBuf::from(vault_path)).map_err(CommandError::from)
}
