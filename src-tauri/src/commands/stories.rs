use std::path::PathBuf;

use crate::{
    domain::stories::{
        ImportStoryResponseRequest, ImportStoryResponseResult, StoryStatus, StorySummary,
    },
    error::{CommandError, CommandResult},
    services::{canonical_store, story_service},
};

#[tauri::command]
pub fn import_story_response(
    vault_path: String,
    request: ImportStoryResponseRequest,
) -> CommandResult<ImportStoryResponseResult> {
    let vault_path = PathBuf::from(vault_path);
    let result = story_service::import_synthesis_response(
        &vault_path,
        &request.interview_id,
        &PathBuf::from(request.response_path),
    )
    .map_err(CommandError::from)?;
    canonical_store::migrate_prototype(&vault_path).map_err(CommandError::from)?;
    Ok(result)
}

#[tauri::command]
pub fn list_stories(vault_path: String) -> CommandResult<Vec<StorySummary>> {
    story_service::list_stories(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn set_story_status(
    vault_path: String,
    story_id: String,
    status: StoryStatus,
) -> CommandResult<StorySummary> {
    let vault_path = PathBuf::from(vault_path);
    let result = story_service::set_story_status(&vault_path, &story_id, status)
        .map_err(CommandError::from)?;
    canonical_store::migrate_prototype(&vault_path).map_err(CommandError::from)?;
    Ok(result)
}
