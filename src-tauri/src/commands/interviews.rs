use std::path::PathBuf;

use crate::{
    domain::interviews::{InterviewSummary, SubmitInterviewResponseRequest},
    error::{CommandError, CommandResult},
    services::{canonical_store, interview_service},
};

#[tauri::command]
pub fn start_guided_interview(
    vault_path: String,
    candidate_id: String,
) -> CommandResult<InterviewSummary> {
    let vault_path = PathBuf::from(vault_path);
    let result = interview_service::start_interview(&vault_path, &candidate_id)
        .map_err(CommandError::from)?;
    canonical_store::migrate_prototype(&vault_path).map_err(CommandError::from)?;
    Ok(result)
}

#[tauri::command]
pub fn list_guided_interviews(vault_path: String) -> CommandResult<Vec<InterviewSummary>> {
    interview_service::list_interviews(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn submit_guided_interview_response(
    vault_path: String,
    request: SubmitInterviewResponseRequest,
) -> CommandResult<InterviewSummary> {
    let vault_path = PathBuf::from(vault_path);
    let result =
        interview_service::submit_response(&vault_path, request).map_err(CommandError::from)?;
    canonical_store::migrate_prototype(&vault_path).map_err(CommandError::from)?;
    Ok(result)
}

#[tauri::command]
pub fn resume_guided_interview(
    vault_path: String,
    interview_id: String,
) -> CommandResult<InterviewSummary> {
    let vault_path = PathBuf::from(vault_path);
    let result = interview_service::resume_interview(&vault_path, &interview_id)
        .map_err(CommandError::from)?;
    canonical_store::migrate_prototype(&vault_path).map_err(CommandError::from)?;
    Ok(result)
}
