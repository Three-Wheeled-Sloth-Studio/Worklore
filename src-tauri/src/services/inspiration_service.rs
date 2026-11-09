use std::{path::Path, time::Duration};

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
pub enum InspirationLifecycle {
    Saved,
    Processed,
    Archived,
}

impl InspirationLifecycle {
    fn as_str(self) -> &'static str {
        match self {
            Self::Saved => "saved",
            Self::Processed => "processed",
            Self::Archived => "archived",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "saved" => Ok(Self::Saved),
            "processed" => Ok(Self::Processed),
            "archived" => Ok(Self::Archived),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Inspiration lifecycle {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InspirationRelationKind {
    Topic,
    Theme,
}

impl InspirationRelationKind {
    fn relationship_type(self) -> &'static str {
        match self {
            Self::Topic => "topic_inspiration",
            Self::Theme => "inspiration_theme",
        }
    }

    fn target_type(self) -> &'static str {
        match self {
            Self::Topic => "topic_candidate",
            Self::Theme => "theme",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InspirationExcerptInput {
    pub text: String,
    pub locator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InspirationExcerptView {
    pub text: String,
    pub locator: String,
    pub source_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationSourceView {
    pub source_id: String,
    pub source_type: String,
    pub display_name: String,
    pub source_origin: String,
    pub original_file_name: String,
    pub stored_path: String,
    pub source_url: Option<String>,
    pub captured_text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationRelationshipView {
    pub relationship_id: String,
    pub relation_kind: InspirationRelationKind,
    pub target_id: String,
    pub target_label: String,
    pub target_detail: String,
    pub target_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationRecordView {
    pub inspiration_id: String,
    pub source: Option<InspirationSourceView>,
    pub title: String,
    pub lifecycle: InspirationLifecycle,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
    pub source_published_at: Option<String>,
    pub summary: String,
    pub takeaways: Vec<String>,
    pub excerpts: Vec<InspirationExcerptView>,
    pub why_interesting: String,
    pub user_reaction: String,
    pub concepts: Vec<String>,
    pub questions: Vec<String>,
    pub counterpoints: Vec<String>,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
    pub relationships: Vec<InspirationRelationshipView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInspirationResult {
    pub inspiration: InspirationRecordView,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInspirationRequest {
    pub inspiration_id: String,
    pub title: String,
    pub lifecycle: InspirationLifecycle,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
    pub source_published_at: Option<String>,
    pub summary: String,
    pub takeaways: Vec<String>,
    pub excerpts: Vec<InspirationExcerptInput>,
    pub why_interesting: String,
    pub user_reaction: String,
    pub concepts: Vec<String>,
    pub questions: Vec<String>,
    pub counterpoints: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationLinkTargetView {
    pub relation_kind: InspirationRelationKind,
    pub target_id: String,
    pub label: String,
    pub detail: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationRelationshipMutationResult {
    pub inspiration_id: String,
    pub relation_kind: InspirationRelationKind,
    pub target_id: String,
    pub changed: bool,
}

pub fn create_inspiration_from_source(
    vault_path: &Path,
    source_id: &str,
) -> ServiceResult<CreateInspirationResult> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let source = load_source_from_connection(&connection, source_id)?
        .ok_or(WorkLoreError::SourceNotFound)?;

    if let Some(existing_id) = connection
        .query_row(
            "SELECT inspiration_id FROM inspirations WHERE source_id=?1 ORDER BY created_at LIMIT 1",
            [source_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(CreateInspirationResult {
            inspiration: load_inspiration_from_connection(&connection, &existing_id)?,
            created: false,
        });
    }

    let inspiration_id = format!("inspiration_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    let source_url = source.source_url.clone().or_else(|| {
        let candidate = source.captured_text.trim();
        if candidate.starts_with("https://") || candidate.starts_with("http://") {
            Some(
                candidate
                    .lines()
                    .next()
                    .unwrap_or(candidate)
                    .trim()
                    .to_string(),
            )
        } else {
            None
        }
    });

    connection.execute(
        "INSERT INTO inspirations(
           inspiration_id,source_id,title,notes,status,provenance_json,created_at,updated_at,revision,
           source_url,source_title,summary,takeaways_json,excerpts_json,why_interesting,user_reaction,
           concepts_json,questions_json,counterpoints_json)
         VALUES (?1,?2,?3,'','saved',?4,?5,?5,1,?6,?7,'','[]','[]','','','[]','[]','[]')",
        params![
            &inspiration_id,
            source_id,
            &source.display_name,
            json!({
                "creationActor": "user",
                "creationPath": "inspiration_from_source",
                "sourceId": source_id
            })
            .to_string(),
            &now,
            &source_url,
            &source.display_name,
        ],
    )?;
    connection.execute(
        "INSERT OR IGNORE INTO record_relationships(
           relationship_id,from_type,from_id,relationship_type,to_type,to_id,provenance_json,created_at)
         VALUES (?1,'source',?2,'source_inspiration','inspiration',?3,?4,?5)",
        params![
            format!("relationship_{}", Uuid::now_v7()),
            source_id,
            &inspiration_id,
            json!({"creationActor":"user"}).to_string(),
            &now,
        ],
    )?;
    audit(
        &connection,
        "inspiration_created",
        &inspiration_id,
        json!({"sourceId": source_id}),
    )?;

    Ok(CreateInspirationResult {
        inspiration: load_inspiration_from_connection(&connection, &inspiration_id)?,
        created: true,
    })
}

pub fn load_inspiration(
    vault_path: &Path,
    inspiration_id: &str,
) -> ServiceResult<InspirationRecordView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    load_inspiration_from_connection(&connection, inspiration_id)
}

pub fn list_inspirations(vault_path: &Path) -> ServiceResult<Vec<InspirationRecordView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let ids = {
        let mut statement = connection.prepare(
            "SELECT inspiration_id FROM inspirations ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    ids.into_iter()
        .map(|id| load_inspiration_from_connection(&connection, &id))
        .collect()
}

pub fn update_inspiration(
    vault_path: &Path,
    request: UpdateInspirationRequest,
) -> ServiceResult<InspirationRecordView> {
    canonical_store::initialize(vault_path)?;
    let title = required_text(&request.title, "Inspiration title")?;
    let mut connection = open_connection(vault_path)?;
    let before = load_inspiration_from_connection(&connection, &request.inspiration_id)?;
    let excerpt_source_id = before
        .source
        .as_ref()
        .map(|source| source.source_id.clone());
    if !request.excerpts.is_empty() && excerpt_source_id.is_none() {
        return Err(WorkLoreError::InvalidVault(
            "An Inspiration needs Source provenance before excerpts can be stored.".to_string(),
        ));
    }

    let takeaways = normalize_string_list(request.takeaways);
    let concepts = normalize_string_list(request.concepts);
    let questions = normalize_string_list(request.questions);
    let counterpoints = normalize_string_list(request.counterpoints);
    let excerpts = normalize_excerpts(request.excerpts, excerpt_source_id.as_deref().unwrap_or(""));
    let source_url = optional_trimmed(request.source_url);
    let source_title = optional_trimmed(request.source_title);
    let source_author = optional_trimmed(request.source_author);
    let source_published_at = optional_trimmed(request.source_published_at);
    let now = Utc::now().to_rfc3339();
    let tx = connection.transaction()?;
    let changed = tx.execute(
        "UPDATE inspirations SET
           title=?2,status=?3,source_url=?4,source_title=?5,source_author=?6,source_published_at=?7,
           summary=?8,takeaways_json=?9,excerpts_json=?10,why_interesting=?11,user_reaction=?12,
           concepts_json=?13,questions_json=?14,counterpoints_json=?15,notes=?16,
           updated_at=?17,revision=revision+1
         WHERE inspiration_id=?1",
        params![
            &request.inspiration_id,
            &title,
            request.lifecycle.as_str(),
            &source_url,
            &source_title,
            &source_author,
            &source_published_at,
            request.summary.trim(),
            serde_json::to_string(&takeaways)?,
            serde_json::to_string(&excerpts)?,
            request.why_interesting.trim(),
            request.user_reaction.trim(),
            serde_json::to_string(&concepts)?,
            serde_json::to_string(&questions)?,
            serde_json::to_string(&counterpoints)?,
            request.notes.trim(),
            &now,
        ],
    )?;
    if changed == 0 {
        return Err(inspiration_not_found(&request.inspiration_id));
    }
    audit(
        &tx,
        "inspiration_updated",
        &request.inspiration_id,
        json!({
            "beforeLifecycle": before.lifecycle.as_str(),
            "afterLifecycle": request.lifecycle.as_str(),
            "sourceId": before.source.as_ref().map(|source| source.source_id.clone())
        }),
    )?;
    tx.commit()?;
    load_inspiration(vault_path, &request.inspiration_id)
}

pub fn add_inspiration_relationship(
    vault_path: &Path,
    inspiration_id: &str,
    relation_kind: InspirationRelationKind,
    target_id: &str,
) -> ServiceResult<InspirationRelationshipMutationResult> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    ensure_inspiration_exists(&connection, inspiration_id)?;
    ensure_target_exists(&connection, relation_kind, target_id)?;
    let now = Utc::now().to_rfc3339();
    let tx = connection.transaction()?;
    let changed = match relation_kind {
        InspirationRelationKind::Topic => tx.execute(
            "INSERT OR IGNORE INTO record_relationships(
               relationship_id,from_type,from_id,relationship_type,to_type,to_id,provenance_json,created_at)
             VALUES (?1,'topic_candidate',?2,'topic_inspiration','inspiration',?3,?4,?5)",
            params![
                format!("relationship_{}", Uuid::now_v7()),
                target_id,
                inspiration_id,
                json!({"creationActor":"user"}).to_string(),
                &now,
            ],
        )? > 0,
        InspirationRelationKind::Theme => tx.execute(
            "INSERT OR IGNORE INTO record_relationships(
               relationship_id,from_type,from_id,relationship_type,to_type,to_id,provenance_json,created_at)
             VALUES (?1,'inspiration',?2,'inspiration_theme','theme',?3,?4,?5)",
            params![
                format!("relationship_{}", Uuid::now_v7()),
                inspiration_id,
                target_id,
                json!({"creationActor":"user"}).to_string(),
                &now,
            ],
        )? > 0,
    };
    if changed {
        audit(
            &tx,
            "inspiration_relationship_added",
            inspiration_id,
            json!({
                "relationKind": relation_kind.relationship_type(),
                "targetType": relation_kind.target_type(),
                "targetId": target_id
            }),
        )?;
    }
    tx.commit()?;
    Ok(InspirationRelationshipMutationResult {
        inspiration_id: inspiration_id.to_string(),
        relation_kind,
        target_id: target_id.to_string(),
        changed,
    })
}

pub fn remove_inspiration_relationship(
    vault_path: &Path,
    inspiration_id: &str,
    relation_kind: InspirationRelationKind,
    target_id: &str,
) -> ServiceResult<InspirationRelationshipMutationResult> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    ensure_inspiration_exists(&connection, inspiration_id)?;
    let tx = connection.transaction()?;
    let changed = match relation_kind {
        InspirationRelationKind::Topic => tx.execute(
            "DELETE FROM record_relationships
             WHERE from_type='topic_candidate' AND from_id=?1 AND relationship_type='topic_inspiration'
               AND to_type='inspiration' AND to_id=?2",
            params![target_id, inspiration_id],
        )? > 0,
        InspirationRelationKind::Theme => tx.execute(
            "DELETE FROM record_relationships
             WHERE from_type='inspiration' AND from_id=?1 AND relationship_type='inspiration_theme'
               AND to_type='theme' AND to_id=?2",
            params![inspiration_id, target_id],
        )? > 0,
    };
    if changed {
        audit(
            &tx,
            "inspiration_relationship_removed",
            inspiration_id,
            json!({
                "relationKind": relation_kind.relationship_type(),
                "targetType": relation_kind.target_type(),
                "targetId": target_id
            }),
        )?;
    }
    tx.commit()?;
    Ok(InspirationRelationshipMutationResult {
        inspiration_id: inspiration_id.to_string(),
        relation_kind,
        target_id: target_id.to_string(),
        changed,
    })
}

pub fn list_inspiration_link_targets(
    vault_path: &Path,
    relation_kind: InspirationRelationKind,
) -> ServiceResult<Vec<InspirationLinkTargetView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let sql = match relation_kind {
        InspirationRelationKind::Topic => {
            "SELECT topic_id,title,summary,status FROM topic_candidates ORDER BY updated_at DESC"
        }
        InspirationRelationKind::Theme => {
            "SELECT theme_id,name,description,status FROM themes ORDER BY updated_at DESC"
        }
    };
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map([], |row| {
        Ok(InspirationLinkTargetView {
            relation_kind,
            target_id: row.get(0)?,
            label: row.get(1)?,
            detail: row.get(2)?,
            status: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn load_inspiration_from_connection(
    connection: &Connection,
    inspiration_id: &str,
) -> ServiceResult<InspirationRecordView> {
    let raw = connection
        .query_row(
            "SELECT inspiration_id,source_id,title,notes,status,source_url,source_title,source_author,
                    source_published_at,summary,takeaways_json,excerpts_json,why_interesting,user_reaction,
                    concepts_json,questions_json,counterpoints_json,created_at,updated_at,revision
             FROM inspirations WHERE inspiration_id=?1",
            [inspiration_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, String>(13)?,
                    row.get::<_, String>(14)?,
                    row.get::<_, String>(15)?,
                    row.get::<_, String>(16)?,
                    row.get::<_, String>(17)?,
                    row.get::<_, String>(18)?,
                    row.get::<_, u32>(19)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| inspiration_not_found(inspiration_id))?;

    let source = match raw.1.as_deref() {
        Some(source_id) => load_source_from_connection(connection, source_id)?,
        None => None,
    };
    Ok(InspirationRecordView {
        inspiration_id: raw.0,
        source,
        title: raw.2,
        lifecycle: InspirationLifecycle::parse(&raw.4)?,
        source_url: raw.5,
        source_title: raw.6,
        source_author: raw.7,
        source_published_at: raw.8,
        summary: raw.9,
        takeaways: parse_json_list(&raw.10)?,
        excerpts: parse_json_excerpts(&raw.11)?,
        why_interesting: raw.12,
        user_reaction: raw.13,
        concepts: parse_json_list(&raw.14)?,
        questions: parse_json_list(&raw.15)?,
        counterpoints: parse_json_list(&raw.16)?,
        notes: raw.3,
        created_at: raw.17,
        updated_at: raw.18,
        revision: raw.19,
        relationships: load_relationships(connection, inspiration_id)?,
    })
}

fn load_source_from_connection(
    connection: &Connection,
    source_id: &str,
) -> ServiceResult<Option<InspirationSourceView>> {
    let raw = connection
        .query_row(
            "SELECT source_id,source_type,display_name,source_origin,original_file_name,stored_path,
                    provenance_json,COALESCE(captured_text,'')
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
                ))
            },
        )
        .optional()?;
    raw.map(|row| {
        Ok(InspirationSourceView {
            source_id: row.0,
            source_type: row.1,
            display_name: row.2,
            source_origin: row.3,
            original_file_name: row.4,
            stored_path: row.5,
            source_url: source_url_from_provenance(&row.6),
            captured_text: row.7,
        })
    })
    .transpose()
}

fn load_relationships(
    connection: &Connection,
    inspiration_id: &str,
) -> ServiceResult<Vec<InspirationRelationshipView>> {
    let mut out = Vec::new();
    {
        let mut statement = connection.prepare(
            "SELECT r.relationship_id,t.topic_id,t.title,t.summary,t.status
             FROM record_relationships r
             JOIN topic_candidates t ON t.topic_id=r.from_id
             WHERE r.from_type='topic_candidate' AND r.relationship_type='topic_inspiration'
               AND r.to_type='inspiration' AND r.to_id=?1
             ORDER BY r.created_at,r.relationship_id",
        )?;
        let rows = statement.query_map([inspiration_id], |row| {
            Ok(InspirationRelationshipView {
                relationship_id: row.get(0)?,
                relation_kind: InspirationRelationKind::Topic,
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
             WHERE r.from_type='inspiration' AND r.from_id=?1
               AND r.relationship_type='inspiration_theme' AND r.to_type='theme'
             ORDER BY r.created_at,r.relationship_id",
        )?;
        let rows = statement.query_map([inspiration_id], |row| {
            Ok(InspirationRelationshipView {
                relationship_id: row.get(0)?,
                relation_kind: InspirationRelationKind::Theme,
                target_id: row.get(1)?,
                target_label: row.get(2)?,
                target_detail: row.get(3)?,
                target_status: row.get(4)?,
            })
        })?;
        out.extend(rows.collect::<Result<Vec<_>, _>>()?);
    }
    Ok(out)
}

fn ensure_inspiration_exists(connection: &Connection, inspiration_id: &str) -> ServiceResult<()> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM inspirations WHERE inspiration_id=?1)",
        [inspiration_id],
        |row| row.get(0),
    )?;
    if exists {
        Ok(())
    } else {
        Err(inspiration_not_found(inspiration_id))
    }
}

fn ensure_target_exists(
    connection: &Connection,
    relation_kind: InspirationRelationKind,
    target_id: &str,
) -> ServiceResult<()> {
    let sql = match relation_kind {
        InspirationRelationKind::Topic => {
            "SELECT EXISTS(SELECT 1 FROM topic_candidates WHERE topic_id=?1)"
        }
        InspirationRelationKind::Theme => "SELECT EXISTS(SELECT 1 FROM themes WHERE theme_id=?1)",
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

fn normalize_excerpts(
    values: Vec<InspirationExcerptInput>,
    source_id: &str,
) -> Vec<InspirationExcerptView> {
    let mut out = Vec::new();
    for value in values {
        let text = value.text.trim().to_string();
        if text.is_empty() {
            continue;
        }
        let locator = value.locator.trim().to_string();
        if out.iter().any(|existing: &InspirationExcerptView| {
            existing.text == text && existing.locator == locator
        }) {
            continue;
        }
        out.push(InspirationExcerptView {
            text,
            locator,
            source_id: source_id.to_string(),
        });
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

fn optional_trimmed(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_json_list(value: &str) -> ServiceResult<Vec<String>> {
    Ok(serde_json::from_str(value)?)
}

fn parse_json_excerpts(value: &str) -> ServiceResult<Vec<InspirationExcerptView>> {
    Ok(serde_json::from_str(value)?)
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

fn inspiration_not_found(inspiration_id: &str) -> WorkLoreError {
    WorkLoreError::InvalidVault(format!("Inspiration {inspiration_id} was not found."))
}

fn audit(
    connection: &Connection,
    event_type: &str,
    inspiration_id: &str,
    details: Value,
) -> ServiceResult<()> {
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,?2,'inspiration',?3,'user',?4,?5)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            event_type,
            inspiration_id,
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
        let path = std::env::temp_dir().join(format!("worklore-inspiration-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Inspiration Test").expect("create vault");
        path
    }

    fn row_count(connection: &Connection, table: &str) -> i64 {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        connection
            .query_row(&sql, [], |row| row.get(0))
            .expect("count rows")
    }

    #[test]
    fn pasted_url_source_stays_neutral_until_inspiration_creation_and_reopens() {
        let path = vault();
        let source = capture_service::create_capture_source(
            &path,
            "https://example.com/architecture-note",
            SourceType::Other,
        )
        .expect("capture URL");

        let connection = open_connection(&path).expect("open database");
        assert_eq!(row_count(&connection, "inspirations"), 0);
        let paths: (String, String) = connection
            .query_row(
                "SELECT stored_path,original_file_name FROM sources WHERE source_id=?1",
                [&source.source_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("source paths");
        assert_eq!(paths, (String::new(), String::new()));
        drop(connection);

        let created =
            create_inspiration_from_source(&path, &source.source_id).expect("create inspiration");
        assert!(created.created);
        assert_eq!(
            created.inspiration.source_url.as_deref(),
            Some("https://example.com/architecture-note")
        );
        let reopened =
            load_inspiration(&path, &created.inspiration.inspiration_id).expect("reopen");
        assert_eq!(reopened.inspiration_id, created.inspiration.inspiration_id);
        assert_eq!(
            reopened
                .source
                .as_ref()
                .map(|source| source.source_id.as_str()),
            Some(source.source_id.as_str())
        );

        fs::remove_dir_all(path).expect("cleanup");
    }

    #[test]
    fn structured_updates_preserve_identity_and_excerpt_source_attribution() {
        let path = vault();
        let source = capture_service::create_capture_source(
            &path,
            "External article excerpt about product discovery.",
            SourceType::Other,
        )
        .expect("capture text");
        let created =
            create_inspiration_from_source(&path, &source.source_id).expect("create inspiration");
        let inspiration_id = created.inspiration.inspiration_id.clone();

        let updated = update_inspiration(
            &path,
            UpdateInspirationRequest {
                inspiration_id: inspiration_id.clone(),
                title: "Discovery article".into(),
                lifecycle: InspirationLifecycle::Processed,
                source_url: Some("https://example.com/discovery".into()),
                source_title: Some("Discovery article".into()),
                source_author: Some("Synthetic Author".into()),
                source_published_at: Some("2026-09-01".into()),
                summary: "A concise external argument.".into(),
                takeaways: vec!["Start with evidence".into(), "Start with evidence".into()],
                excerpts: vec![InspirationExcerptInput {
                    text: "External wording stays attributed.".into(),
                    locator: "paragraph 4".into(),
                }],
                why_interesting: "It challenges a default assumption.".into(),
                user_reaction: "I agree with the evidence point but not the sequencing.".into(),
                concepts: vec!["source-of-truth validation".into()],
                questions: vec!["Where does the baseline come from?".into()],
                counterpoints: vec!["Baseline data can also encode old assumptions.".into()],
                notes: "Follow up with a concrete example.".into(),
            },
        )
        .expect("update inspiration");

        assert_eq!(updated.inspiration_id, inspiration_id);
        assert_eq!(updated.lifecycle, InspirationLifecycle::Processed);
        assert_eq!(updated.takeaways, vec!["Start with evidence"]);
        assert_eq!(updated.excerpts.len(), 1);
        assert_eq!(updated.excerpts[0].source_id, source.source_id);
        assert_eq!(
            updated.user_reaction,
            "I agree with the evidence point but not the sequencing."
        );

        let reopened = load_inspiration(&path, &inspiration_id).expect("reopen");
        assert_eq!(reopened.revision, 2);
        assert_eq!(reopened.excerpts[0].locator, "paragraph 4");
        fs::remove_dir_all(path).expect("cleanup");
    }

    #[test]
    fn imported_file_source_can_back_inspiration_without_semantic_collapse() {
        let path = vault();
        canonical_store::initialize(&path).expect("initialize");
        let connection = open_connection(&path).expect("open database");
        let now = Utc::now().to_rfc3339();
        connection
            .execute(
                "INSERT INTO sources(
                   source_id,source_type,display_name,stored_path,original_file_name,media_type,byte_size,
                   content_hash,lifecycle_status,imported_at,updated_at,extraction_json,privacy_json,
                   provenance_json,tags_json,revision,source_origin,captured_text)
                 VALUES (
                   'source_file','other','Saved paper','sources/other/paper.pdf','paper.pdf','application/pdf',42,
                   ?1,'active',?2,?2,'{}','{}',?3,'[]',1,'imported_file',NULL)",
                params![
                    "0".repeat(64),
                    &now,
                    json!({"creationActor":"user","importMethod":"file_copy"}).to_string()
                ],
            )
            .expect("insert imported source");
        drop(connection);

        let created =
            create_inspiration_from_source(&path, "source_file").expect("create from file source");
        let source = created.inspiration.source.expect("source view");
        assert_eq!(source.source_origin, "imported_file");
        assert_eq!(source.original_file_name, "paper.pdf");
        assert_eq!(source.stored_path, "sources/other/paper.pdf");

        let connection = open_connection(&path).expect("reopen database");
        assert_eq!(row_count(&connection, "evidence_records"), 0);
        assert_eq!(row_count(&connection, "proof_points"), 0);
        assert_eq!(row_count(&connection, "stories"), 0);
        drop(connection);
        fs::remove_dir_all(path).expect("cleanup");
    }

    #[test]
    fn topic_theme_links_are_idempotent_and_never_establish_user_evidence() {
        let path = vault();
        let source =
            capture_service::create_capture_source(&path, "External material.", SourceType::Other)
                .expect("capture");
        let inspiration =
            create_inspiration_from_source(&path, &source.source_id).expect("inspiration");
        let topic = topic_service::create_topic(
            &path,
            CreateTopicRequest {
                title: "Validation as product discipline".into(),
                summary: "Use baselines before optimizing.".into(),
                timing_class: TopicTimingClass::Evergreen,
                relevant_until: None,
                timely_note: None,
            },
        )
        .expect("topic");
        let theme = topic_service::create_theme(
            &path,
            CreateThemeRequest {
                name: "Product craft".into(),
                description: String::new(),
            },
        )
        .expect("theme");

        let first = add_inspiration_relationship(
            &path,
            &inspiration.inspiration.inspiration_id,
            InspirationRelationKind::Topic,
            &topic.topic_id,
        )
        .expect("link topic");
        let duplicate = add_inspiration_relationship(
            &path,
            &inspiration.inspiration.inspiration_id,
            InspirationRelationKind::Topic,
            &topic.topic_id,
        )
        .expect("duplicate topic");
        let theme_link = add_inspiration_relationship(
            &path,
            &inspiration.inspiration.inspiration_id,
            InspirationRelationKind::Theme,
            &theme.theme_id,
        )
        .expect("link theme");
        assert!(first.changed);
        assert!(!duplicate.changed);
        assert!(theme_link.changed);

        let reopened =
            load_inspiration(&path, &inspiration.inspiration.inspiration_id).expect("reopen");
        assert_eq!(reopened.relationships.len(), 2);
        assert!(reopened
            .relationships
            .iter()
            .any(|relationship| relationship.relation_kind == InspirationRelationKind::Topic));
        assert!(reopened
            .relationships
            .iter()
            .any(|relationship| relationship.relation_kind == InspirationRelationKind::Theme));

        let removed = remove_inspiration_relationship(
            &path,
            &inspiration.inspiration.inspiration_id,
            InspirationRelationKind::Topic,
            &topic.topic_id,
        )
        .expect("remove topic");
        assert!(removed.changed);
        topic_service::load_topic(&path, &topic.topic_id).expect("topic remains");
        topic_service::load_theme(&path, &theme.theme_id).expect("theme remains");

        let connection = open_connection(&path).expect("open database");
        assert_eq!(row_count(&connection, "evidence_records"), 0);
        assert_eq!(row_count(&connection, "proof_points"), 0);
        assert_eq!(row_count(&connection, "stories"), 0);
        let voice_links: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM record_relationships
                 WHERE from_id=?1 AND (to_type='voice_evidence' OR relationship_type LIKE '%voice%')",
                [&inspiration.inspiration.inspiration_id],
                |row| row.get(0),
            )
            .expect("voice links");
        assert_eq!(voice_links, 0);
        drop(connection);
        fs::remove_dir_all(path).expect("cleanup");
    }
}
