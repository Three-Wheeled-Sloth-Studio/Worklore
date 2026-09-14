use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Duration,
};

use chrono::Utc;
use rusqlite::{params, Connection, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    domain::{
        edit_learning::{
            DecideEditLearningProposalRequest, EditChangeKind, EditChangeView,
            EditLearningAnalysisView, EditLearningDecisionKind, EditLearningDecisionView,
            EditObservationView, EditPairProvenanceView, RecurringEditPreferenceProposalView,
        },
        posts::{PostRevisionAuthorship, PostRevisionOrigin},
    },
    error::{ServiceResult, WorkLoreError},
    services::canonical_store,
};

const MIN_RECURRING_DISTINCT_POSTS: usize = 2;
const MAX_PROPOSAL_FRAGMENT_CHARS: usize = 120;
const DECISION_RECORD_TYPE: &str = "edit_learning_decision";
const ACCEPTED_EVENT_TYPE: &str = "edit_learning_proposal_accepted";
const REJECTED_EVENT_TYPE: &str = "edit_learning_proposal_rejected";

pub fn analyze_edit_learning(vault_path: &Path) -> ServiceResult<EditLearningAnalysisView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut analysis = derive_raw_analysis(&connection)?;
    let decided_keys = load_decision_keys(&connection)?;
    analysis.proposals.retain(|proposal| {
        let proposal_fingerprint = fingerprint(&proposal.proposal_key);
        let support_fingerprint = fingerprint(&pair_signature(&proposal.supporting_pairs));
        !decided_keys.contains(&(proposal_fingerprint, support_fingerprint))
    });
    Ok(analysis)
}

pub fn decide_edit_learning_proposal(
    vault_path: &Path,
    request: DecideEditLearningProposalRequest,
) -> ServiceResult<EditLearningDecisionView> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    let analysis = derive_raw_analysis(&connection)?;
    let requested_pairs = normalize_pairs(&request.supporting_pairs)?;
    if requested_pairs.is_empty() {
        return Err(WorkLoreError::InvalidVault(
            "An edit-learning decision requires supporting Revision pairs.".to_string(),
        ));
    }
    let requested_signature = pair_signature(&requested_pairs);

    let proposal = analysis
        .proposals
        .into_iter()
        .find(|proposal| {
            proposal.proposal_key == request.proposal_key
                && pair_signature(&proposal.supporting_pairs) == requested_signature
        })
        .ok_or_else(|| {
            WorkLoreError::InvalidVault(
                "The edit-learning proposal is stale, tampered, or no longer reproducible from current Revision history."
                    .to_string(),
            )
        })?;

    let proposal_fingerprint = fingerprint(&proposal.proposal_key);
    let support_fingerprint = fingerprint(&requested_signature);
    if load_decision_keys(&connection)?
        .contains(&(proposal_fingerprint.clone(), support_fingerprint.clone()))
    {
        return Err(WorkLoreError::InvalidVault(
            "This exact edit-learning proposal evidence set already has an explicit decision."
                .to_string(),
        ));
    }

    let decision_id = format!("edit_learning_decision_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    let tx = connection.transaction()?;
    let (resulting_artifact_type, resulting_artifact_id) = match request.decision {
        EditLearningDecisionKind::Accepted => {
            let rule_id = create_proposed_writing_rule_tx(
                &tx,
                &decision_id,
                &proposal,
                &proposal_fingerprint,
                &now,
            )?;
            (Some("writing_rule".to_string()), Some(rule_id))
        }
        EditLearningDecisionKind::Rejected => (None, None),
    };

    let details = DecisionAuditDetails {
        decision: request.decision,
        proposal_fingerprint: proposal_fingerprint.clone(),
        support_fingerprint,
        supporting_pairs: requested_pairs.clone(),
        resulting_artifact_type: resulting_artifact_type.clone(),
        resulting_artifact_id: resulting_artifact_id.clone(),
    };
    audit_tx(
        &tx,
        match request.decision {
            EditLearningDecisionKind::Accepted => ACCEPTED_EVENT_TYPE,
            EditLearningDecisionKind::Rejected => REJECTED_EVENT_TYPE,
        },
        DECISION_RECORD_TYPE,
        &decision_id,
        serde_json::to_value(&details)?,
        &now,
    )?;
    tx.commit()?;

    Ok(EditLearningDecisionView {
        decision_id,
        proposal_fingerprint,
        decision: request.decision,
        supporting_pairs: requested_pairs,
        resulting_artifact_type,
        resulting_artifact_id,
        decided_at: now,
    })
}

