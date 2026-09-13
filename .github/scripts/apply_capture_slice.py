from pathlib import Path
from textwrap import dedent


def replace_once(path: str, before: str, after: str) -> None:
    file_path = Path(path)
    text = file_path.read_text(encoding="utf-8")
    if after in text:
        return
    count = text.count(before)
    if count != 1:
        raise SystemExit(f"Expected one match in {path}, found {count}: {before[:100]!r}")
    file_path.write_text(text.replace(before, after, 1), encoding="utf-8")


def replace_between(path: str, start: str, end: str, replacement: str) -> None:
    file_path = Path(path)
    text = file_path.read_text(encoding="utf-8")
    start_index = text.find(start)
    if start_index < 0:
        raise SystemExit(f"Start marker missing in {path}: {start!r}")
    end_index = text.find(end, start_index)
    if end_index < 0:
        raise SystemExit(f"End marker missing in {path}: {end!r}")
    file_path.write_text(text[:start_index] + replacement + text[end_index:], encoding="utf-8")


canonical = "src-tauri/src/services/canonical_store.rs"
replace_once(canonical, "const CURRENT_SCHEMA_VERSION: i64 = 1;", "const CURRENT_SCHEMA_VERSION: i64 = 2;")

schema_anchor = 'CREATE INDEX idx_lineage_legacy ON migration_lineage(legacy_type, legacy_id);\n"#;\n'
schema_v2 = dedent(
    '''

    const SCHEMA_V2: &str = r#"
    ALTER TABLE sources ADD COLUMN source_origin TEXT NOT NULL DEFAULT 'imported_file';
    ALTER TABLE sources ADD COLUMN captured_text TEXT;
    CREATE INDEX idx_sources_origin_imported_at ON sources(source_origin, imported_at DESC);
    "#;
    '''
)
replace_once(canonical, schema_anchor, schema_anchor + schema_v2)

new_migrate = dedent(
    '''
    fn migrate_schema(connection: &mut Connection) -> ServiceResult<()> {
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations(
            version INTEGER PRIMARY KEY,name TEXT NOT NULL,applied_at TEXT NOT NULL);",
        )?;
        let mut version: i64 = connection.query_row(
            "SELECT COALESCE(MAX(version),0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )?;
        if version > CURRENT_SCHEMA_VERSION {
            return Err(WorkLoreError::InvalidVault(format!(
                "Canonical database schema version {version} is newer than this WorkLore build supports."
            )));
        }
        if version == 0 {
            let tx = connection.transaction()?;
            tx.execute_batch(SCHEMA_V1)?;
            tx.execute(
                "INSERT INTO schema_migrations(version,name,applied_at) VALUES (1,'professional_memory_v1',?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
            version = 1;
        }
        if version == 1 {
            let tx = connection.transaction()?;
            tx.execute_batch(SCHEMA_V2)?;
            tx.execute(
                "INSERT INTO schema_migrations(version,name,applied_at) VALUES (2,'capture_text_sources_v2',?1)",
                [Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        Ok(())
    }

    '''
)
replace_between(canonical, "fn migrate_schema(connection: &mut Connection) -> ServiceResult<()> {", "fn migrate_source(", new_migrate)
replace_once(canonical, "assert_eq!(schema_version(&p).unwrap(), 1);", "assert_eq!(schema_version(&p).unwrap(), 2);")

replace_once(
    "src-tauri/src/services/mod.rs",
    "pub mod canonical_store;\n",
    "pub mod canonical_store;\npub mod capture_service;\n",
)
replace_once(
    "src-tauri/src/commands/mod.rs",
    "pub mod candidates;\n",
    "pub mod candidates;\npub mod capture;\n",
)
replace_once(
    "src-tauri/src/lib.rs",
    "use commands::{\n    candidates::{extract_resume_candidates, list_story_candidates, set_story_candidate_status},\n",
    "use commands::{\n    candidates::{extract_resume_candidates, list_story_candidates, set_story_candidate_status},\n    capture::{classify_capture_source, create_capture_source, get_capture_source, list_unclassified_captures},\n",
)
replace_once(
    "src-tauri/src/lib.rs",
    "            create_default_vault,\n            open_vault,\n",
    "            create_default_vault,\n            create_capture_source,\n            get_capture_source,\n            list_unclassified_captures,\n            classify_capture_source,\n            open_vault,\n",
)

