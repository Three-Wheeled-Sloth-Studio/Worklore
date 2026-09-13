from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"Expected exactly one match in {path}, found {count}: {old[:120]!r}")
    write(path, text.replace(old, new, 1))


# Canonical schema v3: Topic timing metadata only.
replace_once(
    "src-tauri/src/services/canonical_store.rs",
    'const CURRENT_SCHEMA_VERSION: i64 = 2;',
    'const CURRENT_SCHEMA_VERSION: i64 = 3;',
)
replace_once(
    "src-tauri/src/services/canonical_store.rs",
    '''const SCHEMA_V2: &str = r#"\nALTER TABLE sources ADD COLUMN source_origin TEXT NOT NULL DEFAULT 'imported_file';\nALTER TABLE sources ADD COLUMN captured_text TEXT;\nCREATE INDEX idx_sources_origin_imported_at ON sources(source_origin, imported_at DESC);\n"#;\n''',
    '''const SCHEMA_V2: &str = r#"\nALTER TABLE sources ADD COLUMN source_origin TEXT NOT NULL DEFAULT 'imported_file';\nALTER TABLE sources ADD COLUMN captured_text TEXT;\nCREATE INDEX idx_sources_origin_imported_at ON sources(source_origin, imported_at DESC);\n"#;\n\nconst SCHEMA_V3: &str = r#"\nALTER TABLE topic_candidates ADD COLUMN timing_class TEXT NOT NULL DEFAULT 'evergreen';\nALTER TABLE topic_candidates ADD COLUMN relevant_until TEXT;\nALTER TABLE topic_candidates ADD COLUMN timely_note TEXT;\nCREATE INDEX idx_topics_status_timing ON topic_candidates(status, timing_class, updated_at DESC);\n"#;\n''',
)
replace_once(
    "src-tauri/src/services/canonical_store.rs",
    '''    if version == 1 {\n        let tx = connection.transaction()?;\n        tx.execute_batch(SCHEMA_V2)?;\n        tx.execute(\n            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (2,'capture_text_sources_v2',?1)",\n            [Utc::now().to_rfc3339()],\n        )?;\n        tx.commit()?;\n    }\n    Ok(())\n''',
    '''    if version == 1 {\n        let tx = connection.transaction()?;\n        tx.execute_batch(SCHEMA_V2)?;\n        tx.execute(\n            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (2,'capture_text_sources_v2',?1)",\n            [Utc::now().to_rfc3339()],\n        )?;\n        tx.commit()?;\n        version = 2;\n    }\n    if version == 2 {\n        let tx = connection.transaction()?;\n        tx.execute_batch(SCHEMA_V3)?;\n        tx.execute(\n            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (3,'topic_timing_v3',?1)",\n            [Utc::now().to_rfc3339()],\n        )?;\n        tx.commit()?;\n    }\n    Ok(())\n''',
)
replace_once(
    "src-tauri/src/services/canonical_store.rs",
    'assert_eq!(schema_version(&p).unwrap(), 2);',
    'assert_eq!(schema_version(&p).unwrap(), 3);',
)

