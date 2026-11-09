use std::path::PathBuf;

use crate::{
    domain::roles::RoleSummary,
    error::{CommandError, CommandResult},
    services::role_service,
};

#[tauri::command]
pub fn list_roles(vault_path: String) -> CommandResult<Vec<RoleSummary>> {
    role_service::list_role_summaries(&PathBuf::from(vault_path)).map_err(CommandError::from)
}
