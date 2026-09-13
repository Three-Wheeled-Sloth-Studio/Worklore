from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8", newline="\n")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


# Schema v5: Target Context working fields.
path = "src-tauri/src/services/canonical_store.rs"
text = read(path)
text = replace_once(text, "const CURRENT_SCHEMA_VERSION: i64 = 4;", "const CURRENT_SCHEMA_VERSION: i64 = 5;", "schema version")
schema_v4 = '''const SCHEMA_V4: &str = r#"
ALTER TABLE inspirations ADD COLUMN source_url TEXT;
ALTER TABLE inspirations ADD COLUMN source_title TEXT;
ALTER TABLE inspirations ADD COLUMN source_author TEXT;
ALTER TABLE inspirations ADD COLUMN source_published_at TEXT;
ALTER TABLE inspirations ADD COLUMN summary TEXT NOT NULL DEFAULT '';
ALTER TABLE inspirations ADD COLUMN takeaways_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE inspirations ADD COLUMN excerpts_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE inspirations ADD COLUMN why_interesting TEXT NOT NULL DEFAULT '';
ALTER TABLE inspirations ADD COLUMN user_reaction TEXT NOT NULL DEFAULT '';
ALTER TABLE inspirations ADD COLUMN concepts_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE inspirations ADD COLUMN questions_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE inspirations ADD COLUMN counterpoints_json TEXT NOT NULL DEFAULT '[]';
CREATE INDEX idx_inspirations_status_updated ON inspirations(status, updated_at DESC);
CREATE INDEX idx_inspirations_source ON inspirations(source_id);
"#;
'''
schema_v5 = '''
const SCHEMA_V5: &str = r#"
ALTER TABLE target_contexts ADD COLUMN source_url TEXT;
ALTER TABLE target_contexts ADD COLUMN organization_name TEXT;
ALTER TABLE target_contexts ADD COLUMN role_title TEXT;
ALTER TABLE target_contexts ADD COLUMN location TEXT;
ALTER TABLE target_contexts ADD COLUMN summary TEXT NOT NULL DEFAULT '';
ALTER TABLE target_contexts ADD COLUMN responsibilities_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE target_contexts ADD COLUMN skills_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE target_contexts ADD COLUMN concepts_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE target_contexts ADD COLUMN language_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE target_contexts ADD COLUMN tensions_json TEXT NOT NULL DEFAULT '[]';
CREATE INDEX idx_target_contexts_status_type ON target_contexts(status, context_type, updated_at DESC);
CREATE INDEX idx_target_contexts_source ON target_contexts(source_id);
"#;
'''
text = replace_once(text, schema_v4, schema_v4 + schema_v5, "schema v5 insertion")
old_v4 = '''    if version == 3 {
        let tx = connection.transaction()?;
        tx.execute_batch(SCHEMA_V4)?;
        tx.execute(
            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (4,'inspiration_working_fields_v4',?1)",
            [Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
    }
    Ok(())
}
'''
new_v5_migration = '''    if version == 3 {
        let tx = connection.transaction()?;
        tx.execute_batch(SCHEMA_V4)?;
        tx.execute(
            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (4,'inspiration_working_fields_v4',?1)",
            [Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
        version = 4;
    }
    if version == 4 {
        let tx = connection.transaction()?;
        tx.execute_batch(SCHEMA_V5)?;
        tx.execute(
            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (5,'target_context_working_fields_v5',?1)",
            [Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
    }
    Ok(())
}
'''
text = replace_once(text, old_v4, new_v5_migration, "schema migration v5")
text = text.replace("schema_version(&p).unwrap(), 4", "schema_version(&p).unwrap(), 5")
write(path, text)

# Existing Topic schema-version proof advances with the canonical store.
path = "src-tauri/src/services/topic_service.rs"
text = read(path)
text = text.replace("schema_version(&path).unwrap(), 4", "schema_version(&path).unwrap(), 5")
write(path, text)

