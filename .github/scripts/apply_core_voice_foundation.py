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


# Canonical schema v7: Core Voice, Tone Modes, Voice Direction, and Writing Rules.
path = "src-tauri/src/services/canonical_store.rs"
text = read(path)
text = replace_once(
    text,
    "const CURRENT_SCHEMA_VERSION: i64 = 6;",
    "const CURRENT_SCHEMA_VERSION: i64 = 7;",
    "schema version",
)
anchor = '''const SCHEMA_V6: &str = r#"\nCREATE TABLE voice_evidence (\n  voice_evidence_id TEXT PRIMARY KEY,\n  source_id TEXT NOT NULL,\n  source_locator TEXT NOT NULL,\n  text_snapshot TEXT NOT NULL,\n  content_hash TEXT NOT NULL,\n  authorship_state TEXT NOT NULL,\n  status TEXT NOT NULL,\n  eligibility_reason TEXT NOT NULL,\n  approval_state TEXT NOT NULL,\n  approved_at TEXT,\n  provenance_json TEXT NOT NULL,\n  created_at TEXT NOT NULL,\n  updated_at TEXT NOT NULL,\n  revision INTEGER NOT NULL CHECK (revision >= 1),\n  UNIQUE(source_id, source_locator)\n);\nCREATE INDEX idx_voice_evidence_status_updated ON voice_evidence(status, updated_at DESC);\nCREATE INDEX idx_voice_evidence_source ON voice_evidence(source_id);\n"#;\n'''
schema_v7 = anchor + '''\nconst SCHEMA_V7: &str = r#"\nCREATE TABLE core_voices (\n  voice_id TEXT PRIMARY KEY,\n  version_number INTEGER NOT NULL UNIQUE CHECK (version_number >= 1),\n  label TEXT NOT NULL,\n  status TEXT NOT NULL CHECK (status IN ('proposed','active','superseded')),\n  provenance_json TEXT NOT NULL,\n  activated_at TEXT,\n  superseded_at TEXT,\n  created_at TEXT NOT NULL,\n  updated_at TEXT NOT NULL,\n  revision INTEGER NOT NULL CHECK (revision >= 1)\n);\nCREATE UNIQUE INDEX idx_core_voice_single_active ON core_voices(status) WHERE status='active';\nCREATE INDEX idx_core_voices_version ON core_voices(version_number DESC);\n\nCREATE TABLE core_voice_traits (\n  trait_id TEXT PRIMARY KEY,\n  voice_id TEXT NOT NULL,\n  name TEXT NOT NULL,\n  value TEXT NOT NULL,\n  user_guidance TEXT,\n  provenance_json TEXT NOT NULL,\n  created_at TEXT NOT NULL,\n  updated_at TEXT NOT NULL,\n  revision INTEGER NOT NULL CHECK (revision >= 1),\n  UNIQUE(voice_id, name),\n  FOREIGN KEY(voice_id) REFERENCES core_voices(voice_id) ON DELETE CASCADE\n);\nCREATE INDEX idx_core_voice_traits_voice ON core_voice_traits(voice_id, name);\n\nCREATE TABLE core_voice_trait_evidence (\n  trait_id TEXT NOT NULL,\n  voice_evidence_id TEXT NOT NULL,\n  linked_at TEXT NOT NULL,\n  PRIMARY KEY(trait_id, voice_evidence_id),\n  FOREIGN KEY(trait_id) REFERENCES core_voice_traits(trait_id) ON DELETE CASCADE,\n  FOREIGN KEY(voice_evidence_id) REFERENCES voice_evidence(voice_evidence_id)\n);\nCREATE INDEX idx_core_voice_trait_evidence_evidence ON core_voice_trait_evidence(voice_evidence_id);\n\nCREATE TABLE tone_modes (\n  tone_id TEXT PRIMARY KEY,\n  name TEXT NOT NULL,\n  description TEXT NOT NULL,\n  instructions TEXT NOT NULL,\n  status TEXT NOT NULL CHECK (status IN ('active','disabled','retired')),\n  provenance_json TEXT NOT NULL,\n  created_at TEXT NOT NULL,\n  updated_at TEXT NOT NULL,\n  revision INTEGER NOT NULL CHECK (revision >= 1)\n);\nCREATE INDEX idx_tone_modes_status_updated ON tone_modes(status, updated_at DESC);\n\nCREATE TABLE voice_directions (\n  voice_direction_id TEXT PRIMARY KEY,\n  statement TEXT NOT NULL,\n  rationale TEXT NOT NULL,\n  proposed_by TEXT NOT NULL,\n  status TEXT NOT NULL CHECK (status IN ('proposed','accepted','completed','retired')),\n  provenance_json TEXT NOT NULL,\n  accepted_at TEXT,\n  completed_at TEXT,\n  created_at TEXT NOT NULL,\n  updated_at TEXT NOT NULL,\n  revision INTEGER NOT NULL CHECK (revision >= 1)\n);\nCREATE INDEX idx_voice_directions_status_updated ON voice_directions(status, updated_at DESC);\n\nCREATE TABLE writing_rules (\n  rule_id TEXT PRIMARY KEY,\n  name TEXT NOT NULL,\n  instruction TEXT NOT NULL,\n  status TEXT NOT NULL CHECK (status IN ('proposed','active','disabled','retired')),\n  provenance_json TEXT NOT NULL,\n  created_at TEXT NOT NULL,\n  updated_at TEXT NOT NULL,\n  revision INTEGER NOT NULL CHECK (revision >= 1)\n);\nCREATE INDEX idx_writing_rules_status_updated ON writing_rules(status, updated_at DESC);\n"#;\n'''
text = replace_once(text, anchor, schema_v7, "voice foundation schema insertion")
old_migration = '''    if version == 5 {\n        let tx = connection.transaction()?;\n        tx.execute_batch(SCHEMA_V6)?;\n        tx.execute(\n            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (6,'voice_evidence_provenance_v6',?1)",\n            [Utc::now().to_rfc3339()],\n        )?;\n        tx.commit()?;\n    }\n    Ok(())\n}\n'''
new_migration = '''    if version == 5 {\n        let tx = connection.transaction()?;\n        tx.execute_batch(SCHEMA_V6)?;\n        tx.execute(\n            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (6,'voice_evidence_provenance_v6',?1)",\n            [Utc::now().to_rfc3339()],\n        )?;\n        tx.commit()?;\n        version = 6;\n    }\n    if version == 6 {\n        let tx = connection.transaction()?;\n        tx.execute_batch(SCHEMA_V7)?;\n        tx.execute(\n            "INSERT INTO schema_migrations(version,name,applied_at) VALUES (7,'core_voice_foundation_v7',?1)",\n            [Utc::now().to_rfc3339()],\n        )?;\n        tx.commit()?;\n    }\n    Ok(())\n}\n'''
text = replace_once(text, old_migration, new_migration, "schema v7 migration")
text = replace_once(
    text,
    "assert_eq!(schema_version(&p).unwrap(), 6);",
    "assert_eq!(schema_version(&p).unwrap(), 7);",
    "schema version test",
)
write(path, text)


