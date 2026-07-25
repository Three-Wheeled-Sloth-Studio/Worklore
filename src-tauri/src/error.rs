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

    #[error("The requested source record was not found.")]
    SourceNotFound,

    #[error("This source is not ready for that operation: {0}")]
    SourceNotReady(String),

    #[error("WorkLore does not support this source file type yet.")]
    UnsupportedSourceType,

    #[error("The selected path cannot be represented safely inside the vault.")]
    InvalidPath,

    #[error("The vault record is invalid: {0}")]
    InvalidVault(String),

    #[error("WorkLore could not extract text from this document: {0}")]
    DocumentExtraction(String),

    #[error("The requested story candidate was not found.")]
    CandidateNotFound,

    #[error("The requested interview session was not found.")]
    InterviewNotFound,

    #[error("This interview action is not valid: {0}")]
    InvalidInterviewAction(String),

    #[error("The requested role record was not found.")]
    RoleNotFound,

    #[error("WorkLore could not infer a role from this employment-history heading: {0}")]
    RoleInferenceFailed(String),

    #[error("The requested story record was not found.")]
    StoryNotFound,

    #[error("The story synthesis response is invalid: {0}")]
    InvalidStoryResponse(String),

    #[error("The requested privacy review item was not found.")]
    ReviewItemNotFound,

    #[error("The selected private entity was not found.")]
    EntityNotFound,

    #[error("This privacy review resolution is not valid: {0}")]
    InvalidReviewResolution(String),

    #[error("This provider request is blocked by privacy preflight: {0}")]
    ProviderPreflightBlocked(String),

    #[error("The manual workspace could not be created: {0}")]
    ManualWorkspace(String),

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

impl CommandError {
    pub fn background_task(error: impl ToString) -> Self {
        Self {
            code: "background_task_failed".to_string(),
            message: "A background WorkLore operation stopped unexpectedly.".to_string(),
            detail: Some(error.to_string()),
        }
    }
}

impl From<WorkLoreError> for CommandError {
    fn from(value: WorkLoreError) -> Self {
        let code = match &value {
            WorkLoreError::NotAVault => "not_a_vault",
            WorkLoreError::VaultAlreadyExists => "vault_already_exists",
            WorkLoreError::InvalidSourceFile => "invalid_source_file",
            WorkLoreError::SourceNotFound => "source_not_found",
            WorkLoreError::SourceNotReady(_) => "source_not_ready",
            WorkLoreError::UnsupportedSourceType => "unsupported_source_type",
            WorkLoreError::InvalidPath => "invalid_path",
            WorkLoreError::InvalidVault(_) => "invalid_vault",
            WorkLoreError::DocumentExtraction(_) => "document_extraction_failed",
            WorkLoreError::CandidateNotFound => "candidate_not_found",
            WorkLoreError::InterviewNotFound => "interview_not_found",
            WorkLoreError::InvalidInterviewAction(_) => "invalid_interview_action",
            WorkLoreError::RoleNotFound => "role_not_found",
            WorkLoreError::RoleInferenceFailed(_) => "role_inference_failed",
            WorkLoreError::StoryNotFound => "story_not_found",
            WorkLoreError::InvalidStoryResponse(_) => "invalid_story_response",
            WorkLoreError::ReviewItemNotFound => "review_item_not_found",
            WorkLoreError::EntityNotFound => "entity_not_found",
            WorkLoreError::InvalidReviewResolution(_) => "invalid_review_resolution",
            WorkLoreError::ProviderPreflightBlocked(_) => "provider_preflight_blocked",
            WorkLoreError::ManualWorkspace(_) => "manual_workspace_failed",
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
