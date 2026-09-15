use std::{path::Path, time::Duration};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use uuid::Uuid;

use crate::{
    domain::models::SourceType,
    error::{ServiceResult, WorkLoreError},
    services::{canonical_store, capture_service},
};

pub fn update_capture_source_type(
    vault_path: &Path,
    source_id: &str,
    source_type: SourceType,
) -> ServiceResult<capture_service::CaptureSourceView> {
    canonical_store::initialize(vault_path)?;
    let source_id = source_id.trim();
    if source_id.is_empty() {
        return Err(WorkLoreError::InvalidVault(
            "Capture source ID is required.".to_string(),
        ));
    }

    let mut connection = open_connection(vault_path)?;
    let current: Option<(String, String)> = connection
        .query_row(
            "SELECT source_type,source_origin FROM sources WHERE source_id=?1",
            [source_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let (before, origin) = current.ok_or(WorkLoreError::SourceNotFound)?;
    if origin != "captured_text" {
        return Err(WorkLoreError::InvalidVault(
            "Only captured text can be re-tagged in place. Imported files keep their original source type because it is part of import provenance."
                .to_string(),
        ));
    }

    let after = source_type_text(source_type);
    if before == after {
        return capture_service::get_capture_source(vault_path, source_id);
    }

    let now = Utc::now().to_rfc3339();
    let tx = connection.transaction()?;
    tx.execute(
        "UPDATE sources SET source_type=?2,updated_at=?3,revision=revision+1 WHERE source_id=?1",
        params![source_id, after, &now],
    )?;
    tx.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)\n         VALUES (?1,'capture_source_type_changed','source',?2,'user',?3,?4)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            source_id,
            json!({"before":before,"after":after}).to_string(),
            &now
        ],
    )?;
    tx.commit()?;

    capture_service::get_capture_source(vault_path, source_id)
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    let connection = Connection::open(canonical_store::database_path(vault_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

fn source_type_text(source_type: SourceType) -> &'static str {
    match source_type {
        SourceType::Resume => "resume",
        SourceType::JobDescription => "job_description",
        SourceType::WritingSample => "writing_sample",
        SourceType::InterviewTranscript => "interview_transcript",
        SourceType::GitSnapshot => "git_snapshot",
        SourceType::Other => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::vault_service;

    #[test]
    fn captured_text_can_be_retagged_without_rewriting_the_source() {
        let path = std::env::temp_dir().join(format!("worklore-capture-retag-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Capture Retag Test").unwrap();
        let created = capture_service::create_capture_source(
            &path,
            "Reduced cycle time by 80% while preserving auditability.",
            SourceType::Other,
        )
        .unwrap();

        let updated =
            update_capture_source_type(&path, &created.source_id, SourceType::Resume).unwrap();

        assert_eq!(updated.source_type, SourceType::Resume);
        assert_eq!(updated.text, created.text);
        assert_eq!(updated.source_id, created.source_id);
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn retag_rolls_back_when_audit_write_fails() {
        let path = std::env::temp_dir().join(format!(
            "worklore-capture-retag-rollback-{}",
            Uuid::now_v7()
        ));
        vault_service::create_vault(&path, "Capture Retag Rollback Test").unwrap();
        let created = capture_service::create_capture_source(
            &path,
            "Kept the source mutation and audit record in one unit of work.",
            SourceType::Other,
        )
        .unwrap();

        let sabotage = open_connection(&path).unwrap();
        sabotage
            .execute_batch(
                "CREATE TRIGGER force_capture_audit_failure
                 BEFORE INSERT ON audit_events
                 WHEN NEW.event_type='capture_source_type_changed'
                 BEGIN
                   SELECT RAISE(ABORT,'forced capture audit failure');
                 END;",
            )
            .unwrap();
        drop(sabotage);

        assert!(update_capture_source_type(&path, &created.source_id, SourceType::Resume).is_err());

        let connection = open_connection(&path).unwrap();
        let stored_type: String = connection
            .query_row(
                "SELECT source_type FROM sources WHERE source_id=?1",
                [&created.source_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored_type, "other");

        std::fs::remove_dir_all(path).unwrap();
    }
}