write(
    "src-tauri/src/services/voice_profile_service.rs",
    r'''use std::{
    collections::BTreeSet,
    path::Path,
    time::Duration,
};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    error::{ServiceResult, WorkLoreError},
    services::canonical_store,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreVoiceStatus {
    Proposed,
    Active,
    Superseded,
}

impl CoreVoiceStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Active => "active",
            Self::Superseded => "superseded",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "proposed" => Ok(Self::Proposed),
            "active" => Ok(Self::Active),
            "superseded" => Ok(Self::Superseded),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Core Voice status {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToneModeStatus {
    Active,
    Disabled,
    Retired,
}

impl ToneModeStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Retired => "retired",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            "retired" => Ok(Self::Retired),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Tone Mode status {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VoiceDirectionStatus {
    Proposed,
    Accepted,
    Completed,
    Retired,
}

impl VoiceDirectionStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Completed => "completed",
            Self::Retired => "retired",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "completed" => Ok(Self::Completed),
            "retired" => Ok(Self::Retired),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Voice Direction status {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WritingRuleStatus {
    Proposed,
    Active,
    Disabled,
    Retired,
}

impl WritingRuleStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Retired => "retired",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "proposed" => Ok(Self::Proposed),
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            "retired" => Ok(Self::Retired),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Writing Rule status {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CoreVoiceTraitEvidenceView {
    pub voice_evidence_id: String,
    pub current_status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CoreVoiceTraitView {
    pub trait_id: String,
    pub name: String,
    pub value: String,
    pub user_guidance: Option<String>,
    pub evidence: Vec<CoreVoiceTraitEvidenceView>,
    pub provenance_kind: String,
    pub provenance_valid: bool,
    pub invalidated_evidence_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CoreVoiceRecordView {
    pub voice_id: String,
    pub version_number: u32,
    pub label: String,
    pub status: CoreVoiceStatus,
    pub traits: Vec<CoreVoiceTraitView>,
    pub activated_at: Option<String>,
    pub superseded_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCoreVoiceRequest {
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCoreVoiceTraitRequest {
    pub voice_id: String,
    pub trait_id: Option<String>,
    pub name: String,
    pub value: String,
    pub user_guidance: Option<String>,
    #[serde(default)]
    pub voice_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToneModeRecordView {
    pub tone_id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub status: ToneModeStatus,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateToneModeRequest {
    pub name: String,
    pub description: String,
    pub instructions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateToneModeRequest {
    pub tone_id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub status: ToneModeStatus,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VoiceDirectionRecordView {
    pub voice_direction_id: String,
    pub statement: String,
    pub rationale: String,
    pub proposed_by: String,
    pub status: VoiceDirectionStatus,
    pub accepted_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVoiceDirectionRequest {
    pub statement: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVoiceDirectionStatusRequest {
    pub voice_direction_id: String,
    pub status: VoiceDirectionStatus,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WritingRuleRecordView {
    pub rule_id: String,
    pub name: String,
    pub instruction: String,
    pub status: WritingRuleStatus,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWritingRuleRequest {
    pub name: String,
    pub instruction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWritingRuleRequest {
    pub rule_id: String,
    pub name: String,
    pub instruction: String,
    pub status: WritingRuleStatus,
}

pub fn list_core_voices(vault_path: &Path) -> ServiceResult<Vec<CoreVoiceRecordView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut statement = connection.prepare(
        "SELECT voice_id FROM core_voices ORDER BY version_number DESC,created_at DESC",
    )?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|id| load_core_voice_from_connection(&connection, &id))
        .collect()
}

pub fn create_core_voice(
    vault_path: &Path,
    request: CreateCoreVoiceRequest,
) -> ServiceResult<CoreVoiceRecordView> {
    canonical_store::initialize(vault_path)?;
    let label = required_text(&request.label, "Core Voice label")?;
    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    let version_number: u32 = tx.query_row(
        "SELECT COALESCE(MAX(version_number),0)+1 FROM core_voices",
        [],
        |row| row.get(0),
    )?;
    let voice_id = format!("voice_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    tx.execute(
        "INSERT INTO core_voices(voice_id,version_number,label,status,provenance_json,activated_at,superseded_at,created_at,updated_at,revision)
         VALUES (?1,?2,?3,'proposed',?4,NULL,NULL,?5,?5,1)",
        params![
            &voice_id,
            version_number,
            &label,
            json!({"creationActor":"user","foundation":"manual"}).to_string(),
            &now
        ],
    )?;
    audit_tx(
        &tx,
        "core_voice_created",
        "core_voice",
        &voice_id,
        json!({"versionNumber":version_number,"status":"proposed"}),
        &now,
    )?;
    tx.commit()?;
    load_core_voice_from_connection(&connection, &voice_id)
}

pub fn save_core_voice_trait(
    vault_path: &Path,
    request: SaveCoreVoiceTraitRequest,
) -> ServiceResult<CoreVoiceRecordView> {
    canonical_store::initialize(vault_path)?;
    let name = required_text(&request.name, "Core Voice trait name")?;
    let value = required_text(&request.value, "Core Voice trait value")?;
    let guidance = optional_text(request.user_guidance.as_deref());
    let evidence_ids = request
        .voice_evidence_ids
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();
    if guidance.is_none() && evidence_ids.is_empty() {
        return Err(WorkLoreError::InvalidVault(
            "A Core Voice trait requires eligible Voice Evidence or explicit user guidance."
                .to_string(),
        ));
    }

    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    require_proposed_voice(&tx, &request.voice_id)?;
    validate_eligible_evidence(&tx, &evidence_ids)?;
    let now = Utc::now().to_rfc3339();
    let (trait_id, event_type) = if let Some(trait_id) = request.trait_id.as_deref() {
        let owner: Option<String> = tx
            .query_row(
                "SELECT voice_id FROM core_voice_traits WHERE trait_id=?1",
                [trait_id],
                |row| row.get(0),
            )
            .optional()?;
        if owner.as_deref() != Some(request.voice_id.as_str()) {
            return Err(WorkLoreError::InvalidVault(
                "Core Voice trait does not belong to the proposed Core Voice version.".to_string(),
            ));
        }
        tx.execute(
            "UPDATE core_voice_traits SET name=?2,value=?3,user_guidance=?4,provenance_json=?5,
             updated_at=?6,revision=revision+1 WHERE trait_id=?1",
            params![
                trait_id,
                &name,
                &value,
                &guidance,
                json!({"creationActor":"user","source":"manual_review"}).to_string(),
                &now
            ],
        )?;
        tx.execute(
            "DELETE FROM core_voice_trait_evidence WHERE trait_id=?1",
            [trait_id],
        )?;
        (trait_id.to_string(), "core_voice_trait_updated")
    } else {
        let trait_id = format!("voice_trait_{}", Uuid::now_v7());
        tx.execute(
            "INSERT INTO core_voice_traits(trait_id,voice_id,name,value,user_guidance,provenance_json,created_at,updated_at,revision)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?7,1)",
            params![
                &trait_id,
                &request.voice_id,
                &name,
                &value,
                &guidance,
                json!({"creationActor":"user","source":"manual_review"}).to_string(),
                &now
            ],
        )?;
        (trait_id, "core_voice_trait_created")
    };
    for voice_evidence_id in &evidence_ids {
        tx.execute(
            "INSERT INTO core_voice_trait_evidence(trait_id,voice_evidence_id,linked_at)
             VALUES (?1,?2,?3)",
            params![&trait_id, voice_evidence_id, &now],
        )?;
    }
    touch_voice(&tx, &request.voice_id, &now)?;
    audit_tx(
        &tx,
        event_type,
        "core_voice_trait",
        &trait_id,
        json!({
            "voiceId":request.voice_id,
            "evidenceIds":evidence_ids,
            "hasUserGuidance":guidance.is_some()
        }),
        &now,
    )?;
    tx.commit()?;
    load_core_voice_from_connection(&connection, &request.voice_id)
}

pub fn delete_core_voice_trait(
    vault_path: &Path,
    voice_id: &str,
    trait_id: &str,
) -> ServiceResult<CoreVoiceRecordView> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    require_proposed_voice(&tx, voice_id)?;
    let deleted = tx.execute(
        "DELETE FROM core_voice_traits WHERE trait_id=?1 AND voice_id=?2",
        params![trait_id, voice_id],
    )?;
    if deleted == 0 {
        return Err(WorkLoreError::InvalidVault(
            "Core Voice trait was not found on this proposed version.".to_string(),
        ));
    }
    let now = Utc::now().to_rfc3339();
    touch_voice(&tx, voice_id, &now)?;
    audit_tx(
        &tx,
        "core_voice_trait_deleted",
        "core_voice_trait",
        trait_id,
        json!({"voiceId":voice_id}),
        &now,
    )?;
    tx.commit()?;
    load_core_voice_from_connection(&connection, voice_id)
}

pub fn activate_core_voice(vault_path: &Path, voice_id: &str) -> ServiceResult<CoreVoiceRecordView> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    require_proposed_voice(&tx, voice_id)?;
    validate_voice_for_activation(&tx, voice_id)?;
    let now = Utc::now().to_rfc3339();

    let active_ids = {
        let mut statement = tx.prepare("SELECT voice_id FROM core_voices WHERE status='active'")?;
        statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?
    };
    for active_id in active_ids {
        tx.execute(
            "UPDATE core_voices SET status='superseded',superseded_at=?2,updated_at=?2,revision=revision+1
             WHERE voice_id=?1",
            params![&active_id, &now],
        )?;
        audit_tx(
            &tx,
            "core_voice_superseded",
            "core_voice",
            &active_id,
            json!({"supersededBy":voice_id}),
            &now,
        )?;
    }
    tx.execute(
        "UPDATE core_voices SET status='active',activated_at=?2,updated_at=?2,revision=revision+1
         WHERE voice_id=?1",
        params![voice_id, &now],
    )?;
    audit_tx(
        &tx,
        "core_voice_activated",
        "core_voice",
        voice_id,
        json!({"status":"active"}),
        &now,
    )?;
    tx.commit()?;
    load_core_voice_from_connection(&connection, voice_id)
}

pub fn list_tone_modes(vault_path: &Path) -> ServiceResult<Vec<ToneModeRecordView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut statement = connection.prepare(
        "SELECT tone_id,name,description,instructions,status,created_at,updated_at,revision
         FROM tone_modes ORDER BY CASE status WHEN 'active' THEN 0 WHEN 'disabled' THEN 1 ELSE 2 END,updated_at DESC",
    )?;
    statement
        .query_map([], map_tone_mode)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub fn create_tone_mode(
    vault_path: &Path,
    request: CreateToneModeRequest,
) -> ServiceResult<ToneModeRecordView> {
    canonical_store::initialize(vault_path)?;
    let name = required_text(&request.name, "Tone Mode name")?;
    let description = request.description.trim().to_string();
    let instructions = required_text(&request.instructions, "Tone Mode instructions")?;
    let connection = open_connection(vault_path)?;
    let tone_id = format!("tone_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO tone_modes(tone_id,name,description,instructions,status,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,?4,'active',?5,?6,?6,1)",
        params![
            &tone_id,
            &name,
            &description,
            &instructions,
            json!({"creationActor":"user"}).to_string(),
            &now
        ],
    )?;
    audit_connection(
        &connection,
        "tone_mode_created",
        "tone_mode",
        &tone_id,
        json!({"status":"active"}),
        &now,
    )?;
    load_tone_mode(&connection, &tone_id)
}

pub fn update_tone_mode(
    vault_path: &Path,
    request: UpdateToneModeRequest,
) -> ServiceResult<ToneModeRecordView> {
    canonical_store::initialize(vault_path)?;
    let name = required_text(&request.name, "Tone Mode name")?;
    let description = request.description.trim().to_string();
    let instructions = required_text(&request.instructions, "Tone Mode instructions")?;
    let connection = open_connection(vault_path)?;
    let existing = load_tone_mode(&connection, &request.tone_id)?;
    if existing.status == ToneModeStatus::Retired && request.status != ToneModeStatus::Retired {
        return Err(WorkLoreError::InvalidVault(
            "Retired Tone Modes cannot be reactivated through the normal path.".to_string(),
        ));
    }
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "UPDATE tone_modes SET name=?2,description=?3,instructions=?4,status=?5,updated_at=?6,revision=revision+1
         WHERE tone_id=?1",
        params![
            &request.tone_id,
            &name,
            &description,
            &instructions,
            request.status.as_str(),
            &now
        ],
    )?;
    audit_connection(
        &connection,
        "tone_mode_updated",
        "tone_mode",
        &request.tone_id,
        json!({"status":request.status.as_str()}),
        &now,
    )?;
    load_tone_mode(&connection, &request.tone_id)
}

pub fn list_voice_directions(vault_path: &Path) -> ServiceResult<Vec<VoiceDirectionRecordView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut statement = connection.prepare(
        "SELECT voice_direction_id,statement,rationale,proposed_by,status,accepted_at,completed_at,created_at,updated_at,revision
         FROM voice_directions ORDER BY updated_at DESC,created_at DESC",
    )?;
    statement
        .query_map([], map_voice_direction)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub fn create_voice_direction(
    vault_path: &Path,
    request: CreateVoiceDirectionRequest,
) -> ServiceResult<VoiceDirectionRecordView> {
    canonical_store::initialize(vault_path)?;
    let statement = required_text(&request.statement, "Voice Direction statement")?;
    let rationale = request.rationale.trim().to_string();
    let connection = open_connection(vault_path)?;
    let voice_direction_id = format!("voice_direction_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO voice_directions(voice_direction_id,statement,rationale,proposed_by,status,provenance_json,
         accepted_at,completed_at,created_at,updated_at,revision)
         VALUES (?1,?2,?3,'user','proposed',?4,NULL,NULL,?5,?5,1)",
        params![
            &voice_direction_id,
            &statement,
            &rationale,
            json!({"creationActor":"user","proposalSource":"explicit_user_guidance"}).to_string(),
            &now
        ],
    )?;
    audit_connection(
        &connection,
        "voice_direction_created",
        "voice_direction",
        &voice_direction_id,
        json!({"status":"proposed","proposedBy":"user"}),
        &now,
    )?;
    load_voice_direction(&connection, &voice_direction_id)
}

pub fn set_voice_direction_status(
    vault_path: &Path,
    request: SetVoiceDirectionStatusRequest,
) -> ServiceResult<VoiceDirectionRecordView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let existing = load_voice_direction(&connection, &request.voice_direction_id)?;
    if existing.status == request.status {
        return Ok(existing);
    }
    if !valid_direction_transition(existing.status, request.status) {
        return Err(WorkLoreError::InvalidVault(format!(
            "Voice Direction cannot move from {} to {} through the normal path.",
            existing.status.as_str(),
            request.status.as_str()
        )));
    }
    let now = Utc::now().to_rfc3339();
    let accepted_at = if request.status == VoiceDirectionStatus::Accepted {
        Some(now.clone())
    } else {
        existing.accepted_at.clone()
    };
    let completed_at = if request.status == VoiceDirectionStatus::Completed {
        Some(now.clone())
    } else {
        existing.completed_at.clone()
    };
    connection.execute(
        "UPDATE voice_directions SET status=?2,accepted_at=?3,completed_at=?4,updated_at=?5,revision=revision+1
         WHERE voice_direction_id=?1",
        params![
            &request.voice_direction_id,
            request.status.as_str(),
            &accepted_at,
            &completed_at,
            &now
        ],
    )?;
    audit_connection(
        &connection,
        "voice_direction_status_changed",
        "voice_direction",
        &request.voice_direction_id,
        json!({"from":existing.status.as_str(),"to":request.status.as_str()}),
        &now,
    )?;
    load_voice_direction(&connection, &request.voice_direction_id)
}

pub fn list_writing_rules(vault_path: &Path) -> ServiceResult<Vec<WritingRuleRecordView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut statement = connection.prepare(
        "SELECT rule_id,name,instruction,status,created_at,updated_at,revision
         FROM writing_rules ORDER BY CASE status WHEN 'active' THEN 0 WHEN 'proposed' THEN 1 WHEN 'disabled' THEN 2 ELSE 3 END,updated_at DESC",
    )?;
    statement
        .query_map([], map_writing_rule)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub fn create_writing_rule(
    vault_path: &Path,
    request: CreateWritingRuleRequest,
) -> ServiceResult<WritingRuleRecordView> {
    canonical_store::initialize(vault_path)?;
    let name = required_text(&request.name, "Writing Rule name")?;
    let instruction = required_text(&request.instruction, "Writing Rule instruction")?;
    let connection = open_connection(vault_path)?;
    let rule_id = format!("rule_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO writing_rules(rule_id,name,instruction,status,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,'proposed',?4,?5,?5,1)",
        params![
            &rule_id,
            &name,
            &instruction,
            json!({"creationActor":"user","source":"explicit_user_instruction"}).to_string(),
            &now
        ],
    )?;
    audit_connection(
        &connection,
        "writing_rule_created",
        "writing_rule",
        &rule_id,
        json!({"status":"proposed"}),
        &now,
    )?;
    load_writing_rule(&connection, &rule_id)
}

pub fn update_writing_rule(
    vault_path: &Path,
    request: UpdateWritingRuleRequest,
) -> ServiceResult<WritingRuleRecordView> {
    canonical_store::initialize(vault_path)?;
    let name = required_text(&request.name, "Writing Rule name")?;
    let instruction = required_text(&request.instruction, "Writing Rule instruction")?;
    let connection = open_connection(vault_path)?;
    let existing = load_writing_rule(&connection, &request.rule_id)?;
    if existing.status != request.status && !valid_rule_transition(existing.status, request.status) {
        return Err(WorkLoreError::InvalidVault(format!(
            "Writing Rule cannot move from {} to {} through the normal path.",
            existing.status.as_str(),
            request.status.as_str()
        )));
    }
    let now = Utc::now().to_rfc3339();
    connection.execute(
        "UPDATE writing_rules SET name=?2,instruction=?3,status=?4,updated_at=?5,revision=revision+1
         WHERE rule_id=?1",
        params![
            &request.rule_id,
            &name,
            &instruction,
            request.status.as_str(),
            &now
        ],
    )?;
    audit_connection(
        &connection,
        "writing_rule_updated",
        "writing_rule",
        &request.rule_id,
        json!({"from":existing.status.as_str(),"to":request.status.as_str()}),
        &now,
    )?;
    load_writing_rule(&connection, &request.rule_id)
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    let connection = Connection::open(canonical_store::database_path(vault_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

fn required_text(value: &str, label: &str) -> ServiceResult<String> {
    let value = value.trim();
    if value.is_empty() {
        Err(WorkLoreError::InvalidVault(format!("{label} is required.")))
    } else {
        Ok(value.to_string())
    }
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn require_proposed_voice(tx: &Transaction<'_>, voice_id: &str) -> ServiceResult<()> {
    let status: Option<String> = tx
        .query_row(
            "SELECT status FROM core_voices WHERE voice_id=?1",
            [voice_id],
            |row| row.get(0),
        )
        .optional()?;
    match status.as_deref() {
        Some("proposed") => Ok(()),
        Some(_) => Err(WorkLoreError::InvalidVault(
            "Only a proposed Core Voice version can be edited.".to_string(),
        )),
        None => Err(WorkLoreError::InvalidVault(format!(
            "Core Voice {voice_id} was not found."
        ))),
    }
}

fn validate_eligible_evidence(
    tx: &Transaction<'_>,
    evidence_ids: &BTreeSet<String>,
) -> ServiceResult<()> {
    for voice_evidence_id in evidence_ids {
        let status: Option<String> = tx
            .query_row(
                "SELECT status FROM voice_evidence WHERE voice_evidence_id=?1",
                [voice_evidence_id],
                |row| row.get(0),
            )
            .optional()?;
        match status.as_deref() {
            Some("eligible") => {}
            Some(other) => {
                return Err(WorkLoreError::InvalidVault(format!(
                    "Voice Evidence {voice_evidence_id} is {other} and cannot support a new Core Voice trait."
                )))
            }
            None => {
                return Err(WorkLoreError::InvalidVault(format!(
                    "Voice Evidence {voice_evidence_id} was not found."
                )))
            }
        }
    }
    Ok(())
}

fn validate_voice_for_activation(tx: &Transaction<'_>, voice_id: &str) -> ServiceResult<()> {
    let trait_ids = {
        let mut statement = tx.prepare(
            "SELECT trait_id FROM core_voice_traits WHERE voice_id=?1 ORDER BY trait_id",
        )?;
        statement
            .query_map([voice_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?
    };
    if trait_ids.is_empty() {
        return Err(WorkLoreError::InvalidVault(
            "A Core Voice version needs at least one attributable trait before activation."
                .to_string(),
        ));
    }
    for trait_id in trait_ids {
        let guidance: Option<String> = tx.query_row(
            "SELECT user_guidance FROM core_voice_traits WHERE trait_id=?1",
            [&trait_id],
            |row| row.get(0),
        )?;
        let statuses = {
            let mut statement = tx.prepare(
                "SELECT ve.voice_evidence_id,ve.status FROM core_voice_trait_evidence cve
                 JOIN voice_evidence ve ON ve.voice_evidence_id=cve.voice_evidence_id
                 WHERE cve.trait_id=?1 ORDER BY ve.voice_evidence_id",
            )?;
            statement
                .query_map([&trait_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?
        };
        if optional_text(guidance.as_deref()).is_none() && statuses.is_empty() {
            return Err(WorkLoreError::InvalidVault(format!(
                "Core Voice trait {trait_id} has no attributable provenance."
            )));
        }
        if let Some((evidence_id, status)) = statuses.iter().find(|(_, status)| status != "eligible") {
            return Err(WorkLoreError::InvalidVault(format!(
                "Core Voice trait {trait_id} still cites Voice Evidence {evidence_id} with status {status}. Remove or replace that provenance before activation."
            )));
        }
    }
    Ok(())
}

fn touch_voice(tx: &Transaction<'_>, voice_id: &str, now: &str) -> ServiceResult<()> {
    tx.execute(
        "UPDATE core_voices SET updated_at=?2,revision=revision+1 WHERE voice_id=?1",
        params![voice_id, now],
    )?;
    Ok(())
}

fn load_core_voice_from_connection(
    connection: &Connection,
    voice_id: &str,
) -> ServiceResult<CoreVoiceRecordView> {
    let base = connection
        .query_row(
            "SELECT voice_id,version_number,label,status,activated_at,superseded_at,created_at,updated_at,revision
             FROM core_voices WHERE voice_id=?1",
            [voice_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, u32>(8)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| WorkLoreError::InvalidVault(format!("Core Voice {voice_id} was not found.")))?;
    let traits = load_traits(connection, voice_id)?;
    Ok(CoreVoiceRecordView {
        voice_id: base.0,
        version_number: base.1,
        label: base.2,
        status: CoreVoiceStatus::parse(&base.3)?,
        traits,
        activated_at: base.4,
        superseded_at: base.5,
        created_at: base.6,
        updated_at: base.7,
        revision: base.8,
    })
}

fn load_traits(connection: &Connection, voice_id: &str) -> ServiceResult<Vec<CoreVoiceTraitView>> {
    let rows = {
        let mut statement = connection.prepare(
            "SELECT trait_id,name,value,user_guidance,created_at,updated_at,revision
             FROM core_voice_traits WHERE voice_id=?1 ORDER BY name,trait_id",
        )?;
        statement
            .query_map([voice_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, u32>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
    };
    rows.into_iter()
        .map(|row| {
            let evidence = {
                let mut statement = connection.prepare(
                    "SELECT ve.voice_evidence_id,ve.status FROM core_voice_trait_evidence cve
                     JOIN voice_evidence ve ON ve.voice_evidence_id=cve.voice_evidence_id
                     WHERE cve.trait_id=?1 ORDER BY ve.voice_evidence_id",
                )?;
                statement
                    .query_map([&row.0], |evidence_row| {
                        Ok(CoreVoiceTraitEvidenceView {
                            voice_evidence_id: evidence_row.get(0)?,
                            current_status: evidence_row.get(1)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?
            };
            let invalidated_evidence_ids = evidence
                .iter()
                .filter(|item| item.current_status != "eligible")
                .map(|item| item.voice_evidence_id.clone())
                .collect::<Vec<_>>();
            let has_guidance = optional_text(row.3.as_deref()).is_some();
            let provenance_kind = match (has_guidance, evidence.is_empty()) {
                (true, false) => "mixed",
                (true, true) => "user_guidance",
                (false, false) => "voice_evidence",
                (false, true) => "missing",
            }
            .to_string();
            let provenance_valid = provenance_kind != "missing" && invalidated_evidence_ids.is_empty();
            Ok(CoreVoiceTraitView {
                trait_id: row.0,
                name: row.1,
                value: row.2,
                user_guidance: row.3,
                evidence,
                provenance_kind,
                provenance_valid,
                invalidated_evidence_ids,
                created_at: row.4,
                updated_at: row.5,
                revision: row.6,
            })
        })
        .collect()
}

fn map_tone_mode(row: &rusqlite::Row<'_>) -> rusqlite::Result<ToneModeRecordView> {
    let status: String = row.get(4)?;
    Ok(ToneModeRecordView {
        tone_id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        instructions: row.get(3)?,
        status: match status.as_str() {
            "active" => ToneModeStatus::Active,
            "disabled" => ToneModeStatus::Disabled,
            "retired" => ToneModeStatus::Retired,
            _ => return Err(rusqlite::Error::InvalidQuery),
        },
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        revision: row.get(7)?,
    })
}

fn load_tone_mode(connection: &Connection, tone_id: &str) -> ServiceResult<ToneModeRecordView> {
    connection
        .query_row(
            "SELECT tone_id,name,description,instructions,status,created_at,updated_at,revision
             FROM tone_modes WHERE tone_id=?1",
            [tone_id],
            map_tone_mode,
        )
        .optional()?
        .ok_or_else(|| WorkLoreError::InvalidVault(format!("Tone Mode {tone_id} was not found.")))
}

fn map_voice_direction(row: &rusqlite::Row<'_>) -> rusqlite::Result<VoiceDirectionRecordView> {
    let status: String = row.get(4)?;
    let status = match status.as_str() {
        "proposed" => VoiceDirectionStatus::Proposed,
        "accepted" => VoiceDirectionStatus::Accepted,
        "completed" => VoiceDirectionStatus::Completed,
        "retired" => VoiceDirectionStatus::Retired,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(VoiceDirectionRecordView {
        voice_direction_id: row.get(0)?,
        statement: row.get(1)?,
        rationale: row.get(2)?,
        proposed_by: row.get(3)?,
        status,
        accepted_at: row.get(5)?,
        completed_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        revision: row.get(9)?,
    })
}

fn load_voice_direction(
    connection: &Connection,
    voice_direction_id: &str,
) -> ServiceResult<VoiceDirectionRecordView> {
    connection
        .query_row(
            "SELECT voice_direction_id,statement,rationale,proposed_by,status,accepted_at,completed_at,created_at,updated_at,revision
             FROM voice_directions WHERE voice_direction_id=?1",
            [voice_direction_id],
            map_voice_direction,
        )
        .optional()?
        .ok_or_else(|| {
            WorkLoreError::InvalidVault(format!(
                "Voice Direction {voice_direction_id} was not found."
            ))
        })
}

fn map_writing_rule(row: &rusqlite::Row<'_>) -> rusqlite::Result<WritingRuleRecordView> {
    let status: String = row.get(3)?;
    let status = match status.as_str() {
        "proposed" => WritingRuleStatus::Proposed,
        "active" => WritingRuleStatus::Active,
        "disabled" => WritingRuleStatus::Disabled,
        "retired" => WritingRuleStatus::Retired,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(WritingRuleRecordView {
        rule_id: row.get(0)?,
        name: row.get(1)?,
        instruction: row.get(2)?,
        status,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        revision: row.get(6)?,
    })
}

fn load_writing_rule(connection: &Connection, rule_id: &str) -> ServiceResult<WritingRuleRecordView> {
    connection
        .query_row(
            "SELECT rule_id,name,instruction,status,created_at,updated_at,revision
             FROM writing_rules WHERE rule_id=?1",
            [rule_id],
            map_writing_rule,
        )
        .optional()?
        .ok_or_else(|| WorkLoreError::InvalidVault(format!("Writing Rule {rule_id} was not found.")))
}

fn valid_direction_transition(from: VoiceDirectionStatus, to: VoiceDirectionStatus) -> bool {
    matches!(
        (from, to),
        (VoiceDirectionStatus::Proposed, VoiceDirectionStatus::Accepted)
            | (VoiceDirectionStatus::Proposed, VoiceDirectionStatus::Retired)
            | (VoiceDirectionStatus::Accepted, VoiceDirectionStatus::Completed)
            | (VoiceDirectionStatus::Accepted, VoiceDirectionStatus::Retired)
            | (VoiceDirectionStatus::Completed, VoiceDirectionStatus::Retired)
    )
}

fn valid_rule_transition(from: WritingRuleStatus, to: WritingRuleStatus) -> bool {
    matches!(
        (from, to),
        (WritingRuleStatus::Proposed, WritingRuleStatus::Active)
            | (WritingRuleStatus::Proposed, WritingRuleStatus::Retired)
            | (WritingRuleStatus::Active, WritingRuleStatus::Disabled)
            | (WritingRuleStatus::Active, WritingRuleStatus::Retired)
            | (WritingRuleStatus::Disabled, WritingRuleStatus::Active)
            | (WritingRuleStatus::Disabled, WritingRuleStatus::Retired)
    )
}

fn audit_tx(
    tx: &Transaction<'_>,
    event_type: &str,
    record_type: &str,
    record_id: &str,
    details: Value,
    now: &str,
) -> ServiceResult<()> {
    tx.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,?2,?3,?4,'user',?5,?6)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            event_type,
            record_type,
            record_id,
            details.to_string(),
            now
        ],
    )?;
    Ok(())
}

fn audit_connection(
    connection: &Connection,
    event_type: &str,
    record_type: &str,
    record_id: &str,
    details: Value,
    now: &str,
) -> ServiceResult<()> {
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,?2,?3,?4,'user',?5,?6)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            event_type,
            record_type,
            record_id,
            details.to_string(),
            now
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::vault_service;

    fn vault() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-voice-profile-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Voice Profile Test").expect("create vault");
        path
    }

    fn add_voice_evidence(path: &Path, status: &str) -> String {
        canonical_store::initialize(path).unwrap();
        let connection = open_connection(path).unwrap();
        let id = format!("voice_evidence_{}", Uuid::now_v7());
        let now = Utc::now().to_rfc3339();
        connection
            .execute(
                "INSERT INTO voice_evidence(voice_evidence_id,source_id,source_locator,text_snapshot,content_hash,authorship_state,status,
                 eligibility_reason,approval_state,approved_at,provenance_json,created_at,updated_at,revision)
                 VALUES (?1,?2,'full_source','Synthetic user text','synthetic','user_authored',?3,'synthetic_test','approved',?4,'{}',?4,?4,1)",
                params![&id, format!("source_{}", Uuid::now_v7()), status, &now],
            )
            .unwrap();
        id
    }

    #[test]
    fn core_voice_requires_attributable_provenance_and_supersedes_transactionally() {
        let path = vault();
        let voice_one = create_core_voice(
            &path,
            CreateCoreVoiceRequest {
                label: "Observed voice v1".into(),
            },
        )
        .unwrap();
        assert!(save_core_voice_trait(
            &path,
            SaveCoreVoiceTraitRequest {
                voice_id: voice_one.voice_id.clone(),
                trait_id: None,
                name: "Concise".into(),
                value: "Gets to the point quickly".into(),
                user_guidance: None,
                voice_evidence_ids: vec![],
            }
        )
        .is_err());

        let eligible = add_voice_evidence(&path, "eligible");
        save_core_voice_trait(
            &path,
            SaveCoreVoiceTraitRequest {
                voice_id: voice_one.voice_id.clone(),
                trait_id: None,
                name: "Direct".into(),
                value: "States the point before elaborating".into(),
                user_guidance: None,
                voice_evidence_ids: vec![eligible],
            },
        )
        .unwrap();
        let active_one = activate_core_voice(&path, &voice_one.voice_id).unwrap();
        assert_eq!(active_one.status, CoreVoiceStatus::Active);

        let voice_two = create_core_voice(
            &path,
            CreateCoreVoiceRequest {
                label: "Observed voice v2".into(),
            },
        )
        .unwrap();
        save_core_voice_trait(
            &path,
            SaveCoreVoiceTraitRequest {
                voice_id: voice_two.voice_id.clone(),
                trait_id: None,
                name: "Humor".into(),
                value: "Uses dry humor without turning the post into a bit".into(),
                user_guidance: Some("This is an explicit preference I want preserved.".into()),
                voice_evidence_ids: vec![],
            },
        )
        .unwrap();
        activate_core_voice(&path, &voice_two.voice_id).unwrap();

        let voices = list_core_voices(&path).unwrap();
        assert_eq!(
            voices.iter().filter(|voice| voice.status == CoreVoiceStatus::Active).count(),
            1
        );
        assert_eq!(
            voices
                .iter()
                .find(|voice| voice.voice_id == voice_one.voice_id)
                .unwrap()
                .status,
            CoreVoiceStatus::Superseded
        );
        assert_eq!(
            voices
                .iter()
                .find(|voice| voice.voice_id == voice_two.voice_id)
                .unwrap()
                .status,
            CoreVoiceStatus::Active
        );
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejected_or_retired_voice_evidence_cannot_support_new_traits() {
        let path = vault();
        let voice = create_core_voice(
            &path,
            CreateCoreVoiceRequest {
                label: "Proposed".into(),
            },
        )
        .unwrap();
        for status in ["rejected", "retired"] {
            let evidence = add_voice_evidence(&path, status);
            assert!(save_core_voice_trait(
                &path,
                SaveCoreVoiceTraitRequest {
                    voice_id: voice.voice_id.clone(),
                    trait_id: None,
                    name: format!("Trait {status}"),
                    value: "Synthetic".into(),
                    user_guidance: None,
                    voice_evidence_ids: vec![evidence],
                }
            )
            .is_err());
        }
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn later_evidence_retirement_surfaces_without_rewriting_historical_voice() {
        let path = vault();
        let evidence = add_voice_evidence(&path, "eligible");
        let voice = create_core_voice(
            &path,
            CreateCoreVoiceRequest {
                label: "Historical".into(),
            },
        )
        .unwrap();
        save_core_voice_trait(
            &path,
            SaveCoreVoiceTraitRequest {
                voice_id: voice.voice_id.clone(),
                trait_id: None,
                name: "Analytical".into(),
                value: "Explains the reasoning chain".into(),
                user_guidance: None,
                voice_evidence_ids: vec![evidence.clone()],
            },
        )
        .unwrap();
        activate_core_voice(&path, &voice.voice_id).unwrap();
        let connection = open_connection(&path).unwrap();
        connection
            .execute(
                "UPDATE voice_evidence SET status='retired',approval_state='revoked',revision=revision+1 WHERE voice_evidence_id=?1",
                [&evidence],
            )
            .unwrap();
        drop(connection);

        canonical_store::initialize(&path).unwrap();
        let reopened = list_core_voices(&path).unwrap();
        let historical = reopened
            .iter()
            .find(|candidate| candidate.voice_id == voice.voice_id)
            .unwrap();
        assert_eq!(historical.status, CoreVoiceStatus::Active);
        assert!(!historical.traits[0].provenance_valid);
        assert_eq!(historical.traits[0].invalidated_evidence_ids, vec![evidence]);
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn tone_direction_and_rules_have_independent_lifecycles() {
        let path = vault();
        let tone = create_tone_mode(
            &path,
            CreateToneModeRequest {
                name: "Reflective".into(),
                description: "Slower and more personal".into(),
                instructions: "Prefer concrete reflection over advice.".into(),
            },
        )
        .unwrap();
        let direction = create_voice_direction(
            &path,
            CreateVoiceDirectionRequest {
                statement: "Become more concise".into(),
                rationale: "Reduce setup before the useful point.".into(),
            },
        )
        .unwrap();
        let accepted = set_voice_direction_status(
            &path,
            SetVoiceDirectionStatusRequest {
                voice_direction_id: direction.voice_direction_id.clone(),
                status: VoiceDirectionStatus::Accepted,
            },
        )
        .unwrap();
        let rule = create_writing_rule(
            &path,
            CreateWritingRuleRequest {
                name: "No forced questions".into(),
                instruction: "Do not end a post with a question unless the question is genuine.".into(),
            },
        )
        .unwrap();
        let active_rule = update_writing_rule(
            &path,
            UpdateWritingRuleRequest {
                rule_id: rule.rule_id.clone(),
                name: rule.name,
                instruction: rule.instruction,
                status: WritingRuleStatus::Active,
            },
        )
        .unwrap();

        assert_eq!(tone.status, ToneModeStatus::Active);
        assert_eq!(accepted.status, VoiceDirectionStatus::Accepted);
        assert_eq!(active_rule.status, WritingRuleStatus::Active);
        assert!(list_core_voices(&path).unwrap().is_empty());
        assert_eq!(list_tone_modes(&path).unwrap()[0].tone_id, tone.tone_id);
        assert_eq!(
            list_voice_directions(&path).unwrap()[0].voice_direction_id,
            direction.voice_direction_id
        );
        assert_eq!(list_writing_rules(&path).unwrap()[0].rule_id, rule.rule_id);
        std::fs::remove_dir_all(path).unwrap();
    }
}
''',
)


