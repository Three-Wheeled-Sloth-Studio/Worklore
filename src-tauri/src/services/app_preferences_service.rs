use std::{
    env, fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    domain::providers::{ProviderSettingsView, GEMINI_PROVIDER_ID, OPENAI_PROVIDER_ID},
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, write_json_atomic},
    services::provider_secret_service,
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
    #[serde(default)]
    selected_provider_id: Option<String>,
    #[serde(default = "default_ollama_base_url")]
    ollama_base_url: String,
    #[serde(default)]
    ollama_model_id: Option<String>,
    #[serde(default)]
    openai_model_id: Option<String>,
    #[serde(default)]
    openai_api_key_ciphertext: Option<String>,
    #[serde(default)]
    gemini_model_id: Option<String>,
    #[serde(default)]
    gemini_api_key_ciphertext: Option<String>,
    #[serde(default)]
    brave_search_api_key_ciphertext: Option<String>,
    updated_at: String,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            last_vault_path: None,
            last_import_directory: None,
            selected_provider_id: None,
            ollama_base_url: default_ollama_base_url(),
            ollama_model_id: None,
            openai_model_id: None,
            openai_api_key_ciphertext: None,
            gemini_model_id: None,
            gemini_api_key_ciphertext: None,
            brave_search_api_key_ciphertext: None,
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

pub struct ProviderSecretUpdate {
    pub openai_api_key: Option<String>,
    pub clear_openai_api_key: bool,
    pub gemini_api_key: Option<String>,
    pub clear_gemini_api_key: bool,
    pub brave_search_api_key: Option<String>,
    pub clear_brave_search_api_key: bool,
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
    let executable = env::current_exe().map_err(WorkLoreError::Io)?;
    default_vault_root_from_executable(&executable)
}

pub fn get_default_vault_root() -> ServiceResult<String> {
    Ok(default_vault_root()?.to_string_lossy().to_string())
}

pub fn get_provider_settings() -> ServiceResult<ProviderSettingsView> {
    let preferences = read_preferences_or_default(&preferences_path()?)?;
    Ok(ProviderSettingsView {
        selected_provider_id: preferences.selected_provider_id,
        ollama_base_url: preferences.ollama_base_url,
        ollama_model_id: preferences.ollama_model_id,
        openai_model_id: preferences.openai_model_id,
        openai_api_key_configured: has_ciphertext(&preferences.openai_api_key_ciphertext),
        gemini_model_id: preferences.gemini_model_id,
        gemini_api_key_configured: has_ciphertext(&preferences.gemini_api_key_ciphertext),
        brave_search_api_key_configured: has_ciphertext(
            &preferences.brave_search_api_key_ciphertext,
        ),
    })
}

pub fn save_provider_settings(
    settings: ProviderSettingsView,
    secrets: ProviderSecretUpdate,
) -> ServiceResult<()> {
    let protected_openai = protect_optional_secret(secrets.openai_api_key.as_deref())?;
    let protected_gemini = protect_optional_secret(secrets.gemini_api_key.as_deref())?;
    let protected_brave =
        protect_optional_secret(secrets.brave_search_api_key.as_deref())?;
    update_preferences(|preferences| {
        preferences.schema_version = 4;
        preferences.selected_provider_id = settings.selected_provider_id.clone();
        preferences.ollama_base_url = settings.ollama_base_url.clone();
        preferences.ollama_model_id = settings.ollama_model_id.clone();
        preferences.openai_model_id = settings.openai_model_id.clone();
        preferences.gemini_model_id = settings.gemini_model_id.clone();
        if secrets.clear_openai_api_key {
            preferences.openai_api_key_ciphertext = None;
        } else if let Some(ciphertext) = &protected_openai {
            preferences.openai_api_key_ciphertext = Some(ciphertext.clone());
        }
        if secrets.clear_gemini_api_key {
            preferences.gemini_api_key_ciphertext = None;
        } else if let Some(ciphertext) = &protected_gemini {
            preferences.gemini_api_key_ciphertext = Some(ciphertext.clone());
        }
        if secrets.clear_brave_search_api_key {
            preferences.brave_search_api_key_ciphertext = None;
        } else if let Some(ciphertext) = &protected_brave {
            preferences.brave_search_api_key_ciphertext = Some(ciphertext.clone());
        }
    })
}

pub fn get_brave_search_api_key() -> ServiceResult<Option<String>> {
    let preferences = read_preferences_or_default(&preferences_path()?)?;
    preferences
        .brave_search_api_key_ciphertext
        .map(|value| provider_secret_service::unprotect_secret(&value))
        .transpose()
}

pub fn get_provider_api_key(provider_id: &str) -> ServiceResult<Option<String>> {
    let preferences = read_preferences_or_default(&preferences_path()?)?;
    let ciphertext = match provider_id {
        OPENAI_PROVIDER_ID => preferences.openai_api_key_ciphertext,
        GEMINI_PROVIDER_ID => preferences.gemini_api_key_ciphertext,
        _ => None,
    };
    ciphertext
        .map(|value| provider_secret_service::unprotect_secret(&value))
        .transpose()
}

fn protect_optional_secret(secret: Option<&str>) -> ServiceResult<Option<String>> {
    secret
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(provider_secret_service::protect_secret)
        .transpose()
}

fn has_ciphertext(value: &Option<String>) -> bool {
    value
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
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

fn default_vault_root_from_executable(executable: &Path) -> ServiceResult<PathBuf> {
    executable
        .parent()
        .map(Path::to_path_buf)
        .ok_or(WorkLoreError::InvalidPath)
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
    4
}

fn default_ollama_base_url() -> String {
    "http://127.0.0.1:11434".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preferences_keep_vault_import_and_provider_secrets_separate() {
        let preferences = AppPreferences::default();
        assert_eq!(preferences.schema_version, 4);
        assert!(preferences.last_vault_path.is_none());
        assert!(preferences.last_import_directory.is_none());
        assert!(preferences.selected_provider_id.is_none());
        assert_eq!(preferences.ollama_base_url, "http://127.0.0.1:11434");
        assert!(preferences.ollama_model_id.is_none());
        assert!(preferences.openai_model_id.is_none());
        assert!(preferences.openai_api_key_ciphertext.is_none());
        assert!(preferences.gemini_model_id.is_none());
        assert!(preferences.gemini_api_key_ciphertext.is_none());
        assert!(preferences.brave_search_api_key_ciphertext.is_none());
    }

    #[test]
    fn missing_paths_are_not_returned_as_defaults() {
        let stamp = Utc::now().timestamp_nanos_opt().unwrap_or_default();
        let missing = std::env::temp_dir().join(format!("missing-{stamp}"));
        assert!(
            existing_directory_value(Some(missing.to_string_lossy().to_string()), false).is_none()
        );
    }

    #[test]
    fn default_vault_root_is_the_executable_folder() {
        let executable = PathBuf::from(r"D:\Portable\WorkLore\WorkLore-QA.exe");
        assert_eq!(
            default_vault_root_from_executable(&executable).unwrap(),
            PathBuf::from(r"D:\Portable\WorkLore")
        );
    }
}
