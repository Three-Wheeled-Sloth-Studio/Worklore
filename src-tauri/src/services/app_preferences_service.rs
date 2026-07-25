use std::{
    env,
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
    last_vault_path: Option<String>,
    updated_at: String,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            last_vault_path: None,
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

pub fn get_last_vault_path() -> ServiceResult<Option<String>> {
    let path = preferences_path()?;
    if !path.is_file() {
        return Ok(None);
    }

    let preferences: AppPreferences = read_json(&path)?;
    Ok(preferences
        .last_vault_path
        .filter(|value| !value.trim().is_empty()))
}

pub fn remember_last_vault(path: &Path) -> ServiceResult<()> {
    if !path.join("vault.json").is_file() {
        return Err(WorkLoreError::NotAVault);
    }

    let preferences_path = preferences_path()?;
    let mut preferences = read_preferences_or_default(&preferences_path)?;
    preferences.last_vault_path = Some(path.to_string_lossy().to_string());
    preferences.updated_at = Utc::now().to_rfc3339();
    write_json_atomic(&preferences_path, &preferences)
}

pub fn clear_last_vault() -> ServiceResult<()> {
    let preferences_path = preferences_path()?;
    let mut preferences = read_preferences_or_default(&preferences_path)?;
    preferences.last_vault_path = None;
    preferences.updated_at = Utc::now().to_rfc3339();
    write_json_atomic(&preferences_path, &preferences)
}

fn read_preferences_or_default(path: &Path) -> ServiceResult<AppPreferences> {
    if path.is_file() {
        read_json(path)
    } else {
        Ok(AppPreferences::default())
    }
}

fn preferences_path() -> ServiceResult<PathBuf> {
    let base = local_config_root().ok_or(WorkLoreError::InvalidPath)?;
    Ok(base.join("WorkLore").join("preferences.json"))
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
    fn default_preferences_do_not_assume_a_vault() {
        let preferences = AppPreferences::default();
        assert_eq!(preferences.schema_version, 1);
        assert!(preferences.last_vault_path.is_none());
    }
}