# Register the provider-free voice profile service.
path = "src-tauri/src/services/mod.rs"
text = read(path)
text = replace_once(
    text,
    "pub mod voice_evidence_service;\n",
    "pub mod voice_evidence_service;\npub mod voice_profile_service;\n",
    "voice profile service module",
)
write(path, text)


# Extend Tauri voice commands while keeping Voice Evidence commands intact.
write(
    "src-tauri/src/commands/voice.rs",
    r'''use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::{
        voice_evidence_service::{
            self, CreateVoiceEvidenceResult, ReviewVoiceEvidenceRequest, VoiceEvidenceRecordView,
            VoiceSourceCandidateView,
        },
        voice_profile_service::{
            self, CoreVoiceRecordView, CreateCoreVoiceRequest, CreateToneModeRequest,
            CreateVoiceDirectionRequest, CreateWritingRuleRequest, SaveCoreVoiceTraitRequest,
            SetVoiceDirectionStatusRequest, ToneModeRecordView, UpdateToneModeRequest,
            UpdateWritingRuleRequest, VoiceDirectionRecordView, WritingRuleRecordView,
        },
    },
};

#[tauri::command]
pub fn list_voice_source_candidates(
    vault_path: String,
) -> CommandResult<Vec<VoiceSourceCandidateView>> {
    voice_evidence_service::list_voice_source_candidates(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_voice_evidence_from_source(
    vault_path: String,
    source_id: String,
) -> CommandResult<CreateVoiceEvidenceResult> {
    voice_evidence_service::create_voice_evidence_from_source(
        &PathBuf::from(vault_path),
        &source_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_voice_evidence(
    vault_path: String,
    voice_evidence_id: String,
) -> CommandResult<VoiceEvidenceRecordView> {
    voice_evidence_service::load_voice_evidence(&PathBuf::from(vault_path), &voice_evidence_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_voice_evidence(vault_path: String) -> CommandResult<Vec<VoiceEvidenceRecordView>> {
    voice_evidence_service::list_voice_evidence(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn review_voice_evidence(
    vault_path: String,
    request: ReviewVoiceEvidenceRequest,
) -> CommandResult<VoiceEvidenceRecordView> {
    voice_evidence_service::review_voice_evidence(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_core_voices(vault_path: String) -> CommandResult<Vec<CoreVoiceRecordView>> {
    voice_profile_service::list_core_voices(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_core_voice(
    vault_path: String,
    request: CreateCoreVoiceRequest,
) -> CommandResult<CoreVoiceRecordView> {
    voice_profile_service::create_core_voice(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn save_core_voice_trait(
    vault_path: String,
    request: SaveCoreVoiceTraitRequest,
) -> CommandResult<CoreVoiceRecordView> {
    voice_profile_service::save_core_voice_trait(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn delete_core_voice_trait(
    vault_path: String,
    voice_id: String,
    trait_id: String,
) -> CommandResult<CoreVoiceRecordView> {
    voice_profile_service::delete_core_voice_trait(&PathBuf::from(vault_path), &voice_id, &trait_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn activate_core_voice(
    vault_path: String,
    voice_id: String,
) -> CommandResult<CoreVoiceRecordView> {
    voice_profile_service::activate_core_voice(&PathBuf::from(vault_path), &voice_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_tone_modes(vault_path: String) -> CommandResult<Vec<ToneModeRecordView>> {
    voice_profile_service::list_tone_modes(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_tone_mode(
    vault_path: String,
    request: CreateToneModeRequest,
) -> CommandResult<ToneModeRecordView> {
    voice_profile_service::create_tone_mode(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_tone_mode(
    vault_path: String,
    request: UpdateToneModeRequest,
) -> CommandResult<ToneModeRecordView> {
    voice_profile_service::update_tone_mode(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_voice_directions(
    vault_path: String,
) -> CommandResult<Vec<VoiceDirectionRecordView>> {
    voice_profile_service::list_voice_directions(&PathBuf::from(vault_path))
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_voice_direction(
    vault_path: String,
    request: CreateVoiceDirectionRequest,
) -> CommandResult<VoiceDirectionRecordView> {
    voice_profile_service::create_voice_direction(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn set_voice_direction_status(
    vault_path: String,
    request: SetVoiceDirectionStatusRequest,
) -> CommandResult<VoiceDirectionRecordView> {
    voice_profile_service::set_voice_direction_status(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_writing_rules(vault_path: String) -> CommandResult<Vec<WritingRuleRecordView>> {
    voice_profile_service::list_writing_rules(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_writing_rule(
    vault_path: String,
    request: CreateWritingRuleRequest,
) -> CommandResult<WritingRuleRecordView> {
    voice_profile_service::create_writing_rule(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn update_writing_rule(
    vault_path: String,
    request: UpdateWritingRuleRequest,
) -> CommandResult<WritingRuleRecordView> {
    voice_profile_service::update_writing_rule(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}
''',
)


