use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{ServiceResult, WorkLoreError};

pub fn read_json<T: DeserializeOwned>(path: &Path) -> ServiceResult<T> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> ServiceResult<()> {
    let parent = path.parent().ok_or(WorkLoreError::InvalidPath)?;
    fs::create_dir_all(parent)?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(WorkLoreError::InvalidPath)?;
    let temporary = parent.join(format!(".{file_name}.{}.tmp", Uuid::now_v7()));
    let backup = parent.join(format!(".{file_name}.previous"));

    let bytes = serde_json::to_vec_pretty(value)?;
    {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }

    if path.exists() {
        if backup.exists() {
            fs::remove_file(&backup)?;
        }
        fs::rename(path, &backup)?;
    }

    if let Err(error) = fs::rename(&temporary, path) {
        if backup.exists() && !path.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(error.into());
    }

    if backup.exists() {
        fs::remove_file(backup)?;
    }

    Ok(())
}

pub fn sha256_file(path: &Path) -> ServiceResult<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

pub fn sanitize_file_name(input: &str) -> String {
    let mut value = String::with_capacity(input.len());
    for character in input.chars() {
        let safe = character.is_ascii_alphanumeric()
            || matches!(character, '.' | '-' | '_' | ' ' | '(' | ')');
        value.push(if safe { character } else { '_' });
    }

    let trimmed = value.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        "source".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn unique_destination(directory: &Path, file_name: &str) -> PathBuf {
    let direct = directory.join(file_name);
    if !direct.exists() {
        return direct;
    }

    let original = Path::new(file_name);
    let stem = original
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("source");
    let extension = original.extension().and_then(|value| value.to_str());

    for suffix in 2..10_000 {
        let candidate_name = match extension {
            Some(extension) => format!("{stem}-{suffix}.{extension}"),
            None => format!("{stem}-{suffix}"),
        };
        let candidate = directory.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    directory.join(format!("{}-{}", stem, Uuid::now_v7()))
}

pub fn to_vault_relative(vault_path: &Path, path: &Path) -> ServiceResult<String> {
    let relative = path
        .strip_prefix(vault_path)
        .map_err(|_| WorkLoreError::InvalidPath)?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}
