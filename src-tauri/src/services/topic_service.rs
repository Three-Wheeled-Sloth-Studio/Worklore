use std::{path::Path, time::Duration};

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

pub fn create_topic(
    vault_path: &Path,
    request: CreateTopicRequest,
) -> ServiceResult<TopicRecordView> {
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
        "INSERT INTO topic_candidates(topic_id,title,summary,status,provenance_json,created_at,updated_at,revision, \
         timing_class,relevant_until,timely_note) \
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
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    ids.into_iter()
        .map(|topic_id| load_topic_from_connection(&connection, &topic_id))
        .collect()
}

pub fn update_topic(
    vault_path: &Path,
    request: UpdateTopicRequest,
) -> ServiceResult<TopicRecordView> {
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
        "UPDATE topic_candidates SET title=?2,summary=?3,status=?4,timing_class=?5,relevant_until=?6, \
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

pub fn create_theme(
    vault_path: &Path,
    request: CreateThemeRequest,
) -> ServiceResult<ThemeRecordView> {
    canonical_store::initialize(vault_path)?;
    let name = required_text(&request.name, "Theme name")?;
    let description = request.description.trim().to_string();
    let connection = open_connection(vault_path)?;
    let theme_id = format!("theme_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO themes(theme_id,name,description,status,provenance_json,created_at,updated_at,revision) \
         VALUES (?1,?2,?3,'emerging',?4,?5,?5,1)",
        params![
            &theme_id,
            &name,
            &description,
            json!({"creationActor":"user"}).to_string(),
            &now
        ],
    )?;
    audit(&connection, "theme_created", "theme", &theme_id, json!({}))?;
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
            "SELECT theme_id,name,description,status,created_at,updated_at,revision \
             FROM themes ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = statement
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
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    raw.into_iter().map(theme_from_row).collect()
}

pub fn update_theme(
    vault_path: &Path,
    request: UpdateThemeRequest,
) -> ServiceResult<ThemeRecordView> {
    canonical_store::initialize(vault_path)?;
    let name = required_text(&request.name, "Theme name")?;
    let description = request.description.trim().to_string();
    let mut connection = open_connection(vault_path)?;
    let before = load_theme_from_connection(&connection, &request.theme_id)?;
    let now = Utc::now().to_rfc3339();
    let tx = connection.transaction()?;
    let changed = tx.execute(
        "UPDATE themes SET name=?2,description=?3,status=?4,updated_at=?5,revision=revision+1 \
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
        "INSERT OR IGNORE INTO record_relationships(relationship_id,from_type,from_id,relationship_type,to_type,to_id, \
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
        "DELETE FROM record_relationships WHERE from_type='topic_candidate' AND from_id=?1 \
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
            "SELECT topic_id,title,summary,status,timing_class,relevant_until,timely_note,created_at,updated_at,revision \
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
            "SELECT relationship_id,relationship_type,to_id FROM record_relationships \
             WHERE from_type='topic_candidate' AND from_id=?1 AND relationship_type LIKE 'topic_%' \
             ORDER BY created_at,relationship_id",
        )?;
        let rows = statement
            .query_map([topic_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
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
        TopicRelationKind::Theme => "SELECT name,description,status FROM themes WHERE theme_id=?1",
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
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at) \
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
        assert_eq!(canonical_store::schema_version(&path).unwrap(), 8);

        let captured = load_topic(&path, &topic_id).unwrap();
        let listed = list_topics(&path).unwrap();
        assert!(listed.iter().any(|topic| topic.topic_id == topic_id));
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
        let story_a = canonical_store::create_story(
            &path,
            "Earth baseline",
            "Validated generation against known data",
        )
        .unwrap();
        let story_b = canonical_store::create_story(
            &path,
            "Audit baseline",
            "Compared observed behavior to a trusted control",
        )
        .unwrap();
        let proof_a = capture_proof(&path, "Reduced a validation cycle from months to hours.");
        let proof_b = capture_proof(
            &path,
            "Used known Earth data as a source-of-truth baseline.",
        );
        let inspiration_id =
            canonical_store::create_inspiration(&path, "External article").unwrap();
        let target_id = canonical_store::create_target_context(
            &path,
            "job_description",
            "Product leadership role",
        )
        .unwrap();

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
            assert!(
                add_topic_relationship(&path, &topic_id, *kind, target_id)
                    .unwrap()
                    .changed
            );
        }
        assert!(
            !add_topic_relationship(
                &path,
                &topic_id,
                TopicRelationKind::Theme,
                &theme_a.theme_id
            )
            .unwrap()
            .changed
        );

        let topic = load_topic(&path, &topic_id).unwrap();
        assert_eq!(topic.relationships.len(), 8);
        assert_eq!(
            topic
                .relationships
                .iter()
                .filter(|link| link.category == "standing")
                .count(),
            4
        );
        assert_eq!(
            topic
                .relationships
                .iter()
                .filter(|link| link.category != "standing")
                .count(),
            4
        );

        let listed_themes = list_themes(&path).unwrap();
        assert_eq!(listed_themes.len(), 2);

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
            .query_row("SELECT COUNT(*) FROM evidence_records", [], |row| {
                row.get(0)
            })
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
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }
}
