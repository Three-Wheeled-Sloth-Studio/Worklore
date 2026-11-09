use std::path::PathBuf;

use crate::{
    domain::edit_learning::{
        DecideEditLearningProposalRequest, EditLearningAnalysisView, EditLearningDecisionView,
    },
    error::{CommandError, CommandResult},
    services::{
        edit_learning_service,
        voice_evidence_service::{
            self, CreateVoiceEvidenceResult, ReviewVoiceEvidenceRequest, VoiceEvidenceRecordView,
            VoiceSourceCandidateView,
        },
        voice_profile_service::{
            self, CoreVoiceRecordView, CreateCoreVoiceRequest, CreateToneModeRequest,
            CreateVoiceDirectionRequest, CreateWritingRuleRequest, SaveCoreVoiceTraitRequest,
            SetVoiceDirectionStatusRequest, ToneModeRecordView, UpdateToneModeRequest,
            UpdateWritingRuleRequest, VoiceDirectionRecordView, WritingRuleRecordView,
        },
    },
};

#[tauri::command]
pub fn analyze_edit_learning(vault_path: String) -> CommandResult<EditLearningAnalysisView> {
    edit_learning_service::analyze_edit_learning(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn decide_edit_learning_proposal(
    vault_path: String,
    request: DecideEditLearningProposalRequest,
) -> CommandResult<EditLearningDecisionView> {
    edit_learning_service::decide_edit_learning_proposal(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_edit_learning_decisions(
    vault_path: String,
) -> CommandResult<Vec<EditLearningDecisionView>> {
    edit_learning_service::list_edit_learning_decisions(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

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

#[tauri::command]
pub fn list_core_voices(vault_path: String) -> CommandResult<Vec<CoreVoiceRecordView>> {
    voice_profile_service::list_core_voices(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_core_voice(
    vault_path: String,
    request: CreateCoreVoiceRequest,
) -> CommandResult<CoreVoiceRecordView> {
    voice_profile_service::create_core_voice(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn save_core_voice_trait(
    vault_path: String,
    request: SaveCoreVoiceTraitRequest,
) -> CommandResult<CoreVoiceRecordView> {
    voice_profile_service::save_core_voice_trait(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn delete_core_voice_trait(
    vault_path: String,
    voice_id: String,
    trait_id: String,
) -> CommandResult<CoreVoiceRecordView> {
    voice_profile_service::delete_core_voice_trait(&PathBuf::from(vault_path), &voice_id, &trait_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn activate_core_voice(
    vault_path: String,
    voice_id: String,
) -> CommandResult<CoreVoiceRecordView> {
    voice_profile_service::activate_core_voice(&PathBuf::from(vault_path), &voice_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_tone_modes(vault_path: String) -> CommandResult<Vec<ToneModeRecordView>> {
    voice_profile_service::list_tone_modes(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_tone_mode(
    vault_path: String,
    request: CreateToneModeRequest,
) -> CommandResult<ToneModeRecordView> {
    voice_profile_service::create_tone_mode(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_tone_mode(
    vault_path: String,
    request: UpdateToneModeRequest,
) -> CommandResult<ToneModeRecordView> {
    voice_profile_service::update_tone_mode(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_voice_directions(vault_path: String) -> CommandResult<Vec<VoiceDirectionRecordView>> {
    voice_profile_service::list_voice_directions(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_voice_direction(
    vault_path: String,
    request: CreateVoiceDirectionRequest,
) -> CommandResult<VoiceDirectionRecordView> {
    voice_profile_service::create_voice_direction(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn set_voice_direction_status(
    vault_path: String,
    request: SetVoiceDirectionStatusRequest,
) -> CommandResult<VoiceDirectionRecordView> {
    voice_profile_service::set_voice_direction_status(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_writing_rules(vault_path: String) -> CommandResult<Vec<WritingRuleRecordView>> {
    voice_profile_service::list_writing_rules(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_writing_rule(
    vault_path: String,
    request: CreateWritingRuleRequest,
) -> CommandResult<WritingRuleRecordView> {
    voice_profile_service::create_writing_rule(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_writing_rule(
    vault_path: String,
    request: UpdateWritingRuleRequest,
) -> CommandResult<WritingRuleRecordView> {
    voice_profile_service::update_writing_rule(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}