# Dedicated Topic/Theme service.
write(
    "src-tauri/src/services/topic_service.rs",
    r'''use std::{path::Path, time::Duration};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    error::{ServiceResult, WorkLoreError},
    services::canonical_store,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TopicLifecycle {
    Captured,
    Exploring,
    Ready,
    Drafted,
    Parked,
    Retired,
}

impl TopicLifecycle {
    fn as_str(self) -> &'static str {
        match self {
            Self::Captured => "captured",
            Self::Exploring => "exploring",
            Self::Ready => "ready",
            Self::Drafted => "drafted",
            Self::Parked => "parked",
            Self::Retired => "retired",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "captured" => Ok(Self::Captured),
            "exploring" => Ok(Self::Exploring),
            "ready" => Ok(Self::Ready),
            "drafted" => Ok(Self::Drafted),
            "parked" => Ok(Self::Parked),
            "retired" => Ok(Self::Retired),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Topic lifecycle {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TopicTimingClass {
    Evergreen,
    Timely,
}

impl TopicTimingClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Evergreen => "evergreen",
            Self::Timely => "timely",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "evergreen" => Ok(Self::Evergreen),
            "timely" => Ok(Self::Timely),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Topic timing class {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemeLifecycle {
    Emerging,
    Active,
    Retired,
}

impl ThemeLifecycle {
    fn as_str(self) -> &'static str {
        match self {
            Self::Emerging => "emerging",
            Self::Active => "active",
            Self::Retired => "retired",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "emerging" => Ok(Self::Emerging),
            "active" => Ok(Self::Active),
            "retired" => Ok(Self::Retired),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Theme lifecycle {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TopicRelationKind {
    Story,
    ProofPoint,
    Theme,
    Inspiration,
    TargetContext,
}

impl TopicRelationKind {
    fn relationship_type(self) -> &'static str {
        match self {
            Self::Story => "topic_story",
            Self::ProofPoint => "topic_proof_point",
            Self::Theme => "topic_theme",
            Self::Inspiration => "topic_inspiration",
            Self::TargetContext => "topic_target_context",
        }
    }

    fn target_type(self) -> &'static str {
        match self {
            Self::Story => "story",
            Self::ProofPoint => "proof_point",
            Self::Theme => "theme",
            Self::Inspiration => "inspiration",
            Self::TargetContext => "target_context",
        }
    }

    fn category(self) -> &'static str {
        match self {
            Self::Story | Self::ProofPoint => "standing",
            Self::Theme => "organizing_context",
            Self::Inspiration => "creative_context",
            Self::TargetContext => "target_context",
        }
    }

    fn from_relationship_type(value: &str) -> Option<Self> {
        match value {
            "topic_story" => Some(Self::Story),
            "topic_proof_point" => Some(Self::ProofPoint),
            "topic_theme" => Some(Self::Theme),
            "topic_inspiration" => Some(Self::Inspiration),
            "topic_target_context" => Some(Self::TargetContext),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTopicRequest {
    pub title: String,
    pub summary: String,
    pub timing_class: TopicTimingClass,
    pub relevant_until: Option<String>,
    pub timely_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTopicRequest {
    pub topic_id: String,
    pub title: String,
    pub summary: String,
    pub lifecycle: TopicLifecycle,
    pub timing_class: TopicTimingClass,
    pub relevant_until: Option<String>,
    pub timely_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThemeRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateThemeRequest {
    pub theme_id: String,
    pub name: String,
    pub description: String,
    pub lifecycle: ThemeLifecycle,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicRelationshipView {
    pub relationship_id: String,
    pub relation_kind: TopicRelationKind,
    pub target_id: String,
    pub target_label: String,
    pub target_detail: String,
    pub target_status: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicRecordView {
    pub topic_id: String,
    pub title: String,
    pub summary: String,
    pub lifecycle: TopicLifecycle,
    pub timing_class: TopicTimingClass,
    pub relevant_until: Option<String>,
    pub timely_note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
    pub relationships: Vec<TopicRelationshipView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeRecordView {
    pub theme_id: String,
    pub name: String,
    pub description: String,
    pub lifecycle: ThemeLifecycle,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicLinkTargetView {
    pub relation_kind: TopicRelationKind,
    pub target_id: String,
    pub label: String,
    pub detail: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicRelationshipMutationResult {
    pub topic_id: String,
    pub relation_kind: TopicRelationKind,
    pub target_id: String,
    pub changed: bool,
}

pub fn create_topic(vault_path: &Path, request: CreateTopicRequest) -> ServiceResult<TopicRecordView> {
    canonical_store::initialize(vault_path)?;
    let title = required_text(&request.title, "Topic title")?;
    let summary = request.summary.trim().to_string();
    let (relevant_until, timely_note) = timing_fields(
        request.timing_class,
        request.relevant_until,
        request.timely_note,
    );
    let connection = open_connection(vault_path)?;
    let topic_id = format!("topic_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO topic_candidates(topic_id,title,summary,status,provenance_json,created_at,updated_at,revision,\
         timing_class,relevant_until,timely_note)\
         VALUES (?1,?2,?3,'captured',?4,?5,?5,1,?6,?7,?8)",
        params![
            &topic_id,
            &title,
            &summary,
            json!({"creationActor":"user","creationPath":"topic_api"}).to_string(),
            &now,
            request.timing_class.as_str(),
            &relevant_until,
            &timely_note
        ],
    )?;
    audit(
        &connection,
        "topic_created",
        "topic_candidate",
        &topic_id,
        json!({"timingClass":request.timing_class.as_str()}),
    )?;
    load_topic(vault_path, &topic_id)
}

pub fn load_topic(vault_path: &Path, topic_id: &str) -> ServiceResult<TopicRecordView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    load_topic_from_connection(&connection, topic_id)
}

pub fn list_topics(vault_path: &Path) -> ServiceResult<Vec<TopicRecordView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let ids = {
        let mut statement = connection.prepare(
            "SELECT topic_id FROM topic_candidates ORDER BY updated_at DESC, created_at DESC",
        )?;
        statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?
    };
    ids.into_iter()
        .map(|topic_id| load_topic_from_connection(&connection, &topic_id))
        .collect()
}

pub fn update_topic(vault_path: &Path, request: UpdateTopicRequest) -> ServiceResult<TopicRecordView> {
    canonical_store::initialize(vault_path)?;
    let title = required_text(&request.title, "Topic title")?;
    let summary = request.summary.trim().to_string();
    let (relevant_until, timely_note) = timing_fields(
        request.timing_class,
        request.relevant_until,
        request.timely_note,
    );
    let mut connection = open_connection(vault_path)?;
    let before = load_topic_from_connection(&connection, &request.topic_id)?;
    let now = Utc::now().to_rfc3339();
    let tx = connection.transaction()?;
    let changed = tx.execute(
        "UPDATE topic_candidates SET title=?2,summary=?3,status=?4,timing_class=?5,relevant_until=?6,\
         timely_note=?7,updated_at=?8,revision=revision+1 WHERE topic_id=?1",
        params![
            &request.topic_id,
            &title,
            &summary,
            request.lifecycle.as_str(),
            request.timing_class.as_str(),
            &relevant_until,
            &timely_note,
            &now
        ],
    )?;
    if changed == 0 {
        return Err(topic_not_found(&request.topic_id));
    }
    audit(
        &tx,
        "topic_updated",
        "topic_candidate",
        &request.topic_id,
        json!({
            "before": {
                "lifecycle": before.lifecycle.as_str(),
                "timingClass": before.timing_class.as_str(),
                "relevantUntil": before.relevant_until,
                "timelyNote": before.timely_note
            },
            "after": {
                "lifecycle": request.lifecycle.as_str(),
                "timingClass": request.timing_class.as_str(),
                "relevantUntil": relevant_until,
                "timelyNote": timely_note
            }
        }),
    )?;
    tx.commit()?;
    load_topic(vault_path, &request.topic_id)
}

pub fn create_theme(vault_path: &Path, request: CreateThemeRequest) -> ServiceResult<ThemeRecordView> {
    canonical_store::initialize(vault_path)?;
    let name = required_text(&request.name, "Theme name")?;
    let description = request.description.trim().to_string();
    let connection = open_connection(vault_path)?;
    let theme_id = format!("theme_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO themes(theme_id,name,description,status,provenance_json,created_at,updated_at,revision)\
         VALUES (?1,?2,?3,'emerging',?4,?5,?5,1)",
        params![
            &theme_id,
            &name,
            &description,
            json!({"creationActor":"user"}).to_string(),
            &now
        ],
    )?;
    audit(
        &connection,
        "theme_created",
        "theme",
        &theme_id,
        json!({}),
    )?;
    load_theme(vault_path, &theme_id)
}

pub fn load_theme(vault_path: &Path, theme_id: &str) -> ServiceResult<ThemeRecordView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    load_theme_from_connection(&connection, theme_id)
}

pub fn list_themes(vault_path: &Path) -> ServiceResult<Vec<ThemeRecordView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let raw = {
        let mut statement = connection.prepare(
            "SELECT theme_id,name,description,status,created_at,updated_at,revision\
             FROM themes ORDER BY updated_at DESC, created_at DESC",
        )?;
        statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, u32>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
    };
    raw.into_iter().map(theme_from_row).collect()
}

pub fn update_theme(vault_path: &Path, request: UpdateThemeRequest) -> ServiceResult<ThemeRecordView> {
    canonical_store::initialize(vault_path)?;
    let name = required_text(&request.name, "Theme name")?;
    let description = request.description.trim().to_string();
    let mut connection = open_connection(vault_path)?;
    let before = load_theme_from_connection(&connection, &request.theme_id)?;
    let now = Utc::now().to_rfc3339();
    let tx = connection.transaction()?;
    let changed = tx.execute(
        "UPDATE themes SET name=?2,description=?3,status=?4,updated_at=?5,revision=revision+1\
         WHERE theme_id=?1",
        params![
            &request.theme_id,
            &name,
            &description,
            request.lifecycle.as_str(),
            &now
        ],
    )?;
    if changed == 0 {
        return Err(theme_not_found(&request.theme_id));
    }
    audit(
        &tx,
        "theme_updated",
        "theme",
        &request.theme_id,
        json!({
            "beforeLifecycle": before.lifecycle.as_str(),
            "afterLifecycle": request.lifecycle.as_str()
        }),
    )?;
    tx.commit()?;
    load_theme(vault_path, &request.theme_id)
}

pub fn add_topic_relationship(
    vault_path: &Path,
    topic_id: &str,
    relation_kind: TopicRelationKind,
    target_id: &str,
) -> ServiceResult<TopicRelationshipMutationResult> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    ensure_topic_exists(&connection, topic_id)?;
    lookup_target(&connection, relation_kind, target_id)?.ok_or_else(|| {
        WorkLoreError::InvalidVault(format!(
            "{} target {target_id} was not found.",
            relation_kind.target_type()
        ))
    })?;
    let now = Utc::now().to_rfc3339();
    let tx = connection.transaction()?;
    let changed = tx.execute(
        "INSERT OR IGNORE INTO record_relationships(relationship_id,from_type,from_id,relationship_type,to_type,to_id,\
         provenance_json,created_at) VALUES (?1,'topic_candidate',?2,?3,?4,?5,?6,?7)",
        params![
            format!("relationship_{}", Uuid::now_v7()),
            topic_id,
            relation_kind.relationship_type(),
            relation_kind.target_type(),
            target_id,
            json!({"creationActor":"user"}).to_string(),
            &now
        ],
    )? > 0;
    if changed {
        audit(
            &tx,
            "topic_relationship_added",
            "topic_candidate",
            topic_id,
            json!({
                "relationKind": relation_kind.relationship_type(),
                "targetType": relation_kind.target_type(),
                "targetId": target_id,
                "category": relation_kind.category()
            }),
        )?;
    }
    tx.commit()?;
    Ok(TopicRelationshipMutationResult {
        topic_id: topic_id.to_string(),
        relation_kind,
        target_id: target_id.to_string(),
        changed,
    })
}

pub fn remove_topic_relationship(
    vault_path: &Path,
    topic_id: &str,
    relation_kind: TopicRelationKind,
    target_id: &str,
) -> ServiceResult<TopicRelationshipMutationResult> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    ensure_topic_exists(&connection, topic_id)?;
    let now = Utc::now().to_rfc3339();
    let tx = connection.transaction()?;
    let changed = tx.execute(
        "DELETE FROM record_relationships WHERE from_type='topic_candidate' AND from_id=?1\
         AND relationship_type=?2 AND to_type=?3 AND to_id=?4",
        params![
            topic_id,
            relation_kind.relationship_type(),
            relation_kind.target_type(),
            target_id
        ],
    )? > 0;
    if changed {
        audit(
            &tx,
            "topic_relationship_removed",
            "topic_candidate",
            topic_id,
            json!({
                "relationKind": relation_kind.relationship_type(),
                "targetType": relation_kind.target_type(),
                "targetId": target_id,
                "removedAt": now
            }),
        )?;
    }
    tx.commit()?;
    Ok(TopicRelationshipMutationResult {
        topic_id: topic_id.to_string(),
        relation_kind,
        target_id: target_id.to_string(),
        changed,
    })
}

pub fn list_topic_link_targets(
    vault_path: &Path,
    relation_kind: TopicRelationKind,
) -> ServiceResult<Vec<TopicLinkTargetView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let sql = match relation_kind {
        TopicRelationKind::Story => {
            "SELECT story_id,title,summary,lifecycle_status FROM stories ORDER BY updated_at DESC"
        }
        TopicRelationKind::ProofPoint => {
            "SELECT proof_id,statement,'',status FROM proof_points ORDER BY updated_at DESC"
        }
        TopicRelationKind::Theme => {
            "SELECT theme_id,name,description,status FROM themes ORDER BY updated_at DESC"
        }
        TopicRelationKind::Inspiration => {
            "SELECT inspiration_id,title,notes,status FROM inspirations ORDER BY updated_at DESC"
        }
        TopicRelationKind::TargetContext => {
            "SELECT target_id,title,notes,status FROM target_contexts ORDER BY updated_at DESC"
        }
    };
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map([], |row| {
        Ok(TopicLinkTargetView {
            relation_kind,
            target_id: row.get(0)?,
            label: row.get(1)?,
            detail: row.get(2)?,
            status: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn load_topic_from_connection(
    connection: &Connection,
    topic_id: &str,
) -> ServiceResult<TopicRecordView> {
    let raw = connection
        .query_row(
            "SELECT topic_id,title,summary,status,timing_class,relevant_until,timely_note,created_at,updated_at,revision\
             FROM topic_candidates WHERE topic_id=?1",
            [topic_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, u32>(9)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| topic_not_found(topic_id))?;
    Ok(TopicRecordView {
        topic_id: raw.0,
        title: raw.1,
        summary: raw.2,
        lifecycle: TopicLifecycle::parse(&raw.3)?,
        timing_class: TopicTimingClass::parse(&raw.4)?,
        relevant_until: raw.5,
        timely_note: raw.6,
        created_at: raw.7,
        updated_at: raw.8,
        revision: raw.9,
        relationships: load_relationships(connection, topic_id)?,
    })
}

fn load_theme_from_connection(
    connection: &Connection,
    theme_id: &str,
) -> ServiceResult<ThemeRecordView> {
    let raw = connection
        .query_row(
            "SELECT theme_id,name,description,status,created_at,updated_at,revision FROM themes WHERE theme_id=?1",
            [theme_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, u32>(6)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| theme_not_found(theme_id))?;
    theme_from_row(raw)
}

fn theme_from_row(
    raw: (String, String, String, String, String, String, u32),
) -> ServiceResult<ThemeRecordView> {
    Ok(ThemeRecordView {
        theme_id: raw.0,
        name: raw.1,
        description: raw.2,
        lifecycle: ThemeLifecycle::parse(&raw.3)?,
        created_at: raw.4,
        updated_at: raw.5,
        revision: raw.6,
    })
}

fn load_relationships(
    connection: &Connection,
    topic_id: &str,
) -> ServiceResult<Vec<TopicRelationshipView>> {
    let raw = {
        let mut statement = connection.prepare(
            "SELECT relationship_id,relationship_type,to_id FROM record_relationships\
             WHERE from_type='topic_candidate' AND from_id=?1 AND relationship_type LIKE 'topic_%'\
             ORDER BY created_at,relationship_id",
        )?;
        statement
            .query_map([topic_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
    };
    let mut relationships = Vec::new();
    for (relationship_id, relationship_type, target_id) in raw {
        let Some(relation_kind) = TopicRelationKind::from_relationship_type(&relationship_type)
        else {
            continue;
        };
        let target = lookup_target(connection, relation_kind, &target_id)?.ok_or_else(|| {
            WorkLoreError::InvalidVault(format!(
                "Topic relationship {relationship_id} points to missing {} {target_id}.",
                relation_kind.target_type()
            ))
        })?;
        relationships.push(TopicRelationshipView {
            relationship_id,
            relation_kind,
            target_id,
            target_label: target.0,
            target_detail: target.1,
            target_status: target.2,
            category: relation_kind.category().to_string(),
        });
    }
    Ok(relationships)
}

fn lookup_target(
    connection: &Connection,
    relation_kind: TopicRelationKind,
    target_id: &str,
) -> ServiceResult<Option<(String, String, String)>> {
    let sql = match relation_kind {
        TopicRelationKind::Story => {
            "SELECT title,summary,lifecycle_status FROM stories WHERE story_id=?1"
        }
        TopicRelationKind::ProofPoint => {
            "SELECT statement,'',status FROM proof_points WHERE proof_id=?1"
        }
        TopicRelationKind::Theme => {
            "SELECT name,description,status FROM themes WHERE theme_id=?1"
        }
        TopicRelationKind::Inspiration => {
            "SELECT title,notes,status FROM inspirations WHERE inspiration_id=?1"
        }
        TopicRelationKind::TargetContext => {
            "SELECT title,notes,status FROM target_contexts WHERE target_id=?1"
        }
    };
    Ok(connection
        .query_row(sql, [target_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .optional()?)
}

fn ensure_topic_exists(connection: &Connection, topic_id: &str) -> ServiceResult<()> {
    let found = connection
        .query_row(
            "SELECT 1 FROM topic_candidates WHERE topic_id=?1",
            [topic_id],
            |_| Ok(()),
        )
        .optional()?;
    found.ok_or_else(|| topic_not_found(topic_id))
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    let connection = Connection::open(canonical_store::database_path(vault_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

fn audit(
    connection: &Connection,
    event_type: &str,
    record_type: &str,
    record_id: &str,
    details: serde_json::Value,
) -> ServiceResult<()> {
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)\
         VALUES (?1,?2,?3,?4,'user',?5,?6)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            event_type,
            record_type,
            record_id,
            details.to_string(),
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

fn timing_fields(
    timing_class: TopicTimingClass,
    relevant_until: Option<String>,
    timely_note: Option<String>,
) -> (Option<String>, Option<String>) {
    if timing_class == TopicTimingClass::Evergreen {
        return (None, None);
    }
    (clean_optional(relevant_until), clean_optional(timely_note))
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn required_text(value: &str, label: &str) -> ServiceResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(WorkLoreError::InvalidVault(format!(
            "{label} cannot be empty."
        )))
    } else {
        Ok(trimmed.to_string())
    }
}

fn topic_not_found(topic_id: &str) -> WorkLoreError {
    WorkLoreError::InvalidVault(format!("Topic {topic_id} was not found."))
}

fn theme_not_found(theme_id: &str) -> WorkLoreError {
    WorkLoreError::InvalidVault(format!("Theme {theme_id} was not found."))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;
    use crate::{
        domain::models::SourceType,
        services::{capture_service, vault_service},
    };

    fn vault() -> PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-topic-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Topics Test").expect("create vault");
        path
    }

    fn capture_topic(path: &Path) -> String {
        let source = capture_service::create_capture_source(
            path,
            "Baseline source-of-truth data can validate a product approach before scaling it.",
            SourceType::Other,
        )
        .unwrap();
        capture_service::classify_capture_source(
            path,
            &source.source_id,
            capture_service::CaptureRole::TopicCandidate,
        )
        .unwrap()
        .target_id
    }

    fn capture_proof(path: &Path, text: &str) -> String {
        let source = capture_service::create_capture_source(path, text, SourceType::Other).unwrap();
        capture_service::classify_capture_source(
            path,
            &source.source_id,
            capture_service::CaptureRole::ProofPoint,
        )
        .unwrap()
        .target_id
    }

    #[test]
    fn captured_topic_survives_reopen_and_timing_updates_preserve_identity() {
        let path = vault();
        let topic_id = capture_topic(&path);
        canonical_store::initialize_and_migrate(&path).unwrap();
        assert_eq!(canonical_store::schema_version(&path).unwrap(), 3);

        let captured = load_topic(&path, &topic_id).unwrap();
        assert_eq!(captured.lifecycle, TopicLifecycle::Captured);
        assert_eq!(captured.timing_class, TopicTimingClass::Evergreen);
        assert_eq!(captured.relevant_until, None);

        let timely = update_topic(
            &path,
            UpdateTopicRequest {
                topic_id: topic_id.clone(),
                title: "Validate with a baseline".into(),
                summary: captured.summary.clone(),
                lifecycle: TopicLifecycle::Exploring,
                timing_class: TopicTimingClass::Timely,
                relevant_until: Some("2026-10-01".into()),
                timely_note: Some("Relevant while the baseline experiment is active.".into()),
            },
        )
        .unwrap();
        assert_eq!(timely.topic_id, topic_id);
        assert_eq!(timely.lifecycle, TopicLifecycle::Exploring);
        assert_eq!(timely.timing_class, TopicTimingClass::Timely);
        assert_eq!(timely.relevant_until.as_deref(), Some("2026-10-01"));

        let evergreen = update_topic(
            &path,
            UpdateTopicRequest {
                topic_id: topic_id.clone(),
                title: timely.title,
                summary: timely.summary,
                lifecycle: TopicLifecycle::Ready,
                timing_class: TopicTimingClass::Evergreen,
                relevant_until: Some("stale-value".into()),
                timely_note: Some("stale note".into()),
            },
        )
        .unwrap();
        assert_eq!(evergreen.topic_id, topic_id);
        assert_eq!(evergreen.lifecycle, TopicLifecycle::Ready);
        assert_eq!(evergreen.relevant_until, None);
        assert_eq!(evergreen.timely_note, None);
        assert!(evergreen.revision > captured.revision);

        let audit_count: i64 = open_connection(&path)
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM audit_events WHERE record_id=?1 AND event_type='topic_updated'",
                [&topic_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(audit_count, 2);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn topic_relationships_are_typed_idempotent_and_context_never_becomes_evidence() {
        let path = vault();
        let topic_id = capture_topic(&path);
        let topic_source_link_before: i64 = open_connection(&path)
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM record_relationships WHERE to_id=?1 AND relationship_type='capture_topic_candidate'",
                [&topic_id],
                |row| row.get(0),
            )
            .unwrap();

        let theme_a = create_theme(
            &path,
            CreateThemeRequest {
                name: "Evidence-led product work".into(),
                description: "How product teams prove an approach before scaling.".into(),
            },
        )
        .unwrap();
        let theme_b = create_theme(
            &path,
            CreateThemeRequest {
                name: "Decision quality".into(),
                description: "Making ambiguity explicit and testable.".into(),
            },
        )
        .unwrap();
        let story_a = canonical_store::create_story(&path, "Earth baseline", "Validated generation against known data").unwrap();
        let story_b = canonical_store::create_story(&path, "Audit baseline", "Compared observed behavior to a trusted control").unwrap();
        let proof_a = capture_proof(&path, "Reduced a validation cycle from months to hours.");
        let proof_b = capture_proof(&path, "Used known Earth data as a source-of-truth baseline.");
        let inspiration_id = canonical_store::create_inspiration(&path, "External article").unwrap();
        let target_id = canonical_store::create_target_context(&path, "job_description", "Product leadership role").unwrap();

        let links = [
            (TopicRelationKind::Theme, theme_a.theme_id.clone()),
            (TopicRelationKind::Theme, theme_b.theme_id.clone()),
            (TopicRelationKind::Story, story_a.story_id.clone()),
            (TopicRelationKind::Story, story_b.story_id.clone()),
            (TopicRelationKind::ProofPoint, proof_a.clone()),
            (TopicRelationKind::ProofPoint, proof_b.clone()),
            (TopicRelationKind::Inspiration, inspiration_id.clone()),
            (TopicRelationKind::TargetContext, target_id.clone()),
        ];
        for (kind, target_id) in &links {
            assert!(add_topic_relationship(&path, &topic_id, *kind, target_id)
                .unwrap()
                .changed);
        }
        assert!(!add_topic_relationship(&path, &topic_id, TopicRelationKind::Theme, &theme_a.theme_id)
            .unwrap()
            .changed);

        let topic = load_topic(&path, &topic_id).unwrap();
        assert_eq!(topic.relationships.len(), 8);
        assert_eq!(topic.relationships.iter().filter(|link| link.category == "standing").count(), 4);
        assert_eq!(topic.relationships.iter().filter(|link| link.category != "standing").count(), 4);

        let updated_theme = update_theme(
            &path,
            UpdateThemeRequest {
                theme_id: theme_a.theme_id.clone(),
                name: theme_a.name.clone(),
                description: theme_a.description.clone(),
                lifecycle: ThemeLifecycle::Active,
            },
        )
        .unwrap();
        let parked_topic = update_topic(
            &path,
            UpdateTopicRequest {
                topic_id: topic_id.clone(),
                title: topic.title.clone(),
                summary: topic.summary.clone(),
                lifecycle: TopicLifecycle::Parked,
                timing_class: TopicTimingClass::Evergreen,
                relevant_until: None,
                timely_note: None,
            },
        )
        .unwrap();
        assert_eq!(updated_theme.lifecycle, ThemeLifecycle::Active);
        assert_eq!(parked_topic.lifecycle, TopicLifecycle::Parked);

        let removed = remove_topic_relationship(
            &path,
            &topic_id,
            TopicRelationKind::Story,
            &story_a.story_id,
        )
        .unwrap();
        assert!(removed.changed);
        assert!(canonical_store::load_story(&path, &story_a.story_id).is_ok());
        assert!(load_theme(&path, &theme_a.theme_id).is_ok());

        let connection = open_connection(&path).unwrap();
        let evidence_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM evidence_records", [], |row| row.get(0))
            .unwrap();
        let proof_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM proof_points", [], |row| row.get(0))
            .unwrap();
        let source_link_after: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM record_relationships WHERE to_id=?1 AND relationship_type='capture_topic_candidate'",
                [&topic_id],
                |row| row.get(0),
            )
            .unwrap();
        let removal_audit: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM audit_events WHERE record_id=?1 AND event_type='topic_relationship_removed'",
                [&topic_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(evidence_count, 0);
        assert_eq!(proof_count, 2);
        assert_eq!(source_link_after, topic_source_link_before);
        assert_eq!(removal_audit, 1);

        let theme_targets = list_topic_link_targets(&path, TopicRelationKind::Theme).unwrap();
        assert_eq!(theme_targets.len(), 2);
        fs::remove_dir_all(path).unwrap();
    }
}
''',
)

