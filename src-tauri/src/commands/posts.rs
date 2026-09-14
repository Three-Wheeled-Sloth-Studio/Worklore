use std::path::PathBuf;

use crate::{
    domain::posts::{
        AppendPostRevisionRequest, ApprovePostRevisionRequest, CreatePostRequest,
        LinkPostSupportingMaterialRequest, PostLineageView, PostRecordView,
    },
    error::{CommandError, CommandResult, WorkLoreError},
    services::{
        confidentiality_service::{
            self, ConfidentialityState, ConfidentialityTransformRequest,
        },
        post_catalog_service, post_lineage_service,
    },
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
    let vault_path = PathBuf::from(vault_path);
    let lineage = post_lineage_service::get_post_lineage(&vault_path, &request.post_id)
        .map_err(CommandError::from)?;
    let revision = lineage
        .revisions
        .iter()
        .find(|revision| revision.revision_id == request.revision_id)
        .ok_or_else(|| {
            CommandError::from(WorkLoreError::InvalidVault(format!(
                "Post Revision {} was not found for final approval.",
                request.revision_id
            )))
        })?;
    let preflight = confidentiality_service::transform_for_public_use(
        &vault_path,
        ConfidentialityTransformRequest {
            text: revision.text.clone(),
        },
    )
    .map_err(CommandError::from)?;

    if preflight.state != ConfidentialityState::Ready {
        return Err(CommandError::from(WorkLoreError::InvalidVault(
            "Final approval is blocked until confidentiality review is resolved.".to_string(),
        )));
    }
    if preflight.public_safe_text != revision.text {
        return Err(CommandError::from(WorkLoreError::InvalidVault(
            "Final approval requires the exact saved Revision to match WorkLore's public-safe text. Load the public-safe text, save a new Revision, and challenge it again."
                .to_string(),
        )));
    }

    post_lineage_service::approve_revision(&vault_path, request).map_err(CommandError::from)
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
