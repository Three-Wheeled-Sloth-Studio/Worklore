use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::writing_lint_service::{self, LintDraftRequest, LintDraftResult},
};

#[tauri::command]
pub fn lint_draft(vault_path: String, request: LintDraftRequest) -> CommandResult<LintDraftResult> {
    writing_lint_service::lint_draft(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}