replace_once(
    "src-tauri/src/services/mod.rs",
    "pub mod story_service;\npub mod vault_service;\n",
    "pub mod story_service;\npub mod topic_service;\npub mod vault_service;\n",
)

# Tauri command boundary.
write(
    "src-tauri/src/commands/topics.rs",
    r'''use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::topic_service::{
        self, CreateThemeRequest, CreateTopicRequest, ThemeRecordView, TopicLifecycle,
        TopicLinkTargetView, TopicRecordView, TopicRelationKind, TopicRelationshipMutationResult,
        TopicTimingClass, UpdateThemeRequest, UpdateTopicRequest,
    },
};

#[tauri::command]
pub fn create_topic(
    vault_path: String,
    request: CreateTopicRequest,
) -> CommandResult<TopicRecordView> {
    topic_service::create_topic(&PathBuf::from(vault_path), request).map_err(CommandError::from)
}

#[tauri::command]
pub fn get_topic(vault_path: String, topic_id: String) -> CommandResult<TopicRecordView> {
    topic_service::load_topic(&PathBuf::from(vault_path), &topic_id).map_err(CommandError::from)
}

#[tauri::command]
pub fn list_topics(vault_path: String) -> CommandResult<Vec<TopicRecordView>> {
    topic_service::list_topics(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn update_topic(
    vault_path: String,
    request: UpdateTopicRequest,
) -> CommandResult<TopicRecordView> {
    topic_service::update_topic(&PathBuf::from(vault_path), request).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_theme(
    vault_path: String,
    request: CreateThemeRequest,
) -> CommandResult<ThemeRecordView> {
    topic_service::create_theme(&PathBuf::from(vault_path), request).map_err(CommandError::from)
}

#[tauri::command]
pub fn get_theme(vault_path: String, theme_id: String) -> CommandResult<ThemeRecordView> {
    topic_service::load_theme(&PathBuf::from(vault_path), &theme_id).map_err(CommandError::from)
}

#[tauri::command]
pub fn list_themes(vault_path: String) -> CommandResult<Vec<ThemeRecordView>> {
    topic_service::list_themes(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn update_theme(
    vault_path: String,
    request: UpdateThemeRequest,
) -> CommandResult<ThemeRecordView> {
    topic_service::update_theme(&PathBuf::from(vault_path), request).map_err(CommandError::from)
}

#[tauri::command]
pub fn add_topic_relationship(
    vault_path: String,
    topic_id: String,
    relation_kind: TopicRelationKind,
    target_id: String,
) -> CommandResult<TopicRelationshipMutationResult> {
    topic_service::add_topic_relationship(
        &PathBuf::from(vault_path),
        &topic_id,
        relation_kind,
        &target_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn remove_topic_relationship(
    vault_path: String,
    topic_id: String,
    relation_kind: TopicRelationKind,
    target_id: String,
) -> CommandResult<TopicRelationshipMutationResult> {
    topic_service::remove_topic_relationship(
        &PathBuf::from(vault_path),
        &topic_id,
        relation_kind,
        &target_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_topic_link_targets(
    vault_path: String,
    relation_kind: TopicRelationKind,
) -> CommandResult<Vec<TopicLinkTargetView>> {
    topic_service::list_topic_link_targets(&PathBuf::from(vault_path), relation_kind)
        .map_err(CommandError::from)
}

// Keep these domain enums visible at the command boundary for generated/debug tooling.
#[allow(dead_code)]
fn _topic_command_contract(
    lifecycle: TopicLifecycle,
    timing: TopicTimingClass,
) -> (TopicLifecycle, TopicTimingClass) {
    (lifecycle, timing)
}
''',
)
replace_once(
    "src-tauri/src/commands/mod.rs",
    "pub mod stories;\npub mod vault;\n",
    "pub mod stories;\npub mod topics;\npub mod vault;\n",
)
replace_once(
    "src-tauri/src/lib.rs",
    '''    stories::{import_story_response, list_stories, set_story_status},\n    vault::{\n''',
    '''    stories::{import_story_response, list_stories, set_story_status},\n    topics::{\n        add_topic_relationship, create_theme, create_topic, get_theme, get_topic,\n        list_themes, list_topic_link_targets, list_topics, remove_topic_relationship, update_theme,\n        update_topic,\n    },\n    vault::{\n''',
)
replace_once(
    "src-tauri/src/lib.rs",
    '''            create_story_from_seed_development,\n            open_vault,\n''',
    '''            create_story_from_seed_development,\n            create_topic,\n            get_topic,\n            list_topics,\n            update_topic,\n            create_theme,\n            get_theme,\n            list_themes,\n            update_theme,\n            add_topic_relationship,\n            remove_topic_relationship,\n            list_topic_link_targets,\n            open_vault,\n''',
)

