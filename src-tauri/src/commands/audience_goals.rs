use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::audience_goals_service::{self, AudienceGoals, PostAudienceIntent},
};

#[tauri::command]
pub fn get_audience_goals(vault_path: String) -> CommandResult<AudienceGoals> {
    audience_goals_service::get(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn save_audience_goals(vault_path: String, goals: AudienceGoals) -> CommandResult<AudienceGoals> {
    audience_goals_service::save(&PathBuf::from(vault_path), goals).map_err(CommandError::from)
}

#[tauri::command]
pub fn get_post_audience_intent(
    vault_path: String,
    post_id: String,
) -> CommandResult<Option<PostAudienceIntent>> {
    audience_goals_service::get_post_intent(&PathBuf::from(vault_path), &post_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn save_post_audience_intent(
    vault_path: String,
    intent: PostAudienceIntent,
) -> CommandResult<PostAudienceIntent> {
    audience_goals_service::save_post_intent(&PathBuf::from(vault_path), intent)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn clear_post_audience_intent(
    vault_path: String,
    post_id: String,
) -> CommandResult<()> {
    audience_goals_service::clear_post_intent(&PathBuf::from(vault_path), &post_id)
        .map_err(CommandError::from)
}