pub fn list_edit_learning_decisions(
    vault_path: &Path,
) -> ServiceResult<Vec<EditLearningDecisionView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    load_decisions(&connection)
}

fn derive_raw_analysis(connection: &Connection) -> ServiceResult<EditLearningAnalysisView> {
    let pairs = load_user_edit_pairs(connection)?;
    let mut observations = Vec::new();

    for pair in pairs {
        let Some(change) = diff_text(&pair.parent_text, &pair.child_text) else {
            continue;
        };
        observations.push(EditObservationView {
            post_id: pair.post_id,
            parent_revision_id: pair.parent_revision_id,
            child_revision_id: pair.child_revision_id,
            parent_origin: parse_origin(&pair.parent_origin)?,
            parent_authorship_state: PostRevisionAuthorship::parse(&pair.parent_authorship_state)
                .map_err(WorkLoreError::InvalidVault)?,
            child_origin: parse_origin(&pair.child_origin)?,
            child_authorship_state: PostRevisionAuthorship::parse(&pair.child_authorship_state)
                .map_err(WorkLoreError::InvalidVault)?,
            changes: vec![change],
        });
    }

    let proposals = build_proposals(&observations);
    Ok(EditLearningAnalysisView {
        observations,
        proposals,
    })
}

struct RevisionPairRaw {
    post_id: String,
    parent_revision_id: String,
    child_revision_id: String,
    parent_origin: String,
    parent_authorship_state: String,
    child_origin: String,
    child_authorship_state: String,
    parent_text: String,
    child_text: String,
}

