use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use uuid::Uuid;

use crate::{
    domain::models::{
        CloudIdentifierMode, VaultDocument, VaultFeatures, VaultPrivacy, VaultSummary,
    },
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, sanitize_file_name, write_json_atomic},
    services::{
        entity_scan::{count_pending_review_items, initialize_registry, save_registry},
        performance_service, privacy_scan_migration,
    },
};

const VAULT_DIRECTORIES: &[&str] = &[
    "sources/resumes",
    "sources/job-descriptions",
    "sources/writing-samples",
    "sources/interview-transcripts",
    "sources/git-snapshots",
    "sources/other",
    "sources/metadata",
    "roles",
    "candidates",
    "stories",
    "interviews",
    "jobs",
    "voice",
    "privacy/review-items",
    "privacy/scan-log",
    "exports/manual-workspaces",
    "exports/markdown",
    "exports/json",
    "backups",
    ".worklore/extraction-cache",
    ".worklore/provider-logs",
    ".worklore/operation-journal",
    ".worklore/operation-metrics/active",
];

pub fn create_vault_in_parent(parent_path: &Path, name: &str) -> ServiceResult<VaultSummary> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(WorkLoreError::InvalidVault(
            "Vault name cannot be empty.".to_string(),
        ));
    }

    fs::create_dir_all(parent_path)?;
    let folder_name = vault_folder_name(trimmed_name);
    let vault_path = unique_vault_directory(parent_path, &folder_name);
    create_vault(&vault_path, trimmed_name)
}

pub fn create_vault(path: &Path, name: &str) -> ServiceResult<VaultSummary> {
    if path.join("vault.json").exists() {
        return Err(WorkLoreError::VaultAlreadyExists);
    }

    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(WorkLoreError::InvalidVault(
            "Vault name cannot be empty.".to_string(),
        ));
    }

    fs::create_dir_all(path)?;
    for directory in VAULT_DIRECTORIES {
        fs::create_dir_all(path.join(directory))?;
    }

    let now = Utc::now().to_rfc3339();
    let vault = VaultDocument {
        schema_version: 1,
        vault_id: format!("vault_{}", Uuid::now_v7()),
        name: trimmed_name.to_string(),
        description: String::new(),
        created_at: now.clone(),
        updated_at: now,
        privacy: VaultPrivacy {
            cloud_identifier_mode: CloudIdentifierMode::Redact,
            block_cloud_when_high_risk_review_pending: true,
            retain_provider_bodies: false,
        },
        features: VaultFeatures {
            entity_registry: true,
            ollama: true,
            gemini: true,
            manual_workspace: true,
        },
        metadata: Default::default(),
    };

    write_json_atomic(&path.join("vault.json"), &vault)?;
    let registry = initialize_registry(&vault.vault_id);
    save_registry(path, &registry)?;

    summarize(path, vault)
}

pub fn open_vault(path: &Path) -> ServiceResult<VaultSummary> {
    let vault_file = path.join("vault.json");
    if !vault_file.is_file() {
        return Err(WorkLoreError::NotAVault);
    }

    let vault: VaultDocument = read_json(&vault_file)?;
    if vault.schema_version != 1 {
        return Err(WorkLoreError::InvalidVault(format!(
            "Unsupported vault schema version {}.",
            vault.schema_version
        )));
    }

    if !path.join("privacy/entity-registry.json").is_file() {
        return Err(WorkLoreError::InvalidVault(
            "The Private Entity Registry is missing.".to_string(),
        ));
    }

    for directory in VAULT_DIRECTORIES {
        fs::create_dir_all(path.join(directory))?;
    }
    performance_service::recover_interrupted(path)?;
    privacy_scan_migration::migrate_legacy_review_noise(path)?;

    summarize(path, vault)
}

pub fn update_cloud_identifier_mode(
    path: &Path,
    mode: CloudIdentifierMode,
) -> ServiceResult<VaultSummary> {
    let mut vault: VaultDocument = read_json(&path.join("vault.json"))?;
    vault.privacy.cloud_identifier_mode = mode;
    vault.updated_at = Utc::now().to_rfc3339();
    write_json_atomic(&path.join("vault.json"), &vault)?;
    summarize(path, vault)
}

fn summarize(path: &Path, vault: VaultDocument) -> ServiceResult<VaultSummary> {
    Ok(VaultSummary {
        schema_version: vault.schema_version,
        vault_id: vault.vault_id,
        name: vault.name,
        path: path.to_string_lossy().to_string(),
        created_at: vault.created_at,
        updated_at: vault.updated_at,
        source_count: count_json_files(&path.join("sources/metadata"))?,
        story_count: count_json_files(&path.join("stories"))?,
        pending_privacy_review_count: count_pending_review_items(path)?,
        cloud_identifier_mode: vault.privacy.cloud_identifier_mode,
    })
}

fn count_json_files(directory: &Path) -> ServiceResult<usize> {
    if !directory.exists() {
        return Ok(0);
    }

    let mut count = 0;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
        {
            count += 1;
        }
    }
    Ok(count)
}

fn vault_folder_name(name: &str) -> String {
    let sanitized = sanitize_file_name(name);
    let upper = sanitized.to_ascii_uppercase();
    let reserved = matches!(
        upper.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    );

    if sanitized == "source" || reserved {
        "WorkLore Vault".to_string()
    } else {
        sanitized
    }
}

fn unique_vault_directory(parent_path: &Path, folder_name: &str) -> PathBuf {
    let direct = parent_path.join(folder_name);
    if !direct.exists() {
        return direct;
    }

    for suffix in 2..10_000 {
        let candidate = parent_path.join(format!("{folder_name}-{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    parent_path.join(format!("{folder_name}-{}", Uuid::now_v7()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_directories_do_not_use_parent_traversal() {
        assert!(VAULT_DIRECTORIES
            .iter()
            .all(|directory| !directory.contains("..")));
    }

    #[test]
    fn vault_creation_uses_a_named_child_folder() {
        let parent = std::env::temp_dir().join(format!("worklore-vault-test-{}", Uuid::now_v7()));
        let summary = create_vault_in_parent(&parent, "My Career Stories")
            .expect("vault should be created");
        let expected = parent.join("My Career Stories");

        assert_eq!(summary.path, expected.to_string_lossy());
        assert!(expected.join("vault.json").is_file());

        fs::remove_dir_all(parent).expect("test vault should be removable");
    }

    #[test]
    fn existing_named_folder_gets_a_unique_sibling() {
        let parent = std::env::temp_dir().join(format!("worklore-vault-test-{}", Uuid::now_v7()));
        fs::create_dir_all(parent.join("My Career Stories"))
            .expect("existing folder should be created");

        let summary = create_vault_in_parent(&parent, "My Career Stories")
            .expect("vault should be created");
        let expected = parent.join("My Career Stories-2");

        assert_eq!(summary.path, expected.to_string_lossy());
        assert!(expected.join("vault.json").is_file());

        fs::remove_dir_all(parent).expect("test vault should be removable");
    }
}