# TypeScript domain contract.
replace_once(
    "src/domain/types.ts",
    '''export interface CaptureClassificationResult {\n  sourceId: string;\n  role: CaptureRole;\n  targetId: string;\n  created: boolean;\n}\n\n''',
    '''export interface CaptureClassificationResult {\n  sourceId: string;\n  role: CaptureRole;\n  targetId: string;\n  created: boolean;\n}\n\nexport type TopicLifecycle =\n  | "captured"\n  | "exploring"\n  | "ready"\n  | "drafted"\n  | "parked"\n  | "retired";\n\nexport type TopicTimingClass = "evergreen" | "timely";\nexport type ThemeLifecycle = "emerging" | "active" | "retired";\nexport type TopicRelationKind =\n  | "story"\n  | "proof_point"\n  | "theme"\n  | "inspiration"\n  | "target_context";\n\nexport interface CreateTopicRequest {\n  title: string;\n  summary: string;\n  timingClass: TopicTimingClass;\n  relevantUntil?: string | null;\n  timelyNote?: string | null;\n}\n\nexport interface UpdateTopicRequest extends CreateTopicRequest {\n  topicId: string;\n  lifecycle: TopicLifecycle;\n}\n\nexport interface TopicRelationship {\n  relationshipId: string;\n  relationKind: TopicRelationKind;\n  targetId: string;\n  targetLabel: string;\n  targetDetail: string;\n  targetStatus: string;\n  category: "standing" | "organizing_context" | "creative_context" | "target_context" | string;\n}\n\nexport interface TopicRecord {\n  topicId: string;\n  title: string;\n  summary: string;\n  lifecycle: TopicLifecycle;\n  timingClass: TopicTimingClass;\n  relevantUntil: string | null;\n  timelyNote: string | null;\n  createdAt: string;\n  updatedAt: string;\n  revision: number;\n  relationships: TopicRelationship[];\n}\n\nexport interface CreateThemeRequest {\n  name: string;\n  description: string;\n}\n\nexport interface UpdateThemeRequest extends CreateThemeRequest {\n  themeId: string;\n  lifecycle: ThemeLifecycle;\n}\n\nexport interface ThemeRecord {\n  themeId: string;\n  name: string;\n  description: string;\n  lifecycle: ThemeLifecycle;\n  createdAt: string;\n  updatedAt: string;\n  revision: number;\n}\n\nexport interface TopicLinkTarget {\n  relationKind: TopicRelationKind;\n  targetId: string;\n  label: string;\n  detail: string;\n  status: string;\n}\n\nexport interface TopicRelationshipMutationResult {\n  topicId: string;\n  relationKind: TopicRelationKind;\n  targetId: string;\n  changed: boolean;\n}\n\n''',
)