service = r'''use std::{
    collections::HashMap,
    fs,
    path::{Component, Path},
    time::Duration,
};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    error::{ServiceResult, WorkLoreError},
    services::canonical_store,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetContextLifecycle {
    Active,
    Archived,
}

impl TargetContextLifecycle {
    fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Target Context lifecycle {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetContextRelationKind {
    Topic,
    Theme,
    Story,
}

impl TargetContextRelationKind {
    fn relationship_type(self) -> &'static str {
        match self {
            Self::Topic => "topic_target_context",
            Self::Theme => "target_theme",
            Self::Story => "target_story",
        }
    }

    fn target_type(self) -> &'static str {
        match self {
            Self::Topic => "topic_candidate",
            Self::Theme => "theme",
            Self::Story => "story",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetContextSourceView {
    pub source_id: String,
    pub source_type: String,
    pub display_name: String,
    pub source_origin: String,
    pub original_file_name: String,
    pub stored_path: String,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone)]
struct SourceRecord {
    view: TargetContextSourceView,
    captured_text: String,
    extraction_json: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetContextRelationshipView {
    pub relationship_id: String,
    pub relation_kind: TargetContextRelationKind,
    pub target_id: String,
    pub target_label: String,
    pub target_detail: String,
    pub target_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetContextRecordView {
    pub target_id: String,
    pub source: Option<TargetContextSourceView>,
    pub context_type: String,
    pub title: String,
    pub lifecycle: TargetContextLifecycle,
    pub source_url: Option<String>,
    pub organization_name: Option<String>,
    pub role_title: Option<String>,
    pub location: Option<String>,
    pub summary: String,
    pub responsibilities: Vec<String>,
    pub skills: Vec<String>,
    pub concepts: Vec<String>,
    pub language: Vec<String>,
    pub tensions: Vec<String>,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
    pub relationships: Vec<TargetContextRelationshipView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTargetContextResult {
    pub target_context: TargetContextRecordView,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTargetContextRequest {
    pub target_id: String,
    pub title: String,
    pub lifecycle: TargetContextLifecycle,
    pub source_url: Option<String>,
    pub organization_name: Option<String>,
    pub role_title: Option<String>,
    pub location: Option<String>,
    pub summary: String,
    pub responsibilities: Vec<String>,
    pub skills: Vec<String>,
    pub concepts: Vec<String>,
    pub language: Vec<String>,
    pub tensions: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetContextExtractionResult {
    pub target_context: TargetContextRecordView,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetContextLinkTargetView {
    pub relation_kind: TargetContextRelationKind,
    pub target_id: String,
    pub label: String,
    pub detail: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetContextRelationshipMutationResult {
    pub target_context_id: String,
    pub relation_kind: TargetContextRelationKind,
    pub target_id: String,
    pub changed: bool,
}

#[derive(Debug, Default)]
struct DerivedSignals {
    responsibilities: Vec<String>,
    skills: Vec<String>,
    concepts: Vec<String>,
    language: Vec<String>,
    tensions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Other,
    Responsibilities,
    Skills,
    Tensions,
}

pub fn create_target_context_from_source(
    vault_path: &Path,
    source_id: &str,
) -> ServiceResult<CreateTargetContextResult> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let source = load_source_record(&connection, source_id)?.ok_or(WorkLoreError::SourceNotFound)?;

    if let Some(existing_id) = connection
        .query_row(
            "SELECT target_id FROM target_contexts WHERE source_id=?1 ORDER BY created_at LIMIT 1",
            [source_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(CreateTargetContextResult {
            target_context: load_target_context_from_connection(&connection, &existing_id)?,
            created: false,
        });
    }

    let target_id = format!("target_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    let context_type = if source.view.source_type == "job_description" {
        "job_description"
    } else {
        "captured_context"
    };
    let source_url = source
        .view
        .source_url
        .clone()
        .or_else(|| url_from_text(&source.captured_text));

    connection.execute(
        "INSERT INTO target_contexts(
           target_id,source_id,context_type,title,notes,status,provenance_json,created_at,updated_at,revision,source_url)
         VALUES (?1,?2,?3,?4,'','active',?5,?6,?6,1,?7)",
        params![
            &target_id,
            source_id,
            context_type,
            &source.view.display_name,
            json!({
                "creationActor": "user",
                "creationPath": "target_context_from_source",
                "sourceId": source_id
            })
            .to_string(),
            &now,
            &source_url,
        ],
    )?;
    audit(
        &connection,
        "target_context_created",
        &target_id,
        json!({"sourceId": source_id, "contextType": context_type}),
    )?;

    Ok(CreateTargetContextResult {
        target_context: load_target_context_from_connection(&connection, &target_id)?,
        created: true,
    })
}

pub fn load_target_context(
    vault_path: &Path,
    target_id: &str,
) -> ServiceResult<TargetContextRecordView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    load_target_context_from_connection(&connection, target_id)
}

pub fn list_target_contexts(vault_path: &Path) -> ServiceResult<Vec<TargetContextRecordView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let ids = {
        let mut statement = connection.prepare(
            "SELECT target_id FROM target_contexts ORDER BY updated_at DESC, created_at DESC",
        )?;
        statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?
    };
    ids.into_iter()
        .map(|id| load_target_context_from_connection(&connection, &id))
        .collect()
}

pub fn update_target_context(
    vault_path: &Path,
    request: UpdateTargetContextRequest,
) -> ServiceResult<TargetContextRecordView> {
    canonical_store::initialize(vault_path)?;
    let title = required_text(&request.title, "Target Context title")?;
    let connection = open_connection(vault_path)?;
    let before = load_target_context_from_connection(&connection, &request.target_id)?;
    let source_url = optional_trimmed(request.source_url);
    let organization_name = optional_trimmed(request.organization_name);
    let role_title = optional_trimmed(request.role_title);
    let location = optional_trimmed(request.location);
    let responsibilities = normalize_string_list(request.responsibilities);
    let skills = normalize_string_list(request.skills);
    let concepts = normalize_string_list(request.concepts);
    let language = normalize_string_list(request.language);
    let tensions = normalize_string_list(request.tensions);
    let now = Utc::now().to_rfc3339();

    connection.execute(
        "UPDATE target_contexts SET
           title=?2,status=?3,source_url=?4,organization_name=?5,role_title=?6,location=?7,summary=?8,
           responsibilities_json=?9,skills_json=?10,concepts_json=?11,language_json=?12,tensions_json=?13,
           notes=?14,updated_at=?15,revision=revision+1
         WHERE target_id=?1",
        params![
            &request.target_id,
            &title,
            request.lifecycle.as_str(),
            &source_url,
            &organization_name,
            &role_title,
            &location,
            request.summary.trim(),
            serde_json::to_string(&responsibilities)?,
            serde_json::to_string(&skills)?,
            serde_json::to_string(&concepts)?,
            serde_json::to_string(&language)?,
            serde_json::to_string(&tensions)?,
            request.notes.trim(),
            &now,
        ],
    )?;
    audit(
        &connection,
        "target_context_updated",
        &request.target_id,
        json!({
            "previousLifecycle": before.lifecycle.as_str(),
            "lifecycle": request.lifecycle.as_str(),
            "responsibilityCount": responsibilities.len(),
            "skillCount": skills.len(),
            "conceptCount": concepts.len(),
            "languageCount": language.len(),
            "tensionCount": tensions.len()
        }),
    )?;
    load_target_context_from_connection(&connection, &request.target_id)
}

pub fn extract_target_context_signals(
    vault_path: &Path,
    target_id: &str,
) -> ServiceResult<TargetContextExtractionResult> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let before = load_target_context_from_connection(&connection, target_id)?;
    let source_id = before
        .source
        .as_ref()
        .map(|source| source.source_id.clone())
        .ok_or_else(|| {
            WorkLoreError::InvalidVault(
                "Target Context needs Source provenance before source signals can be extracted."
                    .to_string(),
            )
        })?;
    let source = load_source_record(&connection, &source_id)?.ok_or(WorkLoreError::SourceNotFound)?;
    let text = load_source_text(vault_path, &source)?;
    if text.trim().is_empty() {
        return Err(WorkLoreError::InvalidVault(
            "The attached Source does not contain extractable text.".to_string(),
        ));
    }

    let derived = derive_signals(&text);
    let responsibilities = merge_unique(&before.responsibilities, derived.responsibilities);
    let skills = merge_unique(&before.skills, derived.skills);
    let concepts = merge_unique(&before.concepts, derived.concepts);
    let language = merge_unique(&before.language, derived.language);
    let tensions = merge_unique(&before.tensions, derived.tensions);
    let changed = responsibilities != before.responsibilities
        || skills != before.skills
        || concepts != before.concepts
        || language != before.language
        || tensions != before.tensions;

    if !changed {
        return Ok(TargetContextExtractionResult {
            target_context: before,
            changed: false,
        });
    }

    let now = Utc::now().to_rfc3339();
    connection.execute(
        "UPDATE target_contexts SET responsibilities_json=?2,skills_json=?3,concepts_json=?4,
           language_json=?5,tensions_json=?6,updated_at=?7,revision=revision+1 WHERE target_id=?1",
        params![
            target_id,
            serde_json::to_string(&responsibilities)?,
            serde_json::to_string(&skills)?,
            serde_json::to_string(&concepts)?,
            serde_json::to_string(&language)?,
            serde_json::to_string(&tensions)?,
            &now,
        ],
    )?;
    audit(
        &connection,
        "target_context_signals_extracted",
        target_id,
        json!({
            "extractor": "deterministic-target-context-v1",
            "sourceId": source_id,
            "responsibilityCount": responsibilities.len(),
            "skillCount": skills.len(),
            "conceptCount": concepts.len(),
            "languageCount": language.len(),
            "tensionCount": tensions.len()
        }),
    )?;
    Ok(TargetContextExtractionResult {
        target_context: load_target_context_from_connection(&connection, target_id)?,
        changed: true,
    })
}

pub fn add_target_context_relationship(
    vault_path: &Path,
    target_context_id: &str,
    relation_kind: TargetContextRelationKind,
    related_id: &str,
) -> ServiceResult<TargetContextRelationshipMutationResult> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    ensure_target_context_exists(&connection, target_context_id)?;
    ensure_relation_target_exists(&connection, relation_kind, related_id)?;
    let now = Utc::now().to_rfc3339();
    let inserted = match relation_kind {
        TargetContextRelationKind::Topic => connection.execute(
            "INSERT OR IGNORE INTO record_relationships(
               relationship_id,from_type,from_id,relationship_type,to_type,to_id,provenance_json,created_at)
             VALUES (?1,'topic_candidate',?2,'topic_target_context','target_context',?3,?4,?5)",
            params![
                format!("relationship_{}", Uuid::now_v7()),
                related_id,
                target_context_id,
                json!({"creationActor":"user"}).to_string(),
                &now,
            ],
        )?,
        TargetContextRelationKind::Theme => connection.execute(
            "INSERT OR IGNORE INTO record_relationships(
               relationship_id,from_type,from_id,relationship_type,to_type,to_id,provenance_json,created_at)
             VALUES (?1,'target_context',?2,'target_theme','theme',?3,?4,?5)",
            params![
                format!("relationship_{}", Uuid::now_v7()),
                target_context_id,
                related_id,
                json!({"creationActor":"user"}).to_string(),
                &now,
            ],
        )?,
        TargetContextRelationKind::Story => connection.execute(
            "INSERT OR IGNORE INTO record_relationships(
               relationship_id,from_type,from_id,relationship_type,to_type,to_id,provenance_json,created_at)
             VALUES (?1,'target_context',?2,'target_story','story',?3,?4,?5)",
            params![
                format!("relationship_{}", Uuid::now_v7()),
                target_context_id,
                related_id,
                json!({"creationActor":"user"}).to_string(),
                &now,
            ],
        )?,
    };
    if inserted > 0 {
        audit(
            &connection,
            "target_context_relationship_added",
            target_context_id,
            json!({"relationKind": relation_kind, "targetId": related_id}),
        )?;
    }
    Ok(TargetContextRelationshipMutationResult {
        target_context_id: target_context_id.to_string(),
        relation_kind,
        target_id: related_id.to_string(),
        changed: inserted > 0,
    })
}

pub fn remove_target_context_relationship(
    vault_path: &Path,
    target_context_id: &str,
    relation_kind: TargetContextRelationKind,
    related_id: &str,
) -> ServiceResult<TargetContextRelationshipMutationResult> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    ensure_target_context_exists(&connection, target_context_id)?;
    let removed = match relation_kind {
        TargetContextRelationKind::Topic => connection.execute(
            "DELETE FROM record_relationships WHERE from_type='topic_candidate' AND from_id=?1
               AND relationship_type='topic_target_context' AND to_type='target_context' AND to_id=?2",
            params![related_id, target_context_id],
        )?,
        TargetContextRelationKind::Theme => connection.execute(
            "DELETE FROM record_relationships WHERE from_type='target_context' AND from_id=?1
               AND relationship_type='target_theme' AND to_type='theme' AND to_id=?2",
            params![target_context_id, related_id],
        )?,
        TargetContextRelationKind::Story => connection.execute(
            "DELETE FROM record_relationships WHERE from_type='target_context' AND from_id=?1
               AND relationship_type='target_story' AND to_type='story' AND to_id=?2",
            params![target_context_id, related_id],
        )?,
    };
    if removed > 0 {
        audit(
            &connection,
            "target_context_relationship_removed",
            target_context_id,
            json!({"relationKind": relation_kind, "targetId": related_id}),
        )?;
    }
    Ok(TargetContextRelationshipMutationResult {
        target_context_id: target_context_id.to_string(),
        relation_kind,
        target_id: related_id.to_string(),
        changed: removed > 0,
    })
}

pub fn list_target_context_link_targets(
    vault_path: &Path,
    relation_kind: TargetContextRelationKind,
) -> ServiceResult<Vec<TargetContextLinkTargetView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut out = Vec::new();
    match relation_kind {
        TargetContextRelationKind::Topic => {
            let mut statement = connection.prepare(
                "SELECT topic_id,title,summary,status FROM topic_candidates ORDER BY updated_at DESC,title",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(TargetContextLinkTargetView {
                    relation_kind,
                    target_id: row.get(0)?,
                    label: row.get(1)?,
                    detail: row.get(2)?,
                    status: row.get(3)?,
                })
            })?;
            out.extend(rows.collect::<Result<Vec<_>, _>>()?);
        }
        TargetContextRelationKind::Theme => {
            let mut statement = connection.prepare(
                "SELECT theme_id,name,description,status FROM themes ORDER BY updated_at DESC,name",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(TargetContextLinkTargetView {
                    relation_kind,
                    target_id: row.get(0)?,
                    label: row.get(1)?,
                    detail: row.get(2)?,
                    status: row.get(3)?,
                })
            })?;
            out.extend(rows.collect::<Result<Vec<_>, _>>()?);
        }
        TargetContextRelationKind::Story => {
            let mut statement = connection.prepare(
                "SELECT story_id,title,summary,lifecycle_status,maturity FROM stories ORDER BY updated_at DESC,title",
            )?;
            let rows = statement.query_map([], |row| {
                let lifecycle = row.get::<_, String>(3)?;
                let maturity = row.get::<_, String>(4)?;
                Ok(TargetContextLinkTargetView {
                    relation_kind,
                    target_id: row.get(0)?,
                    label: row.get(1)?,
                    detail: row.get(2)?,
                    status: format!("{lifecycle} · {maturity}"),
                })
            })?;
            out.extend(rows.collect::<Result<Vec<_>, _>>()?);
        }
    }
    Ok(out)
}

fn load_target_context_from_connection(
    connection: &Connection,
    target_id: &str,
) -> ServiceResult<TargetContextRecordView> {
    let raw = connection
        .query_row(
            "SELECT target_id,source_id,context_type,title,notes,status,source_url,organization_name,role_title,
                    location,summary,responsibilities_json,skills_json,concepts_json,language_json,tensions_json,
                    created_at,updated_at,revision
             FROM target_contexts WHERE target_id=?1",
            [target_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, String>(13)?,
                    row.get::<_, String>(14)?,
                    row.get::<_, String>(15)?,
                    row.get::<_, String>(16)?,
                    row.get::<_, String>(17)?,
                    row.get::<_, u32>(18)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| target_context_not_found(target_id))?;
    let source = raw
        .1
        .as_deref()
        .map(|source_id| load_source_record(connection, source_id))
        .transpose()?
        .flatten()
        .map(|record| record.view);
    Ok(TargetContextRecordView {
        target_id: raw.0,
        source,
        context_type: raw.2,
        title: raw.3,
        lifecycle: TargetContextLifecycle::parse(&raw.5)?,
        source_url: raw.6,
        organization_name: raw.7,
        role_title: raw.8,
        location: raw.9,
        summary: raw.10,
        responsibilities: parse_json_list(&raw.11)?,
        skills: parse_json_list(&raw.12)?,
        concepts: parse_json_list(&raw.13)?,
        language: parse_json_list(&raw.14)?,
        tensions: parse_json_list(&raw.15)?,
        notes: raw.4,
        created_at: raw.16,
        updated_at: raw.17,
        revision: raw.18,
        relationships: load_relationships(connection, target_id)?,
    })
}

fn load_source_record(connection: &Connection, source_id: &str) -> ServiceResult<Option<SourceRecord>> {
    let raw = connection
        .query_row(
            "SELECT source_id,source_type,display_name,source_origin,original_file_name,stored_path,
                    provenance_json,COALESCE(captured_text,''),extraction_json
             FROM sources WHERE source_id=?1",
            [source_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            },
        )
        .optional()?;
    Ok(raw.map(|row| SourceRecord {
        view: TargetContextSourceView {
            source_id: row.0,
            source_type: row.1,
            display_name: row.2,
            source_origin: row.3,
            original_file_name: row.4,
            stored_path: row.5,
            source_url: source_url_from_provenance(&row.6),
        },
        captured_text: row.7,
        extraction_json: row.8,
    }))
}

fn load_relationships(
    connection: &Connection,
    target_context_id: &str,
) -> ServiceResult<Vec<TargetContextRelationshipView>> {
    let mut out = Vec::new();
    {
        let mut statement = connection.prepare(
            "SELECT r.relationship_id,t.topic_id,t.title,t.summary,t.status
             FROM record_relationships r
             JOIN topic_candidates t ON t.topic_id=r.from_id
             WHERE r.from_type='topic_candidate' AND r.relationship_type='topic_target_context'
               AND r.to_type='target_context' AND r.to_id=?1
             ORDER BY r.created_at,r.relationship_id",
        )?;
        let rows = statement.query_map([target_context_id], |row| {
            Ok(TargetContextRelationshipView {
                relationship_id: row.get(0)?,
                relation_kind: TargetContextRelationKind::Topic,
                target_id: row.get(1)?,
                target_label: row.get(2)?,
                target_detail: row.get(3)?,
                target_status: row.get(4)?,
            })
        })?;
        out.extend(rows.collect::<Result<Vec<_>, _>>()?);
    }
    {
        let mut statement = connection.prepare(
            "SELECT r.relationship_id,t.theme_id,t.name,t.description,t.status
             FROM record_relationships r
             JOIN themes t ON t.theme_id=r.to_id
             WHERE r.from_type='target_context' AND r.from_id=?1
               AND r.relationship_type='target_theme' AND r.to_type='theme'
             ORDER BY r.created_at,r.relationship_id",
        )?;
        let rows = statement.query_map([target_context_id], |row| {
            Ok(TargetContextRelationshipView {
                relationship_id: row.get(0)?,
                relation_kind: TargetContextRelationKind::Theme,
                target_id: row.get(1)?,
                target_label: row.get(2)?,
                target_detail: row.get(3)?,
                target_status: row.get(4)?,
            })
        })?;
        out.extend(rows.collect::<Result<Vec<_>, _>>()?);
    }
    {
        let mut statement = connection.prepare(
            "SELECT r.relationship_id,s.story_id,s.title,s.summary,s.lifecycle_status,s.maturity
             FROM record_relationships r
             JOIN stories s ON s.story_id=r.to_id
             WHERE r.from_type='target_context' AND r.from_id=?1
               AND r.relationship_type='target_story' AND r.to_type='story'
             ORDER BY r.created_at,r.relationship_id",
        )?;
        let rows = statement.query_map([target_context_id], |row| {
            let lifecycle = row.get::<_, String>(4)?;
            let maturity = row.get::<_, String>(5)?;
            Ok(TargetContextRelationshipView {
                relationship_id: row.get(0)?,
                relation_kind: TargetContextRelationKind::Story,
                target_id: row.get(1)?,
                target_label: row.get(2)?,
                target_detail: row.get(3)?,
                target_status: format!("{lifecycle} · {maturity}"),
            })
        })?;
        out.extend(rows.collect::<Result<Vec<_>, _>>()?);
    }
    Ok(out)
}

fn ensure_target_context_exists(connection: &Connection, target_id: &str) -> ServiceResult<()> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM target_contexts WHERE target_id=?1)",
        [target_id],
        |row| row.get(0),
    )?;
    if exists {
        Ok(())
    } else {
        Err(target_context_not_found(target_id))
    }
}

fn ensure_relation_target_exists(
    connection: &Connection,
    relation_kind: TargetContextRelationKind,
    target_id: &str,
) -> ServiceResult<()> {
    let sql = match relation_kind {
        TargetContextRelationKind::Topic => {
            "SELECT EXISTS(SELECT 1 FROM topic_candidates WHERE topic_id=?1)"
        }
        TargetContextRelationKind::Theme => "SELECT EXISTS(SELECT 1 FROM themes WHERE theme_id=?1)",
        TargetContextRelationKind::Story => "SELECT EXISTS(SELECT 1 FROM stories WHERE story_id=?1)",
    };
    let exists: bool = connection.query_row(sql, [target_id], |row| row.get(0))?;
    if exists {
        Ok(())
    } else {
        Err(WorkLoreError::InvalidVault(format!(
            "{} target {target_id} was not found.",
            relation_kind.target_type()
        )))
    }
}

fn load_source_text(vault_path: &Path, source: &SourceRecord) -> ServiceResult<String> {
    if !source.captured_text.trim().is_empty() {
        return Ok(source.captured_text.clone());
    }
    let extraction: Value = serde_json::from_str(&source.extraction_json).unwrap_or(Value::Null);
    let Some(text_path) = extraction.get("textPath").and_then(Value::as_str) else {
        return Ok(String::new());
    };
    let relative = Path::new(text_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(WorkLoreError::InvalidPath);
    }
    Ok(fs::read_to_string(vault_path.join(relative))?)
}

fn derive_signals(text: &str) -> DerivedSignals {
    let mut signals = DerivedSignals::default();
    let mut section = Section::Other;
    let lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    for raw in &lines {
        let cleaned = strip_list_marker(raw);
        if let Some(next_section) = section_heading(&cleaned) {
            section = next_section;
            continue;
        }
        if cleaned.chars().count() < 4 || cleaned.chars().count() > 280 {
            continue;
        }
        match section {
            Section::Responsibilities => push_unique(&mut signals.responsibilities, &cleaned, 12),
            Section::Skills => push_unique(&mut signals.skills, &cleaned, 12),
            Section::Tensions => push_unique(&mut signals.tensions, &cleaned, 10),
            Section::Other => {}
        }
        if has_tension_language(&cleaned) {
            push_unique(&mut signals.tensions, &cleaned, 10);
        }
    }

    if signals.responsibilities.is_empty() {
        for raw in &lines {
            if is_list_item(raw) {
                let cleaned = strip_list_marker(raw);
                if starts_with_action_verb(&cleaned) {
                    push_unique(&mut signals.responsibilities, &cleaned, 12);
                }
            }
        }
    }
    if signals.skills.is_empty() {
        for raw in &lines {
            let cleaned = strip_list_marker(raw);
            if looks_like_skill_statement(&cleaned) {
                push_unique(&mut signals.skills, &cleaned, 12);
            }
        }
    }

    signals.language = salient_terms(text, 12);
    signals.concepts = salient_bigrams(text, 10);
    signals
}

fn section_heading(value: &str) -> Option<Section> {
    if value.chars().count() > 90 {
        return None;
    }
    let normalized = value
        .trim()
        .trim_end_matches(':')
        .replace('’', "'")
        .to_ascii_lowercase();
    match normalized.as_str() {
        "responsibilities" | "what you'll do" | "what you will do" | "your impact"
        | "what you do" | "the role" | "key responsibilities" => Some(Section::Responsibilities),
        "requirements" | "qualifications" | "skills" | "what you bring"
        | "what we're looking for" | "what we are looking for" | "preferred qualifications"
        | "minimum qualifications" => Some(Section::Skills),
        "challenges" | "tensions" | "problems to solve" | "what success looks like" => {
            Some(Section::Tensions)
        }
        _ => None,
    }
}

fn strip_list_marker(value: &str) -> String {
    let mut trimmed = value.trim();
    loop {
        let Some(first) = trimmed.chars().next() else {
            return String::new();
        };
        if matches!(first, '-' | '*' | '•' | '–' | '—') {
            trimmed = trimmed[first.len_utf8()..].trim_start();
        } else {
            break;
        }
    }
    if let Some((prefix, rest)) = trimmed.split_once(' ') {
        let number = prefix.trim_end_matches(|c: char| c == '.' || c == ')');
        if !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()) {
            return rest.trim().to_string();
        }
    }
    trimmed.to_string()
}

fn is_list_item(value: &str) -> bool {
    let trimmed = value.trim_start();
    let Some(first) = trimmed.chars().next() else {
        return false;
    };
    if matches!(first, '-' | '*' | '•' | '–' | '—') {
        return true;
    }
    trimmed
        .split_whitespace()
        .next()
        .map(|prefix| {
            let number = prefix.trim_end_matches(|c: char| c == '.' || c == ')');
            !number.is_empty() && number.chars().all(|c| c.is_ascii_digit())
        })
        .unwrap_or(false)
}

fn starts_with_action_verb(value: &str) -> bool {
    let first = value
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_ascii_lowercase();
    matches!(
        first.as_str(),
        "lead"
            | "own"
            | "build"
            | "partner"
            | "manage"
            | "develop"
            | "define"
            | "drive"
            | "create"
            | "turn"
            | "translate"
            | "collaborate"
            | "deliver"
            | "design"
            | "shape"
            | "evaluate"
            | "establish"
            | "scale"
            | "improve"
            | "reduce"
            | "increase"
            | "support"
            | "identify"
    )
}

fn looks_like_skill_statement(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "experience ",
        "experience with",
        "years of",
        "knowledge of",
        "proficiency",
        "ability to",
        "expertise",
        "background in",
        "familiarity",
        "required",
        "preferred",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn has_tension_language(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "balance ",
        "tradeoff",
        "trade-off",
        " while ",
        " without ",
        "ambiguity",
        "constraint",
        "challenge",
        "risk",
        "complex",
        "scale",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn salient_terms(text: &str, limit: usize) -> Vec<String> {
    let mut counts: HashMap<String, (usize, usize)> = HashMap::new();
    let mut position = 0usize;
    for token in text.split(|c: char| !c.is_alphanumeric() && c != '-') {
        let token = token.trim_matches('-').to_ascii_lowercase();
        if significant_word(&token) {
            let entry = counts.entry(token).or_insert((0, position));
            entry.0 += 1;
            position += 1;
        }
    }
    let mut ranked = counts.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .0
            .cmp(&left.1.0)
            .then_with(|| left.1.1.cmp(&right.1.1))
            .then_with(|| left.0.cmp(&right.0))
    });
    ranked
        .into_iter()
        .take(limit)
        .map(|(term, _)| term)
        .collect()
}

fn salient_bigrams(text: &str, limit: usize) -> Vec<String> {
    let mut counts: HashMap<String, (usize, usize)> = HashMap::new();
    let mut position = 0usize;
    for line in text.lines() {
        let tokens = line
            .split(|c: char| !c.is_alphanumeric() && c != '-')
            .map(|token| token.trim_matches('-').to_ascii_lowercase())
            .collect::<Vec<_>>();
        for pair in tokens.windows(2) {
            if significant_word(&pair[0]) && significant_word(&pair[1]) && pair[0] != pair[1] {
                let phrase = format!("{} {}", pair[0], pair[1]);
                let entry = counts.entry(phrase).or_insert((0, position));
                entry.0 += 1;
                position += 1;
            }
        }
    }
    let mut ranked = counts.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .0
            .cmp(&left.1.0)
            .then_with(|| left.1.1.cmp(&right.1.1))
            .then_with(|| left.0.cmp(&right.0))
    });
    ranked
        .into_iter()
        .take(limit)
        .map(|(phrase, _)| phrase)
        .collect()
}

fn significant_word(value: &str) -> bool {
    if value.len() < 4 || value.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    !matches!(
        value,
        "about"
            | "above"
            | "across"
            | "after"
            | "again"
            | "against"
            | "along"
            | "also"
            | "among"
            | "another"
            | "because"
            | "before"
            | "being"
            | "between"
            | "both"
            | "could"
            | "from"
            | "have"
            | "into"
            | "more"
            | "most"
            | "other"
            | "over"
            | "should"
            | "their"
            | "there"
            | "these"
            | "they"
            | "this"
            | "those"
            | "through"
            | "under"
            | "using"
            | "what"
            | "when"
            | "where"
            | "which"
            | "while"
            | "with"
            | "within"
            | "without"
            | "would"
            | "your"
            | "you'll"
            | "will"
            | "role"
            | "team"
            | "work"
    )
}

fn push_unique(target: &mut Vec<String>, value: &str, limit: usize) {
    let normalized = value.trim().to_string();
    if !normalized.is_empty() && target.len() < limit && !target.contains(&normalized) {
        target.push(normalized);
    }
}

fn merge_unique(existing: &[String], additions: Vec<String>) -> Vec<String> {
    let mut out = normalize_string_list(existing.to_vec());
    for addition in additions {
        let normalized = addition.trim().to_string();
        if !normalized.is_empty() && !out.contains(&normalized) {
            out.push(normalized);
        }
    }
    out
}

fn normalize_string_list(values: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in values {
        let normalized = value.trim().to_string();
        if !normalized.is_empty() && !out.contains(&normalized) {
            out.push(normalized);
        }
    }
    out
}

fn parse_json_list(value: &str) -> ServiceResult<Vec<String>> {
    Ok(serde_json::from_str(value)?)
}

fn optional_trimmed(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn required_text(value: &str, label: &str) -> ServiceResult<String> {
    let value = value.trim();
    if value.is_empty() {
        Err(WorkLoreError::InvalidVault(format!(
            "{label} cannot be empty."
        )))
    } else {
        Ok(value.to_string())
    }
}

fn source_url_from_provenance(value: &str) -> Option<String> {
    serde_json::from_str::<Value>(value)
        .ok()
        .and_then(|json| {
            json.get("sourceUrl")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .filter(|value| !value.trim().is_empty())
}

fn url_from_text(value: &str) -> Option<String> {
    value
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("https://") || line.starts_with("http://"))
        .map(str::to_string)
}

fn target_context_not_found(target_id: &str) -> WorkLoreError {
    WorkLoreError::InvalidVault(format!("Target Context {target_id} was not found."))
}

fn audit(
    connection: &Connection,
    event_type: &str,
    target_id: &str,
    details: Value,
) -> ServiceResult<()> {
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,?2,'target_context',?3,'user',?4,?5)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            event_type,
            target_id,
            details.to_string(),
            Utc::now().to_rfc3339(),
        ],
    )?;
    Ok(())
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    let connection = Connection::open(canonical_store::database_path(vault_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::models::SourceType,
        services::{
            capture_service,
            topic_service,
            topic_service::{CreateThemeRequest, CreateTopicRequest, TopicTimingClass},
            vault_service,
        },
    };
    use std::{fs, path::PathBuf};

    fn vault() -> PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-target-context-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Target Context Test").expect("create vault");
        path
    }

    fn row_count(connection: &Connection, table: &str) -> i64 {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        connection
            .query_row(&sql, [], |row| row.get(0))
            .expect("count rows")
    }

    fn captured_job(path: &Path) -> (String, String) {
        let source = capture_service::create_capture_source(
            path,
            "Synthetic product leadership job description.",
            SourceType::JobDescription,
        )
        .expect("capture job description");
        let result = capture_service::classify_capture_source(
            path,
            &source.source_id,
            capture_service::CaptureRole::TargetContext,
        )
        .expect("classify target context");
        (source.source_id, result.target_id)
    }

    #[test]
    fn captured_job_becomes_durable_target_context_and_structured_updates_preserve_identity() {
        let path = vault();
        let source = capture_service::create_capture_source(
            &path,
            "Synthetic Director of Product opportunity.",
            SourceType::JobDescription,
        )
        .expect("capture");
        let connection = open_connection(&path).expect("open database");
        assert_eq!(row_count(&connection, "target_contexts"), 0);
        drop(connection);

        let classified = capture_service::classify_capture_source(
            &path,
            &source.source_id,
            capture_service::CaptureRole::TargetContext,
        )
        .expect("classify");
        assert_eq!(canonical_store::schema_version(&path).unwrap(), 5);
        let before = load_target_context(&path, &classified.target_id).expect("load target context");
        assert_eq!(before.context_type, "job_description");
        assert_eq!(before.lifecycle, TargetContextLifecycle::Active);
        assert_eq!(before.source.as_ref().map(|source| source.source_id.as_str()), Some(source.source_id.as_str()));

        let updated = update_target_context(
            &path,
            UpdateTargetContextRequest {
                target_id: before.target_id.clone(),
                title: "Director of Product — synthetic".into(),
                lifecycle: TargetContextLifecycle::Active,
                source_url: Some("https://example.com/jobs/product".into()),
                organization_name: Some("Synthetic Health Co".into()),
                role_title: Some("Director of Product".into()),
                location: Some("Remote".into()),
                summary: "Mission-critical product leadership context.".into(),
                responsibilities: vec!["Lead continuous discovery".into()],
                skills: vec!["Product leadership".into()],
                concepts: vec!["forward-deployed discovery".into()],
                language: vec!["clinical intelligence".into()],
                tensions: vec!["Move quickly without compromising clinical safety".into()],
                notes: "Context only; not evidence about the user.".into(),
            },
        )
        .expect("update");
        assert_eq!(updated.target_id, before.target_id);
        assert_eq!(updated.revision, before.revision + 1);
        let reopened = load_target_context(&path, &before.target_id).expect("reopen");
        assert_eq!(reopened.organization_name.as_deref(), Some("Synthetic Health Co"));
        assert_eq!(reopened.skills, vec!["Product leadership"]);
        fs::remove_dir_all(path).expect("cleanup");
    }

    #[test]
    fn deterministic_signal_extraction_is_idempotent_and_never_creates_standing() {
        let path = vault();
        let text = "What You'll Do\n- Lead continuous discovery with providers and care teams.\n- Partner with Engineering to evaluate technical tradeoffs and iterate quickly.\n- Turn complex clinical workflows into clear product opportunities.\nWhat You Bring\n- Experience building and scaling product teams in healthcare.\n- Product leadership experience in regulated domains.\nChallenges\n- Balance rapid iteration with clinical safety and operational reliability.";
        let source = capture_service::create_capture_source(&path, text, SourceType::JobDescription)
            .expect("capture JD");
        let classified = capture_service::classify_capture_source(
            &path,
            &source.source_id,
            capture_service::CaptureRole::TargetContext,
        )
        .expect("classify");

        let first = extract_target_context_signals(&path, &classified.target_id).expect("extract");
        assert!(first.changed);
        assert!(first.target_context.responsibilities.len() >= 3);
        assert!(first.target_context.skills.len() >= 2);
        assert!(!first.target_context.concepts.is_empty());
        assert!(!first.target_context.language.is_empty());
        assert!(!first.target_context.tensions.is_empty());
        let second = extract_target_context_signals(&path, &classified.target_id).expect("extract again");
        assert!(!second.changed);
        assert_eq!(second.target_context.revision, first.target_context.revision);

        let connection = open_connection(&path).expect("database");
        assert_eq!(row_count(&connection, "evidence_records"), 0);
        assert_eq!(row_count(&connection, "proof_points"), 0);
        assert_eq!(row_count(&connection, "stories"), 0);
        assert_eq!(row_count(&connection, "topic_candidates"), 0);
        drop(connection);
        fs::remove_dir_all(path).expect("cleanup");
    }

    #[test]
    fn topic_theme_story_links_are_explicit_idempotent_and_removable_without_deleting_records() {
        let path = vault();
        let (_, target_id) = captured_job(&path);
        let topic = topic_service::create_topic(
            &path,
            CreateTopicRequest {
                title: "Product discovery".into(),
                summary: "How discovery works in complex domains.".into(),
                timing_class: TopicTimingClass::Evergreen,
                relevant_until: None,
                timely_note: None,
            },
        )
        .expect("topic");
        let theme = topic_service::create_theme(
            &path,
            CreateThemeRequest {
                name: "Product leadership".into(),
                description: "Synthetic theme".into(),
            },
        )
        .expect("theme");
        let story = canonical_store::create_story(&path, "A real user story", "Synthetic story summary")
            .expect("story");

        assert!(add_target_context_relationship(&path, &target_id, TargetContextRelationKind::Topic, &topic.topic_id).unwrap().changed);
        assert!(add_target_context_relationship(&path, &target_id, TargetContextRelationKind::Theme, &theme.theme_id).unwrap().changed);
        assert!(add_target_context_relationship(&path, &target_id, TargetContextRelationKind::Story, &story.story_id).unwrap().changed);
        assert!(!add_target_context_relationship(&path, &target_id, TargetContextRelationKind::Topic, &topic.topic_id).unwrap().changed);

        let loaded = load_target_context(&path, &target_id).expect("load");
        assert_eq!(loaded.relationships.len(), 3);
        let topic_reopened = topic_service::load_topic(&path, &topic.topic_id).expect("load topic");
        assert!(topic_reopened.relationships.iter().any(|relationship| {
            relationship.target_id == target_id && relationship.category == "target_context"
        }));

        let removed = remove_target_context_relationship(
            &path,
            &target_id,
            TargetContextRelationKind::Story,
            &story.story_id,
        )
        .expect("remove story link");
        assert!(removed.changed);
        assert!(canonical_store::load_story(&path, &story.story_id).is_ok());
        assert!(load_target_context(&path, &target_id).is_ok());
        let connection = open_connection(&path).expect("database");
        assert_eq!(row_count(&connection, "evidence_records"), 0);
        assert_eq!(row_count(&connection, "proof_points"), 0);
        drop(connection);
        fs::remove_dir_all(path).expect("cleanup");
    }

    #[test]
    fn imported_file_extraction_cache_can_feed_target_context_without_fake_capture_paths() {
        let path = vault();
        canonical_store::initialize(&path).expect("initialize");
        let cache_dir = path.join(".worklore/extraction-cache");
        fs::create_dir_all(&cache_dir).expect("cache dir");
        fs::write(
            cache_dir.join("source_file.txt"),
            "Responsibilities\n- Lead platform strategy across integrations.\nRequirements\n- Experience with enterprise APIs and product leadership.",
        )
        .expect("write extraction");
        let connection = open_connection(&path).expect("database");
        let now = Utc::now().to_rfc3339();
        connection
            .execute(
                "INSERT INTO sources(
                   source_id,source_type,display_name,stored_path,original_file_name,media_type,byte_size,
                   content_hash,lifecycle_status,imported_at,updated_at,extraction_json,privacy_json,
                   provenance_json,tags_json,revision,source_origin,captured_text)
                 VALUES ('source_file','job_description','Synthetic JD','sources/job_description/jd.txt','jd.txt',
                   'text/plain',42,?1,'active',?2,?2,?3,'{}',?4,'[]',1,'imported_file',NULL)",
                params![
                    "0".repeat(64),
                    &now,
                    json!({"textPath":".worklore/extraction-cache/source_file.txt"}).to_string(),
                    json!({"creationActor":"user","importMethod":"file_copy"}).to_string()
                ],
            )
            .expect("insert source");
        drop(connection);

        let created = create_target_context_from_source(&path, "source_file").expect("create target");
        assert!(created.created);
        assert_eq!(created.target_context.context_type, "job_description");
        assert_eq!(created.target_context.source.as_ref().unwrap().source_origin, "imported_file");
        let extracted = extract_target_context_signals(&path, &created.target_context.target_id)
            .expect("extract imported source");
        assert!(extracted.changed);
        assert!(!extracted.target_context.responsibilities.is_empty());
        assert!(!extracted.target_context.skills.is_empty());
        fs::remove_dir_all(path).expect("cleanup");
    }
}
'''
write("src-tauri/src/services/target_context_service.rs", service)

