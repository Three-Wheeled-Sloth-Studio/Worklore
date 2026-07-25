use std::path::PathBuf;

use crate::{
    domain::stories::{
        ImportStoryResponseRequest, ImportStoryResponseResult, StoryStatus, StorySummary,
    },
    error::{CommandError, CommandResult},
    services::story_service,
};

#[tauri::command]
pub fn import_story_response(
    vault_path: String,
    request: ImportStoryResponseRequest,
) -> CommandResult<ImportStoryResponseResult> {
    story_service::import_synthesis_response(
        &PathBuf::from(vault_path),
        &request.interview_id,
        &PathBuf::from(request.response_path),
    )
    .map_err(CommandError::from)
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
    story_service::set_story_status(&PathBuf::from(vault_path), &story_id, status)
        .map_err(CommandError::from)
}