# Application API.
replace_once(
    "src/lib/workloreApi.ts",
    '''  CaptureSource,\n  CloudIdentifierMode,\n''',
    '''  CaptureSource,\n  CloudIdentifierMode,\n  CreateThemeRequest,\n  CreateTopicRequest,\n''',
)
replace_once(
    "src/lib/workloreApi.ts",
    '''  StoryStatus,\n  StorySummary,\n  SubmitInterviewResponseRequest,\n  VaultSummary,\n''',
    '''  StoryStatus,\n  StorySummary,\n  SubmitInterviewResponseRequest,\n  ThemeRecord,\n  TopicLinkTarget,\n  TopicRecord,\n  TopicRelationKind,\n  TopicRelationshipMutationResult,\n  UpdateThemeRequest,\n  UpdateTopicRequest,\n  VaultSummary,\n''',
)
replace_once(
    "src/lib/workloreApi.ts",
    '''export async function startStorySeedDevelopment(\n''',
    '''export async function createTopic(\n  vaultPath: string,\n  request: CreateTopicRequest,\n): Promise<TopicRecord> {\n  return invoke<TopicRecord>("create_topic", { vaultPath, request });\n}\n\nexport async function getTopic(vaultPath: string, topicId: string): Promise<TopicRecord> {\n  return invoke<TopicRecord>("get_topic", { vaultPath, topicId });\n}\n\nexport async function listTopics(vaultPath: string): Promise<TopicRecord[]> {\n  return invoke<TopicRecord[]>("list_topics", { vaultPath });\n}\n\nexport async function updateTopic(\n  vaultPath: string,\n  request: UpdateTopicRequest,\n): Promise<TopicRecord> {\n  return invoke<TopicRecord>("update_topic", { vaultPath, request });\n}\n\nexport async function createTheme(\n  vaultPath: string,\n  request: CreateThemeRequest,\n): Promise<ThemeRecord> {\n  return invoke<ThemeRecord>("create_theme", { vaultPath, request });\n}\n\nexport async function listThemes(vaultPath: string): Promise<ThemeRecord[]> {\n  return invoke<ThemeRecord[]>("list_themes", { vaultPath });\n}\n\nexport async function updateTheme(\n  vaultPath: string,\n  request: UpdateThemeRequest,\n): Promise<ThemeRecord> {\n  return invoke<ThemeRecord>("update_theme", { vaultPath, request });\n}\n\nexport async function addTopicRelationship(\n  vaultPath: string,\n  topicId: string,\n  relationKind: TopicRelationKind,\n  targetId: string,\n): Promise<TopicRelationshipMutationResult> {\n  return invoke<TopicRelationshipMutationResult>("add_topic_relationship", {\n    vaultPath,\n    topicId,\n    relationKind,\n    targetId,\n  });\n}\n\nexport async function removeTopicRelationship(\n  vaultPath: string,\n  topicId: string,\n  relationKind: TopicRelationKind,\n  targetId: string,\n): Promise<TopicRelationshipMutationResult> {\n  return invoke<TopicRelationshipMutationResult>("remove_topic_relationship", {\n    vaultPath,\n    topicId,\n    relationKind,\n    targetId,\n  });\n}\n\nexport async function listTopicLinkTargets(\n  vaultPath: string,\n  relationKind: TopicRelationKind,\n): Promise<TopicLinkTarget[]> {\n  return invoke<TopicLinkTarget[]>("list_topic_link_targets", { vaultPath, relationKind });\n}\n\nexport async function startStorySeedDevelopment(\n''',
)

