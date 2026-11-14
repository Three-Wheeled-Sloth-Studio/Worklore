use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::app_preferences_service,
};

#[tauri::command]
pub fn get_last_vault_path() -> CommandResult<Option<String>> {
    app_preferences_service::get_last_vault_path().map_err(CommandError::from)
}

#[tauri::command]
pub fn remember_last_vault(vault_path: String) -> CommandResult<()> {
    app_preferences_service::remember_last_vault(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn clear_last_vault() -> CommandResult<()> {
    app_preferences_service::clear_last_vault().map_err(CommandError::from)
}

#[tauri::command]
pub fn get_last_import_directory() -> CommandResult<Option<String>> {
    app_preferences_service::get_last_import_directory().map_err(CommandError::from)
}

#[tauri::command]
pub fn remember_last_import_file(file_path: String) -> CommandResult<String> {
    app_preferences_service::remember_last_import_file(&PathBuf::from(file_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_default_vault_root() -> CommandResult<String> {
    app_preferences_service::get_default_vault_root().map_err(CommandError::from)
}
