use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkLoreError {
    #[error("The selected folder is not a WorkLore vault.")]
    NotAVault,

    #[error("A WorkLore vault already exists in the selected folder.")]
    VaultAlreadyExists,

    #[error("The selected source file does not exist or is not a file.")]
    InvalidSourceFile,

    #[error("WorkLore does not support this source file type yet.")]
    UnsupportedSourceType,

    #[error("The selected path cannot be represented safely inside the vault.")]
    InvalidPath,

    #[error("The vault record is invalid: {0}")]
    InvalidVault(String),

    #[error("The requested privacy review item was not found.")]
    ReviewItemNotFound,

    #[error("The selected private entity was not found.")]
    EntityNotFound,

    #[error("This privacy review resolution is not valid: {0}")]
    InvalidReviewResolution(String),

    #[error("File operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON operation failed: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub detail: Option<String>,
}

impl From<WorkLoreError> for CommandError {
    fn from(value: WorkLoreError) -> Self {
        let code = match &value {
            WorkLoreError::NotAVault => "not_a_vault",
            WorkLoreError::VaultAlreadyExists => "vault_already_exists",
            WorkLoreError::InvalidSourceFile => "invalid_source_file",
            WorkLoreError::UnsupportedSourceType => "unsupported_source_type",
            WorkLoreError::InvalidPath => "invalid_path",
            WorkLoreError::InvalidVault(_) => "invalid_vault",
            WorkLoreError::ReviewItemNotFound => "review_item_not_found",
            WorkLoreError::EntityNotFound => "entity_not_found",
            WorkLoreError::InvalidReviewResolution(_) => "invalid_review_resolution",
            WorkLoreError::Io(_) => "io_error",
            WorkLoreError::Json(_) => "json_error",
        };

        Self {
            code: code.to_string(),
            message: value.to_string(),
            detail: Some(format!("{value:?}")),
        }
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
pub type ServiceResult<T> = Result<T, WorkLoreError>;
