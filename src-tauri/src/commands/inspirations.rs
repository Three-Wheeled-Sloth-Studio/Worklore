use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::inspiration_service::{
        self, CreateInspirationResult, InspirationLinkTargetView, InspirationRecordView,
        InspirationRelationKind, InspirationRelationshipMutationResult, UpdateInspirationRequest,
    },
};

#[tauri::command]
pub fn create_inspiration_from_source(
    vault_path: String,
    source_id: String,
) -> CommandResult<CreateInspirationResult> {
    inspiration_service::create_inspiration_from_source(&PathBuf::from(vault_path), &source_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_inspiration(
    vault_path: String,
    inspiration_id: String,
) -> CommandResult<InspirationRecordView> {
    inspiration_service::load_inspiration(&PathBuf::from(vault_path), &inspiration_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_inspirations(vault_path: String) -> CommandResult<Vec<InspirationRecordView>> {
    inspiration_service::list_inspirations(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn update_inspiration(
    vault_path: String,
    request: UpdateInspirationRequest,
) -> CommandResult<InspirationRecordView> {
    inspiration_service::update_inspiration(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn add_inspiration_relationship(
    vault_path: String,
    inspiration_id: String,
    relation_kind: InspirationRelationKind,
    target_id: String,
) -> CommandResult<InspirationRelationshipMutationResult> {
    inspiration_service::add_inspiration_relationship(
        &PathBuf::from(vault_path),
        &inspiration_id,
        relation_kind,
        &target_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn remove_inspiration_relationship(
    vault_path: String,
    inspiration_id: String,
    relation_kind: InspirationRelationKind,
    target_id: String,
) -> CommandResult<InspirationRelationshipMutationResult> {
    inspiration_service::remove_inspiration_relationship(
        &PathBuf::from(vault_path),
        &inspiration_id,
        relation_kind,
        &target_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_inspiration_link_targets(
    vault_path: String,
    relation_kind: InspirationRelationKind,
) -> CommandResult<Vec<InspirationLinkTargetView>> {
    inspiration_service::list_inspiration_link_targets(&PathBuf::from(vault_path), relation_kind)
        .map_err(CommandError::from)
}