fn load_user_edit_pairs(connection: &Connection) -> ServiceResult<Vec<RevisionPairRaw>> {
    let mut statement = connection.prepare(
        "SELECT child.post_id,parent.revision_id,child.revision_id,
         parent.origin,parent.authorship_state,child.origin,child.authorship_state,
         parent.text_snapshot,child.text_snapshot
         FROM post_revisions child
         JOIN post_revisions parent
           ON parent.revision_id=child.parent_revision_id AND parent.post_id=child.post_id
         WHERE child.origin='user'
           AND child.authorship_state IN ('user_authored','user_edited_model')
         ORDER BY child.post_id ASC,child.sequence ASC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(RevisionPairRaw {
            post_id: row.get(0)?,
            parent_revision_id: row.get(1)?,
            child_revision_id: row.get(2)?,
            parent_origin: row.get(3)?,
            parent_authorship_state: row.get(4)?,
            child_origin: row.get(5)?,
            child_authorship_state: row.get(6)?,
            parent_text: row.get(7)?,
            child_text: row.get(8)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn diff_text(parent: &str, child: &str) -> Option<EditChangeView> {
    let parent_tokens = parent.split_whitespace().collect::<Vec<_>>();
    let child_tokens = child.split_whitespace().collect::<Vec<_>>();

    let mut prefix = 0;
    while prefix < parent_tokens.len()
        && prefix < child_tokens.len()
        && parent_tokens[prefix] == child_tokens[prefix]
    {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix < parent_tokens.len().saturating_sub(prefix)
        && suffix < child_tokens.len().saturating_sub(prefix)
        && parent_tokens[parent_tokens.len() - 1 - suffix]
            == child_tokens[child_tokens.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let parent_end = parent_tokens.len().saturating_sub(suffix);
    let child_end = child_tokens.len().saturating_sub(suffix);
    let removed = parent_tokens[prefix..parent_end].join(" ");
    let added = child_tokens[prefix..child_end].join(" ");

    match (removed.is_empty(), added.is_empty()) {
        (true, true) => None,
        (true, false) => Some(EditChangeView {
            kind: EditChangeKind::Addition,
            removed_text: None,
            added_text: Some(added),
        }),
        (false, true) => Some(EditChangeView {
            kind: EditChangeKind::Removal,
            removed_text: Some(removed),
            added_text: None,
        }),
        (false, false) => Some(EditChangeView {
            kind: EditChangeKind::Replacement,
            removed_text: Some(removed),
            added_text: Some(added),
        }),
    }
}

struct PatternAccumulator {
    kind: EditChangeKind,
    removed_text: Option<String>,
    added_text: Option<String>,
    post_ids: BTreeSet<String>,
    supporting_pairs: Vec<EditPairProvenanceView>,
}

fn build_proposals(
    observations: &[EditObservationView],
) -> Vec<RecurringEditPreferenceProposalView> {
    let mut patterns = BTreeMap::<String, PatternAccumulator>::new();
    for observation in observations {
        for change in &observation.changes {
            let Some(proposal_key) = proposal_key(change) else {
                continue;
            };
            let entry = patterns
                .entry(proposal_key)
                .or_insert_with(|| PatternAccumulator {
                    kind: change.kind,
                    removed_text: change.removed_text.clone(),
                    added_text: change.added_text.clone(),
                    post_ids: BTreeSet::new(),
                    supporting_pairs: Vec::new(),
                });
            entry.post_ids.insert(observation.post_id.clone());
            entry.supporting_pairs.push(EditPairProvenanceView {
                post_id: observation.post_id.clone(),
                parent_revision_id: observation.parent_revision_id.clone(),
                child_revision_id: observation.child_revision_id.clone(),
            });
        }
    }

    patterns
        .into_iter()
        .filter_map(|(proposal_key, pattern)| {
            let distinct_post_count = pattern.post_ids.len();
            if distinct_post_count < MIN_RECURRING_DISTINCT_POSTS {
                return None;
            }
            Some(RecurringEditPreferenceProposalView {
                proposal_key,
                kind: pattern.kind,
                statement: proposal_statement(
                    pattern.kind,
                    pattern.removed_text.as_deref(),
                    pattern.added_text.as_deref(),
                ),
                support_count: pattern.supporting_pairs.len(),
                distinct_post_count,
                supporting_pairs: pattern.supporting_pairs,
            })
        })
        .collect()
}

fn proposal_key(change: &EditChangeView) -> Option<String> {
    let removed = normalize_fragment(change.removed_text.as_deref().unwrap_or(""));
    let added = normalize_fragment(change.added_text.as_deref().unwrap_or(""));
    if removed.chars().count() > MAX_PROPOSAL_FRAGMENT_CHARS
        || added.chars().count() > MAX_PROPOSAL_FRAGMENT_CHARS
    {
        return None;
    }
    Some(format!("{}|{}|{}", change.kind.as_str(), removed, added))
}

fn normalize_fragment(value: &str) -> String {
    value
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

fn proposal_statement(
    kind: EditChangeKind,
    removed_text: Option<&str>,
    added_text: Option<&str>,
) -> String {
    match kind {
        EditChangeKind::Addition => format!(
            "Consider a writing preference to add \"{}\" when this pattern recurs.",
            added_text.unwrap_or_default()
        ),
        EditChangeKind::Removal => format!(
            "Consider a writing preference to remove \"{}\" when this pattern recurs.",
            removed_text.unwrap_or_default()
        ),
        EditChangeKind::Replacement => format!(
            "Consider preferring \"{}\" over \"{}\" when this pattern recurs.",
            added_text.unwrap_or_default(),
            removed_text.unwrap_or_default()
        ),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DecisionAuditDetails {
    decision: EditLearningDecisionKind,
    proposal_fingerprint: String,
    support_fingerprint: String,
    supporting_pairs: Vec<EditPairProvenanceView>,
    resulting_artifact_type: Option<String>,
    resulting_artifact_id: Option<String>,
}

fn load_decisions(connection: &Connection) -> ServiceResult<Vec<EditLearningDecisionView>> {
    let mut statement = connection.prepare(
        "SELECT audit_id,details_json,occurred_at
         FROM audit_events
         WHERE record_type=?1 AND event_type IN (?2,?3)
         ORDER BY occurred_at DESC,audit_id DESC",
    )?;
    let rows = statement
        .query_map(
            params![DECISION_RECORD_TYPE, ACCEPTED_EVENT_TYPE, REJECTED_EVENT_TYPE],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    rows.into_iter()
        .map(|(decision_id, details_json, decided_at)| {
            let details: DecisionAuditDetails = serde_json::from_str(&details_json)?;
            Ok(EditLearningDecisionView {
                decision_id,
                proposal_fingerprint: details.proposal_fingerprint,
                decision: details.decision,
                supporting_pairs: details.supporting_pairs,
                resulting_artifact_type: details.resulting_artifact_type,
                resulting_artifact_id: details.resulting_artifact_id,
                decided_at,
            })
        })
        .collect()
}

fn load_decision_keys(connection: &Connection) -> ServiceResult<BTreeSet<(String, String)>> {
    let mut statement = connection.prepare(
        "SELECT details_json
         FROM audit_events
         WHERE record_type=?1 AND event_type IN (?2,?3)",
    )?;
    let rows = statement
        .query_map(
            params![DECISION_RECORD_TYPE, ACCEPTED_EVENT_TYPE, REJECTED_EVENT_TYPE],
            |row| row.get::<_, String>(0),
        )?
        .collect::<Result<Vec<_>, _>>()?;
    rows.into_iter()
        .map(|details_json| {
            let details: DecisionAuditDetails = serde_json::from_str(&details_json)?;
            Ok((details.proposal_fingerprint, details.support_fingerprint))
        })
        .collect()
}

fn normalize_pairs(
    pairs: &[EditPairProvenanceView],
) -> ServiceResult<Vec<EditPairProvenanceView>> {
    let mut normalized = BTreeMap::<String, EditPairProvenanceView>::new();
    for pair in pairs {
        let key = pair_key(pair);
        if normalized.insert(key, pair.clone()).is_some() {
            return Err(WorkLoreError::InvalidVault(
                "Edit-learning supporting Revision pairs must be unique.".to_string(),
            ));
        }
    }
    Ok(normalized.into_values().collect())
}

fn pair_signature(pairs: &[EditPairProvenanceView]) -> String {
    let mut keys = pairs.iter().map(pair_key).collect::<Vec<_>>();
    keys.sort();
    keys.join("\n")
}

fn pair_key(pair: &EditPairProvenanceView) -> String {
    format!(
        "{}\u{1f}{}\u{1f}{}",
        pair.post_id, pair.parent_revision_id, pair.child_revision_id
    )
}

fn fingerprint(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

fn create_proposed_writing_rule_tx(
    tx: &Transaction<'_>,
    decision_id: &str,
    proposal: &RecurringEditPreferenceProposalView,
    proposal_fingerprint: &str,
    now: &str,
) -> ServiceResult<String> {
    let rule_id = format!("rule_{}", Uuid::now_v7());
    tx.execute(
        "INSERT INTO writing_rules(rule_id,name,instruction,status,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,'proposed',?4,?5,?5,1)",
        params![
            &rule_id,
            "Recurring edit preference",
            &proposal.statement,
            json!({
                "creationActor":"user",
                "source":"edit_learning_decision",
                "decisionId":decision_id,
                "proposalFingerprint":proposal_fingerprint,
                "supportingPairCount":proposal.supporting_pairs.len()
            })
            .to_string(),
            now
        ],
    )?;
    audit_tx(
        tx,
        "writing_rule_created",
        "writing_rule",
        &rule_id,
        json!({
            "status":"proposed",
            "source":"edit_learning_decision",
            "decisionId":decision_id
        }),
        now,
    )?;
    Ok(rule_id)
}

fn audit_tx(
    tx: &Transaction<'_>,
    event_type: &str,
    record_type: &str,
    record_id: &str,
    details: serde_json::Value,
    occurred_at: &str,
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
            occurred_at
        ],
    )?;
    Ok(())
}

fn parse_origin(value: &str) -> ServiceResult<PostRevisionOrigin> {
    match value {
        "user" => Ok(PostRevisionOrigin::User),
        "model" => Ok(PostRevisionOrigin::Model),
        _ => Err(WorkLoreError::InvalidVault(format!(
            "Unknown post revision origin {value}."
        ))),
    }
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    let connection = Connection::open(canonical_store::database_path(vault_path))?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::posts::{AppendPostRevisionRequest, CreatePostRequest},
        services::{
            post_lineage_service, vault_service, voice_evidence_service, voice_profile_service,
        },
    };
    use std::fs;

    fn vault() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-edit-learning-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Edit Learning Test").expect("create vault");
        path
    }

    fn create_user_edit_pair(
        path: &Path,
        title: &str,
        parent_text: &str,
        child_text: &str,
    ) -> String {
        let created = post_lineage_service::create_post(
            path,
            CreatePostRequest {
                title: title.to_string(),
                text: parent_text.to_string(),
                origin: PostRevisionOrigin::User,
                authorship_state: PostRevisionAuthorship::UserAuthored,
                provider_run_id: None,
                provider_id: None,
                model_id: None,
            },
        )
        .unwrap();
        post_lineage_service::append_revision(
            path,
            AppendPostRevisionRequest {
                post_id: created.post.post_id.clone(),
                text: child_text.to_string(),
                origin: PostRevisionOrigin::User,
                authorship_state: PostRevisionAuthorship::UserAuthored,
                provider_run_id: None,
                provider_id: None,
                model_id: None,
            },
        )
        .unwrap();
        created.post.post_id
    }

    fn create_model_edit_pair(
        path: &Path,
        title: &str,
        parent_text: &str,
        child_text: &str,
    ) -> String {
        let created = post_lineage_service::create_post(
            path,
            CreatePostRequest {
                title: title.to_string(),
                text: parent_text.to_string(),
                origin: PostRevisionOrigin::Model,
                authorship_state: PostRevisionAuthorship::ModelGenerated,
                provider_run_id: Some(format!("run_{}", Uuid::now_v7())),
                provider_id: Some("synthetic_provider".to_string()),
                model_id: Some("synthetic_model".to_string()),
            },
        )
        .unwrap();
        post_lineage_service::append_revision(
            path,
            AppendPostRevisionRequest {
                post_id: created.post.post_id.clone(),
                text: child_text.to_string(),
                origin: PostRevisionOrigin::User,
                authorship_state: PostRevisionAuthorship::UserEditedModel,
                provider_run_id: None,
                provider_id: None,
                model_id: None,
            },
        )
        .unwrap();
        created.post.post_id
    }

    fn recurring_proposal(path: &Path) -> RecurringEditPreferenceProposalView {
        analyze_edit_learning(path).unwrap().proposals.remove(0)
    }

    #[test]
    fn empty_history_yields_no_observations_or_proposals() {
        let path = vault();
        let analysis = analyze_edit_learning(&path).unwrap();
        assert!(analysis.observations.is_empty());
        assert!(analysis.proposals.is_empty());
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn persisted_edits_expose_addition_removal_and_replacement() {
        let path = vault();
        let addition_post = create_user_edit_pair(
            &path,
            "Addition",
            "Plain sentence.",
            "Plain concise sentence.",
        );
        let removal_post = create_user_edit_pair(
            &path,
            "Removal",
            "Plain very concise sentence.",
            "Plain concise sentence.",
        );
        let replacement_post = create_user_edit_pair(
            &path,
            "Replacement",
            "Plain very concise sentence.",
            "Plain sharply concise sentence.",
        );

        let analysis = analyze_edit_learning(&path).unwrap();
        assert_eq!(analysis.observations.len(), 3);
        assert!(analysis.proposals.is_empty());

        let addition = analysis
            .observations
            .iter()
            .find(|item| item.post_id == addition_post)
            .unwrap();
        assert_eq!(addition.changes[0].kind, EditChangeKind::Addition);
        assert_eq!(addition.changes[0].added_text.as_deref(), Some("concise"));

        let removal = analysis
            .observations
            .iter()
            .find(|item| item.post_id == removal_post)
            .unwrap();
        assert_eq!(removal.changes[0].kind, EditChangeKind::Removal);
        assert_eq!(removal.changes[0].removed_text.as_deref(), Some("very"));

        let replacement = analysis
            .observations
            .iter()
            .find(|item| item.post_id == replacement_post)
            .unwrap();
        assert_eq!(replacement.changes[0].kind, EditChangeKind::Replacement);
        assert_eq!(replacement.changes[0].removed_text.as_deref(), Some("very"));
        assert_eq!(
            replacement.changes[0].added_text.as_deref(),
            Some("sharply")
        );
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn model_text_is_context_not_voice_evidence() {
        let path = vault();
        let created = post_lineage_service::create_post(
            &path,
            CreatePostRequest {
                title: "Model edit".to_string(),
                text: "AI phrase stays context.".to_string(),
                origin: PostRevisionOrigin::Model,
                authorship_state: PostRevisionAuthorship::ModelGenerated,
                provider_run_id: Some("run_synthetic".to_string()),
                provider_id: Some("synthetic_provider".to_string()),
                model_id: Some("synthetic_model".to_string()),
            },
        )
        .unwrap();
        let model_revision_id = created.revisions[0].revision_id.clone();
        let edited = post_lineage_service::append_revision(
            &path,
            AppendPostRevisionRequest {
                post_id: created.post.post_id.clone(),
                text: "Personal phrase stays context.".to_string(),
                origin: PostRevisionOrigin::User,
                authorship_state: PostRevisionAuthorship::UserEditedModel,
                provider_run_id: None,
                provider_id: None,
                model_id: None,
            },
        )
        .unwrap();

        let analysis = analyze_edit_learning(&path).unwrap();
        let observation = analysis
            .observations
            .iter()
            .find(|item| item.post_id == created.post.post_id)
            .unwrap();
        assert_eq!(observation.parent_revision_id, model_revision_id);
        assert_eq!(observation.parent_origin, PostRevisionOrigin::Model);
        assert_eq!(
            observation.parent_authorship_state,
            PostRevisionAuthorship::ModelGenerated
        );
        assert_eq!(observation.child_origin, PostRevisionOrigin::User);
        assert_eq!(
            observation.child_authorship_state,
            PostRevisionAuthorship::UserEditedModel
        );
        assert_eq!(observation.changes[0].kind, EditChangeKind::Replacement);

        let reopened =
            post_lineage_service::get_post_lineage(&path, &created.post.post_id).unwrap();
        assert_eq!(reopened.revisions[0].origin, PostRevisionOrigin::Model);
        assert_eq!(
            reopened.revisions[0].authorship_state,
            PostRevisionAuthorship::ModelGenerated
        );
        assert_eq!(edited.revisions[1], reopened.revisions[1]);
        assert!(voice_evidence_service::list_voice_evidence(&path)
            .unwrap()
            .is_empty());
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn repeated_correction_across_distinct_posts_proposes_review_only_preference() {
        let path = vault();
        let first_post = create_user_edit_pair(
            &path,
            "First",
            "We leverage data carefully.",
            "We use data carefully.",
        );
        let second_post = create_user_edit_pair(
            &path,
            "Second",
            "Teams leverage evidence carefully.",
            "Teams use evidence carefully.",
        );

        let analysis = analyze_edit_learning(&path).unwrap();
        assert_eq!(analysis.proposals.len(), 1);
        let proposal = &analysis.proposals[0];
        assert_eq!(proposal.kind, EditChangeKind::Replacement);
        assert_eq!(proposal.support_count, 2);
        assert_eq!(proposal.distinct_post_count, 2);
        assert!(proposal.statement.contains("use"));
        assert!(proposal.statement.contains("leverage"));
        let supporting_posts = proposal
            .supporting_pairs
            .iter()
            .map(|pair| pair.post_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(supporting_posts.len(), 2);
        assert!(supporting_posts.contains(first_post.as_str()));
        assert!(supporting_posts.contains(second_post.as_str()));
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn analysis_does_not_mutate_governed_voice_state() {
        let path = vault();
        let core_voice = voice_profile_service::create_core_voice(
            &path,
            voice_profile_service::CreateCoreVoiceRequest {
                label: "Synthetic voice".to_string(),
            },
        )
        .unwrap();
        voice_profile_service::save_core_voice_trait(
            &path,
            voice_profile_service::SaveCoreVoiceTraitRequest {
                voice_id: core_voice.voice_id.clone(),
                trait_id: None,
                name: "Direct".to_string(),
                value: "Prefer direct statements.".to_string(),
                user_guidance: Some("Explicit synthetic guidance.".to_string()),
                voice_evidence_ids: Vec::new(),
            },
        )
        .unwrap();
        voice_profile_service::activate_core_voice(&path, &core_voice.voice_id).unwrap();
        voice_profile_service::create_tone_mode(
            &path,
            voice_profile_service::CreateToneModeRequest {
                name: "Analytical".to_string(),
                description: "Synthetic tone.".to_string(),
                instructions: "Stay precise.".to_string(),
            },
        )
        .unwrap();
        voice_profile_service::create_voice_direction(
            &path,
            voice_profile_service::CreateVoiceDirectionRequest {
                statement: "Become more concise.".to_string(),
                rationale: "Synthetic direction.".to_string(),
            },
        )
        .unwrap();
        voice_profile_service::create_writing_rule(
            &path,
            voice_profile_service::CreateWritingRuleRequest {
                name: "No filler".to_string(),
                instruction: "Avoid filler phrases.".to_string(),
            },
        )
        .unwrap();

        let before_core = voice_profile_service::list_core_voices(&path).unwrap();
        let before_tones = voice_profile_service::list_tone_modes(&path).unwrap();
        let before_directions = voice_profile_service::list_voice_directions(&path).unwrap();
        let before_rules = voice_profile_service::list_writing_rules(&path).unwrap();
        let before_evidence_count = voice_evidence_service::list_voice_evidence(&path)
            .unwrap()
            .len();

        create_user_edit_pair(
            &path,
            "First recurring edit",
            "We leverage data carefully.",
            "We use data carefully.",
        );
        create_user_edit_pair(
            &path,
            "Second recurring edit",
            "Teams leverage evidence carefully.",
            "Teams use evidence carefully.",
        );
        let analysis = analyze_edit_learning(&path).unwrap();
        assert_eq!(analysis.proposals.len(), 1);

        assert_eq!(
            voice_profile_service::list_core_voices(&path).unwrap(),
            before_core
        );
        assert_eq!(
            voice_profile_service::list_tone_modes(&path).unwrap(),
            before_tones
        );
        assert_eq!(
            voice_profile_service::list_voice_directions(&path).unwrap(),
            before_directions
        );
        assert_eq!(
            voice_profile_service::list_writing_rules(&path).unwrap(),
            before_rules
        );
        assert_eq!(
            voice_evidence_service::list_voice_evidence(&path)
                .unwrap()
                .len(),
            before_evidence_count
        );
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn accepted_decision_is_explicit_durable_and_creates_only_a_proposed_rule() {
        let path = vault();
        create_user_edit_pair(
            &path,
            "First accepted",
            "We leverage data carefully.",
            "We use data carefully.",
        );
        create_user_edit_pair(
            &path,
            "Second accepted",
            "Teams leverage evidence carefully.",
            "Teams use evidence carefully.",
        );
        assert!(voice_profile_service::list_writing_rules(&path)
            .unwrap()
            .is_empty());

        let proposal = recurring_proposal(&path);
        let decision = decide_edit_learning_proposal(
            &path,
            DecideEditLearningProposalRequest {
                proposal_key: proposal.proposal_key.clone(),
                supporting_pairs: proposal.supporting_pairs.clone(),
                decision: EditLearningDecisionKind::Accepted,
            },
        )
        .unwrap();
        assert_eq!(decision.decision, EditLearningDecisionKind::Accepted);
        assert_eq!(decision.resulting_artifact_type.as_deref(), Some("writing_rule"));
        assert!(decision.resulting_artifact_id.is_some());

        let rules = voice_profile_service::list_writing_rules(&path).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0].status,
            voice_profile_service::WritingRuleStatus::Proposed
        );
        assert_eq!(decision.resulting_artifact_id.as_deref(), Some(rules[0].rule_id.as_str()));
        assert!(voice_profile_service::list_core_voices(&path).unwrap().is_empty());
        assert!(voice_evidence_service::list_voice_evidence(&path)
            .unwrap()
            .is_empty());

        let decisions = list_edit_learning_decisions(&path).unwrap();
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0], decision);
        assert_eq!(decisions[0].supporting_pairs.len(), 2);
        assert!(analyze_edit_learning(&path).unwrap().proposals.is_empty());
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejected_evidence_is_suppressed_until_real_support_expands() {
        let path = vault();
        create_user_edit_pair(
            &path,
            "First rejected",
            "We leverage data carefully.",
            "We use data carefully.",
        );
        create_user_edit_pair(
            &path,
            "Second rejected",
            "Teams leverage evidence carefully.",
            "Teams use evidence carefully.",
        );
        let proposal = recurring_proposal(&path);
        let decision = decide_edit_learning_proposal(
            &path,
            DecideEditLearningProposalRequest {
                proposal_key: proposal.proposal_key,
                supporting_pairs: proposal.supporting_pairs,
                decision: EditLearningDecisionKind::Rejected,
            },
        )
        .unwrap();
        assert_eq!(decision.decision, EditLearningDecisionKind::Rejected);
        assert!(decision.resulting_artifact_id.is_none());
        assert!(voice_profile_service::list_writing_rules(&path)
            .unwrap()
            .is_empty());
        assert!(analyze_edit_learning(&path).unwrap().proposals.is_empty());

        create_user_edit_pair(
            &path,
            "Third support",
            "Leaders leverage judgment carefully.",
            "Leaders use judgment carefully.",
        );
        let renewed = analyze_edit_learning(&path).unwrap();
        assert_eq!(renewed.proposals.len(), 1);
        assert_eq!(renewed.proposals[0].support_count, 3);
        assert_eq!(renewed.proposals[0].distinct_post_count, 3);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn tampered_or_stale_decision_evidence_is_rejected() {
        let path = vault();
        create_user_edit_pair(
            &path,
            "First tamper",
            "We leverage data carefully.",
            "We use data carefully.",
        );
        create_user_edit_pair(
            &path,
            "Second tamper",
            "Teams leverage evidence carefully.",
            "Teams use evidence carefully.",
        );
        let proposal = recurring_proposal(&path);
        let mut tampered_pairs = proposal.supporting_pairs.clone();
        tampered_pairs[0].child_revision_id = "post_revision_tampered".to_string();
        assert!(decide_edit_learning_proposal(
            &path,
            DecideEditLearningProposalRequest {
                proposal_key: proposal.proposal_key.clone(),
                supporting_pairs: tampered_pairs,
                decision: EditLearningDecisionKind::Accepted,
            },
        )
        .is_err());
        assert!(list_edit_learning_decisions(&path).unwrap().is_empty());

        let stale_pairs = proposal.supporting_pairs.clone();
        create_user_edit_pair(
            &path,
            "Third stale",
            "Leaders leverage judgment carefully.",
            "Leaders use judgment carefully.",
        );
        assert!(decide_edit_learning_proposal(
            &path,
            DecideEditLearningProposalRequest {
                proposal_key: proposal.proposal_key,
                supporting_pairs: stale_pairs,
                decision: EditLearningDecisionKind::Rejected,
            },
        )
        .is_err());
        assert!(list_edit_learning_decisions(&path).unwrap().is_empty());
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn accepting_recurring_model_edits_preserves_model_ancestry_and_voice_boundary() {
        let path = vault();
        let first_post = create_model_edit_pair(
            &path,
            "First model",
            "We leverage data carefully.",
            "We use data carefully.",
        );
        let second_post = create_model_edit_pair(
            &path,
            "Second model",
            "Teams leverage evidence carefully.",
            "Teams use evidence carefully.",
        );
        let proposal = recurring_proposal(&path);
        decide_edit_learning_proposal(
            &path,
            DecideEditLearningProposalRequest {
                proposal_key: proposal.proposal_key,
                supporting_pairs: proposal.supporting_pairs,
                decision: EditLearningDecisionKind::Accepted,
            },
        )
        .unwrap();

        for post_id in [first_post, second_post] {
            let lineage = post_lineage_service::get_post_lineage(&path, &post_id).unwrap();
            assert_eq!(lineage.revisions[0].origin, PostRevisionOrigin::Model);
            assert_eq!(
                lineage.revisions[0].authorship_state,
                PostRevisionAuthorship::ModelGenerated
            );
            assert_eq!(
                lineage.revisions[1].authorship_state,
                PostRevisionAuthorship::UserEditedModel
            );
        }
        assert!(voice_evidence_service::list_voice_evidence(&path)
            .unwrap()
            .is_empty());
        fs::remove_dir_all(path).unwrap();
    }
}
