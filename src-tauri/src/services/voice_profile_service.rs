use std::{collections::BTreeSet, path::Path, time::Duration};

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
    let mut statement = connection
        .prepare("SELECT voice_id FROM core_voices ORDER BY version_number DESC,created_at DESC")?;
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

pub fn activate_core_voice(
    vault_path: &Path,
    voice_id: &str,
) -> ServiceResult<CoreVoiceRecordView> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    require_proposed_voice(&tx, voice_id)?;
    validate_voice_for_activation(&tx, voice_id)?;
    let now = Utc::now().to_rfc3339();

    let active_ids = {
        let mut statement = tx.prepare("SELECT voice_id FROM core_voices WHERE status='active'")?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
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
    let rows = statement
        .query_map([], map_tone_mode)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
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
    let rows = statement
        .query_map([], map_voice_direction)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
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
    let rows = statement
        .query_map([], map_writing_rule)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
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
    if existing.status != request.status && !valid_rule_transition(existing.status, request.status)
    {
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
        let rows = statement
            .query_map([voice_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
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
            let rows = statement
                .query_map([&trait_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        };
        if optional_text(guidance.as_deref()).is_none() && statuses.is_empty() {
            return Err(WorkLoreError::InvalidVault(format!(
                "Core Voice trait {trait_id} has no attributable provenance."
            )));
        }
        if let Some((evidence_id, status)) =
            statuses.iter().find(|(_, status)| status != "eligible")
        {
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
        let rows = statement
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
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    rows.into_iter()
        .map(|row| {
            let evidence = {
                let mut statement = connection.prepare(
                    "SELECT ve.voice_evidence_id,ve.status FROM core_voice_trait_evidence cve
                     JOIN voice_evidence ve ON ve.voice_evidence_id=cve.voice_evidence_id
                     WHERE cve.trait_id=?1 ORDER BY ve.voice_evidence_id",
                )?;
                let rows = statement
                    .query_map([&row.0], |evidence_row| {
                        Ok(CoreVoiceTraitEvidenceView {
                            voice_evidence_id: evidence_row.get(0)?,
                            current_status: evidence_row.get(1)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                rows
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
            let provenance_valid =
                provenance_kind != "missing" && invalidated_evidence_ids.is_empty();
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

fn load_writing_rule(
    connection: &Connection,
    rule_id: &str,
) -> ServiceResult<WritingRuleRecordView> {
    connection
        .query_row(
            "SELECT rule_id,name,instruction,status,created_at,updated_at,revision
             FROM writing_rules WHERE rule_id=?1",
            [rule_id],
            map_writing_rule,
        )
        .optional()?
        .ok_or_else(|| {
            WorkLoreError::InvalidVault(format!("Writing Rule {rule_id} was not found."))
        })
}

fn valid_direction_transition(from: VoiceDirectionStatus, to: VoiceDirectionStatus) -> bool {
    matches!(
        (from, to),
        (
            VoiceDirectionStatus::Proposed,
            VoiceDirectionStatus::Accepted
        ) | (
            VoiceDirectionStatus::Proposed,
            VoiceDirectionStatus::Retired
        ) | (
            VoiceDirectionStatus::Accepted,
            VoiceDirectionStatus::Completed
        ) | (
            VoiceDirectionStatus::Accepted,
            VoiceDirectionStatus::Retired
        ) | (
            VoiceDirectionStatus::Completed,
            VoiceDirectionStatus::Retired
        )
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
            voices
                .iter()
                .filter(|voice| voice.status == CoreVoiceStatus::Active)
                .count(),
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
        assert_eq!(
            historical.traits[0].invalidated_evidence_ids,
            vec![evidence]
        );
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
                instruction: "Do not end a post with a question unless the question is genuine."
                    .into(),
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
