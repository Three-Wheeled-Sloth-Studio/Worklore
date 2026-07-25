use std::{
    env, fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, write_json_atomic},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppPreferences {
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    #[serde(default)]
    last_vault_path: Option<String>,
    #[serde(default)]
    last_import_directory: Option<String>,
    updated_at: String,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            last_vault_path: None,
            last_import_directory: None,
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

pub fn get_last_vault_path() -> ServiceResult<Option<String>> {
    let preferences = read_preferences_or_default(&preferences_path()?)?;
    Ok(existing_directory_value(preferences.last_vault_path, true))
}

pub fn remember_last_vault(path: &Path) -> ServiceResult<()> {
    if !path.join("vault.json").is_file() {
        return Err(WorkLoreError::NotAVault);
    }

    update_preferences(|preferences| {
        preferences.last_vault_path = Some(path.to_string_lossy().to_string());
    })
}

pub fn clear_last_vault() -> ServiceResult<()> {
    update_preferences(|preferences| {
        preferences.last_vault_path = None;
    })
}

pub fn get_last_import_directory() -> ServiceResult<Option<String>> {
    let preferences = read_preferences_or_default(&preferences_path()?)?;
    Ok(existing_directory_value(
        preferences.last_import_directory,
        false,
    ))
}

pub fn remember_last_import_file(file_path: &Path) -> ServiceResult<String> {
    let parent = file_path.parent().ok_or(WorkLoreError::InvalidPath)?;
    if !parent.is_dir() {
        return Err(WorkLoreError::InvalidPath);
    }

    let value = parent.to_string_lossy().to_string();
    update_preferences(|preferences| {
        preferences.last_import_directory = Some(value.clone());
    })?;
    Ok(value)
}

pub fn default_vault_root() -> ServiceResult<PathBuf> {
    let root = application_root()?.join("Vaults");
    fs::create_dir_all(&root)?;
    Ok(root)
}

pub fn get_default_vault_root() -> ServiceResult<String> {
    Ok(default_vault_root()?.to_string_lossy().to_string())
}

fn update_preferences(mut update: impl FnMut(&mut AppPreferences)) -> ServiceResult<()> {
    let preferences_path = preferences_path()?;
    let mut preferences = read_preferences_or_default(&preferences_path)?;
    update(&mut preferences);
    preferences.updated_at = Utc::now().to_rfc3339();
    write_json_atomic(&preferences_path, &preferences)
}

fn existing_directory_value(value: Option<String>, require_vault: bool) -> Option<String> {
    value.filter(|candidate| {
        let path = Path::new(candidate);
        path.is_dir() && (!require_vault || path.join("vault.json").is_file())
    })
}

fn read_preferences_or_default(path: &Path) -> ServiceResult<AppPreferences> {
    if !path.is_file() {
        return Ok(AppPreferences::default());
    }

    match read_json(path) {
        Ok(preferences) => Ok(preferences),
        Err(WorkLoreError::Json(_)) => Ok(AppPreferences::default()),
        Err(error) => Err(error),
    }
}

fn preferences_path() -> ServiceResult<PathBuf> {
    Ok(application_root()?.join("preferences.json"))
}

fn application_root() -> ServiceResult<PathBuf> {
    let base = local_config_root().ok_or(WorkLoreError::InvalidPath)?;
    let root = base.join("WorkLore");
    fs::create_dir_all(&root)?;
    Ok(root)
}

#[cfg(target_os = "windows")]
fn local_config_root() -> Option<PathBuf> {
    env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn local_config_root() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library").join("Application Support"))
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn local_config_root() -> Option<PathBuf> {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(path));
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config"))
}

const fn default_schema_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preferences_keep_vault_and_import_memory_separate() {
        let preferences = AppPreferences::default();
        assert_eq!(preferences.schema_version, 1);
        assert!(preferences.last_vault_path.is_none());
        assert!(preferences.last_import_directory.is_none());
    }

    #[test]
    fn missing_paths_are_not_returned_as_defaults() {
        let missing = std::env::temp_dir().join(format!("missing-{}", Utc::now().timestamp_nanos_opt().unwrap_or_default()));
        assert!(existing_directory_value(Some(missing.to_string_lossy().to_string()), false).is_none());
    }
}