capture_service = dedent(
    r'''
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
                .query_row("SELECT COUNT(*) FROM evidence_records", [], |row| row.get(0))
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
    '''
)
Path("src-tauri/src/services/capture_service.rs").write_text(capture_service, encoding="utf-8")

capture_commands = dedent(
    r'''
    use std::path::PathBuf;

    use crate::{
        domain::models::SourceType,
        error::{CommandError, CommandResult},
        services::capture_service::{
            self, CaptureClassificationResult, CaptureRole, CaptureSourceView,
        },
    };

    #[tauri::command]
    pub fn create_capture_source(
        vault_path: String,
        text: String,
        source_type: Option<SourceType>,
    ) -> CommandResult<CaptureSourceView> {
        capture_service::create_capture_source(
            &PathBuf::from(vault_path),
            &text,
            source_type.unwrap_or(SourceType::Other),
        )
        .map_err(CommandError::from)
    }

    #[tauri::command]
    pub fn get_capture_source(
        vault_path: String,
        source_id: String,
    ) -> CommandResult<CaptureSourceView> {
        capture_service::get_capture_source(&PathBuf::from(vault_path), &source_id)
            .map_err(CommandError::from)
    }

    #[tauri::command]
    pub fn list_unclassified_captures(
        vault_path: String,
        limit: Option<u32>,
    ) -> CommandResult<Vec<CaptureSourceView>> {
        capture_service::list_unclassified_captures(
            &PathBuf::from(vault_path),
            limit.unwrap_or(8) as usize,
        )
        .map_err(CommandError::from)
    }

    #[tauri::command]
    pub fn classify_capture_source(
        vault_path: String,
        source_id: String,
        role: CaptureRole,
    ) -> CommandResult<CaptureClassificationResult> {
        capture_service::classify_capture_source(
            &PathBuf::from(vault_path),
            &source_id,
            role,
        )
        .map_err(CommandError::from)
    }
    '''
)
Path("src-tauri/src/commands/capture.rs").write_text(capture_commands, encoding="utf-8")

source_type_block = dedent(
    '''
    export type SourceType =
      | "resume"
      | "job_description"
      | "writing_sample"
      | "interview_transcript"
      | "git_snapshot"
      | "other";
    '''
).lstrip()
capture_types = source_type_block + dedent(
    '''

    export type CaptureRole =
      | "story_seed"
      | "proof_point"
      | "topic_candidate"
      | "inspiration"
      | "target_context";

    export interface CaptureClassification {
      role: CaptureRole;
      targetId: string;
    }

    export interface CaptureSource {
      sourceId: string;
      sourceType: SourceType;
      displayName: string;
      text: string;
      createdAt: string;
      updatedAt: string;
      classifications: CaptureClassification[];
    }

    export interface CaptureClassificationResult {
      sourceId: string;
      role: CaptureRole;
      targetId: string;
      created: boolean;
    }
    '''
)
replace_once("src/domain/types.ts", source_type_block, capture_types)

