use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    domain::{
        candidates::{CandidateStatus, StoryCandidate},
        interviews::InterviewSession,
        models::{SourceDocument, SourceType},
        roles::RoleRecord,
        stories::{StoryRecord, StoryStatus},
    },
    error::{ServiceResult, WorkLoreError},
};

pub const DATABASE_RELATIVE_PATH: &str = "data/worklore.sqlite";
const CURRENT_SCHEMA_VERSION: i64 = 8;
const MIGRATION_NAME: &str = "prototype_to_professional_memory_v1";

const SCHEMA_V1: &str = r#"
CREATE TABLE sources (
  source_id TEXT PRIMARY KEY, source_type TEXT NOT NULL, display_name TEXT NOT NULL,
  stored_path TEXT NOT NULL, original_file_name TEXT NOT NULL, media_type TEXT,
  byte_size INTEGER NOT NULL, content_hash TEXT NOT NULL, lifecycle_status TEXT NOT NULL,
  imported_at TEXT NOT NULL, updated_at TEXT NOT NULL, extraction_json TEXT NOT NULL,
  privacy_json TEXT NOT NULL, provenance_json TEXT NOT NULL, tags_json TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE role_contexts (
  role_id TEXT PRIMARY KEY, title TEXT NOT NULL, organization_entity_id TEXT NOT NULL,
  start_date TEXT, end_date TEXT, is_current INTEGER NOT NULL, summary TEXT NOT NULL,
  client_entity_ids_json TEXT NOT NULL, project_entity_ids_json TEXT NOT NULL,
  lifecycle_status TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE story_seeds (
  seed_id TEXT PRIMARY KEY, title TEXT NOT NULL, summary TEXT NOT NULL, status TEXT NOT NULL,
  provenance_json TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE stories (
  story_id TEXT PRIMARY KEY, title TEXT NOT NULL, summary TEXT NOT NULL,
  lifecycle_status TEXT NOT NULL, maturity TEXT NOT NULL, story_type TEXT NOT NULL,
  content_json TEXT NOT NULL, disclosure_json TEXT NOT NULL, privacy_json TEXT NOT NULL,
  tags_json TEXT NOT NULL, provenance_json TEXT NOT NULL, created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL, revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE evidence_records (
  evidence_id TEXT PRIMARY KEY, evidence_type TEXT NOT NULL, source_id TEXT,
  locator TEXT NOT NULL, captured_text TEXT NOT NULL, status TEXT NOT NULL,
  provenance_json TEXT NOT NULL, captured_at TEXT, created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL, revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE proof_points (
  proof_id TEXT PRIMARY KEY, statement TEXT NOT NULL, status TEXT NOT NULL,
  provenance_json TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE topic_candidates (
  topic_id TEXT PRIMARY KEY, title TEXT NOT NULL, summary TEXT NOT NULL, status TEXT NOT NULL,
  provenance_json TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE themes (
  theme_id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL, status TEXT NOT NULL,
  provenance_json TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE inspirations (
  inspiration_id TEXT PRIMARY KEY, source_id TEXT, title TEXT NOT NULL, notes TEXT NOT NULL,
  status TEXT NOT NULL, provenance_json TEXT NOT NULL, created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL, revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE target_contexts (
  target_id TEXT PRIMARY KEY, source_id TEXT, context_type TEXT NOT NULL, title TEXT NOT NULL,
  notes TEXT NOT NULL, status TEXT NOT NULL, provenance_json TEXT NOT NULL,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE interview_sessions (
  interview_id TEXT PRIMARY KEY, target_seed_id TEXT, target_story_id TEXT, status TEXT NOT NULL,
  payload_json TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE TABLE record_relationships (
  relationship_id TEXT PRIMARY KEY, from_type TEXT NOT NULL, from_id TEXT NOT NULL,
  relationship_type TEXT NOT NULL, to_type TEXT NOT NULL, to_id TEXT NOT NULL,
  provenance_json TEXT NOT NULL, created_at TEXT NOT NULL,
  UNIQUE(from_type, from_id, relationship_type, to_type, to_id)
);
CREATE TABLE migration_lineage (
  lineage_id TEXT PRIMARY KEY, legacy_type TEXT NOT NULL, legacy_id TEXT NOT NULL,
  canonical_type TEXT NOT NULL, canonical_id TEXT NOT NULL, migration_name TEXT NOT NULL,
  details_json TEXT NOT NULL, migrated_at TEXT NOT NULL,
  UNIQUE(legacy_type, legacy_id, canonical_type)
);
CREATE TABLE audit_events (
  audit_id TEXT PRIMARY KEY, event_type TEXT NOT NULL, record_type TEXT NOT NULL,
  record_id TEXT NOT NULL, actor TEXT NOT NULL, details_json TEXT NOT NULL,
  occurred_at TEXT NOT NULL
);
CREATE INDEX idx_relationships_from ON record_relationships(from_type, from_id);
CREATE INDEX idx_relationships_to ON record_relationships(to_type, to_id);
CREATE INDEX idx_lineage_legacy ON migration_lineage(legacy_type, legacy_id);
"#;

const SCHEMA_V2: &str = r#"
ALTER TABLE sources ADD COLUMN source_origin TEXT NOT NULL DEFAULT 'imported_file';
ALTER TABLE sources ADD COLUMN captured_text TEXT;
CREATE INDEX idx_sources_origin_imported_at ON sources(source_origin, imported_at DESC);
"#;

const SCHEMA_V3: &str = r#"
ALTER TABLE topic_candidates ADD COLUMN timing_class TEXT NOT NULL DEFAULT 'evergreen';
ALTER TABLE topic_candidates ADD COLUMN relevant_until TEXT;
ALTER TABLE topic_candidates ADD COLUMN timely_note TEXT;
CREATE INDEX idx_topics_status_timing ON topic_candidates(status, timing_class, updated_at DESC);
"#;

const SCHEMA_V4: &str = r#"
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

const SCHEMA_V6: &str = r#"
CREATE TABLE voice_evidence (
  voice_evidence_id TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  source_locator TEXT NOT NULL,
  text_snapshot TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  authorship_state TEXT NOT NULL,
  status TEXT NOT NULL,
  eligibility_reason TEXT NOT NULL,
  approval_state TEXT NOT NULL,
  approved_at TEXT,
  provenance_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1),
  UNIQUE(source_id, source_locator)
);
CREATE INDEX idx_voice_evidence_status_updated ON voice_evidence(status, updated_at DESC);
CREATE INDEX idx_voice_evidence_source ON voice_evidence(source_id);
"#;

const SCHEMA_V7: &str = r#"
CREATE TABLE core_voices (
  voice_id TEXT PRIMARY KEY,
  version_number INTEGER NOT NULL UNIQUE CHECK (version_number >= 1),
  label TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('proposed','active','superseded')),
  provenance_json TEXT NOT NULL,
  activated_at TEXT,
  superseded_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE UNIQUE INDEX idx_core_voice_single_active ON core_voices(status) WHERE status='active';
CREATE INDEX idx_core_voices_version ON core_voices(version_number DESC);

CREATE TABLE core_voice_traits (
  trait_id TEXT PRIMARY KEY,
  voice_id TEXT NOT NULL,
  name TEXT NOT NULL,
  value TEXT NOT NULL,
  user_guidance TEXT,
  provenance_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1),
  UNIQUE(voice_id, name),
  FOREIGN KEY(voice_id) REFERENCES core_voices(voice_id) ON DELETE CASCADE
);
CREATE INDEX idx_core_voice_traits_voice ON core_voice_traits(voice_id, name);

CREATE TABLE core_voice_trait_evidence (
  trait_id TEXT NOT NULL,
  voice_evidence_id TEXT NOT NULL,
  linked_at TEXT NOT NULL,
  PRIMARY KEY(trait_id, voice_evidence_id),
  FOREIGN KEY(trait_id) REFERENCES core_voice_traits(trait_id) ON DELETE CASCADE,
  FOREIGN KEY(voice_evidence_id) REFERENCES voice_evidence(voice_evidence_id)
);
CREATE INDEX idx_core_voice_trait_evidence_evidence ON core_voice_trait_evidence(voice_evidence_id);

CREATE TABLE tone_modes (
  tone_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  instructions TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('active','disabled','retired')),
  provenance_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE INDEX idx_tone_modes_status_updated ON tone_modes(status, updated_at DESC);

CREATE TABLE voice_directions (
  voice_direction_id TEXT PRIMARY KEY,
  statement TEXT NOT NULL,
  rationale TEXT NOT NULL,
  proposed_by TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('proposed','accepted','completed','retired')),
  provenance_json TEXT NOT NULL,
  accepted_at TEXT,
  completed_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE INDEX idx_voice_directions_status_updated ON voice_directions(status, updated_at DESC);

CREATE TABLE writing_rules (
  rule_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  instruction TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('proposed','active','disabled','retired')),
  provenance_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE INDEX idx_writing_rules_status_updated ON writing_rules(status, updated_at DESC);
"#;

const SCHEMA_V8: &str = r#"
CREATE TABLE posts (
  post_id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('working','final_approved')),
  current_revision_id TEXT,
  final_approved_revision_id TEXT,
  approved_at TEXT,
  provenance_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision >= 1)
);
CREATE INDEX idx_posts_status_updated ON posts(status, updated_at DESC);

CREATE TABLE post_revisions (
  revision_id TEXT PRIMARY KEY,
  post_id TEXT NOT NULL,
  sequence INTEGER NOT NULL CHECK (sequence >= 1),
  parent_revision_id TEXT,
  text_snapshot TEXT NOT NULL,
  origin TEXT NOT NULL CHECK (origin IN ('user','model')),
  authorship_state TEXT NOT NULL CHECK (authorship_state IN ('user_authored','user_edited_model','model_generated')),
  provider_run_id TEXT,
  provider_id TEXT,
  model_id TEXT,
  provenance_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(post_id, sequence),
  FOREIGN KEY(post_id) REFERENCES posts(post_id) ON DELETE CASCADE,
  FOREIGN KEY(parent_revision_id) REFERENCES post_revisions(revision_id)
);
CREATE INDEX idx_post_revisions_post_sequence ON post_revisions(post_id, sequence);
CREATE INDEX idx_post_revisions_parent ON post_revisions(parent_revision_id);
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct StorySeedRecord {
    pub seed_id: String,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub revision: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct CanonicalStoryRecord {
    pub story_id: String,
    pub title: String,
    pub summary: String,
    pub lifecycle_status: String,
    pub maturity: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrototypeMigrationSummary {
    pub sources: usize,
    pub roles: usize,
    pub seeds: usize,
    pub stories: usize,
    pub evidence: usize,
    pub target_contexts: usize,
    pub interviews: usize,
}

pub fn database_path(vault_path: &Path) -> PathBuf {
    vault_path.join(DATABASE_RELATIVE_PATH)
}

pub fn initialize_and_migrate(vault_path: &Path) -> ServiceResult<PrototypeMigrationSummary> {
    initialize(vault_path)?;
    migrate_prototype(vault_path)
}

pub fn initialize(vault_path: &Path) -> ServiceResult<()> {
    ensure_vault(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    migrate_schema(&mut connection)
}

#[allow(dead_code)]
pub fn schema_version(vault_path: &Path) -> ServiceResult<i64> {
    initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    Ok(connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?)
}

#[allow(dead_code)]
pub fn create_story_seed(
    vault_path: &Path,
    title: &str,
    summary: &str,
) -> ServiceResult<StorySeedRecord> {
    initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let now = Utc::now().to_rfc3339();
    let record = StorySeedRecord {
        seed_id: format!("seed_{}", Uuid::now_v7()),
        title: normalized_title(title, "Untitled story seed"),
        summary: summary.trim().to_string(),
        status: "captured".to_string(),
        revision: 1,
    };
    connection.execute(
        "INSERT INTO story_seeds(seed_id,title,summary,status,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,?4,?5,?6,?6,1)",
        params![&record.seed_id,&record.title,&record.summary,&record.status,
            json!({"creationActor":"user"}).to_string(),now],
    )?;
    Ok(record)
}

#[allow(dead_code)]
pub fn load_story_seed(vault_path: &Path, seed_id: &str) -> ServiceResult<StorySeedRecord> {
    initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    connection
        .query_row(
            "SELECT seed_id,title,summary,status,revision FROM story_seeds WHERE seed_id=?1",
            [seed_id],
            |row| {
                Ok(StorySeedRecord {
                    seed_id: row.get(0)?,
                    title: row.get(1)?,
                    summary: row.get(2)?,
                    status: row.get(3)?,
                    revision: row.get(4)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| WorkLoreError::InvalidVault(format!("Story Seed {seed_id} was not found.")))
}

#[allow(dead_code)]
pub fn create_story(
    vault_path: &Path,
    title: &str,
    summary: &str,
) -> ServiceResult<CanonicalStoryRecord> {
    initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let now = Utc::now().to_rfc3339();
    let record = CanonicalStoryRecord {
        story_id: format!("story_{}", Uuid::now_v7()),
        title: normalized_title(title, "Untitled story"),
        summary: summary.trim().to_string(),
        lifecycle_status: "active".to_string(),
        maturity: "developing".to_string(),
        revision: 1,
    };
    connection.execute(
        "INSERT INTO stories(story_id,title,summary,lifecycle_status,maturity,story_type,content_json,
         disclosure_json,privacy_json,tags_json,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,'active','developing','other',?4,'{}','{}','[]',?5,?6,?6,1)",
        params![&record.story_id,&record.title,&record.summary,
            json!({"summary":&record.summary}).to_string(),json!({"creationActor":"user"}).to_string(),now],
    )?;
    Ok(record)
}

#[allow(dead_code)]
pub fn load_story(vault_path: &Path, story_id: &str) -> ServiceResult<CanonicalStoryRecord> {
    initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    connection.query_row(
        "SELECT story_id,title,summary,lifecycle_status,maturity,revision FROM stories WHERE story_id=?1", [story_id],
        |row| Ok(CanonicalStoryRecord { story_id: row.get(0)?, title: row.get(1)?, summary: row.get(2)?,
            lifecycle_status: row.get(3)?, maturity: row.get(4)?, revision: row.get(5)? })
    ).optional()?.ok_or(WorkLoreError::StoryNotFound)
}

#[allow(dead_code)]
pub fn create_evidence(vault_path: &Path, text: &str) -> ServiceResult<String> {
    initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let id = format!("evidence_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO evidence_records(evidence_id,evidence_type,locator,captured_text,status,provenance_json,
         created_at,updated_at,revision) VALUES (?1,'user_confirmation','direct_capture',?2,'candidate',?3,?4,?4,1)",
        params![&id,text.trim(),json!({"creationActor":"user"}).to_string(),now],
    )?;
    Ok(id)
}

#[allow(dead_code)]
pub fn create_inspiration(vault_path: &Path, title: &str) -> ServiceResult<String> {
    initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let id = format!("inspiration_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO inspirations(inspiration_id,title,notes,status,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,'','saved',?3,?4,?4,1)",
        params![&id,normalized_title(title,"Untitled inspiration"),json!({"creationActor":"user"}).to_string(),now],
    )?;
    Ok(id)
}

#[allow(dead_code)]
pub fn create_target_context(
    vault_path: &Path,
    context_type: &str,
    title: &str,
) -> ServiceResult<String> {
    initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let id = format!("target_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO target_contexts(target_id,context_type,title,notes,status,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,'','active',?4,?5,?5,1)",
        params![&id,context_type.trim(),normalized_title(title,"Untitled target context"),
            json!({"creationActor":"user"}).to_string(),now],
    )?;
    Ok(id)
}

pub fn migrate_prototype(vault_path: &Path) -> ServiceResult<PrototypeMigrationSummary> {
    ensure_vault(vault_path)?;
    let sources = read_records::<SourceDocument>(&vault_path.join("sources/metadata"))?;
    let roles = read_records::<RoleRecord>(&vault_path.join("roles"))?;
    let candidates = read_records::<StoryCandidate>(&vault_path.join("candidates"))?;
    let stories = read_records::<StoryRecord>(&vault_path.join("stories"))?;
    let interviews = read_records::<InterviewSession>(&vault_path.join("interviews"))?;
    let mut connection = open_connection(vault_path)?;
    migrate_schema(&mut connection)?;
    let tx = connection.transaction()?;
    let mut summary = PrototypeMigrationSummary::default();
    for source in &sources {
        migrate_source(&tx, source)?;
        summary.sources += 1;
        if source.source_type == SourceType::JobDescription {
            migrate_job_target(&tx, source)?;
            summary.target_contexts += 1;
        }
    }
    for role in &roles {
        migrate_role(&tx, role)?;
        summary.roles += 1;
    }
    for candidate in &candidates {
        migrate_candidate(&tx, candidate)?;
        summary.seeds += 1;
    }
    for story in &stories {
        summary.evidence += migrate_story(&tx, story)?;
        summary.stories += 1;
    }
    for interview in &interviews {
        migrate_interview(&tx, interview)?;
        summary.interviews += 1;
    }
    tx.commit()?;
    Ok(summary)
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    fs::create_dir_all(vault_path.join("data"))?;
    let connection = Connection::open(database_path(vault_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

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
        version = 2;
    }
    if version == 2 {
        let tx = connection.transaction()?;
        tx.execute_batch(SCHEMA_V3)?;
        tx.execute(
            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (3,'topic_timing_v3',?1)",
            [Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
        version = 3;
    }
    if version == 3 {
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
        version = 5;
    }
    if version == 5 {
        let tx = connection.transaction()?;
        tx.execute_batch(SCHEMA_V6)?;
        tx.execute(
            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (6,'voice_evidence_provenance_v6',?1)",
            [Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
        version = 6;
    }
    if version == 6 {
        let tx = connection.transaction()?;
        tx.execute_batch(SCHEMA_V7)?;
        tx.execute(
            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (7,'core_voice_foundation_v7',?1)",
            [Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
        version = 7;
    }
    if version == 7 {
        let tx = connection.transaction()?;
        tx.execute_batch(SCHEMA_V8)?;
        tx.execute(
            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (8,'post_revision_lineage_v8',?1)",
            [Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
    }
    Ok(())
}

fn migrate_source(tx: &Transaction<'_>, source: &SourceDocument) -> ServiceResult<()> {
    validate_relative(&source.stored_path)?;
    let size = i64::try_from(source.byte_size).map_err(|_| {
        WorkLoreError::InvalidVault(format!("Source {} is too large.", source.source_id))
    })?;
    tx.execute(
        "INSERT INTO sources(source_id,source_type,display_name,stored_path,original_file_name,media_type,byte_size,
         content_hash,lifecycle_status,imported_at,updated_at,extraction_json,privacy_json,provenance_json,tags_json,revision)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'active',?9,?10,?11,?12,?13,?14,1)
         ON CONFLICT(source_id) DO UPDATE SET source_type=excluded.source_type,display_name=excluded.display_name,
         stored_path=excluded.stored_path,original_file_name=excluded.original_file_name,media_type=excluded.media_type,
         byte_size=excluded.byte_size,content_hash=excluded.content_hash,updated_at=excluded.updated_at,
         extraction_json=excluded.extraction_json,privacy_json=excluded.privacy_json,
         provenance_json=excluded.provenance_json,tags_json=excluded.tags_json",
        params![&source.source_id,enum_text(&source.source_type)?,&source.display_name,&source.stored_path,
            &source.original_file_name,&source.media_type,size,&source.content_hash.value,&source.imported_at,&source.updated_at,
            serde_json::to_string(&source.extraction)?,serde_json::to_string(&source.privacy_scan)?,
            serde_json::to_string(&source.provenance)?,serde_json::to_string(&source.tags)?],
    )?;
    preserve_id(tx, "source", &source.source_id, "source")
}

fn migrate_job_target(tx: &Transaction<'_>, source: &SourceDocument) -> ServiceResult<()> {
    let id = transformed_id(
        tx,
        "job_description_source",
        &source.source_id,
        "target_context",
        "target",
    )?;
    tx.execute(
        "INSERT INTO target_contexts(target_id,source_id,context_type,title,notes,status,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,'job_description',?3,'','active',?4,?5,?6,1)
         ON CONFLICT(target_id) DO UPDATE SET source_id=excluded.source_id,title=excluded.title,updated_at=excluded.updated_at",
        params![&id,&source.source_id,&source.display_name,
            json!({"creationActor":"migration","legacySourceId":&source.source_id}).to_string(),
            &source.imported_at,&source.updated_at],
    )?;
    Ok(())
}

fn migrate_role(tx: &Transaction<'_>, role: &RoleRecord) -> ServiceResult<()> {
    tx.execute(
        "INSERT INTO role_contexts(role_id,title,organization_entity_id,start_date,end_date,is_current,summary,
         client_entity_ids_json,project_entity_ids_json,lifecycle_status,created_at,updated_at,revision)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,'active',?10,?11,?12)
         ON CONFLICT(role_id) DO UPDATE SET title=excluded.title,organization_entity_id=excluded.organization_entity_id,
         start_date=excluded.start_date,end_date=excluded.end_date,is_current=excluded.is_current,summary=excluded.summary,
         client_entity_ids_json=excluded.client_entity_ids_json,project_entity_ids_json=excluded.project_entity_ids_json,
         updated_at=excluded.updated_at,revision=excluded.revision",
        params![&role.role_id,&role.title,&role.organization_entity_id,&role.start_date,&role.end_date,role.is_current,&role.summary,
            serde_json::to_string(&role.client_entity_ids)?,serde_json::to_string(&role.project_entity_ids)?,
            &role.created_at,&role.updated_at,role.revision],
    )?;
    preserve_id(tx, "role", &role.role_id, "role")?;
    for source in &role.source_ids {
        relationship(tx, "role", &role.role_id, "role_source", "source", source)?;
    }
    for story in &role.story_ids {
        relationship(tx, "story", story, "story_role", "role", &role.role_id)?;
    }
    Ok(())
}

fn migrate_candidate(tx: &Transaction<'_>, candidate: &StoryCandidate) -> ServiceResult<String> {
    let id = transformed_id(
        tx,
        "story_candidate",
        &candidate.candidate_id,
        "story_seed",
        "seed",
    )?;
    tx.execute(
        "INSERT INTO story_seeds(seed_id,title,summary,status,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(seed_id) DO UPDATE SET title=excluded.title,summary=excluded.summary,status=excluded.status,
         provenance_json=excluded.provenance_json,updated_at=excluded.updated_at,revision=excluded.revision",
        params![&id,compact(&candidate.claim,120),&candidate.claim,candidate_status(candidate.status),
            serde_json::to_string(candidate)?,&candidate.created_at,&candidate.updated_at,candidate.revision],
    )?;
    for source in &candidate.source_refs {
        relationship(
            tx,
            "story_seed",
            &id,
            "seed_source",
            "source",
            &source.source_id,
        )?;
    }
    for role in &candidate.context.role_ids {
        relationship(tx, "story_seed", &id, "seed_role_context", "role", role)?;
    }
    Ok(id)
}

fn migrate_story(tx: &Transaction<'_>, story: &StoryRecord) -> ServiceResult<usize> {
    let (lifecycle, maturity) = story_states(story.status);
    tx.execute(
        "INSERT INTO stories(story_id,title,summary,lifecycle_status,maturity,story_type,content_json,disclosure_json,
         privacy_json,tags_json,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
         ON CONFLICT(story_id) DO UPDATE SET title=excluded.title,summary=excluded.summary,lifecycle_status=excluded.lifecycle_status,
         maturity=excluded.maturity,story_type=excluded.story_type,content_json=excluded.content_json,
         disclosure_json=excluded.disclosure_json,privacy_json=excluded.privacy_json,tags_json=excluded.tags_json,
         updated_at=excluded.updated_at,revision=excluded.revision",
        params![&story.story_id,&story.title,&story.content.summary,lifecycle,maturity,enum_text(&story.story_type)?,
            serde_json::to_string(&story.content)?,serde_json::to_string(&story.disclosure)?,
            serde_json::to_string(&story.privacy_scan)?,serde_json::to_string(&story.tags)?,
            json!({"creationActor":"migration","legacyStoryId":&story.story_id}).to_string(),
            &story.created_at,&story.updated_at,story.revision],
    )?;
    preserve_id(tx, "story", &story.story_id, "story")?;
    for role in &story.role_ids {
        relationship(tx, "story", &story.story_id, "story_role", "role", role)?;
    }
    let mut count = 0;
    for evidence in &story.evidence {
        tx.execute(
            "INSERT INTO evidence_records(evidence_id,evidence_type,source_id,locator,captured_text,status,provenance_json,
             captured_at,created_at,updated_at,revision) VALUES (?1,?2,?3,?4,?5,'candidate',?6,?7,?8,?9,1)
             ON CONFLICT(evidence_id) DO UPDATE SET evidence_type=excluded.evidence_type,source_id=excluded.source_id,
             locator=excluded.locator,captured_text=excluded.captured_text,provenance_json=excluded.provenance_json,
             captured_at=excluded.captured_at,updated_at=excluded.updated_at",
            params![&evidence.evidence_id,enum_text(&evidence.evidence_type)?,&evidence.source_id,&evidence.locator,
                &evidence.captured_text,json!({"creationActor":"migration","legacyStoryId":&story.story_id}).to_string(),
                &evidence.captured_at,&story.created_at,&story.updated_at],
        )?;
        preserve_id(tx, "story_evidence", &evidence.evidence_id, "evidence")?;
        relationship(
            tx,
            "story",
            &story.story_id,
            "story_evidence",
            "evidence",
            &evidence.evidence_id,
        )?;
        count += 1;
    }
    Ok(count)
}

fn migrate_interview(tx: &Transaction<'_>, interview: &InterviewSession) -> ServiceResult<()> {
    let seed = interview
        .candidate_ids
        .first()
        .map(|id| transformed_id(tx, "story_candidate", id, "story_seed", "seed"))
        .transpose()?;
    tx.execute(
        "INSERT INTO interview_sessions(interview_id,target_seed_id,target_story_id,status,payload_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(interview_id) DO UPDATE SET target_seed_id=excluded.target_seed_id,target_story_id=excluded.target_story_id,
         status=excluded.status,payload_json=excluded.payload_json,updated_at=excluded.updated_at,revision=excluded.revision",
        params![&interview.interview_id,&seed,&interview.story_id,enum_text(&interview.status)?,serde_json::to_string(interview)?,
            &interview.created_at,&interview.updated_at,interview.revision],
    )?;
    preserve_id(tx, "interview", &interview.interview_id, "interview")?;
    if let (Some(seed), Some(story)) = (seed.as_deref(), interview.story_id.as_deref()) {
        relationship(tx, "story_seed", seed, "seed_story_lineage", "story", story)?;
    }
    Ok(())
}

fn relationship(
    tx: &Transaction<'_>,
    from_type: &str,
    from_id: &str,
    kind: &str,
    to_type: &str,
    to_id: &str,
) -> ServiceResult<()> {
    tx.execute("INSERT OR IGNORE INTO record_relationships(relationship_id,from_type,from_id,relationship_type,to_type,to_id,
        provenance_json,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",params![format!("relationship_{}",Uuid::now_v7()),
        from_type,from_id,kind,to_type,to_id,json!({"creationActor":"migration"}).to_string(),Utc::now().to_rfc3339()])?;
    Ok(())
}

fn preserve_id(
    tx: &Transaction<'_>,
    legacy_type: &str,
    id: &str,
    canonical_type: &str,
) -> ServiceResult<()> {
    lineage(
        tx,
        legacy_type,
        id,
        canonical_type,
        id,
        json!({"idPreserved":true}).to_string(),
    )
}

fn transformed_id(
    tx: &Transaction<'_>,
    legacy_type: &str,
    legacy_id: &str,
    canonical_type: &str,
    prefix: &str,
) -> ServiceResult<String> {
    if let Some(id)=tx.query_row("SELECT canonical_id FROM migration_lineage WHERE legacy_type=?1 AND legacy_id=?2 AND canonical_type=?3",
        params![legacy_type,legacy_id,canonical_type],|r|r.get(0)).optional()? { return Ok(id); }
    let id = format!("{prefix}_{}", Uuid::now_v7());
    lineage(
        tx,
        legacy_type,
        legacy_id,
        canonical_type,
        &id,
        json!({"idPreserved":false}).to_string(),
    )?;
    Ok(id)
}

fn lineage(
    tx: &Transaction<'_>,
    legacy_type: &str,
    legacy_id: &str,
    canonical_type: &str,
    canonical_id: &str,
    details: String,
) -> ServiceResult<()> {
    let now = Utc::now().to_rfc3339();
    let inserted=tx.execute(
        "INSERT OR IGNORE INTO migration_lineage(lineage_id,legacy_type,legacy_id,canonical_type,canonical_id,migration_name,details_json,migrated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",params![format!("migration_{}",Uuid::now_v7()),legacy_type,legacy_id,
            canonical_type,canonical_id,MIGRATION_NAME,details,&now])?;
    if inserted > 0 {
        tx.execute("INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
        VALUES (?1,'migration',?2,?3,'migration',?4,?5)",params![format!("audit_{}",Uuid::now_v7()),canonical_type,canonical_id,
            json!({"legacyType":legacy_type,"legacyId":legacy_id}).to_string(),now])?;
    }
    Ok(())
}

fn read_records<T: DeserializeOwned>(directory: &Path) -> ServiceResult<Vec<T>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(|v| v.to_str()) == Some("json")
        {
            out.push(serde_json::from_slice(&fs::read(entry.path())?)?);
        }
    }
    Ok(out)
}

fn validate_relative(value: &str) -> ServiceResult<()> {
    let v = value.replace('\\', "/");
    let b = v.as_bytes();
    let drive = b.len() >= 3 && b[1] == b':' && b[2] == b'/';
    if v.is_empty() || v.starts_with('/') || drive || v.split('/').any(|p| p == "..") {
        Err(WorkLoreError::InvalidPath)
    } else {
        Ok(())
    }
}

fn enum_text<T: Serialize>(value: &T) -> ServiceResult<String> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(v) => Ok(v),
        _ => Err(WorkLoreError::InvalidVault(
            "Expected string-backed domain enum.".to_string(),
        )),
    }
}
#[allow(dead_code)]
fn normalized_title(value: &str, fallback: &str) -> String {
    let v = value.trim();
    if v.is_empty() {
        fallback.to_string()
    } else {
        v.to_string()
    }
}
fn compact(value: &str, max: usize) -> String {
    let v = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if v.chars().count() <= max {
        v
    } else {
        let mut s = v.chars().take(max.saturating_sub(3)).collect::<String>();
        s.push_str("...");
        s
    }
}
fn candidate_status(status: CandidateStatus) -> &'static str {
    match status {
        CandidateStatus::New | CandidateStatus::SavedForLater => "captured",
        CandidateStatus::ReadyToInterview | CandidateStatus::Interviewing => "developing",
        CandidateStatus::ConvertedToStory => "converted",
        CandidateStatus::Ignored | CandidateStatus::Unsupported => "dismissed",
        CandidateStatus::Merged | CandidateStatus::Split => "archived",
    }
}
fn story_states(status: StoryStatus) -> (&'static str, &'static str) {
    match status {
        StoryStatus::Draft | StoryStatus::Interviewing => ("active", "developing"),
        StoryStatus::ReadyForReview | StoryStatus::Validated => ("active", "evidence_rich"),
        StoryStatus::Finalized => ("active", "ready_to_use"),
        StoryStatus::Archived => ("archived", "ready_to_use"),
    }
}
fn ensure_vault(vault_path: &Path) -> ServiceResult<()> {
    if vault_path.join("vault.json").is_file() {
        Ok(())
    } else {
        Err(WorkLoreError::NotAVault)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            candidates::{CandidateContext, CandidateSourceRef, CandidateType},
            models::{
                ContentHash, ExtractionState, ExtractionStatus, PrivacyScanState,
                PrivacyScanStatus, SourceProvenance,
            },
            stories::{
                EvidenceType, StoryContent, StoryDisclosure, StoryEvidence, StoryPrivacyScan,
                StoryType,
            },
        },
        io_utils::write_json_atomic,
        services::vault_service,
    };

    fn vault() -> PathBuf {
        let p = std::env::temp_dir().join(format!("worklore-sqlite-{}", Uuid::now_v7()));
        vault_service::create_vault(&p, "Test").expect("create vault");
        p
    }

    #[test]
    fn initializes_database() {
        let p = vault();
        assert!(database_path(&p).is_file());
        assert_eq!(schema_version(&p).unwrap(), 8);
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn seed_survives_reopen_without_resume() {
        let p = vault();
        let a = create_story_seed(&p, "Memory", "fragment").unwrap();
        initialize_and_migrate(&p).unwrap();
        assert_eq!(load_story_seed(&p, &a.seed_id).unwrap(), a);
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn story_does_not_require_role() {
        let p = vault();
        let a = create_story(&p, "Direct", "No resume").unwrap();
        assert_eq!(load_story(&p, &a.story_id).unwrap(), a);
        let c = open_connection(&p).unwrap();
        let n:i64=c.query_row(
        "SELECT COUNT(*) FROM record_relationships WHERE from_id=?1 AND relationship_type='story_role'",[&a.story_id],|r|r.get(0)).unwrap();
        assert_eq!(n, 0);
        drop(c);
        fs::remove_dir_all(p).unwrap();
    }
    #[test]
    fn semantic_roles_are_separate() {
        let p = vault();
        create_evidence(&p, "fact").unwrap();
        create_inspiration(&p, "article").unwrap();
        create_target_context(&p, "job_description", "job").unwrap();
        let c = open_connection(&p).unwrap();
        let e: i64 = c
            .query_row("SELECT COUNT(*) FROM evidence_records", [], |r| r.get(0))
            .unwrap();
        let i: i64 = c
            .query_row("SELECT COUNT(*) FROM inspirations", [], |r| r.get(0))
            .unwrap();
        let t: i64 = c
            .query_row("SELECT COUNT(*) FROM target_contexts", [], |r| r.get(0))
            .unwrap();
        assert_eq!((e, i, t), (1, 1, 1));
        drop(c);
        fs::remove_dir_all(p).unwrap();
    }

    #[test]
    fn migration_preserves_ids_and_uses_new_seed_id() {
        let p = vault();
        let now = Utc::now().to_rfc3339();
        let source = SourceDocument {
            schema_version: 1,
            source_id: "source_synthetic".into(),
            source_type: SourceType::Resume,
            display_name: "Synthetic Resume".into(),
            stored_path: "sources/resumes/synthetic.txt".into(),
            original_file_name: "synthetic.txt".into(),
            original_path_hint: Some("C:\\private\\synthetic.txt".into()),
            media_type: Some("text/plain".into()),
            byte_size: 12,
            content_hash: ContentHash {
                algorithm: "sha256".into(),
                value: "0".repeat(64),
            },
            imported_at: now.clone(),
            updated_at: now.clone(),
            extraction: ExtractionState {
                status: ExtractionStatus::Complete,
                extractor_version: Some("test".into()),
                text_path: None,
                character_count: 12,
                warnings: vec![],
                error: None,
            },
            privacy_scan: PrivacyScanState {
                status: PrivacyScanStatus::Complete,
                scan_version: Some("test".into()),
                scanned_at: Some(now.clone()),
                review_item_ids: vec![],
            },
            provenance: SourceProvenance {
                import_method: "file_copy".into(),
                parent_source_id: None,
                notes: String::new(),
            },
            tags: vec![],
        };
        write_json_atomic(&p.join("sources/metadata/source_synthetic.json"), &source).unwrap();
        let candidate = StoryCandidate {
            schema_version: 1,
            candidate_id: "candidate_synthetic".into(),
            source_refs: vec![CandidateSourceRef {
                source_id: source.source_id.clone(),
                fragment_id: "f1".into(),
                captured_text: "Improved workflow".into(),
            }],
            candidate_type: CandidateType::ResumeClaim,
            status: CandidateStatus::New,
            claim: "Improved workflow".into(),
            context: CandidateContext {
                role_ids: vec![],
                entity_ids: vec![],
                skills: vec![],
                tools: vec![],
                metrics: vec![],
                surrounding_heading: None,
            },
            missing_fields: vec![],
            possible_existing_story_ids: vec![],
            merged_into_id: None,
            split_into_ids: vec![],
            created_at: now.clone(),
            updated_at: now.clone(),
            revision: 1,
        };
        write_json_atomic(&p.join("candidates/candidate_synthetic.json"), &candidate).unwrap();
        let story = StoryRecord {
            schema_version: 1,
            story_id: "story_synthetic".into(),
            title: "Synthetic Story".into(),
            status: StoryStatus::Validated,
            story_type: StoryType::ProcessChange,
            role_ids: vec![],
            entity_ids: vec![],
            related_story_ids: vec![],
            job_requirement_ids: vec![],
            content: StoryContent {
                summary: "Improved workflow".into(),
                situation: String::new(),
                problem_or_opportunity: String::new(),
                responsibilities: vec![],
                constraints: vec![],
                actions_and_decisions: vec![],
                alternatives_considered: vec![],
                tools_and_systems: vec![],
                stakeholders: vec![],
                outcomes: vec![],
                metrics: vec![],
                lessons_learned: vec![],
                operating_philosophy: vec![],
                reusable_themes: vec![],
                skills_demonstrated: vec![],
            },
            claims: vec![],
            evidence: vec![StoryEvidence {
                evidence_id: "evidence_synthetic".into(),
                evidence_type: EvidenceType::SourceFragment,
                source_id: Some(source.source_id.clone()),
                locator: "f1".into(),
                captured_text: "Improved workflow".into(),
                captured_at: Some(now.clone()),
            }],
            disclosure: StoryDisclosure {
                default_public_mode: "stable_tokens".into(),
                review_required: true,
                notes: String::new(),
            },
            privacy_scan: StoryPrivacyScan {
                status: PrivacyScanStatus::Complete,
                content_revision: 1,
                scanned_at: Some(now.clone()),
                review_item_ids: vec![],
            },
            tags: vec![],
            created_at: now.clone(),
            updated_at: now,
            revision: 1,
        };
        write_json_atomic(&p.join("stories/story_synthetic.json"), &story).unwrap();
        migrate_prototype(&p).unwrap();
        let c = open_connection(&p).unwrap();
        let source_n: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM sources WHERE source_id='source_synthetic'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let story_n: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM stories WHERE story_id='story_synthetic'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let seed:String=c.query_row("SELECT canonical_id FROM migration_lineage WHERE legacy_type='story_candidate' AND legacy_id='candidate_synthetic' AND canonical_type='story_seed'",[],|r|r.get(0)).unwrap();
        let path_col: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('sources') WHERE name='original_path_hint'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!((source_n, story_n), (1, 1));
        assert!(seed.starts_with("seed_"));
        assert_ne!(seed, candidate.candidate_id);
        assert_eq!(path_col, 0);
        assert!(p.join("sources/metadata/source_synthetic.json").is_file());
        assert!(p.join("stories/story_synthetic.json").is_file());
        drop(c);
        fs::remove_dir_all(p).unwrap();
    }
}
