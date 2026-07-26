use std::path::PathBuf;

use crate::{
    domain::providers::{CreateManualWorkspaceRequest, ManualWorkspaceResult},
    error::{CommandError, CommandResult},
    services::manual_workspace_service,
};

#[tauri::command]
pub fn create_manual_workspace(
    vault_path: String,
    request: CreateManualWorkspaceRequest,
) -> CommandResult<ManualWorkspaceResult> {
    manual_workspace_service::create_manual_workspace(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}
