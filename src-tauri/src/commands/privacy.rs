use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::{
        confidentiality_service::{
            self, ConfidentialityTransformRequest, ConfidentialityTransformResult,
        },
        entity_review::{
            self, EntityReviewView, ResolveEntityReviewRequest, ResolveEntityReviewResult,
        },
    },
};

#[tauri::command]
pub fn list_entity_reviews(
    vault_path: String,
    pending_only: Option<bool>,
) -> CommandResult<Vec<EntityReviewView>> {
    entity_review::list_reviews(&PathBuf::from(vault_path), pending_only.unwrap_or(true))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn resolve_entity_review(
    vault_path: String,
    request: ResolveEntityReviewRequest,
) -> CommandResult<ResolveEntityReviewResult> {
    entity_review::resolve_review(&PathBuf::from(vault_path), request).map_err(CommandError::from)
}

#[tauri::command]
pub fn transform_confidentiality_for_public_use(
    vault_path: String,
    request: ConfidentialityTransformRequest,
) -> CommandResult<ConfidentialityTransformResult> {
    confidentiality_service::transform_for_public_use(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}
