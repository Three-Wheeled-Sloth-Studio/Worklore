use std::path::PathBuf;

use crate::{
    domain::interviews::SubmitInterviewResponseRequest,
    error::{CommandError, CommandResult},
    services::seed_development_service::{
        self, SeedDevelopmentStoryResult, SeedDevelopmentSummary,
    },
};

#[tauri::command]
pub fn start_story_seed_development(
    vault_path: String,
    seed_id: String,
) -> CommandResult<SeedDevelopmentSummary> {
    seed_development_service::start_story_seed_development(&PathBuf::from(vault_path), &seed_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn submit_story_seed_development_response(
    vault_path: String,
    request: SubmitInterviewResponseRequest,
) -> CommandResult<SeedDevelopmentSummary> {
    seed_development_service::submit_story_seed_development_response(
        &PathBuf::from(vault_path),
        request,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_story_from_seed_development(
    vault_path: String,
    interview_id: String,
) -> CommandResult<SeedDevelopmentStoryResult> {
    seed_development_service::create_story_from_seed_development(
        &PathBuf::from(vault_path),
        &interview_id,
    )
    .map_err(CommandError::from)
}