commands = r'''use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::target_context_service::{
        self, CreateTargetContextResult, TargetContextExtractionResult,
        TargetContextLinkTargetView, TargetContextRecordView, TargetContextRelationKind,
        TargetContextRelationshipMutationResult, UpdateTargetContextRequest,
    },
};

#[tauri::command]
pub fn create_target_context_from_source(
    vault_path: String,
    source_id: String,
) -> CommandResult<CreateTargetContextResult> {
    target_context_service::create_target_context_from_source(&PathBuf::from(vault_path), &source_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_target_context(
    vault_path: String,
    target_id: String,
) -> CommandResult<TargetContextRecordView> {
    target_context_service::load_target_context(&PathBuf::from(vault_path), &target_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_target_contexts(vault_path: String) -> CommandResult<Vec<TargetContextRecordView>> {
    target_context_service::list_target_contexts(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_target_context(
    vault_path: String,
    request: UpdateTargetContextRequest,
) -> CommandResult<TargetContextRecordView> {
    target_context_service::update_target_context(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn extract_target_context_signals(
    vault_path: String,
    target_id: String,
) -> CommandResult<TargetContextExtractionResult> {
    target_context_service::extract_target_context_signals(&PathBuf::from(vault_path), &target_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn add_target_context_relationship(
    vault_path: String,
    target_context_id: String,
    relation_kind: TargetContextRelationKind,
    target_id: String,
) -> CommandResult<TargetContextRelationshipMutationResult> {
    target_context_service::add_target_context_relationship(
        &PathBuf::from(vault_path),
        &target_context_id,
        relation_kind,
        &target_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn remove_target_context_relationship(
    vault_path: String,
    target_context_id: String,
    relation_kind: TargetContextRelationKind,
    target_id: String,
) -> CommandResult<TargetContextRelationshipMutationResult> {
    target_context_service::remove_target_context_relationship(
        &PathBuf::from(vault_path),
        &target_context_id,
        relation_kind,
        &target_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_target_context_link_targets(
    vault_path: String,
    relation_kind: TargetContextRelationKind,
) -> CommandResult<Vec<TargetContextLinkTargetView>> {
    target_context_service::list_target_context_link_targets(&PathBuf::from(vault_path), relation_kind)
        .map_err(CommandError::from)
}
'''
write("src-tauri/src/commands/target_contexts.rs", commands)

