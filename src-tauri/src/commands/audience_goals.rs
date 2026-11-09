use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::audience_goals_service::{self, AudienceGoals},
};

#[tauri::command]
pub fn get_audience_goals(vault_path: String) -> CommandResult<AudienceGoals> {
    audience_goals_service::get(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn save_audience_goals(vault_path: String, goals: AudienceGoals) -> CommandResult<AudienceGoals> {
    audience_goals_service::save(&PathBuf::from(vault_path), goals).map_err(CommandError::from)
}
