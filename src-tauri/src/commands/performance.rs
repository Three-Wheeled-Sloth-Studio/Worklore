use std::path::PathBuf;

use crate::{
    domain::performance::PerformanceSnapshot,
    error::{CommandError, CommandResult},
    services::performance_service,
};

#[tauri::command]
pub fn get_performance_snapshot(
    vault_path: String,
    limit: Option<usize>,
) -> CommandResult<PerformanceSnapshot> {
    performance_service::snapshot(&PathBuf::from(vault_path), limit.unwrap_or(40))
        .map_err(CommandError::from)
}