# Register the commands with Tauri.
path = "src-tauri/src/lib.rs"
text = read(path)
old_voice_import = '''    voice::{\n        create_voice_evidence_from_source, get_voice_evidence, list_voice_evidence,\n        list_voice_source_candidates, review_voice_evidence,\n    },\n'''
new_voice_import = '''    voice::{\n        activate_core_voice, create_core_voice, create_tone_mode, create_voice_direction,\n        create_voice_evidence_from_source, create_writing_rule, delete_core_voice_trait,\n        get_voice_evidence, list_core_voices, list_tone_modes, list_voice_directions,\n        list_voice_evidence, list_voice_source_candidates, list_writing_rules,\n        review_voice_evidence, save_core_voice_trait, set_voice_direction_status,\n        update_tone_mode, update_writing_rule,\n    },\n'''
text = replace_once(text, old_voice_import, new_voice_import, "voice command imports")
old_handler = '''            list_voice_source_candidates,\n            create_voice_evidence_from_source,\n            get_voice_evidence,\n            list_voice_evidence,\n            review_voice_evidence,\n            create_topic,\n'''
new_handler = '''            list_voice_source_candidates,\n            create_voice_evidence_from_source,\n            get_voice_evidence,\n            list_voice_evidence,\n            review_voice_evidence,\n            list_core_voices,\n            create_core_voice,\n            save_core_voice_trait,\n            delete_core_voice_trait,\n            activate_core_voice,\n            list_tone_modes,\n            create_tone_mode,\n            update_tone_mode,\n            list_voice_directions,\n            create_voice_direction,\n            set_voice_direction_status,\n            list_writing_rules,\n            create_writing_rule,\n            update_writing_rule,\n            create_topic,\n'''
text = replace_once(text, old_handler, new_handler, "voice command handler registration")
write(path, text)


