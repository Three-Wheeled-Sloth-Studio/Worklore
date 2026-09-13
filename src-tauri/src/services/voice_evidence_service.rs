use std::{
    fs,
    path::{Component, Path},
    time::Duration,
};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    error::{ServiceResult, WorkLoreError},
    services::canonical_store,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VoiceAuthorshipState {
    Unknown,
    UserAuthored,
    UserEditedModel,
    ModelGenerated,
    ExternalAuthor,
}

impl VoiceAuthorshipState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::UserAuthored => "user_authored",
            Self::UserEditedModel => "user_edited_model",
            Self::ModelGenerated => "model_generated",
            Self::ExternalAuthor => "external_author",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "unknown" => Ok(Self::Unknown),
            "user_authored" => Ok(Self::UserAuthored),
            "user_edited_model" => Ok(Self::UserEditedModel),
            "model_generated" => Ok(Self::ModelGenerated),
            "external_author" => Ok(Self::ExternalAuthor),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Voice Evidence authorship state {value}."
            ))),
        }
    }

    fn is_eligible_authorship(self) -> bool {
        matches!(self, Self::UserAuthored | Self::UserEditedModel)
    }

    fn is_permanently_prohibited(self) -> bool {
        matches!(self, Self::ModelGenerated | Self::ExternalAuthor)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VoiceEvidenceStatus {
    Pending,
    Eligible,
    Rejected,
    Retired,
}

impl VoiceEvidenceStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Eligible => "eligible",
            Self::Rejected => "rejected",
            Self::Retired => "retired",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "eligible" => Ok(Self::Eligible),
            "rejected" => Ok(Self::Rejected),
            "retired" => Ok(Self::Retired),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Voice Evidence status {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VoiceApprovalState {
    Unreviewed,
    Approved,
    Rejected,
    Revoked,
}

impl VoiceApprovalState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Unreviewed => "unreviewed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
        }
    }

    fn parse(value: &str) -> ServiceResult<Self> {
        match value {
            "unreviewed" => Ok(Self::Unreviewed),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "revoked" => Ok(Self::Revoked),
            _ => Err(WorkLoreError::InvalidVault(format!(
                "Unknown Voice Evidence approval state {value}."
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VoiceEvidenceDecision {
    Approve,
    Reject,
    Retire,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VoiceEvidenceRecordView {
    pub voice_evidence_id: String,
    pub source_id: String,
    pub source_display_name: String,
    pub source_origin: String,
    pub text_preview: String,
    pub authorship_state: VoiceAuthorshipState,
    pub status: VoiceEvidenceStatus,
    pub eligibility_reason: String,
    pub approval_state: VoiceApprovalState,
    pub approved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSourceCandidateView {
    pub source_id: String,
    pub display_name: String,
    pub source_origin: String,
    pub text_preview: String,
    pub voice_evidence_id: Option<String>,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVoiceEvidenceResult {
    pub voice_evidence: VoiceEvidenceRecordView,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewVoiceEvidenceRequest {
    pub voice_evidence_id: String,
    pub authorship_state: VoiceAuthorshipState,
    pub decision: VoiceEvidenceDecision,
}

#[derive(Debug, Clone)]
struct SourceRecord {
    source_id: String,
    source_type: String,
    display_name: String,
    source_origin: String,
    captured_text: String,
    extraction_json: String,
}

pub fn list_voice_source_candidates(
    vault_path: &Path,
) -> ServiceResult<Vec<VoiceSourceCandidateView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut statement = connection.prepare(
        "SELECT source_id,source_type,display_name,source_origin,COALESCE(captured_text,''),extraction_json
         FROM sources WHERE source_type='writing_sample' AND lifecycle_status='active'
         ORDER BY imported_at DESC,source_id",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(SourceRecord {
            source_id: row.get(0)?,
            source_type: row.get(1)?,
            display_name: row.get(2)?,
            source_origin: row.get(3)?,
            captured_text: row.get(4)?,
            extraction_json: row.get(5)?,
        })
    })?;
    let sources = rows.collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(sources.len());
    for source in sources {
        let voice_evidence_id = connection
            .query_row(
                "SELECT voice_evidence_id FROM voice_evidence
                 WHERE source_id=?1 AND source_locator='full_source' LIMIT 1",
                [&source.source_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let target_context = is_target_context_source(&connection, &source.source_id)?;
        let text = load_source_text(vault_path, &source).unwrap_or_default();
        let blocked_reason = if target_context {
            Some("target_context_source_prohibited".to_string())
        } else if text.trim().is_empty() {
            Some("source_text_unavailable".to_string())
        } else {
            None
        };
        out.push(VoiceSourceCandidateView {
            source_id: source.source_id,
            display_name: source.display_name,
            source_origin: source.source_origin,
            text_preview: preview(&text, 360),
            voice_evidence_id,
            blocked_reason,
        });
    }
    Ok(out)
}

pub fn create_voice_evidence_from_source(
    vault_path: &Path,
    source_id: &str,
) -> ServiceResult<CreateVoiceEvidenceResult> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    let source = load_source(&connection, source_id)?.ok_or(WorkLoreError::SourceNotFound)?;
    if source.source_type != "writing_sample" {
        return Err(WorkLoreError::SourceNotReady(
            "Only a Source explicitly typed as a writing sample can enter Voice Evidence review."
                .to_string(),
        ));
    }
    if is_target_context_source(&connection, source_id)? {
        return Err(WorkLoreError::SourceNotReady(
            "Target Context is external opportunity/audience context and cannot become Voice Evidence."
                .to_string(),
        ));
    }
    if let Some(existing_id) = connection
        .query_row(
            "SELECT voice_evidence_id FROM voice_evidence
             WHERE source_id=?1 AND source_locator='full_source' LIMIT 1",
            [source_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(CreateVoiceEvidenceResult {
            voice_evidence: load_from_connection(&connection, &existing_id)?,
            created: false,
        });
    }

    let text = load_source_text(vault_path, &source)?;
    if text.trim().is_empty() {
        return Err(WorkLoreError::SourceNotReady(
            "This writing sample has no attributable text available for Voice Evidence review."
                .to_string(),
        ));
    }

    let voice_evidence_id = format!("voice_evidence_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    let tx = connection.transaction()?;
    tx.execute(
        "INSERT INTO voice_evidence(
            voice_evidence_id,source_id,source_locator,text_snapshot,content_hash,authorship_state,status,
            eligibility_reason,approval_state,approved_at,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,'full_source',?3,?4,'unknown','pending','authorship_and_approval_required',
                 'unreviewed',NULL,?5,?6,?6,1)",
        params![
            &voice_evidence_id,
            source_id,
            text,
            hash_text(&text),
            json!({
                "creationActor": "user",
                "sourceId": source_id,
                "sourceRole": "writing_sample",
                "voiceTraining": "pending"
            })
            .to_string(),
            &now
        ],
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO record_relationships(
            relationship_id,from_type,from_id,relationship_type,to_type,to_id,provenance_json,created_at)
         VALUES (?1,'voice_evidence',?2,'voice_evidence_source','source',?3,?4,?5)",
        params![
            format!("relationship_{}", Uuid::now_v7()),
            &voice_evidence_id,
            source_id,
            json!({"creationActor":"user"}).to_string(),
            &now
        ],
    )?;
    audit_tx(
        &tx,
        "voice_evidence_candidate_created",
        &voice_evidence_id,
        json!({"sourceId":source_id,"status":"pending"}),
        &now,
    )?;
    tx.commit()?;

    Ok(CreateVoiceEvidenceResult {
        voice_evidence: load_from_connection(&connection, &voice_evidence_id)?,
        created: true,
    })
}

pub fn load_voice_evidence(
    vault_path: &Path,
    voice_evidence_id: &str,
) -> ServiceResult<VoiceEvidenceRecordView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    load_from_connection(&connection, voice_evidence_id)
}

pub fn list_voice_evidence(vault_path: &Path) -> ServiceResult<Vec<VoiceEvidenceRecordView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut statement = connection.prepare(
        "SELECT voice_evidence_id FROM voice_evidence ORDER BY updated_at DESC,created_at DESC",
    )?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|id| load_from_connection(&connection, &id))
        .collect()
}

pub fn review_voice_evidence(
    vault_path: &Path,
    request: ReviewVoiceEvidenceRequest,
) -> ServiceResult<VoiceEvidenceRecordView> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    let (existing_authorship_text, existing_status_text, existing_revision): (String, String, u32) =
        tx.query_row(
            "SELECT authorship_state,status,revision FROM voice_evidence WHERE voice_evidence_id=?1",
            [&request.voice_evidence_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| voice_evidence_not_found(&request.voice_evidence_id))?;

    let existing_authorship = VoiceAuthorshipState::parse(&existing_authorship_text)?;
    let existing_status = VoiceEvidenceStatus::parse(&existing_status_text)?;
    if existing_status == VoiceEvidenceStatus::Retired {
        return Err(WorkLoreError::InvalidVault(
            "Retired Voice Evidence cannot be reactivated through the normal review path."
                .to_string(),
        ));
    }
    if existing_authorship != VoiceAuthorshipState::Unknown
        && existing_authorship != request.authorship_state
    {
        return Err(WorkLoreError::InvalidVault(
            "Voice Evidence authorship provenance is immutable once explicitly asserted. Retire the record rather than rewriting its origin."
                .to_string(),
        ));
    }
    let authorship = if existing_authorship == VoiceAuthorshipState::Unknown {
        request.authorship_state
    } else {
        existing_authorship
    };

    let now = Utc::now().to_rfc3339();
    let (next_status, reason, approval, approved_at) = match request.decision {
        VoiceEvidenceDecision::Approve => {
            if authorship == VoiceAuthorshipState::Unknown {
                return Err(WorkLoreError::InvalidVault(
                    "Authorship must be explicitly established before Voice Evidence can become eligible."
                        .to_string(),
                ));
            }
            if authorship.is_permanently_prohibited() {
                let reason = if authorship == VoiceAuthorshipState::ModelGenerated {
                    "raw_model_output_prohibited"
                } else {
                    "external_author_prohibited"
                };
                (
                    VoiceEvidenceStatus::Rejected,
                    reason,
                    VoiceApprovalState::Rejected,
                    None,
                )
            } else if authorship.is_eligible_authorship() {
                let reason = if authorship == VoiceAuthorshipState::UserAuthored {
                    "user_authored_and_approved"
                } else {
                    "user_edited_model_and_approved"
                };
                (
                    VoiceEvidenceStatus::Eligible,
                    reason,
                    VoiceApprovalState::Approved,
                    Some(now.clone()),
                )
            } else {
                unreachable!("all VoiceAuthorshipState variants are handled")
            }
        }
        VoiceEvidenceDecision::Reject => {
            let reason = match authorship {
                VoiceAuthorshipState::ModelGenerated => "raw_model_output_prohibited",
                VoiceAuthorshipState::ExternalAuthor => "external_author_prohibited",
                _ => "user_rejected",
            };
            (
                VoiceEvidenceStatus::Rejected,
                reason,
                VoiceApprovalState::Rejected,
                None,
            )
        }
        VoiceEvidenceDecision::Retire => (
            VoiceEvidenceStatus::Retired,
            "user_retired",
            VoiceApprovalState::Revoked,
            None,
        ),
    };

    let next_revision = existing_revision.saturating_add(1);
    tx.execute(
        "UPDATE voice_evidence
         SET authorship_state=?2,status=?3,eligibility_reason=?4,approval_state=?5,approved_at=?6,
             updated_at=?7,revision=?8
         WHERE voice_evidence_id=?1",
        params![
            &request.voice_evidence_id,
            authorship.as_str(),
            next_status.as_str(),
            reason,
            approval.as_str(),
            approved_at,
            &now,
            next_revision
        ],
    )?;
    audit_tx(
        &tx,
        "voice_evidence_eligibility_changed",
        &request.voice_evidence_id,
        json!({
            "beforeStatus": existing_status.as_str(),
            "afterStatus": next_status.as_str(),
            "authorshipState": authorship.as_str(),
            "decision": request.decision,
            "eligibilityReason": reason,
            "revision": next_revision
        }),
        &now,
    )?;
    tx.commit()?;
    load_from_connection(&connection, &request.voice_evidence_id)
}

fn load_from_connection(
    connection: &Connection,
    voice_evidence_id: &str,
) -> ServiceResult<VoiceEvidenceRecordView> {
    let raw = connection
        .query_row(
            "SELECT v.voice_evidence_id,v.source_id,s.display_name,s.source_origin,v.text_snapshot,
                    v.authorship_state,v.status,v.eligibility_reason,v.approval_state,v.approved_at,
                    v.created_at,v.updated_at,v.revision
             FROM voice_evidence v JOIN sources s ON s.source_id=v.source_id
             WHERE v.voice_evidence_id=?1",
            [voice_evidence_id],
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
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, u32>(12)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| voice_evidence_not_found(voice_evidence_id))?;
    Ok(VoiceEvidenceRecordView {
        voice_evidence_id: raw.0,
        source_id: raw.1,
        source_display_name: raw.2,
        source_origin: raw.3,
        text_preview: preview(&raw.4, 700),
        authorship_state: VoiceAuthorshipState::parse(&raw.5)?,
        status: VoiceEvidenceStatus::parse(&raw.6)?,
        eligibility_reason: raw.7,
        approval_state: VoiceApprovalState::parse(&raw.8)?,
        approved_at: raw.9,
        created_at: raw.10,
        updated_at: raw.11,
        revision: raw.12,
    })
}

fn load_source(connection: &Connection, source_id: &str) -> ServiceResult<Option<SourceRecord>> {
    connection
        .query_row(
            "SELECT source_id,source_type,display_name,source_origin,COALESCE(captured_text,''),extraction_json
             FROM sources WHERE source_id=?1",
            [source_id],
            |row| {
                Ok(SourceRecord {
                    source_id: row.get(0)?,
                    source_type: row.get(1)?,
                    display_name: row.get(2)?,
                    source_origin: row.get(3)?,
                    captured_text: row.get(4)?,
                    extraction_json: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(WorkLoreError::from)
}

fn is_target_context_source(connection: &Connection, source_id: &str) -> ServiceResult<bool> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM target_contexts WHERE source_id=?1)",
            [source_id],
            |row| row.get(0),
        )
        .map_err(WorkLoreError::from)
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

fn hash_text(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

fn preview(text: &str, max_chars: usize) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= max_chars {
        compact
    } else {
        let mut out = compact
            .chars()
            .take(max_chars.saturating_sub(3))
            .collect::<String>();
        out.push_str("...");
        out
    }
}

fn audit_tx(
    tx: &Transaction<'_>,
    event_type: &str,
    record_id: &str,
    details: Value,
    occurred_at: &str,
) -> ServiceResult<()> {
    tx.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,?2,'voice_evidence',?3,'user',?4,?5)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            event_type,
            record_id,
            details.to_string(),
            occurred_at
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

fn voice_evidence_not_found(id: &str) -> WorkLoreError {
    WorkLoreError::InvalidVault(format!("Voice Evidence {id} was not found."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::models::SourceType,
        services::{capture_service, vault_service},
    };

    fn vault() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-voice-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Voice Test").expect("create vault");
        path
    }

    fn writing_source(path: &Path, text: &str) -> String {
        capture_service::create_capture_source(path, text, SourceType::WritingSample)
            .expect("create writing source")
            .source_id
    }

    #[test]
    fn writing_sample_starts_pending_then_survives_explicit_approval_and_reopen() {
        let path = vault();
        let source_id = writing_source(
            &path,
            "I write with a direct, specific cadence and concrete examples.",
        );
        let candidates = list_voice_source_candidates(&path).unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].voice_evidence_id.is_none());
        assert!(candidates[0].blocked_reason.is_none());

        let created = create_voice_evidence_from_source(&path, &source_id).unwrap();
        assert!(created.created);
        assert_eq!(created.voice_evidence.status, VoiceEvidenceStatus::Pending);
        assert_eq!(
            created.voice_evidence.authorship_state,
            VoiceAuthorshipState::Unknown
        );
        assert_eq!(
            created.voice_evidence.approval_state,
            VoiceApprovalState::Unreviewed
        );
        assert_eq!(created.voice_evidence.revision, 1);

        let duplicate = create_voice_evidence_from_source(&path, &source_id).unwrap();
        assert!(!duplicate.created);
        assert_eq!(
            duplicate.voice_evidence.voice_evidence_id,
            created.voice_evidence.voice_evidence_id
        );

        let approved = review_voice_evidence(
            &path,
            ReviewVoiceEvidenceRequest {
                voice_evidence_id: created.voice_evidence.voice_evidence_id.clone(),
                authorship_state: VoiceAuthorshipState::UserAuthored,
                decision: VoiceEvidenceDecision::Approve,
            },
        )
        .unwrap();
        assert_eq!(approved.status, VoiceEvidenceStatus::Eligible);
        assert_eq!(approved.approval_state, VoiceApprovalState::Approved);
        assert!(approved.approved_at.is_some());
        assert_eq!(approved.revision, 2);

        canonical_store::initialize(&path).unwrap();
        let reopened = load_voice_evidence(&path, &approved.voice_evidence_id).unwrap();
        assert_eq!(reopened.voice_evidence_id, approved.voice_evidence_id);
        assert_eq!(reopened.status, VoiceEvidenceStatus::Eligible);
        assert_eq!(
            reopened.authorship_state,
            VoiceAuthorshipState::UserAuthored
        );

        let connection = open_connection(&path).unwrap();
        let audit_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM audit_events WHERE record_type='voice_evidence' AND record_id=?1",
                [&approved.voice_evidence_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(audit_count, 2);
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn raw_model_origin_is_rejected_and_cannot_be_rewritten_as_user_authored() {
        let path = vault();
        let source_id =
            writing_source(&path, "Synthetic provider prose that must not train voice.");
        let created = create_voice_evidence_from_source(&path, &source_id).unwrap();
        let rejected = review_voice_evidence(
            &path,
            ReviewVoiceEvidenceRequest {
                voice_evidence_id: created.voice_evidence.voice_evidence_id.clone(),
                authorship_state: VoiceAuthorshipState::ModelGenerated,
                decision: VoiceEvidenceDecision::Approve,
            },
        )
        .unwrap();
        assert_eq!(rejected.status, VoiceEvidenceStatus::Rejected);
        assert_eq!(rejected.eligibility_reason, "raw_model_output_prohibited");
        assert_eq!(rejected.approval_state, VoiceApprovalState::Rejected);

        let rewrite = review_voice_evidence(
            &path,
            ReviewVoiceEvidenceRequest {
                voice_evidence_id: rejected.voice_evidence_id.clone(),
                authorship_state: VoiceAuthorshipState::UserAuthored,
                decision: VoiceEvidenceDecision::Approve,
            },
        );
        assert!(rewrite.is_err());
        let unchanged = load_voice_evidence(&path, &rejected.voice_evidence_id).unwrap();
        assert_eq!(
            unchanged.authorship_state,
            VoiceAuthorshipState::ModelGenerated
        );
        assert_eq!(unchanged.status, VoiceEvidenceStatus::Rejected);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn inspiration_link_does_not_silently_make_writing_voice_evidence() {
        let path = vault();
        let source_id = writing_source(
            &path,
            "My published essay can inspire a topic and separately teach voice.",
        );
        capture_service::classify_capture_source(
            &path,
            &source_id,
            capture_service::CaptureRole::Inspiration,
        )
        .unwrap();
        let created = create_voice_evidence_from_source(&path, &source_id).unwrap();
        assert_eq!(created.voice_evidence.status, VoiceEvidenceStatus::Pending);
        assert_eq!(
            created.voice_evidence.eligibility_reason,
            "authorship_and_approval_required"
        );
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn target_context_source_is_blocked_from_voice_evidence() {
        let path = vault();
        let source_id = writing_source(
            &path,
            "Audience context should never masquerade as my writing voice.",
        );
        capture_service::classify_capture_source(
            &path,
            &source_id,
            capture_service::CaptureRole::TargetContext,
        )
        .unwrap();
        let candidates = list_voice_source_candidates(&path).unwrap();
        assert_eq!(
            candidates[0].blocked_reason.as_deref(),
            Some("target_context_source_prohibited")
        );
        let result = create_voice_evidence_from_source(&path, &source_id);
        assert!(result.is_err());
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn retiring_eligibility_preserves_source_and_records_revisioned_audit() {
        let path = vault();
        let source_id = writing_source(
            &path,
            "A user-authored sample that later stops representing my voice.",
        );
        let created = create_voice_evidence_from_source(&path, &source_id).unwrap();
        let approved = review_voice_evidence(
            &path,
            ReviewVoiceEvidenceRequest {
                voice_evidence_id: created.voice_evidence.voice_evidence_id.clone(),
                authorship_state: VoiceAuthorshipState::UserAuthored,
                decision: VoiceEvidenceDecision::Approve,
            },
        )
        .unwrap();
        let retired = review_voice_evidence(
            &path,
            ReviewVoiceEvidenceRequest {
                voice_evidence_id: approved.voice_evidence_id.clone(),
                authorship_state: VoiceAuthorshipState::UserAuthored,
                decision: VoiceEvidenceDecision::Retire,
            },
        )
        .unwrap();
        assert_eq!(retired.status, VoiceEvidenceStatus::Retired);
        assert_eq!(retired.approval_state, VoiceApprovalState::Revoked);
        assert_eq!(retired.revision, 3);

        let connection = open_connection(&path).unwrap();
        let source_exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sources WHERE source_id=?1)",
                [&source_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(source_exists);
        let audit_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM audit_events WHERE record_type='voice_evidence' AND record_id=?1",
                [&retired.voice_evidence_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(audit_count, 3);
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }
}