# Thin Topic editor.
write(
    "src/components/TopicPanel.tsx",
    r'''import { useEffect, useMemo, useState } from "react";
import type {
  TopicLifecycle,
  TopicLinkTarget,
  TopicRecord,
  TopicRelationKind,
  TopicTimingClass,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  addTopicRelationship,
  createTheme,
  getTopic,
  listTopicLinkTargets,
  removeTopicRelationship,
  updateTopic,
} from "../lib/workloreApi";
import "../topic.css";

const RELATION_KINDS: Array<{ value: TopicRelationKind; label: string }> = [
  { value: "story", label: "Story" },
  { value: "proof_point", label: "Proof point" },
  { value: "theme", label: "Theme" },
  { value: "inspiration", label: "Inspiration" },
  { value: "target_context", label: "Target context" },
];

const LIFECYCLES: Array<{ value: TopicLifecycle; label: string }> = [
  { value: "captured", label: "Captured" },
  { value: "exploring", label: "Exploring" },
  { value: "ready", label: "Ready" },
  { value: "drafted", label: "Drafted" },
  { value: "parked", label: "Parked" },
  { value: "retired", label: "Retired" },
];

export function TopicPanel({
  vaultPath,
  topicId,
  onClose,
}: {
  vaultPath: string;
  topicId: string;
  onClose: () => void;
}) {
  const [topic, setTopic] = useState<TopicRecord | null>(null);
  const [title, setTitle] = useState("");
  const [summary, setSummary] = useState("");
  const [lifecycle, setLifecycle] = useState<TopicLifecycle>("captured");
  const [timingClass, setTimingClass] = useState<TopicTimingClass>("evergreen");
  const [relevantUntil, setRelevantUntil] = useState("");
  const [timelyNote, setTimelyNote] = useState("");
  const [relationKind, setRelationKind] = useState<TopicRelationKind>("story");
  const [targetId, setTargetId] = useState("");
  const [targets, setTargets] = useState<Record<TopicRelationKind, TopicLinkTarget[]>>({
    story: [],
    proof_point: [],
    theme: [],
    inspiration: [],
    target_context: [],
  });
  const [newThemeName, setNewThemeName] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refreshAll();
  }, [vaultPath, topicId]);

  async function refreshAll() {
    setBusy("Opening topic");
    setError(null);
    try {
      const [loaded, ...targetLists] = await Promise.all([
        getTopic(vaultPath, topicId),
        ...RELATION_KINDS.map((option) => listTopicLinkTargets(vaultPath, option.value)),
      ]);
      applyTopic(loaded as TopicRecord);
      const nextTargets = { ...targets };
      RELATION_KINDS.forEach((option, index) => {
        nextTargets[option.value] = targetLists[index] as TopicLinkTarget[];
      });
      setTargets(nextTargets);
      setTargetId("");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  function applyTopic(loaded: TopicRecord) {
    setTopic(loaded);
    setTitle(loaded.title);
    setSummary(loaded.summary);
    setLifecycle(loaded.lifecycle);
    setTimingClass(loaded.timingClass);
    setRelevantUntil(loaded.relevantUntil ?? "");
    setTimelyNote(loaded.timelyNote ?? "");
  }

  async function handleSave() {
    if (!topic || !title.trim()) {
      return;
    }
    setBusy("Saving topic");
    setNotice(null);
    setError(null);
    try {
      const updated = await updateTopic(vaultPath, {
        topicId: topic.topicId,
        title,
        summary,
        lifecycle,
        timingClass,
        relevantUntil: timingClass === "timely" ? relevantUntil || null : null,
        timelyNote: timingClass === "timely" ? timelyNote || null : null,
      });
      applyTopic(updated);
      setNotice("Topic saved locally.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleAddRelationship() {
    if (!topic || !targetId) {
      return;
    }
    setBusy("Connecting topic");
    setNotice(null);
    setError(null);
    try {
      const result = await addTopicRelationship(vaultPath, topic.topicId, relationKind, targetId);
      setNotice(result.changed ? "Connection added." : "That connection already exists.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleRemoveRelationship(kind: TopicRelationKind, relatedId: string) {
    if (!topic) {
      return;
    }
    setBusy("Removing connection");
    setNotice(null);
    setError(null);
    try {
      await removeTopicRelationship(vaultPath, topic.topicId, kind, relatedId);
      setNotice("Connection removed; the linked record was kept.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function handleCreateTheme() {
    if (!topic || !newThemeName.trim()) {
      return;
    }
    setBusy("Creating theme");
    setNotice(null);
    setError(null);
    try {
      const theme = await createTheme(vaultPath, { name: newThemeName, description: "" });
      await addTopicRelationship(vaultPath, topic.topicId, "theme", theme.themeId);
      setNewThemeName("");
      setNotice("Theme created and connected.");
      await refreshAll();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  const linkedTargetIds = useMemo(
    () => new Set(topic?.relationships.map((relationship) => relationship.targetId) ?? []),
    [topic],
  );
  const availableTargets = targets[relationKind].filter(
    (target) => !linkedTargetIds.has(target.targetId),
  );
  const standing = topic?.relationships.filter((relationship) => relationship.category === "standing") ?? [];
  const context = topic?.relationships.filter((relationship) => relationship.category !== "standing") ?? [];

  return (
    <section className="topic-panel" aria-labelledby="topic-panel-heading">
      <div className="topic-panel-heading-row">
        <div>
          <p className="eyebrow">Topic</p>
          <h3 id="topic-panel-heading">Develop the idea, keep the evidence honest</h3>
          <p>
            Stories and proof points establish standing. Themes, inspiration, and target context organize or inform the idea without becoming evidence about you.
          </p>
        </div>
        <button className="quiet-button compact" onClick={onClose}>Close</button>
      </div>

      {topic ? (
        <>
          <div className="topic-editor-grid">
            <label>
              Title
              <input value={title} onChange={(event) => setTitle(event.target.value)} />
            </label>
            <label>
              Lifecycle
              <select value={lifecycle} onChange={(event) => setLifecycle(event.target.value as TopicLifecycle)}>
                {LIFECYCLES.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
              </select>
            </label>
            <label className="topic-wide-field">
              Summary
              <textarea rows={4} value={summary} onChange={(event) => setSummary(event.target.value)} />
            </label>
            <label>
              Timing
              <select value={timingClass} onChange={(event) => setTimingClass(event.target.value as TopicTimingClass)}>
                <option value="evergreen">Evergreen</option>
                <option value="timely">Timely</option>
              </select>
            </label>
            {timingClass === "timely" ? (
              <>
                <label>
                  Relevant until
                  <input type="date" value={relevantUntil} onChange={(event) => setRelevantUntil(event.target.value)} />
                </label>
                <label className="topic-wide-field">
                  Timeliness note
                  <input value={timelyNote} onChange={(event) => setTimelyNote(event.target.value)} placeholder="Why is this timely right now?" />
                </label>
              </>
            ) : null}
          </div>
          <div className="topic-actions-row">
            <button className="primary-button compact" disabled={busy !== null || !title.trim()} onClick={() => void handleSave()}>
              Save topic
            </button>
            <span>{topic.topicId}</span>
          </div>

          <div className="topic-connections-grid">
            <div className="topic-connection-section">
              <h4>Standing</h4>
              <p>Only explicit Story and Proof Point links count here.</p>
              {standing.length === 0 ? <span className="topic-empty">No standing linked yet.</span> : null}
              {standing.map((relationship) => (
                <div className="topic-link-row" key={relationship.relationshipId}>
                  <div>
                    <strong>{relationship.targetLabel}</strong>
                    <span>{kindLabel(relationship.relationKind)} · {relationship.targetStatus}</span>
                  </div>
                  <button className="quiet-button compact" onClick={() => void handleRemoveRelationship(relationship.relationKind, relationship.targetId)}>Remove</button>
                </div>
              ))}
            </div>
            <div className="topic-connection-section">
              <h4>Context</h4>
              <p>Organizing and creative context stays semantically separate from evidence.</p>
              {context.length === 0 ? <span className="topic-empty">No context linked yet.</span> : null}
              {context.map((relationship) => (
                <div className="topic-link-row" key={relationship.relationshipId}>
                  <div>
                    <strong>{relationship.targetLabel}</strong>
                    <span>{kindLabel(relationship.relationKind)} · {relationship.targetStatus}</span>
                  </div>
                  <button className="quiet-button compact" onClick={() => void handleRemoveRelationship(relationship.relationKind, relationship.targetId)}>Remove</button>
                </div>
              ))}
            </div>
          </div>

          <div className="topic-connect-row">
            <select value={relationKind} onChange={(event) => { setRelationKind(event.target.value as TopicRelationKind); setTargetId(""); }}>
              {RELATION_KINDS.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
            </select>
            <select value={targetId} onChange={(event) => setTargetId(event.target.value)}>
              <option value="">Select an existing {kindLabel(relationKind).toLowerCase()}</option>
              {availableTargets.map((target) => <option key={target.targetId} value={target.targetId}>{target.label}</option>)}
            </select>
            <button className="secondary-button compact" disabled={!targetId || busy !== null} onClick={() => void handleAddRelationship()}>Connect</button>
          </div>

          <div className="topic-theme-create-row">
            <input value={newThemeName} onChange={(event) => setNewThemeName(event.target.value)} placeholder="New theme name" />
            <button className="secondary-button compact" disabled={!newThemeName.trim() || busy !== null} onClick={() => void handleCreateTheme()}>
              Create + connect theme
            </button>
          </div>
        </>
      ) : null}

      <div className="topic-feedback" aria-live="polite">
        {busy ? <span>{busy}...</span> : null}
        {notice ? <span>{notice}</span> : null}
        {error ? <span className="capture-error">{error}</span> : null}
      </div>
    </section>
  );
}

function kindLabel(kind: TopicRelationKind): string {
  return RELATION_KINDS.find((option) => option.value === kind)?.label ?? kind;
}
''',
)
write(
    "src/topic.css",
    r'''.topic-panel {
  margin-top: 1.25rem;
  padding: 1.25rem;
  border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.14));
  border-radius: 1rem;
  background: rgba(8, 18, 28, 0.35);
}

.topic-panel-heading-row,
.topic-actions-row,
.topic-connect-row,
.topic-theme-create-row,
.topic-link-row {
  display: flex;
  gap: 0.75rem;
  align-items: center;
  justify-content: space-between;
}

.topic-panel-heading-row {
  align-items: flex-start;
}

.topic-panel-heading-row p,
.topic-connection-section p,
.topic-empty,
.topic-actions-row span,
.topic-link-row span {
  color: var(--text-muted, #a8b3bd);
}

.topic-editor-grid,
.topic-connections-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.9rem;
  margin-top: 1rem;
}

.topic-editor-grid label {
  display: grid;
  gap: 0.4rem;
  font-weight: 600;
}

.topic-editor-grid input,
.topic-editor-grid textarea,
.topic-editor-grid select,
.topic-connect-row select,
.topic-theme-create-row input {
  width: 100%;
}

.topic-wide-field {
  grid-column: 1 / -1;
}

.topic-actions-row,
.topic-connect-row,
.topic-theme-create-row {
  margin-top: 0.9rem;
}

.topic-connection-section {
  min-width: 0;
  padding: 0.9rem;
  border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.1));
  border-radius: 0.8rem;
}

.topic-link-row {
  padding: 0.65rem 0;
  border-top: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
}

.topic-link-row div {
  display: grid;
  gap: 0.2rem;
  min-width: 0;
}

.topic-feedback {
  display: flex;
  gap: 0.75rem;
  margin-top: 0.8rem;
  min-height: 1.4rem;
}

@media (max-width: 780px) {
  .topic-editor-grid,
  .topic-connections-grid {
    grid-template-columns: 1fr;
  }

  .topic-connect-row,
  .topic-theme-create-row {
    align-items: stretch;
    flex-direction: column;
  }
}
''',
)

