use std::path::PathBuf;

use crate::{
    domain::candidates::{
        CandidateStatus, CandidateSummary, ExtractCandidatesResult,
    },
    error::{CommandError, CommandResult},
    services::candidate_service,
};

#[tauri::command]
pub fn extract_resume_candidates(
    vault_path: String,
    source_id: String,
) -> CommandResult<ExtractCandidatesResult> {
    candidate_service::extract_resume_candidates(&PathBuf::from(vault_path), &source_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_story_candidates(vault_path: String) -> CommandResult<Vec<CandidateSummary>> {
    candidate_service::list_candidates(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn set_story_candidate_status(
    vault_path: String,
    candidate_id: String,
    status: CandidateStatus,
) -> CommandResult<CandidateSummary> {
    candidate_service::set_candidate_status(&PathBuf::from(vault_path), &candidate_id, status)
        .map_err(CommandError::from)
}
