use std::path::PathBuf;

use crate::{
    domain::posts::{
        AppendPostRevisionRequest, ApprovePostRevisionRequest, CreatePostRequest,
        LinkPostSupportingMaterialRequest, PostLineageView, PostRecordView,
    },
    error::{CommandError, CommandResult},
    services::{post_catalog_service, post_lineage_service},
};

#[tauri::command]
pub fn create_post(
    vault_path: String,
    request: CreatePostRequest,
) -> CommandResult<PostLineageView> {
    post_lineage_service::create_post(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_posts(vault_path: String) -> CommandResult<Vec<PostRecordView>> {
    post_catalog_service::list_posts(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn append_post_revision(
    vault_path: String,
    request: AppendPostRevisionRequest,
) -> CommandResult<PostLineageView> {
    post_lineage_service::append_revision(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn approve_post_revision(
    vault_path: String,
    request: ApprovePostRevisionRequest,
) -> CommandResult<PostLineageView> {
    post_lineage_service::approve_revision(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_post_lineage(vault_path: String, post_id: String) -> CommandResult<PostLineageView> {
    post_lineage_service::get_post_lineage(&PathBuf::from(vault_path), &post_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn link_post_supporting_material(
    vault_path: String,
    request: LinkPostSupportingMaterialRequest,
) -> CommandResult<PostLineageView> {
    post_lineage_service::link_supporting_material(
        &PathBuf::from(vault_path),
        &request.post_id,
        request.role,
        &request.target_id,
    )
    .map_err(CommandError::from)
}
