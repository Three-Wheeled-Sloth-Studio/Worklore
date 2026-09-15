use std::{path::Path, time::Duration};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    domain::models::SourceType,
    error::{ServiceResult, WorkLoreError},
    services::canonical_store,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaptureRole {
    StorySeed,
    ProofPoint,
    TopicCandidate,
    Inspiration,
    TargetContext,
}

impl CaptureRole {
    fn relationship_type(self) -> &'static str {
        match self {
            Self::StorySeed => "capture_story_seed",
            Self::ProofPoint => "capture_proof_point",
            Self::TopicCandidate => "capture_topic_candidate",
            Self::Inspiration => "capture_inspiration",
            Self::TargetContext => "capture_target_context",
        }
    }

    fn target_type(self) -> &'static str {
        match self {
            Self::StorySeed => "story_seed",
            Self::ProofPoint => "proof_point",
            Self::TopicCandidate => "topic_candidate",
            Self::Inspiration => "inspiration",
            Self::TargetContext => "target_context",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::StorySeed => "story_seed",
            Self::ProofPoint => "proof_point",
            Self::TopicCandidate => "topic_candidate",
            Self::Inspiration => "inspiration",
            Self::TargetContext => "target_context",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CaptureClassificationView {
    pub role: CaptureRole,
    pub target_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSourceView {
    pub source_id: String,
    pub source_type: SourceType,
    pub display_name: String,
    pub text: String,
    pub created_at: String,
    pub updated_at: String,
    pub classifications: Vec<CaptureClassificationView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureClassificationResult {
    pub source_id: String,
    pub role: CaptureRole,
    pub target_id: String,
    pub created: bool,
}

pub fn create_capture_source(
    vault_path: &Path,
    text: &str,
    source_type: SourceType,
) -> ServiceResult<CaptureSourceView> {
    canonical_store::initialize(vault_path)?;
    if text.trim().is_empty() {
        return Err(WorkLoreError::InvalidVault(
            "Capture text cannot be empty.".to_string(),
        ));
    }

    let connection = open_connection(vault_path)?;
    let source_id = format!("source_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    let display_name = capture_title(text);
    let size = i64::try_from(text.len())
        .map_err(|_| WorkLoreError::InvalidVault("Capture text is too large.".to_string()))?;
    let extraction = json!({
        "status": "complete",
        "extractorVersion": "capture-text-v1",
        "textPath": null,
        "characterCount": text.chars().count(),
        "warnings": [],
        "error": null
    });
    let privacy = json!({
        "status": "pending",
        "scanVersion": null,
        "scannedAt": null,
        "reviewItemIds": []
    });
    let provenance = json!({
        "creationActor": "user",
        "importMethod": "capture_text"
    });

    connection.execute(
        "INSERT INTO sources(source_id,source_type,display_name,stored_path,original_file_name,media_type,byte_size,
         content_hash,lifecycle_status,imported_at,updated_at,extraction_json,privacy_json,provenance_json,tags_json,
         revision,source_origin,captured_text)
         VALUES (?1,?2,?3,'','','text/plain',?4,?5,'active',?6,?6,?7,?8,?9,'[]',1,'captured_text',?10)",
        params![
            &source_id,
            source_type_text(source_type),
            &display_name,
            size,
            text_hash(text),
            &now,
            extraction.to_string(),
            privacy.to_string(),
            provenance.to_string(),
            text
        ],
    )?;
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,'capture_saved','source',?2,'user',?3,?4)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            &source_id,
            json!({"sourceType": source_type_text(source_type)}).to_string(),
            &now
        ],
    )?;

    get_capture_source(vault_path, &source_id)
}

