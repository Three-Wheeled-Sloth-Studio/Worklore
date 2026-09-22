use std::path::PathBuf;

use crate::{
    commands::external,
    domain::discovery::{
        DevelopDiscoveryTopicRequest, DevelopDiscoveryTopicResult, DiscoveryFeedbackView,
        DiscoveryOpportunityView, DiscoveryScanResult, RecordDiscoveryFeedbackRequest,
        SaveDiscoveryInspirationResult, ScanDiscoveryRequest,
    },
    error::{CommandError, CommandResult},
    services::discovery_service,
};

#[tauri::command]
pub async fn scan_discovery(
    vault_path: String,
    request: ScanDiscoveryRequest,
) -> CommandResult<DiscoveryScanResult> {
    discovery_service::scan(&PathBuf::from(vault_path), request)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_discovery_opportunities(
    vault_path: String,
    limit: Option<usize>,
) -> CommandResult<Vec<DiscoveryOpportunityView>> {
    discovery_service::list_opportunities(&PathBuf::from(vault_path), limit.unwrap_or(20))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn record_discovery_feedback(
    vault_path: String,
    request: RecordDiscoveryFeedbackRequest,
) -> CommandResult<DiscoveryFeedbackView> {
    discovery_service::record_feedback(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn save_discovery_inspiration(
    vault_path: String,
    opportunity_id: String,
) -> CommandResult<SaveDiscoveryInspirationResult> {
    discovery_service::save_as_inspiration(&PathBuf::from(vault_path), &opportunity_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn develop_discovery_topic(
    vault_path: String,
    request: DevelopDiscoveryTopicRequest,
) -> CommandResult<DevelopDiscoveryTopicResult> {
    discovery_service::develop_topic(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn dismiss_discovery_opportunity(
    vault_path: String,
    opportunity_id: String,
) -> CommandResult<DiscoveryOpportunityView> {
    discovery_service::dismiss_opportunity(&PathBuf::from(vault_path), &opportunity_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn open_discovery_source(
    vault_path: String,
    opportunity_id: String,
    url: String,
) -> CommandResult<()> {
    let approved =
        discovery_service::validate_source_url(&PathBuf::from(vault_path), &opportunity_id, &url)
            .map_err(CommandError::from)?;
    external::open_system_browser(&approved).map_err(|message| CommandError {
        code: "external_link_failed".to_string(),
        message,
        detail: None,
    })
}
