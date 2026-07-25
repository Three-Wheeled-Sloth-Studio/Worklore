use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::entity_review::{
        self, EntityReviewView, ResolveEntityReviewRequest, ResolveEntityReviewResult,
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
