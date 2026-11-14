use std::path::PathBuf;

use crate::{
    domain::posts::{
        AppendPostRevisionRequest, ApprovePostRevisionRequest, CreatePostRequest,
        GeneratePostFromTopicRequest, GeneratePostFromTopicResult,
        LinkPostSupportingMaterialRequest, PostLineageView, PostRecordView,
    },
    error::{CommandError, CommandResult, WorkLoreError},
    services::{
        confidentiality_service::{
            self, ConfidentialityState, ConfidentialityTransformRequest,
            ConfidentialityTransformResult,
        },
        post_catalog_service, post_generation_service, post_lineage_service,
    },
};

#[tauri::command]
pub fn create_post(
    vault_path: String,
    request: CreatePostRequest,
) -> CommandResult<PostLineageView> {
    post_lineage_service::create_post(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_posts(vault_path: String) -> CommandResult<Vec<PostRecordView>> {
    post_catalog_service::list_posts(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub async fn generate_post_from_topic(
    vault_path: String,
    request: GeneratePostFromTopicRequest,
) -> CommandResult<GeneratePostFromTopicResult> {
    post_generation_service::generate_post_from_topic(&PathBuf::from(vault_path), request)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn append_post_revision(
    vault_path: String,
    request: AppendPostRevisionRequest,
) -> CommandResult<PostLineageView> {
    post_lineage_service::append_revision(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn approve_post_revision(
    vault_path: String,
    request: ApprovePostRevisionRequest,
) -> CommandResult<PostLineageView> {
    let vault_path = PathBuf::from(vault_path);
    let lineage = post_lineage_service::get_post_lineage(&vault_path, &request.post_id)
        .map_err(CommandError::from)?;
    let revision = lineage
        .revisions
        .iter()
        .find(|revision| revision.revision_id == request.revision_id)
        .ok_or_else(|| {
            CommandError::from(WorkLoreError::InvalidVault(format!(
                "Post Revision {} was not found for final approval.",
                request.revision_id
            )))
        })?;
    let preflight = confidentiality_service::transform_for_public_use(
        &vault_path,
        ConfidentialityTransformRequest {
            text: revision.text.clone(),
        },
    )
    .map_err(CommandError::from)?;
    ensure_public_safe_revision(&revision.text, &preflight).map_err(CommandError::from)?;

    post_lineage_service::approve_revision(&vault_path, request).map_err(CommandError::from)
}

fn ensure_public_safe_revision(
    revision_text: &str,
    preflight: &ConfidentialityTransformResult,
) -> Result<(), WorkLoreError> {
    if preflight.state != ConfidentialityState::Ready {
        return Err(WorkLoreError::InvalidVault(
            "Final approval is blocked until confidentiality review is resolved.".to_string(),
        ));
    }
    if preflight.public_safe_text != revision_text {
        return Err(WorkLoreError::InvalidVault(
            "Final approval requires the exact saved Revision to match WorkLore's public-safe text. Load the public-safe text, save a new Revision, and challenge it again."
                .to_string(),
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn get_post_lineage(vault_path: String, post_id: String) -> CommandResult<PostLineageView> {
    post_lineage_service::get_post_lineage(&PathBuf::from(vault_path), &post_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn link_post_supporting_material(
    vault_path: String,
    request: LinkPostSupportingMaterialRequest,
) -> CommandResult<PostLineageView> {
    post_lineage_service::link_supporting_material(
        &PathBuf::from(vault_path),
        &request.post_id,
        request.role,
        &request.target_id,
    )
    .map_err(CommandError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preflight(
        state: ConfidentialityState,
        public_safe_text: &str,
    ) -> ConfidentialityTransformResult {
        ConfidentialityTransformResult {
            original_text: "Private draft".to_string(),
            token_redacted_text: public_safe_text.to_string(),
            public_safe_text: public_safe_text.to_string(),
            state,
            replacements: Vec::new(),
            unresolved_risks: Vec::new(),
            registry_revision: 1,
            provider_used: false,
        }
    }

    #[test]
    fn final_approval_requires_ready_exact_public_safe_text() {
        assert!(ensure_public_safe_revision(
            "Private draft",
            &preflight(ConfidentialityState::Blocked, "Private draft")
        )
        .is_err());
        assert!(ensure_public_safe_revision(
            "Private draft",
            &preflight(ConfidentialityState::Ready, "a private employer draft")
        )
        .is_err());
        assert!(ensure_public_safe_revision(
            "Public-safe draft",
            &preflight(ConfidentialityState::Ready, "Public-safe draft")
        )
        .is_ok());
    }
}