# Wire Topic editor into the existing thin Capture surface.
replace_once(
    "src/components/CapturePanel.tsx",
    'import { SeedDevelopmentPanel } from "./SeedDevelopmentPanel";\n',
    'import { SeedDevelopmentPanel } from "./SeedDevelopmentPanel";\nimport { TopicPanel } from "./TopicPanel";\n',
)
replace_once(
    "src/components/CapturePanel.tsx",
    '''  const [developmentSeedId, setDevelopmentSeedId] = useState<string | null>(null);\n''',
    '''  const [developmentSeedId, setDevelopmentSeedId] = useState<string | null>(null);\n  const [developmentTopicId, setDevelopmentTopicId] = useState<string | null>(null);\n''',
)
replace_once(
    "src/components/CapturePanel.tsx",
    '''    setDevelopmentSeedId(null);\n    void refreshRecent();\n''',
    '''    setDevelopmentSeedId(null);\n    setDevelopmentTopicId(null);\n    void refreshRecent();\n''',
)
replace_once(
    "src/components/CapturePanel.tsx",
    '''  const storySeedClassification = saved?.classifications.find(\n    (classification) => classification.role === "story_seed",\n  );\n\n''',
    '''  const storySeedClassification = saved?.classifications.find(\n    (classification) => classification.role === "story_seed",\n  );\n  const topicClassification = saved?.classifications.find(\n    (classification) => classification.role === "topic_candidate",\n  );\n\n''',
)
replace_once(
    "src/components/CapturePanel.tsx",
    '''            {storySeedClassification ? (\n              <button\n                className="primary-button compact"\n                disabled={busy !== null}\n                onClick={() => setDevelopmentSeedId(storySeedClassification.targetId)}\n              >\n                Develop this story seed\n              </button>\n            ) : null}\n''',
    '''            {storySeedClassification ? (\n              <button\n                className="primary-button compact"\n                disabled={busy !== null}\n                onClick={() => setDevelopmentSeedId(storySeedClassification.targetId)}\n              >\n                Develop this story seed\n              </button>\n            ) : null}\n            {topicClassification ? (\n              <button\n                className="primary-button compact"\n                disabled={busy !== null}\n                onClick={() => setDevelopmentTopicId(topicClassification.targetId)}\n              >\n                Open this topic\n              </button>\n            ) : null}\n''',
)
replace_once(
    "src/components/CapturePanel.tsx",
    '''      {developmentSeedId ? (\n        <SeedDevelopmentPanel\n          vaultPath={vaultPath}\n          seedId={developmentSeedId}\n          onClose={() => setDevelopmentSeedId(null)}\n        />\n      ) : null}\n\n''',
    '''      {developmentSeedId ? (\n        <SeedDevelopmentPanel\n          vaultPath={vaultPath}\n          seedId={developmentSeedId}\n          onClose={() => setDevelopmentSeedId(null)}\n        />\n      ) : null}\n\n      {developmentTopicId ? (\n        <TopicPanel\n          vaultPath={vaultPath}\n          topicId={developmentTopicId}\n          onClose={() => setDevelopmentTopicId(null)}\n        />\n      ) : null}\n\n''',
)

# Mark the bounded task active while implementation is under validation.
replace_once(
    "refs/planning/todos.yaml",
    '''  - id: task-028\n    status: next\n''',
    '''  - id: task-028\n    status: in_progress\n''',
)

print("Applied bounded durable Topics/Themes slice.")