# Frontend domain types.
path = "src/domain/types.ts"
text = read(path)
anchor = '''export interface ReviewVoiceEvidenceRequest {\n  voiceEvidenceId: string;\n  authorshipState: VoiceAuthorshipState;\n  decision: VoiceEvidenceDecision;\n}\n'''
addition = anchor + '''\nexport type CoreVoiceStatus = "proposed" | "active" | "superseded";\nexport type ToneModeStatus = "active" | "disabled" | "retired";\nexport type VoiceDirectionStatus = "proposed" | "accepted" | "completed" | "retired";\nexport type WritingRuleStatus = "proposed" | "active" | "disabled" | "retired";\n\nexport interface CoreVoiceTraitEvidence {\n  voiceEvidenceId: string;\n  currentStatus: string;\n}\n\nexport interface CoreVoiceTrait {\n  traitId: string;\n  name: string;\n  value: string;\n  userGuidance: string | null;\n  evidence: CoreVoiceTraitEvidence[];\n  provenanceKind: "voice_evidence" | "user_guidance" | "mixed" | "missing" | string;\n  provenanceValid: boolean;\n  invalidatedEvidenceIds: string[];\n  createdAt: string;\n  updatedAt: string;\n  revision: number;\n}\n\nexport interface CoreVoiceRecord {\n  voiceId: string;\n  versionNumber: number;\n  label: string;\n  status: CoreVoiceStatus;\n  traits: CoreVoiceTrait[];\n  activatedAt: string | null;\n  supersededAt: string | null;\n  createdAt: string;\n  updatedAt: string;\n  revision: number;\n}\n\nexport interface CreateCoreVoiceRequest {\n  label: string;\n}\n\nexport interface SaveCoreVoiceTraitRequest {\n  voiceId: string;\n  traitId?: string | null;\n  name: string;\n  value: string;\n  userGuidance?: string | null;\n  voiceEvidenceIds: string[];\n}\n\nexport interface ToneModeRecord {\n  toneId: string;\n  name: string;\n  description: string;\n  instructions: string;\n  status: ToneModeStatus;\n  createdAt: string;\n  updatedAt: string;\n  revision: number;\n}\n\nexport interface CreateToneModeRequest {\n  name: string;\n  description: string;\n  instructions: string;\n}\n\nexport interface UpdateToneModeRequest extends CreateToneModeRequest {\n  toneId: string;\n  status: ToneModeStatus;\n}\n\nexport interface VoiceDirectionRecord {\n  voiceDirectionId: string;\n  statement: string;\n  rationale: string;\n  proposedBy: string;\n  status: VoiceDirectionStatus;\n  acceptedAt: string | null;\n  completedAt: string | null;\n  createdAt: string;\n  updatedAt: string;\n  revision: number;\n}\n\nexport interface CreateVoiceDirectionRequest {\n  statement: string;\n  rationale: string;\n}\n\nexport interface SetVoiceDirectionStatusRequest {\n  voiceDirectionId: string;\n  status: VoiceDirectionStatus;\n}\n\nexport interface WritingRuleRecord {\n  ruleId: string;\n  name: string;\n  instruction: string;\n  status: WritingRuleStatus;\n  createdAt: string;\n  updatedAt: string;\n  revision: number;\n}\n\nexport interface CreateWritingRuleRequest {\n  name: string;\n  instruction: string;\n}\n\nexport interface UpdateWritingRuleRequest extends CreateWritingRuleRequest {\n  ruleId: string;\n  status: WritingRuleStatus;\n}\n'''
text = replace_once(text, anchor, addition, "voice frontend types")
write(path, text)


