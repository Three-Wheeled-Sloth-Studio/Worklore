use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::voice_evidence_service::{
        self, CreateVoiceEvidenceResult, ReviewVoiceEvidenceRequest, VoiceEvidenceRecordView,
        VoiceSourceCandidateView,
    },
};

#[tauri::command]
pub fn list_voice_source_candidates(
    vault_path: String,
) -> CommandResult<Vec<VoiceSourceCandidateView>> {
    voice_evidence_service::list_voice_source_candidates(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_voice_evidence_from_source(
    vault_path: String,
    source_id: String,
) -> CommandResult<CreateVoiceEvidenceResult> {
    voice_evidence_service::create_voice_evidence_from_source(
        &PathBuf::from(vault_path),
        &source_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_voice_evidence(
    vault_path: String,
    voice_evidence_id: String,
) -> CommandResult<VoiceEvidenceRecordView> {
    voice_evidence_service::load_voice_evidence(&PathBuf::from(vault_path), &voice_evidence_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_voice_evidence(vault_path: String) -> CommandResult<Vec<VoiceEvidenceRecordView>> {
    voice_evidence_service::list_voice_evidence(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn review_voice_evidence(
    vault_path: String,
    request: ReviewVoiceEvidenceRequest,
) -> CommandResult<VoiceEvidenceRecordView> {
    voice_evidence_service::review_voice_evidence(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}