replace_once(
    "src/lib/workloreApi.ts",
    "  CandidateStatus,\n  CandidateSummary,\n",
    "  CandidateStatus,\n  CandidateSummary,\n  CaptureClassificationResult,\n  CaptureRole,\n  CaptureSource,\n",
)
list_sources_block = dedent(
    '''
    export async function listSources(vaultPath: string): Promise<SourceSummary[]> {
      return invoke<SourceSummary[]>("list_sources", { vaultPath });
    }
    '''
).lstrip()
capture_api = list_sources_block + dedent(
    '''

    export async function createCaptureSource(
      vaultPath: string,
      text: string,
      sourceType: SourceType = "other",
    ): Promise<CaptureSource> {
      return invoke<CaptureSource>("create_capture_source", { vaultPath, text, sourceType });
    }

    export async function getCaptureSource(
      vaultPath: string,
      sourceId: string,
    ): Promise<CaptureSource> {
      return invoke<CaptureSource>("get_capture_source", { vaultPath, sourceId });
    }

    export async function listUnclassifiedCaptures(
      vaultPath: string,
      limit = 8,
    ): Promise<CaptureSource[]> {
      return invoke<CaptureSource[]>("list_unclassified_captures", { vaultPath, limit });
    }

    export async function classifyCaptureSource(
      vaultPath: string,
      sourceId: string,
      role: CaptureRole,
    ): Promise<CaptureClassificationResult> {
      return invoke<CaptureClassificationResult>("classify_capture_source", {
        vaultPath,
        sourceId,
        role,
      });
    }
    '''
)
replace_once("src/lib/workloreApi.ts", list_sources_block, capture_api)