# Module registration.
path = "src-tauri/src/services/mod.rs"
text = read(path)
text = replace_once(text, "pub mod story_service;\npub mod topic_service;", "pub mod story_service;\npub mod target_context_service;\npub mod topic_service;", "service module")
write(path, text)

path = "src-tauri/src/commands/mod.rs"
text = read(path)
text = replace_once(text, "pub mod stories;\npub mod topics;", "pub mod stories;\npub mod target_contexts;\npub mod topics;", "command module")
write(path, text)

path = "src-tauri/src/lib.rs"
text = read(path)
text = replace_once(
    text,
    "    stories::{import_story_response, list_stories, set_story_status},\n    topics::{",
    "    stories::{import_story_response, list_stories, set_story_status},\n    target_contexts::{\n        add_target_context_relationship, create_target_context_from_source,\n        extract_target_context_signals, get_target_context, list_target_context_link_targets,\n        list_target_contexts, remove_target_context_relationship, update_target_context,\n    },\n    topics::{",
    "lib target context imports",
)
text = replace_once(
    text,
    "            list_inspiration_link_targets,\n            create_topic,",
    "            list_inspiration_link_targets,\n            create_target_context_from_source,\n            get_target_context,\n            list_target_contexts,\n            update_target_context,\n            extract_target_context_signals,\n            add_target_context_relationship,\n            remove_target_context_relationship,\n            list_target_context_link_targets,\n            create_topic,",
    "lib handler registration",
)
write(path, text)

