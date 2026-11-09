use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::target_context_service::{
        self, CreateTargetContextResult, TargetContextExtractionResult,
        TargetContextLinkTargetView, TargetContextRecordView, TargetContextRelationKind,
        TargetContextRelationshipMutationResult, UpdateTargetContextRequest,
    },
};

#[tauri::command]
pub fn create_target_context_from_source(
    vault_path: String,
    source_id: String,
) -> CommandResult<CreateTargetContextResult> {
    target_context_service::create_target_context_from_source(
        &PathBuf::from(vault_path),
        &source_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_target_context(
    vault_path: String,
    target_id: String,
) -> CommandResult<TargetContextRecordView> {
    target_context_service::load_target_context(&PathBuf::from(vault_path), &target_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_target_contexts(vault_path: String) -> CommandResult<Vec<TargetContextRecordView>> {
    target_context_service::list_target_contexts(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_target_context(
    vault_path: String,
    request: UpdateTargetContextRequest,
) -> CommandResult<TargetContextRecordView> {
    target_context_service::update_target_context(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn extract_target_context_signals(
    vault_path: String,
    target_id: String,
) -> CommandResult<TargetContextExtractionResult> {
    target_context_service::extract_target_context_signals(&PathBuf::from(vault_path), &target_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn add_target_context_relationship(
    vault_path: String,
    target_context_id: String,
    relation_kind: TargetContextRelationKind,
    target_id: String,
) -> CommandResult<TargetContextRelationshipMutationResult> {
    target_context_service::add_target_context_relationship(
        &PathBuf::from(vault_path),
        &target_context_id,
        relation_kind,
        &target_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn remove_target_context_relationship(
    vault_path: String,
    target_context_id: String,
    relation_kind: TargetContextRelationKind,
    target_id: String,
) -> CommandResult<TargetContextRelationshipMutationResult> {
    target_context_service::remove_target_context_relationship(
        &PathBuf::from(vault_path),
        &target_context_id,
        relation_kind,
        &target_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_target_context_link_targets(
    vault_path: String,
    relation_kind: TargetContextRelationKind,
) -> CommandResult<Vec<TargetContextLinkTargetView>> {
    target_context_service::list_target_context_link_targets(
        &PathBuf::from(vault_path),
        relation_kind,
    )
    .map_err(CommandError::from)
}