capture_panel = dedent(
    r'''
    import { useEffect, useState } from "react";
    import type { CaptureRole, CaptureSource, SourceType } from "../domain/types";
    import { errorMessage } from "../domain/types";
    import {
      classifyCaptureSource,
      createCaptureSource,
      getCaptureSource,
      listUnclassifiedCaptures,
    } from "../lib/workloreApi";
    import "../capture.css";

    const CAPTURE_SOURCE_TYPES: Array<{ value: SourceType; label: string }> = [
      { value: "other", label: "Note or pasted text" },
      { value: "job_description", label: "Job description" },
      { value: "writing_sample", label: "Writing sample" },
    ];

    const CAPTURE_ROLES: Array<{ value: CaptureRole; label: string }> = [
      { value: "story_seed", label: "Story seed" },
      { value: "proof_point", label: "Proof point" },
      { value: "topic_candidate", label: "Topic idea" },
      { value: "inspiration", label: "Inspiration" },
      { value: "target_context", label: "Target context" },
    ];

    export function CapturePanel({ vaultPath }: { vaultPath: string }) {
      const [text, setText] = useState("");
      const [sourceType, setSourceType] = useState<SourceType>("other");
      const [saved, setSaved] = useState<CaptureSource | null>(null);
      const [recent, setRecent] = useState<CaptureSource[]>([]);
      const [busy, setBusy] = useState<string | null>(null);
      const [notice, setNotice] = useState<string | null>(null);
      const [error, setError] = useState<string | null>(null);

      useEffect(() => {
        setSaved(null);
        setNotice(null);
        setError(null);
        void refreshRecent();
      }, [vaultPath]);

      async function refreshRecent() {
        try {
          setRecent(await listUnclassifiedCaptures(vaultPath, 8));
        } catch (caught) {
          setError(errorMessage(caught));
        }
      }

      async function handleSave() {
        if (!text.trim()) {
          return;
        }
        setBusy("Saving capture");
        setNotice(null);
        setError(null);
        try {
          const created = await createCaptureSource(vaultPath, text, sourceType);
          setSaved(created);
          setText("");
          setNotice("Saved locally. Classification is optional and happens after save.");
          await refreshRecent();
        } catch (caught) {
          setError(errorMessage(caught));
        } finally {
          setBusy(null);
        }
      }

      async function handleClassify(role: CaptureRole) {
        if (!saved) {
          return;
        }
        setBusy("Connecting capture");
        setNotice(null);
        setError(null);
        try {
          const result = await classifyCaptureSource(vaultPath, saved.sourceId, role);
          setSaved(await getCaptureSource(vaultPath, saved.sourceId));
          setNotice(
            result.created
              ? `Connected as ${roleLabel(role)}.`
              : `This capture is already connected as ${roleLabel(role)}.`,
          );
          await refreshRecent();
        } catch (caught) {
          setError(errorMessage(caught));
        } finally {
          setBusy(null);
        }
      }

      async function handleSelectRecent(sourceId: string) {
        setBusy("Opening capture");
        setError(null);
        try {
          setSaved(await getCaptureSource(vaultPath, sourceId));
        } catch (caught) {
          setError(errorMessage(caught));
        } finally {
          setBusy(null);
        }
      }

      return (
        <section className="workspace-panel capture-panel" aria-labelledby="capture-heading">
          <div className="panel-heading-row capture-heading-row">
            <div>
              <p className="eyebrow">Capture</p>
              <h2 id="capture-heading">Save it before you sort it</h2>
              <p className="capture-intro">
                Paste a memory, result, idea, question, job description, writing sample, or note.
                WorkLore saves the original text first. Classification is optional.
              </p>
            </div>
            <select
              aria-label="Capture source type"
              value={sourceType}
              onChange={(event) => setSourceType(event.target.value as SourceType)}
            >
              {CAPTURE_SOURCE_TYPES.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <textarea
            className="capture-input"
            aria-label="Capture text"
            placeholder="What happened, what did you notice, or what should future-you remember?"
            value={text}
            onChange={(event) => setText(event.target.value)}
            rows={6}
          />
          <div className="capture-save-row">
            <button
              className="primary-button"
              disabled={!text.trim() || busy !== null}
              onClick={() => void handleSave()}
            >
              Save capture
            </button>
            <span className="capture-save-rule">No AI or classification is required to save.</span>
          </div>

          {saved ? (
            <div className="capture-saved-card">
              <div className="capture-saved-heading">
                <div>
                  <span className="status-pill">Saved</span>
                  <h3>{saved.displayName}</h3>
                  <p>{saved.sourceId}</p>
                </div>
                <button className="quiet-button compact" onClick={() => setSaved(null)}>
                  Done
                </button>
              </div>
              <p className="capture-preview">{saved.text}</p>
              <div className="capture-classify">
                <strong>Optional connections</strong>
                <div className="capture-classify-actions">
                  {CAPTURE_ROLES.map((option) => {
                    const linked = saved.classifications.some(
                      (classification) => classification.role === option.value,
                    );
                    return (
                      <button
                        className={linked ? "quiet-button compact" : "secondary-button compact"}
                        disabled={linked || busy !== null}
                        key={option.value}
                        onClick={() => void handleClassify(option.value)}
                      >
                        {linked ? `${option.label} linked` : option.label}
                      </button>
                    );
                  })}
                </div>
                <p className="capture-source-only">
                  Leave it alone to keep this as Source-only material. Writing samples do not become Voice Evidence here.
                </p>
              </div>
            </div>
          ) : null}

          <div className="capture-feedback" aria-live="polite">
            {busy ? <span>{busy}...</span> : null}
            {notice ? <span className="capture-notice">{notice}</span> : null}
            {error ? <span className="capture-error">{error}</span> : null}
          </div>

          {recent.length > 0 ? (
            <div className="capture-recent">
              <div className="capture-recent-heading">
                <h3>Recent unclassified captures</h3>
                <span>{recent.length} shown</span>
              </div>
              <div className="capture-recent-list">
                {recent.map((capture) => (
                  <button
                    className="capture-recent-item"
                    key={capture.sourceId}
                    onClick={() => void handleSelectRecent(capture.sourceId)}
                  >
                    <strong>{capture.displayName}</strong>
                    <span>{formatDate(capture.createdAt)}</span>
                  </button>
                ))}
              </div>
            </div>
          ) : null}
        </section>
      );
    }

    function roleLabel(role: CaptureRole): string {
      return CAPTURE_ROLES.find((option) => option.value === role)?.label.toLowerCase() ?? role;
    }

    function formatDate(value: string): string {
      const date = new Date(value);
      if (Number.isNaN(date.getTime())) {
        return value;
      }
      return new Intl.DateTimeFormat("en-US", {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(date);
    }
    '''
)
Path("src/components/CapturePanel.tsx").write_text(capture_panel, encoding="utf-8")