# Frontend API wrappers.
path = "src/lib/workloreApi.ts"
text = read(path)
old_import_tail = '''  CreateVoiceEvidenceResult,\n  ReviewVoiceEvidenceRequest,\n  VoiceEvidenceRecord,\n  VoiceSourceCandidate,\n} from "../domain/types";\n'''
new_import_tail = '''  CreateVoiceEvidenceResult,\n  ReviewVoiceEvidenceRequest,\n  VoiceEvidenceRecord,\n  VoiceSourceCandidate,\n  CoreVoiceRecord,\n  CreateCoreVoiceRequest,\n  SaveCoreVoiceTraitRequest,\n  ToneModeRecord,\n  CreateToneModeRequest,\n  UpdateToneModeRequest,\n  VoiceDirectionRecord,\n  CreateVoiceDirectionRequest,\n  SetVoiceDirectionStatusRequest,\n  WritingRuleRecord,\n  CreateWritingRuleRequest,\n  UpdateWritingRuleRequest,\n} from "../domain/types";\n'''
text = replace_once(text, old_import_tail, new_import_tail, "voice API imports")
voice_api_anchor = '''export async function reviewVoiceEvidence(\n  vaultPath: string,\n  request: ReviewVoiceEvidenceRequest,\n): Promise<VoiceEvidenceRecord> {\n  return invoke<VoiceEvidenceRecord>("review_voice_evidence", { vaultPath, request });\n}\n'''
voice_api_addition = voice_api_anchor + '''\nexport async function listCoreVoices(vaultPath: string): Promise<CoreVoiceRecord[]> {\n  return invoke<CoreVoiceRecord[]>("list_core_voices", { vaultPath });\n}\n\nexport async function createCoreVoice(\n  vaultPath: string,\n  request: CreateCoreVoiceRequest,\n): Promise<CoreVoiceRecord> {\n  return invoke<CoreVoiceRecord>("create_core_voice", { vaultPath, request });\n}\n\nexport async function saveCoreVoiceTrait(\n  vaultPath: string,\n  request: SaveCoreVoiceTraitRequest,\n): Promise<CoreVoiceRecord> {\n  return invoke<CoreVoiceRecord>("save_core_voice_trait", { vaultPath, request });\n}\n\nexport async function deleteCoreVoiceTrait(\n  vaultPath: string,\n  voiceId: string,\n  traitId: string,\n): Promise<CoreVoiceRecord> {\n  return invoke<CoreVoiceRecord>("delete_core_voice_trait", { vaultPath, voiceId, traitId });\n}\n\nexport async function activateCoreVoice(\n  vaultPath: string,\n  voiceId: string,\n): Promise<CoreVoiceRecord> {\n  return invoke<CoreVoiceRecord>("activate_core_voice", { vaultPath, voiceId });\n}\n\nexport async function listToneModes(vaultPath: string): Promise<ToneModeRecord[]> {\n  return invoke<ToneModeRecord[]>("list_tone_modes", { vaultPath });\n}\n\nexport async function createToneMode(\n  vaultPath: string,\n  request: CreateToneModeRequest,\n): Promise<ToneModeRecord> {\n  return invoke<ToneModeRecord>("create_tone_mode", { vaultPath, request });\n}\n\nexport async function updateToneMode(\n  vaultPath: string,\n  request: UpdateToneModeRequest,\n): Promise<ToneModeRecord> {\n  return invoke<ToneModeRecord>("update_tone_mode", { vaultPath, request });\n}\n\nexport async function listVoiceDirections(vaultPath: string): Promise<VoiceDirectionRecord[]> {\n  return invoke<VoiceDirectionRecord[]>("list_voice_directions", { vaultPath });\n}\n\nexport async function createVoiceDirection(\n  vaultPath: string,\n  request: CreateVoiceDirectionRequest,\n): Promise<VoiceDirectionRecord> {\n  return invoke<VoiceDirectionRecord>("create_voice_direction", { vaultPath, request });\n}\n\nexport async function setVoiceDirectionStatus(\n  vaultPath: string,\n  request: SetVoiceDirectionStatusRequest,\n): Promise<VoiceDirectionRecord> {\n  return invoke<VoiceDirectionRecord>("set_voice_direction_status", { vaultPath, request });\n}\n\nexport async function listWritingRules(vaultPath: string): Promise<WritingRuleRecord[]> {\n  return invoke<WritingRuleRecord[]>("list_writing_rules", { vaultPath });\n}\n\nexport async function createWritingRule(\n  vaultPath: string,\n  request: CreateWritingRuleRequest,\n): Promise<WritingRuleRecord> {\n  return invoke<WritingRuleRecord>("create_writing_rule", { vaultPath, request });\n}\n\nexport async function updateWritingRule(\n  vaultPath: string,\n  request: UpdateWritingRuleRequest,\n): Promise<WritingRuleRecord> {\n  return invoke<WritingRuleRecord>("update_writing_rule", { vaultPath, request });\n}\n'''
text = replace_once(text, voice_api_anchor, voice_api_addition, "voice API wrappers")
write(path, text)


