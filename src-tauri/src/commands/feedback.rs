use std::path::PathBuf;

use crate::{
    domain::feedback::{
        FeedbackSnapshotView, ManualPublicationView, MarkPostPublishedRequest,
        PerformanceRecordView, RecordPostPerformanceRequest,
    },
    error::{CommandError, CommandResult},
    services::publication_feedback_service,
};

#[tauri::command]
pub fn mark_post_published(
    vault_path: String,
    request: MarkPostPublishedRequest,
) -> CommandResult<ManualPublicationView> {
    publication_feedback_service::mark_post_published(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn record_post_performance(
    vault_path: String,
    request: RecordPostPerformanceRequest,
) -> CommandResult<PerformanceRecordView> {
    publication_feedback_service::record_post_performance(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_feedback_snapshot(vault_path: String) -> CommandResult<FeedbackSnapshotView> {
    publication_feedback_service::get_feedback_snapshot(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}