# TypeScript domain contract.
path = "src/domain/types.ts"
text = read(path)
marker = "\n\nexport type PrivacyScanStatus ="
target_types = r'''

export type TargetContextLifecycle = "active" | "archived";
export type TargetContextRelationKind = "topic" | "theme" | "story";

export interface TargetContextSource {
  sourceId: string;
  sourceType: string;
  displayName: string;
  sourceOrigin: string;
  originalFileName: string;
  storedPath: string;
  sourceUrl: string | null;
}

export interface TargetContextRelationship {
  relationshipId: string;
  relationKind: TargetContextRelationKind;
  targetId: string;
  targetLabel: string;
  targetDetail: string;
  targetStatus: string;
}

export interface TargetContextRecord {
  targetId: string;
  source: TargetContextSource | null;
  contextType: string;
  title: string;
  lifecycle: TargetContextLifecycle;
  sourceUrl: string | null;
  organizationName: string | null;
  roleTitle: string | null;
  location: string | null;
  summary: string;
  responsibilities: string[];
  skills: string[];
  concepts: string[];
  language: string[];
  tensions: string[];
  notes: string;
  createdAt: string;
  updatedAt: string;
  revision: number;
  relationships: TargetContextRelationship[];
}

export interface CreateTargetContextResult {
  targetContext: TargetContextRecord;
  created: boolean;
}

export interface UpdateTargetContextRequest {
  targetId: string;
  title: string;
  lifecycle: TargetContextLifecycle;
  sourceUrl?: string | null;
  organizationName?: string | null;
  roleTitle?: string | null;
  location?: string | null;
  summary: string;
  responsibilities: string[];
  skills: string[];
  concepts: string[];
  language: string[];
  tensions: string[];
  notes: string;
}

export interface TargetContextExtractionResult {
  targetContext: TargetContextRecord;
  changed: boolean;
}

export interface TargetContextLinkTarget {
  relationKind: TargetContextRelationKind;
  targetId: string;
  label: string;
  detail: string;
  status: string;
}

export interface TargetContextRelationshipMutationResult {
  targetContextId: string;
  relationKind: TargetContextRelationKind;
  targetId: string;
  changed: boolean;
}
'''
text = replace_once(text, marker, target_types + marker, "target TS types")
write(path, text)

