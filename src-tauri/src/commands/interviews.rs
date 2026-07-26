use std::path::PathBuf;

use crate::{
    domain::interviews::{InterviewSummary, SubmitInterviewResponseRequest},
    error::{CommandError, CommandResult},
    services::interview_service,
};

#[tauri::command]
pub fn start_guided_interview(
    vault_path: String,
    candidate_id: String,
) -> CommandResult<InterviewSummary> {
    interview_service::start_interview(&PathBuf::from(vault_path), &candidate_id)
        .map_err(CommandError::from)
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
    interview_service::submit_response(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn resume_guided_interview(
    vault_path: String,
    interview_id: String,
) -> CommandResult<InterviewSummary> {
    interview_service::resume_interview(&PathBuf::from(vault_path), &interview_id)
        .map_err(CommandError::from)
}