capture_css = dedent(
    r'''
    .capture-panel {
      grid-column: 1 / -1;
      padding: 1rem;
    }

    .capture-heading-row {
      align-items: center;
    }

    .capture-intro {
      max-width: 78ch;
      margin-bottom: 0;
    }

    .capture-input {
      width: 100%;
      min-height: 8.5rem;
      resize: vertical;
      padding: 0.85rem 0.9rem;
      border: 1px solid #596052;
      border-radius: 10px;
      color: #eef1ec;
      background: #151815;
      font: inherit;
      line-height: 1.5;
    }

    .capture-input:focus-visible {
      outline: 3px solid rgba(222, 181, 98, 0.72);
      outline-offset: 2px;
    }

    .capture-save-row,
    .capture-saved-heading,
    .capture-recent-heading {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 0.75rem;
    }

    .capture-save-row {
      margin-top: 0.65rem;
    }

    .capture-save-rule,
    .capture-feedback,
    .capture-recent-heading span {
      color: #929b8f;
      font-size: 0.78rem;
    }

    .capture-saved-card {
      margin-top: 1rem;
      padding: 0.9rem;
      border: 1px solid #4d5548;
      border-radius: 10px;
      background: #1c201b;
    }

    .capture-saved-heading h3 {
      margin-top: 0.35rem;
    }

    .capture-saved-heading p,
    .capture-source-only {
      margin-bottom: 0;
      color: #929b8f;
      font-size: 0.78rem;
    }

    .capture-preview {
      margin: 0.8rem 0;
      padding: 0.75rem;
      border-left: 3px solid #897146;
      color: #dce2d9;
      background: #24271f;
      white-space: pre-wrap;
    }

    .capture-classify-actions {
      display: flex;
      flex-wrap: wrap;
      gap: 0.45rem;
      margin: 0.55rem 0;
    }

    .capture-feedback {
      min-height: 1.4rem;
      margin-top: 0.65rem;
    }

    .capture-notice {
      color: #cfe3c8;
    }

    .capture-error {
      color: #ffd4cd;
    }

    .capture-recent {
      margin-top: 0.75rem;
      padding-top: 0.75rem;
      border-top: 1px solid #3c4238;
    }

    .capture-recent-heading h3 {
      margin-bottom: 0.4rem;
    }

    .capture-recent-list {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 0.45rem;
    }

    .capture-recent-item {
      display: flex;
      flex-direction: column;
      align-items: flex-start;
      gap: 0.2rem;
      padding: 0.65rem 0.7rem;
      border: 1px solid #3c4238;
      border-radius: 8px;
      color: #e4e8e1;
      background: #1a1e19;
      text-align: left;
    }

    .capture-recent-item:hover {
      border-color: #777f70;
      background: #242923;
    }

    .capture-recent-item span {
      color: #929b8f;
      font-size: 0.72rem;
    }

    @media (max-width: 700px) {
      .capture-heading-row,
      .capture-save-row,
      .capture-saved-heading {
        align-items: stretch;
        flex-direction: column;
      }
    }
    '''
)
Path("src/capture.css").write_text(capture_css, encoding="utf-8")

replace_once(
    "src/App.tsx",
    'import { open } from "@tauri-apps/plugin-dialog";\n',
    'import { open } from "@tauri-apps/plugin-dialog";\nimport { CapturePanel } from "./components/CapturePanel";\n',
)
replace_once(
    "src/App.tsx",
    '        <div className="workspace-grid">\n',
    '        <div className="workspace-grid">\n          <CapturePanel vaultPath={vault.path} />\n\n',
)
replace_once(
    "src/App.tsx",
    dedent(
        '''
                    <p>
                      Work-history bullets can now move through interview, privacy-safe synthesis,
                      validated response import, role linking, and canonical story storage.
                    </p>
        '''
    ).lstrip("\n"),
    dedent(
        '''
                    <p>
                      Typed and pasted material can now be saved before classification, then connected
                      to a Story Seed, Proof Point, Topic Candidate, Inspiration, or Target Context.
                    </p>
        '''
    ).lstrip("\n"),
)
