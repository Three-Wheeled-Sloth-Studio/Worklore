use std::{
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
    Stale,
    Archived,
}

impl TargetContextLifecycle {
    fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Stale => "stale",
            Self::Archived => "archived",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "stale" => Ok(Self::Stale),
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
    let source =
        load_source_record(&connection, source_id)?.ok_or(WorkLoreError::SourceNotFound)?;

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
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
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
    let source =
        load_source_record(&connection, &source_id)?.ok_or(WorkLoreError::SourceNotFound)?;
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

fn load_source_record(
    connection: &Connection,
    source_id: &str,
) -> ServiceResult<Option<SourceRecord>> {
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
        TargetContextRelationKind::Story => {
            "SELECT EXISTS(SELECT 1 FROM stories WHERE story_id=?1)"
        }
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
        "responsibilities"
        | "what you'll do"
        | "what you will do"
        | "your impact"
        | "what you do"
        | "the role"
        | "key responsibilities" => Some(Section::Responsibilities),
        "requirements"
        | "qualifications"
        | "skills"
        | "what you bring"
        | "what we're looking for"
        | "what we are looking for"
        | "preferred qualifications"
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
            .cmp(&left.1 .0)
            .then_with(|| left.1 .1.cmp(&right.1 .1))
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
            .cmp(&left.1 .0)
            .then_with(|| left.1 .1.cmp(&right.1 .1))
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
            capture_service, topic_service,
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
        assert_eq!(canonical_store::schema_version(&path).unwrap(), 9);
        let before =
            load_target_context(&path, &classified.target_id).expect("load target context");
        assert_eq!(before.context_type, "job_description");
        assert_eq!(before.lifecycle, TargetContextLifecycle::Active);
        assert_eq!(
            before
                .source
                .as_ref()
                .map(|source| source.source_id.as_str()),
            Some(source.source_id.as_str())
        );

        let updated = update_target_context(
            &path,
            UpdateTargetContextRequest {
                target_id: before.target_id.clone(),
                title: "Director of Product — synthetic".into(),
                lifecycle: TargetContextLifecycle::Stale,
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
        assert_eq!(updated.lifecycle, TargetContextLifecycle::Stale);
        assert_eq!(updated.revision, before.revision + 1);
        let reopened = load_target_context(&path, &before.target_id).expect("reopen");
        assert_eq!(reopened.lifecycle, TargetContextLifecycle::Stale);
        assert_eq!(
            reopened.organization_name.as_deref(),
            Some("Synthetic Health Co")
        );
        assert_eq!(reopened.skills, vec!["Product leadership"]);
        fs::remove_dir_all(path).expect("cleanup");
    }

    #[test]
    fn deterministic_signal_extraction_is_idempotent_and_never_creates_standing() {
        let path = vault();
        let text = "What You'll Do\n- Lead continuous discovery with providers and care teams.\n- Partner with Engineering to evaluate technical tradeoffs and iterate quickly.\n- Turn complex clinical workflows into clear product opportunities.\nWhat You Bring\n- Experience building and scaling product teams in healthcare.\n- Product leadership experience in regulated domains.\nChallenges\n- Balance rapid iteration with clinical safety and operational reliability.";
        let source =
            capture_service::create_capture_source(&path, text, SourceType::JobDescription)
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
        let second =
            extract_target_context_signals(&path, &classified.target_id).expect("extract again");
        assert!(!second.changed);
        assert_eq!(
            second.target_context.revision,
            first.target_context.revision
        );

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
        let story =
            canonical_store::create_story(&path, "A real user story", "Synthetic story summary")
                .expect("story");

        assert!(
            add_target_context_relationship(
                &path,
                &target_id,
                TargetContextRelationKind::Topic,
                &topic.topic_id
            )
            .unwrap()
            .changed
        );
        assert!(
            add_target_context_relationship(
                &path,
                &target_id,
                TargetContextRelationKind::Theme,
                &theme.theme_id
            )
            .unwrap()
            .changed
        );
        assert!(
            add_target_context_relationship(
                &path,
                &target_id,
                TargetContextRelationKind::Story,
                &story.story_id
            )
            .unwrap()
            .changed
        );
        assert!(
            !add_target_context_relationship(
                &path,
                &target_id,
                TargetContextRelationKind::Topic,
                &topic.topic_id
            )
            .unwrap()
            .changed
        );

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

        let created =
            create_target_context_from_source(&path, "source_file").expect("create target");
        assert!(created.created);
        assert_eq!(created.target_context.context_type, "job_description");
        assert_eq!(
            created
                .target_context
                .source
                .as_ref()
                .unwrap()
                .source_origin,
            "imported_file"
        );
        let extracted = extract_target_context_signals(&path, &created.target_context.target_id)
            .expect("extract imported source");
        assert!(extracted.changed);
        assert!(!extracted.target_context.responsibilities.is_empty());
        assert!(!extracted.target_context.skills.is_empty());
        fs::remove_dir_all(path).expect("cleanup");
    }
}