# Thin but functional Voice workspace for evidence, identity, tone, direction, and rules.
write(
    "src/components/VoiceWorkspace.tsx",
    r'''import { useEffect, useMemo, useState } from "react";
import type {
  CoreVoiceRecord,
  CoreVoiceTrait,
  ToneModeRecord,
  VoiceAuthorshipState,
  VoiceDirectionRecord,
  VoiceDirectionStatus,
  VoiceEvidenceDecision,
  VoiceEvidenceRecord,
  VoiceSourceCandidate,
  WritingRuleRecord,
  WritingRuleStatus,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  activateCoreVoice,
  createCoreVoice,
  createToneMode,
  createVoiceDirection,
  createVoiceEvidenceFromSource,
  createWritingRule,
  deleteCoreVoiceTrait,
  listCoreVoices,
  listToneModes,
  listVoiceDirections,
  listVoiceEvidence,
  listVoiceSourceCandidates,
  listWritingRules,
  reviewVoiceEvidence,
  saveCoreVoiceTrait,
  setVoiceDirectionStatus,
  updateToneMode,
  updateWritingRule,
} from "../lib/workloreApi";
import { canApproveVoiceEvidence, voiceEligibilityReasonLabel } from "../voiceEvidencePolicy";
import "../voice.css";

const AUTHORSHIP_OPTIONS: Array<{ value: VoiceAuthorshipState; label: string }> = [
  { value: "unknown", label: "Not established" },
  { value: "user_authored", label: "I wrote this" },
  { value: "user_edited_model", label: "I substantially edited model-origin text" },
  { value: "model_generated", label: "Raw model-generated text" },
  { value: "external_author", label: "Written by someone else" },
];

type TraitDraft = {
  traitId?: string;
  name: string;
  value: string;
  guidance: string;
  evidenceIds: string[];
};

const EMPTY_TRAIT: TraitDraft = { name: "", value: "", guidance: "", evidenceIds: [] };

export function VoiceWorkspace({ vaultPath }: { vaultPath: string }) {
  const [candidates, setCandidates] = useState<VoiceSourceCandidate[]>([]);
  const [evidence, setEvidence] = useState<VoiceEvidenceRecord[]>([]);
  const [voices, setVoices] = useState<CoreVoiceRecord[]>([]);
  const [tones, setTones] = useState<ToneModeRecord[]>([]);
  const [directions, setDirections] = useState<VoiceDirectionRecord[]>([]);
  const [rules, setRules] = useState<WritingRuleRecord[]>([]);
  const [authorship, setAuthorship] = useState<Record<string, VoiceAuthorshipState>>({});
  const [traitDrafts, setTraitDrafts] = useState<Record<string, TraitDraft>>({});
  const [voiceLabel, setVoiceLabel] = useState("");
  const [toneName, setToneName] = useState("");
  const [toneDescription, setToneDescription] = useState("");
  const [toneInstructions, setToneInstructions] = useState("");
  const [directionStatement, setDirectionStatement] = useState("");
  const [directionRationale, setDirectionRationale] = useState("");
  const [ruleName, setRuleName] = useState("");
  const [ruleInstruction, setRuleInstruction] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const eligibleEvidence = useMemo(
    () => evidence.filter((item) => item.status === "eligible"),
    [evidence],
  );

  useEffect(() => {
    setNotice(null);
    setError(null);
    void refresh();
  }, [vaultPath]);

  async function refresh() {
    try {
      const [candidateResult, evidenceResult, voiceResult, toneResult, directionResult, ruleResult] =
        await Promise.all([
          listVoiceSourceCandidates(vaultPath),
          listVoiceEvidence(vaultPath),
          listCoreVoices(vaultPath),
          listToneModes(vaultPath),
          listVoiceDirections(vaultPath),
          listWritingRules(vaultPath),
        ]);
      setCandidates(candidateResult);
      setEvidence(evidenceResult);
      setVoices(voiceResult);
      setTones(toneResult);
      setDirections(directionResult);
      setRules(ruleResult);
      setAuthorship((current) => {
        const next = { ...current };
        for (const item of evidenceResult) {
          if (!(item.voiceEvidenceId in next)) next[item.voiceEvidenceId] = item.authorshipState;
        }
        return next;
      });
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function run(action: string, work: () => Promise<unknown>, success: string) {
    setBusy(action);
    setNotice(null);
    setError(null);
    try {
      await work();
      setNotice(success);
      await refresh();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function createCandidate(sourceId: string) {
    await run(
      "Creating governed Voice Evidence candidate",
      () => createVoiceEvidenceFromSource(vaultPath, sourceId),
      "Voice Evidence candidate is governed and pending explicit authorship and approval.",
    );
  }

  async function review(item: VoiceEvidenceRecord, decision: VoiceEvidenceDecision) {
    const selectedAuthorship = authorship[item.voiceEvidenceId] ?? item.authorshipState;
    await run(
      "Saving Voice Evidence decision",
      () =>
        reviewVoiceEvidence(vaultPath, {
          voiceEvidenceId: item.voiceEvidenceId,
          authorshipState: selectedAuthorship,
          decision,
        }),
      decision === "approve"
        ? "Voice Evidence review saved. Eligible material may now support Core Voice."
        : `Voice Evidence ${decision} decision saved.`,
    );
  }

  async function addVoiceVersion() {
    const label = voiceLabel.trim();
    if (!label) return;
    await run(
      "Creating Core Voice version",
      () => createCoreVoice(vaultPath, { label }),
      "Proposed Core Voice version created. It has no traits until you add attributable ones.",
    );
    setVoiceLabel("");
  }

  function draftFor(voiceId: string): TraitDraft {
    return traitDrafts[voiceId] ?? EMPTY_TRAIT;
  }

  function updateTraitDraft(voiceId: string, patch: Partial<TraitDraft>) {
    setTraitDrafts((current) => ({
      ...current,
      [voiceId]: { ...(current[voiceId] ?? EMPTY_TRAIT), ...patch },
    }));
  }

  function editTrait(voiceId: string, trait: CoreVoiceTrait) {
    setTraitDrafts((current) => ({
      ...current,
      [voiceId]: {
        traitId: trait.traitId,
        name: trait.name,
        value: trait.value,
        guidance: trait.userGuidance ?? "",
        evidenceIds: trait.evidence.map((item) => item.voiceEvidenceId),
      },
    }));
  }

  async function saveTrait(voiceId: string) {
    const draft = draftFor(voiceId);
    await run(
      draft.traitId ? "Updating Core Voice trait" : "Adding Core Voice trait",
      () =>
        saveCoreVoiceTrait(vaultPath, {
          voiceId,
          traitId: draft.traitId ?? null,
          name: draft.name,
          value: draft.value,
          userGuidance: draft.guidance || null,
          voiceEvidenceIds: draft.evidenceIds,
        }),
      "Core Voice trait saved with explicit provenance.",
    );
    setTraitDrafts((current) => ({ ...current, [voiceId]: { ...EMPTY_TRAIT } }));
  }

  async function removeTrait(voiceId: string, traitId: string) {
    await run(
      "Removing proposed Core Voice trait",
      () => deleteCoreVoiceTrait(vaultPath, voiceId, traitId),
      "Proposed trait removed.",
    );
  }

  async function activateVoice(voiceId: string) {
    await run(
      "Activating Core Voice version",
      () => activateCoreVoice(vaultPath, voiceId),
      "Core Voice version activated. Any prior active version was retained as superseded history.",
    );
  }

  async function addTone() {
    if (!toneName.trim() || !toneInstructions.trim()) return;
    await run(
      "Creating Tone Mode",
      () =>
        createToneMode(vaultPath, {
          name: toneName,
          description: toneDescription,
          instructions: toneInstructions,
        }),
      "Tone Mode created as an active expression layer, not a separate identity.",
    );
    setToneName("");
    setToneDescription("");
    setToneInstructions("");
  }

  async function setToneStatus(tone: ToneModeRecord, status: ToneModeRecord["status"]) {
    await run(
      "Updating Tone Mode",
      () => updateToneMode(vaultPath, { ...tone, status }),
      `Tone Mode is now ${status}.`,
    );
  }

  async function addDirection() {
    if (!directionStatement.trim()) return;
    await run(
      "Creating Voice Direction",
      () =>
        createVoiceDirection(vaultPath, {
          statement: directionStatement,
          rationale: directionRationale,
        }),
      "Voice Direction saved as proposed. It does not alter Core Voice unless you deliberately evolve it later.",
    );
    setDirectionStatement("");
    setDirectionRationale("");
  }

  async function setDirectionStatus(item: VoiceDirectionRecord, status: VoiceDirectionStatus) {
    await run(
      "Updating Voice Direction",
      () => setVoiceDirectionStatus(vaultPath, { voiceDirectionId: item.voiceDirectionId, status }),
      `Voice Direction is now ${status}. Core Voice was not rewritten.`,
    );
  }

  async function addRule() {
    if (!ruleName.trim() || !ruleInstruction.trim()) return;
    await run(
      "Creating Writing Rule",
      () => createWritingRule(vaultPath, { name: ruleName, instruction: ruleInstruction }),
      "Writing Rule saved as proposed. It remains separate from identity and tone.",
    );
    setRuleName("");
    setRuleInstruction("");
  }

  async function setRuleStatus(item: WritingRuleRecord, status: WritingRuleStatus) {
    await run(
      "Updating Writing Rule",
      () => updateWritingRule(vaultPath, { ...item, status }),
      `Writing Rule is now ${status}.`,
    );
  }

  return (
    <div className="shell-stack voice-workspace">
      <section className="workspace-panel" aria-labelledby="voice-heading">
        <p className="eyebrow">Phase 2 - governed identity and intentional range</p>
        <h2 id="voice-heading">Voice</h2>
        <p>
          Separate what is observed about your stable voice from how you intentionally vary it, how you
          want it to evolve, and the rules you want generated writing to follow.
        </p>
        <div className="voice-concept-grid">
          <div className="next-step-card"><h3>Core Voice</h3><p>Observed identity, supported only by eligible evidence or explicit guidance.</p></div>
          <div className="next-step-card"><h3>Tone Modes</h3><p>Intentional range. Same author, different register.</p></div>
          <div className="next-step-card"><h3>Voice Direction</h3><p>Desired evolution. It never rewrites history by itself.</p></div>
          <div className="next-step-card"><h3>Writing Rules</h3><p>Behavioral constraints. Useful, but not identity.</p></div>
        </div>
      </section>

      <section className="workspace-panel" aria-labelledby="voice-candidates-heading">
        <div className="panel-heading-row">
          <div><p className="eyebrow">Writing samples</p><h2 id="voice-candidates-heading">Voice Evidence candidates</h2></div>
          <span className="status-pill">{candidates.length}</span>
        </div>
        {candidates.length === 0 ? (
          <div className="empty-state compact-empty"><h3>No writing samples yet</h3><p>Capture or import a Source typed as Writing sample. It will still begin pending.</p></div>
        ) : (
          <div className="voice-card-list">
            {candidates.map((candidate) => (
              <article className="voice-candidate-card" key={candidate.sourceId}>
                <div className="voice-card-heading"><div><h3>{candidate.displayName}</h3><p className="voice-meta">{candidate.sourceOrigin} | {candidate.sourceId}</p></div>{candidate.voiceEvidenceId ? <span className="status-pill">Governed</span> : null}</div>
                <p className="voice-preview">{candidate.textPreview || "No extractable text preview."}</p>
                {candidate.blockedReason ? <p className="voice-warning">Blocked: {candidate.blockedReason.replaceAll("_", " ")}</p> : candidate.voiceEvidenceId ? <p className="voice-rule">This source already has a durable Voice Evidence record below.</p> : <button className="secondary-button compact" disabled={busy !== null} onClick={() => void createCandidate(candidate.sourceId)}>Review for voice</button>}
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="voice-evidence-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Governed material</p><h2 id="voice-evidence-heading">Voice Evidence</h2></div><span className="status-pill">{evidence.length}</span></div>
        {evidence.length === 0 ? <div className="empty-state compact-empty"><h3>No Voice Evidence records yet</h3><p>Creating a candidate does not make it eligible. Authorship and approval stay explicit.</p></div> : (
          <div className="voice-card-list">
            {evidence.map((item) => {
              const selectedAuthorship = authorship[item.voiceEvidenceId] ?? item.authorshipState;
              const authorshipLocked = item.authorshipState !== "unknown";
              return (
                <article className="voice-evidence-card" key={item.voiceEvidenceId}>
                  <div className="voice-card-heading"><div><h3>{item.sourceDisplayName}</h3><p className="voice-meta">{item.voiceEvidenceId} | revision {item.revision}</p></div><span className={`status-pill ${item.status === "eligible" ? "" : "attention"}`}>{item.status}</span></div>
                  <p className="voice-preview">{item.textPreview}</p><p className="voice-rule">{voiceEligibilityReasonLabel(item.eligibilityReason)}</p>
                  <label className="field-label" htmlFor={`authorship-${item.voiceEvidenceId}`}>Authorship provenance</label>
                  <select id={`authorship-${item.voiceEvidenceId}`} value={selectedAuthorship} disabled={authorshipLocked || item.status === "retired" || busy !== null} onChange={(event) => setAuthorship((current) => ({ ...current, [item.voiceEvidenceId]: event.target.value as VoiceAuthorshipState }))}>
                    {AUTHORSHIP_OPTIONS.map((option) => <option key={option.value} value={option.value}>{option.label}</option>)}
                  </select>
                  {authorshipLocked ? <p className="voice-rule">Authorship provenance is locked after the first explicit assertion.</p> : null}
                  <div className="support-actions voice-actions">
                    <button className="primary-button compact" disabled={busy !== null || item.status === "eligible" || !canApproveVoiceEvidence(selectedAuthorship, item.status)} onClick={() => void review(item, "approve")}>Approve for voice</button>
                    <button className="secondary-button compact" disabled={busy !== null || item.status === "retired"} onClick={() => void review(item, "reject")}>Reject / exclude</button>
                    <button className="quiet-button compact" disabled={busy !== null || item.status === "retired"} onClick={() => void review(item, "retire")}>Retire</button>
                  </div>
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="core-voice-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Observed identity</p><h2 id="core-voice-heading">Core Voice versions</h2></div><span className="status-pill">{voices.length}</span></div>
        <div className="voice-inline-form"><input value={voiceLabel} onChange={(event) => setVoiceLabel(event.target.value)} placeholder="Label this proposed version" /><button className="primary-button compact" disabled={busy !== null || !voiceLabel.trim()} onClick={() => void addVoiceVersion()}>Create proposed version</button></div>
        {voices.length === 0 ? <p className="voice-rule">No traits are inferred automatically. Create a version only when you have something attributable to record.</p> : (
          <div className="voice-card-list">
            {voices.map((voice) => {
              const draft = draftFor(voice.voiceId);
              return (
                <article className="voice-model-card" key={voice.voiceId}>
                  <div className="voice-card-heading"><div><h3>v{voice.versionNumber}: {voice.label}</h3><p className="voice-meta">{voice.voiceId} | revision {voice.revision}</p></div><span className={`status-pill ${voice.status === "active" ? "" : "attention"}`}>{voice.status}</span></div>
                  {voice.traits.length === 0 ? <p className="voice-rule">No traits yet. Empty is better than invented.</p> : (
                    <div className="voice-trait-list">{voice.traits.map((trait) => <div className="voice-trait" key={trait.traitId}><div><strong>{trait.name}</strong>: {trait.value}<p className={trait.provenanceValid ? "voice-rule" : "voice-warning"}>Provenance: {trait.provenanceKind}{trait.invalidatedEvidenceIds.length ? ` | needs review: ${trait.invalidatedEvidenceIds.join(", ")}` : ""}</p>{trait.userGuidance ? <p className="voice-meta">Guidance: {trait.userGuidance}</p> : null}{trait.evidence.length ? <p className="voice-meta">Evidence: {trait.evidence.map((item) => `${item.voiceEvidenceId} (${item.currentStatus})`).join(", ")}</p> : null}</div>{voice.status === "proposed" ? <div className="support-actions"><button className="quiet-button compact" onClick={() => editTrait(voice.voiceId, trait)}>Edit</button><button className="quiet-button compact" onClick={() => void removeTrait(voice.voiceId, trait.traitId)}>Remove</button></div> : null}</div>)}</div>
                  )}
                  {voice.status === "proposed" ? (
                    <div className="voice-trait-editor">
                      <h4>{draft.traitId ? "Edit trait" : "Add attributable trait"}</h4>
                      <input value={draft.name} onChange={(event) => updateTraitDraft(voice.voiceId, { name: event.target.value })} placeholder="Trait name" />
                      <textarea value={draft.value} onChange={(event) => updateTraitDraft(voice.voiceId, { value: event.target.value })} placeholder="What is stable about the voice?" rows={2} />
                      <textarea value={draft.guidance} onChange={(event) => updateTraitDraft(voice.voiceId, { guidance: event.target.value })} placeholder="Optional explicit user guidance supporting this trait" rows={2} />
                      <fieldset className="voice-evidence-picker"><legend>Eligible Voice Evidence (optional if guidance is explicit)</legend>{eligibleEvidence.length === 0 ? <p className="voice-rule">No eligible Voice Evidence yet.</p> : eligibleEvidence.map((item) => <label key={item.voiceEvidenceId}><input type="checkbox" checked={draft.evidenceIds.includes(item.voiceEvidenceId)} onChange={(event) => updateTraitDraft(voice.voiceId, { evidenceIds: event.target.checked ? [...draft.evidenceIds, item.voiceEvidenceId] : draft.evidenceIds.filter((id) => id !== item.voiceEvidenceId) })} /> {item.sourceDisplayName}</label>)}</fieldset>
                      <div className="support-actions"><button className="secondary-button compact" disabled={busy !== null || !draft.name.trim() || !draft.value.trim() || (!draft.guidance.trim() && draft.evidenceIds.length === 0)} onClick={() => void saveTrait(voice.voiceId)}>{draft.traitId ? "Save changes" : "Add trait"}</button>{draft.traitId ? <button className="quiet-button compact" onClick={() => updateTraitDraft(voice.voiceId, { ...EMPTY_TRAIT })}>Cancel edit</button> : null}<button className="primary-button compact" disabled={busy !== null || voice.traits.length === 0} onClick={() => void activateVoice(voice.voiceId)}>Activate version</button></div>
                    </div>
                  ) : null}
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="tone-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Intentional expression</p><h2 id="tone-heading">Tone Modes</h2></div><span className="status-pill">{tones.length}</span></div>
        <div className="voice-form-grid"><input value={toneName} onChange={(event) => setToneName(event.target.value)} placeholder="Mode name" /><input value={toneDescription} onChange={(event) => setToneDescription(event.target.value)} placeholder="Short description" /><textarea value={toneInstructions} onChange={(event) => setToneInstructions(event.target.value)} placeholder="How should expression change in this mode?" rows={2} /><button className="secondary-button compact" disabled={busy !== null || !toneName.trim() || !toneInstructions.trim()} onClick={() => void addTone()}>Add Tone Mode</button></div>
        <div className="voice-card-list">{tones.map((tone) => <article className="voice-model-card" key={tone.toneId}><div className="voice-card-heading"><div><h3>{tone.name}</h3><p>{tone.description}</p></div><span className="status-pill">{tone.status}</span></div><p className="voice-preview">{tone.instructions}</p><div className="support-actions">{tone.status === "active" ? <button className="quiet-button compact" onClick={() => void setToneStatus(tone, "disabled")}>Disable</button> : tone.status === "disabled" ? <button className="secondary-button compact" onClick={() => void setToneStatus(tone, "active")}>Enable</button> : null}{tone.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setToneStatus(tone, "retired")}>Retire</button> : null}</div></article>)}</div>
      </section>

      <section className="workspace-panel" aria-labelledby="direction-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Desired evolution</p><h2 id="direction-heading">Voice Direction</h2></div><span className="status-pill">{directions.length}</span></div>
        <div className="voice-form-grid"><input value={directionStatement} onChange={(event) => setDirectionStatement(event.target.value)} placeholder="Example: become more concise" /><textarea value={directionRationale} onChange={(event) => setDirectionRationale(event.target.value)} placeholder="Why do you want this change?" rows={2} /><button className="secondary-button compact" disabled={busy !== null || !directionStatement.trim()} onClick={() => void addDirection()}>Propose direction</button></div>
        <div className="voice-card-list">{directions.map((item) => <article className="voice-model-card" key={item.voiceDirectionId}><div className="voice-card-heading"><div><h3>{item.statement}</h3><p>{item.rationale}</p></div><span className="status-pill">{item.status}</span></div><p className="voice-meta">Proposed by {item.proposedBy}. Accepting this does not mutate Core Voice.</p><div className="support-actions">{item.status === "proposed" ? <button className="primary-button compact" onClick={() => void setDirectionStatus(item, "accepted")}>Accept direction</button> : null}{item.status === "accepted" ? <button className="secondary-button compact" onClick={() => void setDirectionStatus(item, "completed")}>Mark completed</button> : null}{item.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setDirectionStatus(item, "retired")}>Retire</button> : null}</div></article>)}</div>
      </section>

      <section className="workspace-panel" aria-labelledby="rules-heading">
        <div className="panel-heading-row"><div><p className="eyebrow">Behavioral constraints</p><h2 id="rules-heading">Writing Rules</h2></div><span className="status-pill">{rules.length}</span></div>
        <div className="voice-form-grid"><input value={ruleName} onChange={(event) => setRuleName(event.target.value)} placeholder="Rule name" /><textarea value={ruleInstruction} onChange={(event) => setRuleInstruction(event.target.value)} placeholder="Explicit instruction" rows={2} /><button className="secondary-button compact" disabled={busy !== null || !ruleName.trim() || !ruleInstruction.trim()} onClick={() => void addRule()}>Add proposed rule</button></div>
        <div className="voice-card-list">{rules.map((item) => <article className="voice-model-card" key={item.ruleId}><div className="voice-card-heading"><div><h3>{item.name}</h3><p className="voice-preview">{item.instruction}</p></div><span className="status-pill">{item.status}</span></div><div className="support-actions">{item.status === "proposed" ? <button className="primary-button compact" onClick={() => void setRuleStatus(item, "active")}>Activate</button> : null}{item.status === "active" ? <button className="quiet-button compact" onClick={() => void setRuleStatus(item, "disabled")}>Disable</button> : null}{item.status === "disabled" ? <button className="secondary-button compact" onClick={() => void setRuleStatus(item, "active")}>Enable</button> : null}{item.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setRuleStatus(item, "retired")}>Retire</button> : null}</div></article>)}</div>
      </section>

      {busy ? <p className="voice-rule">{busy}</p> : null}
      {notice ? <div className="feedback notice" role="status">{notice}</div> : null}
      {error ? <div className="feedback error" role="alert">{error}</div> : null}
    </div>
  );
}
''',
)