# Frontend API.
path = "src/lib/workloreApi.ts"
text = read(path)
text = replace_once(
    text,
    "  StorySummary,\n  SubmitInterviewResponseRequest,\n  ThemeRecord,",
    "  StorySummary,\n  SubmitInterviewResponseRequest,\n  TargetContextExtractionResult,\n  TargetContextLinkTarget,\n  TargetContextRecord,\n  TargetContextRelationKind,\n  TargetContextRelationshipMutationResult,\n  CreateTargetContextResult,\n  UpdateTargetContextRequest,\n  ThemeRecord,",
    "target API imports",
)
api_marker = "\n\nexport async function startStorySeedDevelopment("
target_api = r'''

export async function createTargetContextFromSource(
  vaultPath: string,
  sourceId: string,
): Promise<CreateTargetContextResult> {
  return invoke<CreateTargetContextResult>("create_target_context_from_source", {
    vaultPath,
    sourceId,
  });
}

export async function getTargetContext(
  vaultPath: string,
  targetId: string,
): Promise<TargetContextRecord> {
  return invoke<TargetContextRecord>("get_target_context", { vaultPath, targetId });
}

export async function listTargetContexts(vaultPath: string): Promise<TargetContextRecord[]> {
  return invoke<TargetContextRecord[]>("list_target_contexts", { vaultPath });
}

export async function updateTargetContext(
  vaultPath: string,
  request: UpdateTargetContextRequest,
): Promise<TargetContextRecord> {
  return invoke<TargetContextRecord>("update_target_context", { vaultPath, request });
}

export async function extractTargetContextSignals(
  vaultPath: string,
  targetId: string,
): Promise<TargetContextExtractionResult> {
  return invoke<TargetContextExtractionResult>("extract_target_context_signals", {
    vaultPath,
    targetId,
  });
}

export async function addTargetContextRelationship(
  vaultPath: string,
  targetContextId: string,
  relationKind: TargetContextRelationKind,
  targetId: string,
): Promise<TargetContextRelationshipMutationResult> {
  return invoke<TargetContextRelationshipMutationResult>("add_target_context_relationship", {
    vaultPath,
    targetContextId,
    relationKind,
    targetId,
  });
}

export async function removeTargetContextRelationship(
  vaultPath: string,
  targetContextId: string,
  relationKind: TargetContextRelationKind,
  targetId: string,
): Promise<TargetContextRelationshipMutationResult> {
  return invoke<TargetContextRelationshipMutationResult>("remove_target_context_relationship", {
    vaultPath,
    targetContextId,
    relationKind,
    targetId,
  });
}

export async function listTargetContextLinkTargets(
  vaultPath: string,
  relationKind: TargetContextRelationKind,
): Promise<TargetContextLinkTarget[]> {
  return invoke<TargetContextLinkTarget[]>("list_target_context_link_targets", {
    vaultPath,
    relationKind,
  });
}
'''
text = replace_once(text, api_marker, target_api + api_marker, "target API functions")
write(path, text)

