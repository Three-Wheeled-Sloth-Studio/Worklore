use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Duration,
};

use rusqlite::Connection;

use crate::{
    domain::{
        edit_learning::{
            EditChangeKind, EditChangeView, EditLearningAnalysisView, EditObservationView,
            EditPairProvenanceView, RecurringEditPreferenceProposalView,
        },
        posts::{PostRevisionAuthorship, PostRevisionOrigin},
    },
    error::{ServiceResult, WorkLoreError},
    services::canonical_store,
};

const MIN_RECURRING_DISTINCT_POSTS: usize = 2;
const MAX_PROPOSAL_FRAGMENT_CHARS: usize = 120;

pub fn analyze_edit_learning(vault_path: &Path) -> ServiceResult<EditLearningAnalysisView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let pairs = load_user_edit_pairs(&connection)?;
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

fn build_proposals(observations: &[EditObservationView]) -> Vec<RecurringEditPreferenceProposalView> {
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
    use uuid::Uuid;

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
        assert_eq!(replacement.changes[0].added_text.as_deref(), Some("sharply"));
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
        let before_evidence_count = voice_evidence_service::list_voice_evidence(&path).unwrap().len();

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

        assert_eq!(voice_profile_service::list_core_voices(&path).unwrap(), before_core);
        assert_eq!(voice_profile_service::list_tone_modes(&path).unwrap(), before_tones);
        assert_eq!(
            voice_profile_service::list_voice_directions(&path).unwrap(),
            before_directions
        );
        assert_eq!(voice_profile_service::list_writing_rules(&path).unwrap(), before_rules);
        assert_eq!(
            voice_evidence_service::list_voice_evidence(&path).unwrap().len(),
            before_evidence_count
        );
        fs::remove_dir_all(path).unwrap();
    }
}
