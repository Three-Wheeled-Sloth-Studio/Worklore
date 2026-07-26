use std::path::PathBuf;

use serde_json::{Map, Value};

use crate::{
    domain::candidates::{CandidateStatus, CandidateSummary, ExtractCandidatesResult},
    error::{CommandError, CommandResult},
    services::{candidate_service, performance_service::OperationSession},
};

#[tauri::command]
pub async fn extract_resume_candidates(
    vault_path: String,
    source_id: String,
) -> CommandResult<ExtractCandidatesResult> {
    tauri::async_runtime::spawn_blocking(move || {
        let vault_path = PathBuf::from(vault_path);
        let mut metadata = Map::new();
        metadata.insert("sourceId".to_string(), Value::String(source_id.clone()));

        let mut operation =
            OperationSession::start(&vault_path, "extract_resume_candidates", metadata)
                .map_err(CommandError::from)?;
        let result = operation.step("parse_and_write_candidates", Map::new(), || {
            candidate_service::extract_resume_candidates(&vault_path, &source_id)
        });
        operation.finish(result).map_err(CommandError::from)
    })
    .await
    .map_err(CommandError::background_task)?
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