pub fn get_capture_source(vault_path: &Path, source_id: &str) -> ServiceResult<CaptureSourceView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let row = connection
        .query_row(
            "SELECT source_id,source_type,display_name,COALESCE(captured_text,''),imported_at,updated_at
             FROM sources WHERE source_id=?1 AND source_origin='captured_text'",
            [source_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or(WorkLoreError::SourceNotFound)?;
    let classifications = load_classifications(&connection, &row.0)?;
    capture_view(row, classifications)
}

pub fn list_unclassified_captures(
    vault_path: &Path,
    limit: usize,
) -> ServiceResult<Vec<CaptureSourceView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut statement = connection.prepare(
        "SELECT s.source_id,s.source_type,s.display_name,COALESCE(s.captured_text,''),s.imported_at,s.updated_at
         FROM sources s
         WHERE s.source_origin='captured_text'
           AND NOT EXISTS (
             SELECT 1 FROM record_relationships r
             WHERE r.from_type='source' AND r.from_id=s.source_id
               AND r.relationship_type LIKE 'capture_%'
           )
         ORDER BY s.imported_at DESC
         LIMIT ?1",
    )?;
    let query_limit = i64::try_from(limit.clamp(1, 100)).unwrap_or(100);
    let rows = statement.query_map([query_limit], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;
    let raw = rows.collect::<Result<Vec<_>, _>>()?;
    raw.into_iter()
        .map(|row| capture_view(row, Vec::new()))
        .collect()
}

pub fn classify_capture_source(
    vault_path: &Path,
    source_id: &str,
    role: CaptureRole,
) -> ServiceResult<CaptureClassificationResult> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    let (source_type_text_value, title, text): (String, String, String) = tx
        .query_row(
            "SELECT source_type,display_name,COALESCE(captured_text,'') FROM sources
             WHERE source_id=?1 AND source_origin='captured_text'",
            [source_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or(WorkLoreError::SourceNotFound)?;

    if let Some(target_id) = tx
        .query_row(
            "SELECT to_id FROM record_relationships
             WHERE from_type='source' AND from_id=?1 AND relationship_type=?2 LIMIT 1",
            params![source_id, role.relationship_type()],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(CaptureClassificationResult {
            source_id: source_id.to_string(),
            role,
            target_id,
            created: false,
        });
    }

    let source_type = source_type_from_text(&source_type_text_value)?;
    let now = Utc::now().to_rfc3339();
    let provenance = json!({
        "creationActor": "user",
        "sourceId": source_id,
        "captureRole": role.label()
    })
    .to_string();
    let target_id = match role {
        CaptureRole::StorySeed => {
            let id = format!("seed_{}", Uuid::now_v7());
            tx.execute(
                "INSERT INTO story_seeds(seed_id,title,summary,status,provenance_json,created_at,updated_at,revision)
                 VALUES (?1,?2,?3,'captured',?4,?5,?5,1)",
                params![&id, &title, &text, &provenance, &now],
            )?;
            id
        }
        CaptureRole::ProofPoint => {
            let id = format!("proof_{}", Uuid::now_v7());
            tx.execute(
                "INSERT INTO proof_points(proof_id,statement,status,provenance_json,created_at,updated_at,revision)
                 VALUES (?1,?2,'candidate',?3,?4,?4,1)",
                params![&id, &text, &provenance, &now],
            )?;
            id
        }
        CaptureRole::TopicCandidate => {
            let id = format!("topic_{}", Uuid::now_v7());
            tx.execute(
                "INSERT INTO topic_candidates(topic_id,title,summary,status,provenance_json,created_at,updated_at,revision)
                 VALUES (?1,?2,?3,'captured',?4,?5,?5,1)",
                params![&id, &title, &text, &provenance, &now],
            )?;
            id
        }
        CaptureRole::Inspiration => {
            let id = format!("inspiration_{}", Uuid::now_v7());
            tx.execute(
                "INSERT INTO inspirations(inspiration_id,source_id,title,notes,status,provenance_json,created_at,updated_at,revision)
                 VALUES (?1,?2,?3,'','saved',?4,?5,?5,1)",
                params![&id, source_id, &title, &provenance, &now],
            )?;
            id
        }
        CaptureRole::TargetContext => {
            let id = format!("target_{}", Uuid::now_v7());
            let context_type = if source_type == SourceType::JobDescription {
                "job_description"
            } else {
                "captured_context"
            };
            tx.execute(
                "INSERT INTO target_contexts(target_id,source_id,context_type,title,notes,status,provenance_json,created_at,updated_at,revision)
                 VALUES (?1,?2,?3,?4,'','active',?5,?6,?6,1)",
                params![&id, source_id, context_type, &title, &provenance, &now],
            )?;
            id
        }
    };

    tx.execute(
        "INSERT INTO record_relationships(relationship_id,from_type,from_id,relationship_type,to_type,to_id,
         provenance_json,created_at) VALUES (?1,'source',?2,?3,?4,?5,?6,?7)",
        params![
            format!("relationship_{}", Uuid::now_v7()),
            source_id,
            role.relationship_type(),
            role.target_type(),
            &target_id,
            json!({"creationActor":"user"}).to_string(),
            &now
        ],
    )?;
    tx.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,'capture_classified',?2,?3,'user',?4,?5)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            role.target_type(),
            &target_id,
            json!({"sourceId":source_id,"role":role.label()}).to_string(),
            &now
        ],
    )?;
    tx.commit()?;

    Ok(CaptureClassificationResult {
        source_id: source_id.to_string(),
        role,
        target_id,
        created: true,
    })
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

