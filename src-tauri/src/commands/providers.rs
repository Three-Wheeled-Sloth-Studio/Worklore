use std::path::PathBuf;

use crate::{
    domain::providers::{
        AnalyzeVoiceEvidenceRequest, CreateManualWorkspaceRequest, ManualWorkspaceResult,
        ProviderConnectionView, ProviderModelView, ProviderSettingsView,
        UpdateProviderSettingsRequest, VoiceAnalysisProposalSet,
    },
    error::{CommandError, CommandResult},
    services::{manual_workspace_service, provider_registry, voice_analysis_service},
};

#[tauri::command]
pub fn create_manual_workspace(
    vault_path: String,
    request: CreateManualWorkspaceRequest,
) -> CommandResult<ManualWorkspaceResult> {
    manual_workspace_service::create_manual_workspace(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_provider_settings() -> CommandResult<ProviderSettingsView> {
    provider_registry::get_settings().map_err(CommandError::from)
}

#[tauri::command]
pub fn update_provider_settings(
    request: UpdateProviderSettingsRequest,
) -> CommandResult<ProviderSettingsView> {
    provider_registry::update_settings(request).map_err(CommandError::from)
}

#[tauri::command]
pub async fn list_provider_models(provider_id: String) -> CommandResult<Vec<ProviderModelView>> {
    provider_registry::list_models(&provider_id)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn test_provider_connection(
    provider_id: String,
) -> CommandResult<ProviderConnectionView> {
    provider_registry::test_connection(&provider_id)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn analyze_voice_evidence(
    vault_path: String,
    request: AnalyzeVoiceEvidenceRequest,
) -> CommandResult<VoiceAnalysisProposalSet> {
    voice_analysis_service::analyze_voice_evidence(&PathBuf::from(vault_path), request)
        .await
        .map_err(CommandError::from)
}