# Voice-specific layout additions.
path = "src/voice.css"
text = read(path)
if ".voice-concept-grid" not in text:
    text += r'''

.voice-concept-grid {
  display: grid;
  gap: 0.8rem;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin-top: 1rem;
}

.voice-model-card {
  border: 1px solid var(--border-color, #d8d8d8);
  border-radius: 0.8rem;
  padding: 1rem;
  background: var(--panel-background, #fff);
}

.voice-inline-form,
.voice-form-grid,
.voice-trait-editor {
  display: grid;
  gap: 0.7rem;
  margin-top: 1rem;
}

.voice-inline-form {
  grid-template-columns: minmax(0, 1fr) auto;
}

.voice-trait-list {
  display: grid;
  gap: 0.7rem;
  margin-top: 0.8rem;
}

.voice-trait {
  border-top: 1px solid var(--border-color, #d8d8d8);
  display: flex;
  gap: 1rem;
  justify-content: space-between;
  padding-top: 0.75rem;
}

.voice-evidence-picker {
  border: 1px solid var(--border-color, #d8d8d8);
  display: grid;
  gap: 0.45rem;
  padding: 0.75rem;
}

.voice-evidence-picker label {
  align-items: center;
  display: flex;
  gap: 0.45rem;
}

@media (max-width: 720px) {
  .voice-inline-form {
    grid-template-columns: 1fr;
  }

  .voice-trait {
    display: grid;
  }
}
'''
write(path, text)