fn capture_view(
    row: (String, String, String, String, String, String),
    classifications: Vec<CaptureClassificationView>,
) -> ServiceResult<CaptureSourceView> {
    Ok(CaptureSourceView {
        source_id: row.0,
        source_type: source_type_from_text(&row.1)?,
        display_name: row.2,
        text: row.3,
        created_at: row.4,
        updated_at: row.5,
        classifications,
    })
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

fn capture_title(text: &str) -> String {
    let first_line = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("Captured note");
    compact(first_line, 90)
}

fn compact(value: &str, max: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max {
        normalized
    } else {
        let mut shortened = normalized
            .chars()
            .take(max.saturating_sub(3))
            .collect::<String>();
        shortened.push_str("...");
        shortened
    }
}

fn text_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::vault_service;
    use std::fs;

    fn vault() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-capture-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Capture Test").expect("create vault");
        path
    }

    #[test]
    fn capture_persists_before_classification_and_reopen() {
        let path = vault();
        let raw = "  A memory I do not want to lose.\nSecond line.  ";
        let captured = create_capture_source(&path, raw, SourceType::Other).unwrap();
        assert_eq!(captured.text, raw);
        assert!(captured.classifications.is_empty());
        canonical_store::initialize_and_migrate(&path).unwrap();
        let reopened = get_capture_source(&path, &captured.source_id).unwrap();
        assert_eq!(reopened.text, raw);
        assert!(reopened.classifications.is_empty());

        let connection = open_connection(&path).unwrap();
        let stored: (String, String, String, String) = connection
            .query_row(
                "SELECT stored_path,original_file_name,source_origin,COALESCE(captured_text,'')
                 FROM sources WHERE source_id=?1",
                [&captured.source_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(stored.0, "");
        assert_eq!(stored.1, "");
        assert_eq!(stored.2, "captured_text");
        assert_eq!(stored.3, raw);
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn story_seed_classification_is_idempotent_and_role_free() {
        let path = vault();
        let captured = create_capture_source(
            &path,
            "I led a difficult launch and changed the release process.",
            SourceType::Other,
        )
        .unwrap();
        let first =
            classify_capture_source(&path, &captured.source_id, CaptureRole::StorySeed).unwrap();
        let second =
            classify_capture_source(&path, &captured.source_id, CaptureRole::StorySeed).unwrap();
        assert!(first.created);
        assert!(!second.created);
        assert_eq!(first.target_id, second.target_id);

        let connection = open_connection(&path).unwrap();
        let seed_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM story_seeds WHERE seed_id=?1",
                [&first.target_id],
                |row| row.get(0),
            )
            .unwrap();
        let relation_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM record_relationships
                 WHERE from_id=?1 AND relationship_type='capture_story_seed'",
                [&captured.source_id],
                |row| row.get(0),
            )
            .unwrap();
        let role_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM record_relationships
                 WHERE from_id=?1 AND relationship_type='seed_role_context'",
                [&first.target_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!((seed_count, relation_count, role_count), (1, 1, 0));
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn target_inspiration_and_writing_sample_boundaries_hold() {
        let path = vault();
        let job = create_capture_source(
            &path,
            "Senior Product Manager - improve clinical workflows",
            SourceType::JobDescription,
        )
        .unwrap();
        assert_eq!(list_unclassified_captures(&path, 10).unwrap().len(), 1);
        classify_capture_source(&path, &job.source_id, CaptureRole::TargetContext).unwrap();
        assert!(list_unclassified_captures(&path, 10).unwrap().is_empty());

        let external = create_capture_source(
            &path,
            "An external article with an interesting counterpoint.",
            SourceType::Other,
        )
        .unwrap();
        classify_capture_source(&path, &external.source_id, CaptureRole::Inspiration).unwrap();

        let writing = create_capture_source(
            &path,
            "A user-authored paragraph saved for possible future voice review.",
            SourceType::WritingSample,
        )
        .unwrap();
        assert!(get_capture_source(&path, &writing.source_id)
            .unwrap()
            .classifications
            .is_empty());

        let connection = open_connection(&path).unwrap();
        let evidence_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM evidence_records", [], |row| {
                row.get(0)
            })
            .unwrap();
        let target_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM target_contexts", [], |row| row.get(0))
            .unwrap();
        let inspiration_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM inspirations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(evidence_count, 0);
        assert_eq!(target_count, 1);
        assert_eq!(inspiration_count, 1);
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }
}