panel = r'''import { useEffect, useMemo, useState } from "react";
import type {
  TargetContextLifecycle,
  TargetContextLinkTarget,
  TargetContextRecord,
  TargetContextRelationKind,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  addTargetContextRelationship,
  extractTargetContextSignals,
  getTargetContext,
  listTargetContextLinkTargets,
  removeTargetContextRelationship,
  updateTargetContext,
} from "../lib/workloreApi";
import "../target-context.css";

const LIFECYCLES: Array<{ value: TargetContextLifecycle; label: string }> = [
  { value: "active", label: "Active" },
  { value: "archived", label: "Archived" },
];

const RELATION_KINDS: Array<{ value: TargetContextRelationKind; label: string }> = [
  { value: "topic", label: "Topic" },
  { value: "theme", label: "Theme" },
  { value: "story", label: "Story" },
];

export function TargetContextPanel({
  vaultPath,
  targetId,
  onClose,
}: {
  vaultPath: string;
  targetId: string;
  onClose: () => void;
}) {
  const [record, setRecord] = useState<TargetContextRecord | null>(null);
  const [title, setTitle] = useState("");
  const [lifecycle, setLifecycle] = useState<TargetContextLifecycle>("active");
  const [sourceUrl, setSourceUrl] = useState("");
  const [organizationName, setOrganizationName] = useState("");
  const [roleTitle, setRoleTitle] = useState("");
  const [location, setLocation] = useState("");
  const [summary, setSummary] = useState("");
  const [responsibilities, setResponsibilities] = useState("");
  const [skills, setSkills] = useState("");
  const [concepts, setConcepts] = useState("");
  const [language, setLanguage] = useState("");
  const [tensions, setTensions] = useState("");
  const [notes, setNotes] = useState("");
  const [relationKind, setRelationKind] = useState<TargetContextRelationKind>("topic");
  const [relatedId, setRelatedId] = useState("");
  const [targets, setTargets] = useState<Record<TargetContextRelationKind, TargetContextLinkTarget[]>>({
    topic: [],
    theme: [],
    story: [],
  });
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refreshAll();
  }, [vaultPath, targetId]);

  async function refreshAll() {
    setBusy("Opening target context");
    setError(null);
    try {
      const [loaded, topicTargets, themeTargets, storyTargets] = await Promise.all([
        getTargetContext(vaultPath, targetId),
        listTargetContextLinkTargets(vaultPath, "topic"),
        listTargetContextLinkTargets(vaultPath, "theme"),
        listTargetContextLinkTargets(vaultPath, "story"),
      ]);
      applyRecord(loaded);
      setTargets({ topic: topicTargets, theme: themeTargets, story: storyTargets });
      setRelatedId("");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  function applyRecord(loaded: TargetContextRecord) {
    setRecord(loaded);
    setTitle(loaded.title);
    setLifecycle(loaded.lifecycle);
    setSourceUrl(loaded.sourceUrl ?? "");
    setOrganizationName(loaded.organizationName ?? "");
    setRoleTitle(loaded.roleTitle ?? "");
    setLocation(loaded.location ?? "");
    setSummary(loaded.summary);
    setResponsibilities(linesFrom(loaded.responsibilities));
    setSkills(linesFrom(loaded.skills));
    setConcepts(linesFrom(loaded.concepts));
    setLanguage(linesFrom(loaded.language));
    setTensions(linesFrom(loaded.tensions));
    setNotes(loaded.notes);
  }

  async function handleSave() {
    if (!record || !title.trim()) {
      return;
    }
    setBusy("Saving target context");
    setNotice(null);
    setError(null);
    try {
      const updated = await updateTargetContext(vaultPath, {
        targetId: record.targetId,
        title,
        lifecycle,
        sourceUrl: sourceUrl || null,
        organizationName: organizationName || null,
        roleTitle: roleTitle || null,
        location: location || null,
        summary,
        responsibilities: listFrom(responsibilities),
        skills: listFrom(skills),
        concepts: listFrom(concepts),
        language: listFrom(language),
        tensions: listFrom(tensions),
        notes,
      });
      applyRecord(updated);
      setNotice("Target context saved locally. Context signals remain separate from evidence about you.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleExtract() {
    if (!record) {
      return;
    }
    setBusy("Extracting source signals");
    setNotice(null);
    setError(null);
    try {
      const result = await extractTargetContextSignals(vaultPath, record.targetId);
      applyRecord(result.targetContext);
      setNotice(
        result.changed
          ? "Source signals added. Review them before using them; no claims or standing were created."
          : "No new source signals were found.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleAddRelationship() {
    if (!record || !relatedId) {
      return;
    }
    setBusy("Connecting target context");
    setNotice(null);
    setError(null);
    try {
      const result = await addTargetContextRelationship(
        vaultPath,
        record.targetId,
        relationKind,
        relatedId,
      );
      setNotice(result.changed ? "Connection added." : "That connection already exists.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleRemoveRelationship(kind: TargetContextRelationKind, relatedTargetId: string) {
    if (!record) {
      return;
    }
    setBusy("Removing connection");
    setNotice(null);
    setError(null);
    try {
      await removeTargetContextRelationship(vaultPath, record.targetId, kind, relatedTargetId);
      setNotice("Connection removed; both records were kept.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  const linkedTargetIds = useMemo(
    () => new Set(record?.relationships.map((relationship) => relationship.targetId) ?? []),
    [record],
  );
  const availableTargets = targets[relationKind].filter(
    (target) => !linkedTargetIds.has(target.targetId),
  );

  return (
    <section className="target-context-panel" aria-labelledby="target-context-heading">
      <div className="target-context-heading-row">
        <div>
          <p className="eyebrow">Target context</p>
          <h3 id="target-context-heading">Understand the opportunity without turning it into an ATS target</h3>
          <p>
            Responsibilities, skills, concepts, and language are audience/opportunity signals. They are not evidence that you have done the work.
          </p>
        </div>
        <button className="quiet-button compact" onClick={onClose}>Close</button>
      </div>

      {record ? (
        <>
          <div className="target-context-provenance">
            <strong>Source provenance</strong>
            {record.source ? (
              <>
                <span>{record.source.displayName}</span>
                <span>{record.source.sourceId} · {record.source.sourceOrigin}</span>
                {record.source.originalFileName ? <span>{record.source.originalFileName}</span> : null}
              </>
            ) : (
              <span>No Source is attached to this legacy Target Context.</span>
            )}
            <span>Type: {record.contextType}</span>
          </div>

          <div className="target-context-editor-grid">
            <label>
              Title
              <input value={title} onChange={(event) => setTitle(event.target.value)} />
            </label>
            <label>
              Lifecycle
              <select
                value={lifecycle}
                onChange={(event) => setLifecycle(event.target.value as TargetContextLifecycle)}
              >
                {LIFECYCLES.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
            </label>
            <label className="target-context-wide">
              Source URL
              <input value={sourceUrl} onChange={(event) => setSourceUrl(event.target.value)} placeholder="https://…" />
            </label>
            <label>
              Organization
              <input value={organizationName} onChange={(event) => setOrganizationName(event.target.value)} />
            </label>
            <label>
              Role / opportunity
              <input value={roleTitle} onChange={(event) => setRoleTitle(event.target.value)} />
            </label>
            <label>
              Location
              <input value={location} onChange={(event) => setLocation(event.target.value)} />
            </label>
            <label className="target-context-wide">
              Summary
              <textarea rows={4} value={summary} onChange={(event) => setSummary(event.target.value)} />
            </label>
            <label>
              Responsibilities
              <textarea rows={7} value={responsibilities} onChange={(event) => setResponsibilities(event.target.value)} placeholder="One source-grounded signal per line" />
            </label>
            <label>
              Skills / qualifications
              <textarea rows={7} value={skills} onChange={(event) => setSkills(event.target.value)} placeholder="One source-grounded signal per line" />
            </label>
            <label>
              Concepts
              <textarea rows={7} value={concepts} onChange={(event) => setConcepts(event.target.value)} placeholder="Potential areas to think about — not claims" />
            </label>
            <label>
              Notable language
              <textarea rows={7} value={language} onChange={(event) => setLanguage(event.target.value)} placeholder="Vocabulary from this source, not keywords to stuff" />
            </label>
            <label className="target-context-wide">
              Tensions / tradeoffs
              <textarea rows={5} value={tensions} onChange={(event) => setTensions(event.target.value)} />
            </label>
            <label className="target-context-wide">
              Notes
              <textarea rows={4} value={notes} onChange={(event) => setNotes(event.target.value)} />
            </label>
          </div>

          <div className="target-context-actions-row">
            <button className="primary-button compact" disabled={busy !== null || !title.trim()} onClick={() => void handleSave()}>
              Save target context
            </button>
            <button className="secondary-button compact" disabled={busy !== null || !record.source} onClick={() => void handleExtract()}>
              Extract source signals
            </button>
            <span>{record.targetId}</span>
          </div>

          <div className="target-context-boundary">
            <strong>Boundary</strong>
            <span>Extraction is deterministic and provider-free. It does not score fit, create Topics, or claim you possess a listed skill.</span>
          </div>

          <div className="target-context-connections">
            <div>
              <h4>Explicit connections</h4>
              <p>Connect this context to existing Topics, Themes, or Stories. A connection is context, not automatic standing.</p>
            </div>
            {record.relationships.length === 0 ? <span>No connections yet.</span> : null}
            {record.relationships.map((relationship) => (
              <div className="target-context-link-row" key={relationship.relationshipId}>
                <div>
                  <strong>{relationship.targetLabel}</strong>
                  <span>{kindLabel(relationship.relationKind)} · {relationship.targetStatus}</span>
                </div>
                <button className="quiet-button compact" onClick={() => void handleRemoveRelationship(relationship.relationKind, relationship.targetId)}>
                  Remove
                </button>
              </div>
            ))}
            <div className="target-context-connect-row">
              <select
                value={relationKind}
                onChange={(event) => {
                  setRelationKind(event.target.value as TargetContextRelationKind);
                  setRelatedId("");
                }}
              >
                {RELATION_KINDS.map((option) => (
                  <option key={option.value} value={option.value}>{option.label}</option>
                ))}
              </select>
              <select value={relatedId} onChange={(event) => setRelatedId(event.target.value)}>
                <option value="">Select an existing {kindLabel(relationKind).toLowerCase()}</option>
                {availableTargets.map((target) => (
                  <option key={target.targetId} value={target.targetId}>{target.label}</option>
                ))}
              </select>
              <button className="secondary-button compact" disabled={!relatedId || busy !== null} onClick={() => void handleAddRelationship()}>
                Connect
              </button>
            </div>
          </div>
        </>
      ) : null}

      <div className="target-context-feedback" aria-live="polite">
        {busy ? <span>{busy}...</span> : null}
        {notice ? <span>{notice}</span> : null}
        {error ? <span className="capture-error">{error}</span> : null}
      </div>
    </section>
  );
}

function linesFrom(values: string[]): string {
  return values.join("\n");
}

function listFrom(value: string): string[] {
  return value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function kindLabel(kind: TargetContextRelationKind): string {
  return RELATION_KINDS.find((option) => option.value === kind)?.label ?? kind;
}
'''
write("src/components/TargetContextPanel.tsx", panel)

