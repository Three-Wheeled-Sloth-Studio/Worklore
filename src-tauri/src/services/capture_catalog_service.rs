use std::{path::Path, time::Duration};

use rusqlite::Connection;

use crate::{
    domain::models::SourceType,
    error::{ServiceResult, WorkLoreError},
    services::{
        canonical_store,
        capture_service::{CaptureClassificationView, CaptureRole, CaptureSourceView},
    },
};

pub fn list_captures(vault_path: &Path, limit: usize) -> ServiceResult<Vec<CaptureSourceView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let query_limit = i64::try_from(limit.clamp(1, 500)).unwrap_or(500);
    let rows = {
        let mut statement = connection.prepare(
            "SELECT source_id,source_type,display_name,COALESCE(captured_text,''),imported_at,updated_at
             FROM sources
             WHERE source_origin='captured_text'
             ORDER BY imported_at DESC,source_id
             LIMIT ?1",
        )?;
        let mapped = statement.query_map([query_limit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        mapped.collect::<Result<Vec<_>, _>>()?
    };

    rows.into_iter()
        .map(|row| {
            let classifications = load_classifications(&connection, &row.0)?;
            Ok(CaptureSourceView {
                source_id: row.0,
                source_type: source_type_from_text(&row.1)?,
                display_name: row.2,
                text: row.3,
                created_at: row.4,
                updated_at: row.5,
                classifications,
            })
        })
        .collect()
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    let connection = Connection::open(canonical_store::database_path(vault_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

fn load_classifications(
    connection: &Connection,
    source_id: &str,
) -> ServiceResult<Vec<CaptureClassificationView>> {
    let mut statement = connection.prepare(
        "SELECT relationship_type,to_id FROM record_relationships
         WHERE from_type='source' AND from_id=?1 AND relationship_type LIKE 'capture_%'
         ORDER BY created_at",
    )?;
    let rows = statement.query_map([source_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut classifications = Vec::new();
    for row in rows {
        let (relationship_type, target_id) = row?;
        if let Some(role) = role_from_relationship(&relationship_type) {
            classifications.push(CaptureClassificationView { role, target_id });
        }
    }
    Ok(classifications)
}

fn source_type_from_text(value: &str) -> ServiceResult<SourceType> {
    match value {
        "resume" => Ok(SourceType::Resume),
        "job_description" => Ok(SourceType::JobDescription),
        "writing_sample" => Ok(SourceType::WritingSample),
        "interview_transcript" => Ok(SourceType::InterviewTranscript),
        "git_snapshot" => Ok(SourceType::GitSnapshot),
        "other" => Ok(SourceType::Other),
        _ => Err(WorkLoreError::InvalidVault(format!(
            "Unknown canonical source type {value}."
        ))),
    }
}

fn role_from_relationship(value: &str) -> Option<CaptureRole> {
    match value {
        "capture_story_seed" => Some(CaptureRole::StorySeed),
        "capture_proof_point" => Some(CaptureRole::ProofPoint),
        "capture_topic_candidate" => Some(CaptureRole::TopicCandidate),
        "capture_inspiration" => Some(CaptureRole::Inspiration),
        "capture_target_context" => Some(CaptureRole::TargetContext),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{capture_service, vault_service};
    use uuid::Uuid;

    #[test]
    fn catalog_keeps_classified_captures_visible() {
        let path =
            std::env::temp_dir().join(format!("worklore-capture-catalog-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Capture Catalog Test").unwrap();
        let capture = capture_service::create_capture_source(
            &path,
            "Reduced delivery cycle time by 80% while rebuilding the release process.",
            SourceType::Resume,
        )
        .unwrap();
        capture_service::classify_capture_source(&path, &capture.source_id, CaptureRole::StorySeed)
            .unwrap();
        capture_service::classify_capture_source(
            &path,
            &capture.source_id,
            CaptureRole::ProofPoint,
        )
        .unwrap();

        let rows = list_captures(&path, 20).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].source_id, capture.source_id);
        assert!(rows[0]
            .classifications
            .iter()
            .any(|item| item.role == CaptureRole::StorySeed));
        assert!(rows[0]
            .classifications
            .iter()
            .any(|item| item.role == CaptureRole::ProofPoint));

        std::fs::remove_dir_all(path).unwrap();
    }
}