css = r'''.target-context-panel {
  margin-top: 1rem;
  padding: 1rem;
  border: 1px solid var(--border-subtle, #d8d8d8);
  border-radius: 0.9rem;
  background: var(--surface-raised, #fff);
}

.target-context-heading-row,
.target-context-actions-row,
.target-context-connect-row,
.target-context-link-row {
  display: flex;
  gap: 0.75rem;
  align-items: center;
  justify-content: space-between;
}

.target-context-heading-row {
  align-items: flex-start;
}

.target-context-heading-row p,
.target-context-connections p {
  max-width: 70ch;
}

.target-context-provenance,
.target-context-boundary {
  display: grid;
  gap: 0.25rem;
  margin: 1rem 0;
  padding: 0.8rem;
  border-radius: 0.7rem;
  background: var(--surface-muted, #f5f5f5);
}

.target-context-editor-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.8rem;
}

.target-context-editor-grid label {
  display: grid;
  gap: 0.35rem;
  font-weight: 600;
}

.target-context-editor-grid input,
.target-context-editor-grid textarea,
.target-context-editor-grid select,
.target-context-connect-row select {
  width: 100%;
}

.target-context-wide {
  grid-column: 1 / -1;
}

.target-context-actions-row {
  justify-content: flex-start;
  flex-wrap: wrap;
  margin-top: 1rem;
}

.target-context-actions-row span {
  font-size: 0.82rem;
  opacity: 0.72;
}

.target-context-connections {
  display: grid;
  gap: 0.65rem;
  margin-top: 1rem;
}

.target-context-link-row {
  padding: 0.65rem 0;
  border-bottom: 1px solid var(--border-subtle, #e3e3e3);
}

.target-context-link-row div {
  display: grid;
  gap: 0.2rem;
}

.target-context-link-row span {
  font-size: 0.82rem;
  opacity: 0.72;
}

.target-context-connect-row {
  justify-content: flex-start;
  flex-wrap: wrap;
}

.target-context-connect-row select {
  max-width: 18rem;
}

.target-context-feedback {
  min-height: 1.4rem;
  margin-top: 0.75rem;
}

@media (max-width: 760px) {
  .target-context-editor-grid {
    grid-template-columns: 1fr;
  }

  .target-context-wide {
    grid-column: auto;
  }

  .target-context-heading-row,
  .target-context-link-row {
    align-items: stretch;
    flex-direction: column;
  }
}
'''
write("src/target-context.css", css)

# Capture entry point for existing capture-classified Target Context.
path = "src/components/CapturePanel.tsx"
text = read(path)
text = replace_once(text, 'import { TopicPanel } from "./TopicPanel";\n', 'import { TopicPanel } from "./TopicPanel";\nimport { TargetContextPanel } from "./TargetContextPanel";\n', "capture Target import")
text = replace_once(
    text,
    "  const [developmentInspirationId, setDevelopmentInspirationId] = useState<string | null>(null);\n",
    "  const [developmentInspirationId, setDevelopmentInspirationId] = useState<string | null>(null);\n  const [developmentTargetContextId, setDevelopmentTargetContextId] = useState<string | null>(null);\n",
    "capture target state",
)
text = replace_once(
    text,
    "    setDevelopmentInspirationId(null);\n    void refreshRecent();",
    "    setDevelopmentInspirationId(null);\n    setDevelopmentTargetContextId(null);\n    void refreshRecent();",
    "capture target reset",
)
text = replace_once(
    text,
    '''  const inspirationClassification = saved?.classifications.find(
    (classification) => classification.role === "inspiration",
  );
''',
    '''  const inspirationClassification = saved?.classifications.find(
    (classification) => classification.role === "inspiration",
  );
  const targetContextClassification = saved?.classifications.find(
    (classification) => classification.role === "target_context",
  );
''',
    "capture target classification",
)
text = replace_once(
    text,
    '''            {inspirationClassification ? (
              <button
                className="primary-button compact"
                disabled={busy !== null}
                onClick={() => setDevelopmentInspirationId(inspirationClassification.targetId)}
              >
                Work with this inspiration
              </button>
            ) : null}
''',
    '''            {inspirationClassification ? (
              <button
                className="primary-button compact"
                disabled={busy !== null}
                onClick={() => setDevelopmentInspirationId(inspirationClassification.targetId)}
              >
                Work with this inspiration
              </button>
            ) : null}
            {targetContextClassification ? (
              <button
                className="primary-button compact"
                disabled={busy !== null}
                onClick={() => setDevelopmentTargetContextId(targetContextClassification.targetId)}
              >
                Work with this target context
              </button>
            ) : null}
''',
    "capture target button",
)
text = replace_once(
    text,
    '''      {developmentInspirationId ? (
        <InspirationPanel
          vaultPath={vaultPath}
          inspirationId={developmentInspirationId}
          onClose={() => setDevelopmentInspirationId(null)}
        />
      ) : null}
''',
    '''      {developmentInspirationId ? (
        <InspirationPanel
          vaultPath={vaultPath}
          inspirationId={developmentInspirationId}
          onClose={() => setDevelopmentInspirationId(null)}
        />
      ) : null}

      {developmentTargetContextId ? (
        <TargetContextPanel
          vaultPath={vaultPath}
          targetId={developmentTargetContextId}
          onClose={() => setDevelopmentTargetContextId(null)}
        />
      ) : null}
''',
    "capture target panel",
)
write(path, text)

print("Applied bounded Target Context slice.")
