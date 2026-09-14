use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::models::{
        CloudIdentifierMode, EntityReviewItem, EntitySensitivity, EntityType, PrivateEntityRegistry,
        VaultDocument,
    },
    error::ServiceResult,
    io_utils::{read_json, write_json_atomic},
    services::{
        entity_review, entity_scan,
        redaction_service::{self, RedactedText},
    },
};

const TRANSIENT_RECORD_ID: &str = "confidentiality_transform_transient";
const KNOWN_ENTITY_WRAPPER_TOLERANCE: usize = 5;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialityTransformRequest {
    pub text: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfidentialityState {
    Ready,
    NeedsReview,
    Blocked,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PublicRepresentationKind {
    PublicDescription,
    StableToken,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfidentialityRiskSource {
    PendingRegistryReview,
    UnregisteredSensitiveText,
    AmbiguousRegistryMatch,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialityReplacement {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub sensitivity: EntitySensitivity,
    pub representation_kind: PublicRepresentationKind,
    pub replacement: String,
    pub occurrence_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialityRisk {
    pub source: ConfidentialityRiskSource,
    pub risk: String,
    pub review_item_id: Option<String>,
    pub entity_type: Option<EntityType>,
    pub locator: Option<String>,
    pub occurrence_count: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialityTransformResult {
    pub original_text: String,
    pub token_redacted_text: String,
    pub public_safe_text: String,
    pub state: ConfidentialityState,
    pub replacements: Vec<ConfidentialityReplacement>,
    pub unresolved_risks: Vec<ConfidentialityRisk>,
    pub registry_revision: u32,
    pub provider_used: bool,
}

#[derive(Debug, Clone, Copy)]
struct TextRange {
    start: usize,
    end: usize,
}

pub fn transform_for_public_use(
    vault_path: &Path,
    request: ConfidentialityTransformRequest,
) -> ServiceResult<ConfidentialityTransformResult> {
    let vault: VaultDocument = read_json(&vault_path.join("vault.json"))?;
    let registry = entity_scan::load_registry(vault_path)?;
    let persistent_reviews = entity_review::list_reviews(vault_path, true)?;

    let transient = TransientPrivacyVault::create(&vault, &registry)?;
    let RedactedText {
        text: token_redacted_text,
        preflight,
    } = redaction_service::redact_for_external_use(&transient.path, &request.text)?;

    let (public_safe_text, replacements) = build_public_safe_text(
        &token_redacted_text,
        &preflight.replacements,
        &registry,
    );

    let mut unresolved_risks = persistent_reviews
        .into_iter()
        .map(|review| ConfidentialityRisk {
            source: ConfidentialityRiskSource::PendingRegistryReview,
            risk: review.risk,
            review_item_id: Some(review.review_item_id),
            entity_type: None,
            locator: None,
            occurrence_count: 1,
            reason: "A pending Private Entity Registry review remains unresolved.".to_string(),
        })
        .collect::<Vec<_>>();

    unresolved_risks.extend(scan_transient_risks(
        &transient.path,
        &registry,
        &request.text,
    )?);

    let persistent_block = vault.privacy.block_cloud_when_high_risk_review_pending
        && unresolved_risks.iter().any(|risk| {
            risk.source == ConfidentialityRiskSource::PendingRegistryReview
                && is_high_risk(&risk.risk)
        });
    let transient_block = unresolved_risks.iter().any(|risk| {
        risk.source != ConfidentialityRiskSource::PendingRegistryReview
            && is_high_risk(&risk.risk)
    });

    let state = if persistent_block || transient_block {
        ConfidentialityState::Blocked
    } else if unresolved_risks.is_empty() {
        ConfidentialityState::Ready
    } else {
        ConfidentialityState::NeedsReview
    };

    Ok(ConfidentialityTransformResult {
        original_text: request.text,
        token_redacted_text,
        public_safe_text,
        state,
        replacements,
        unresolved_risks,
        registry_revision: registry.revision,
        provider_used: false,
    })
}

fn build_public_safe_text(
    token_redacted_text: &str,
    redaction_replacements: &[crate::domain::providers::RedactionReplacement],
    registry: &PrivateEntityRegistry,
) -> (String, Vec<ConfidentialityReplacement>) {
    let mut staged_text = token_redacted_text.to_string();
    let mut staged_replacements = Vec::new();
    let mut replacements = Vec::new();

    for (index, redaction) in redaction_replacements.iter().enumerate() {
        let Some(entity) = registry
            .entities
            .iter()
            .find(|entity| entity.entity_id == redaction.entity_id)
        else {
            continue;
        };

        let public_description = entity
            .public_description
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let (representation_kind, public_representation) = match public_description {
            Some(description) => (
                PublicRepresentationKind::PublicDescription,
                description.to_string(),
            ),
            None => (
                PublicRepresentationKind::StableToken,
                entity.public_token.clone(),
            ),
        };

        let marker = format!("\u{001f}WORKLORE_PUBLIC_{index}_{}\u{001f}", Uuid::now_v7());
        staged_text = staged_text.replace(&entity.public_token, &marker);
        staged_replacements.push((marker, public_representation.clone()));
        replacements.push(ConfidentialityReplacement {
            entity_id: entity.entity_id.clone(),
            entity_type: entity.entity_type,
            sensitivity: entity.sensitivity,
            representation_kind,
            replacement: public_representation,
            occurrence_count: redaction.occurrence_count,
        });
    }

    for (marker, representation) in staged_replacements {
        staged_text = staged_text.replace(&marker, &representation);
    }

    (staged_text, replacements)
}

fn scan_transient_risks(
    transient_vault_path: &Path,
    original_registry: &PrivateEntityRegistry,
    text: &str,
) -> ServiceResult<Vec<ConfidentialityRisk>> {
    let known_ids = original_registry
        .entities
        .iter()
        .map(|entity| entity.entity_id.clone())
        .collect::<HashSet<_>>();

    let outcome = entity_scan::scan_text(
        transient_vault_path,
        "confidentiality_transform",
        TRANSIENT_RECORD_ID,
        text,
    )?;
    let scanned_registry = entity_scan::load_registry(transient_vault_path)?;
    let known_ranges = scanned_registry
        .entities
        .iter()
        .filter(|entity| known_ids.contains(&entity.entity_id))
        .flat_map(|entity| entity.occurrences.iter())
        .filter(|occurrence| occurrence.record_id == TRANSIENT_RECORD_ID)
        .filter_map(|occurrence| parse_char_locator(&occurrence.locator))
        .collect::<Vec<_>>();
    let mut risks = Vec::new();

    for entity in scanned_registry
        .entities
        .iter()
        .filter(|entity| !known_ids.contains(&entity.entity_id))
    {
        let occurrences = entity
            .occurrences
            .iter()
            .filter(|occurrence| occurrence.record_id == TRANSIENT_RECORD_ID)
            .filter(|occurrence| {
                parse_char_locator(&occurrence.locator).is_none_or(|candidate| {
                    !known_ranges
                        .iter()
                        .any(|known| is_known_entity_scanner_wrapper(candidate, *known))
                })
            })
            .collect::<Vec<_>>();
        if occurrences.is_empty() {
            continue;
        }

        risks.push(ConfidentialityRisk {
            source: ConfidentialityRiskSource::UnregisteredSensitiveText,
            risk: transient_risk(entity.entity_type).to_string(),
            review_item_id: None,
            entity_type: Some(entity.entity_type),
            locator: occurrences.first().map(|occurrence| occurrence.locator.clone()),
            occurrence_count: occurrences.len(),
            reason: "Sensitive-looking text was detected but is not represented by a reviewed canonical private entity.".to_string(),
        });
    }

    for review_item_id in outcome.review_item_ids {
        let item: EntityReviewItem = read_json(
            &transient_vault_path
                .join("privacy/review-items")
                .join(format!("{review_item_id}.json")),
        )?;
        risks.push(ConfidentialityRisk {
            source: ConfidentialityRiskSource::AmbiguousRegistryMatch,
            risk: item.risk,
            review_item_id: None,
            entity_type: parse_entity_type(&item.suggested_entity_type),
            locator: Some(item.locator),
            occurrence_count: 1,
            reason: "The supplied text matches more than one registered entity and needs identity review.".to_string(),
        });
    }

    Ok(risks)
}

fn parse_char_locator(locator: &str) -> Option<TextRange> {
    let range = locator.strip_prefix("chars:")?;
    let (start, end) = range.split_once('-')?;
    let start = start.parse::<usize>().ok()?;
    let end = end.parse::<usize>().ok()?;
    (end >= start).then_some(TextRange { start, end })
}

fn is_known_entity_scanner_wrapper(candidate: TextRange, known: TextRange) -> bool {
    if known.start <= candidate.start && candidate.end <= known.end {
        return true;
    }

    if candidate.start <= known.start && known.end <= candidate.end {
        let extra_prefix = known.start.saturating_sub(candidate.start);
        let extra_suffix = candidate.end.saturating_sub(known.end);
        return extra_prefix + extra_suffix <= KNOWN_ENTITY_WRAPPER_TOLERANCE;
    }

    false
}

fn transient_risk(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Email
        | EntityType::Phone
        | EntityType::Account
        | EntityType::Identifier
        | EntityType::Person => "high",
        _ => "medium",
    }
}

fn is_high_risk(risk: &str) -> bool {
    matches!(risk, "high" | "critical")
}

fn parse_entity_type(value: &str) -> Option<EntityType> {
    match value {
        "employer" => Some(EntityType::Employer),
        "client" => Some(EntityType::Client),
        "project" => Some(EntityType::Project),
        "product" => Some(EntityType::Product),
        "system" => Some(EntityType::System),
        "repository" => Some(EntityType::Repository),
        "person" => Some(EntityType::Person),
        "location" => Some(EntityType::Location),
        "email" => Some(EntityType::Email),
        "phone" => Some(EntityType::Phone),
        "url" => Some(EntityType::Url),
        "account" => Some(EntityType::Account),
        "identifier" => Some(EntityType::Identifier),
        "organization" => Some(EntityType::Organization),
        "userdefined" | "user_defined" => Some(EntityType::UserDefined),
        _ => None,
    }
}

struct TransientPrivacyVault {
    path: PathBuf,
}

impl TransientPrivacyVault {
    fn create(vault: &VaultDocument, registry: &PrivateEntityRegistry) -> ServiceResult<Self> {
        let transient = Self {
            path: std::env::temp_dir().join(format!(
                "worklore-confidentiality-{}",
                Uuid::now_v7()
            )),
        };
        fs::create_dir_all(transient.path.join("privacy/review-items"))?;

        let mut transient_vault = vault.clone();
        transient_vault.privacy.cloud_identifier_mode = CloudIdentifierMode::Redact;
        transient_vault.privacy.block_cloud_when_high_risk_review_pending = false;
        write_json_atomic(&transient.path.join("vault.json"), &transient_vault)?;
        entity_scan::save_registry(&transient.path, registry)?;
        Ok(transient)
    }
}

impl Drop for TransientPrivacyVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::models::{
            EntityAlias, EntityStatus, PrivateEntity, TokenRedirect, VaultFeatures, VaultPrivacy,
        },
        io_utils::write_json_atomic,
        services::redaction_service,
    };

    struct TestVault {
        path: PathBuf,
    }

    impl TestVault {
        fn create(
            entities: Vec<PrivateEntity>,
            block_high_risk_review: bool,
            identifier_mode: CloudIdentifierMode,
        ) -> Self {
            let path = std::env::temp_dir().join(format!("worklore-test-{}", Uuid::now_v7()));
            fs::create_dir_all(path.join("privacy/review-items")).expect("create test privacy dir");
            let vault = VaultDocument {
                schema_version: 1,
                vault_id: "vault_test".to_string(),
                name: "Synthetic test vault".to_string(),
                description: String::new(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
                privacy: VaultPrivacy {
                    cloud_identifier_mode: identifier_mode,
                    block_cloud_when_high_risk_review_pending: block_high_risk_review,
                    retain_provider_bodies: false,
                },
                features: VaultFeatures {
                    entity_registry: true,
                    ollama: false,
                    gemini: false,
                    manual_workspace: false,
                },
                metadata: Default::default(),
            };
            write_json_atomic(&path.join("vault.json"), &vault).expect("write test vault");
            let registry = PrivateEntityRegistry {
                schema_version: 1,
                registry_id: "registry_test".to_string(),
                vault_id: "vault_test".to_string(),
                token_counters: Default::default(),
                entities,
                token_redirects: Vec::<TokenRedirect>::new(),
                ignored_terms: Vec::new(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
                revision: 7,
            };
            entity_scan::save_registry(&path, &registry).expect("write test registry");
            Self { path }
        }

        fn add_review(&self, id: &str, risk: &str) {
            let item = EntityReviewItem {
                schema_version: 1,
                review_item_id: id.to_string(),
                record_type: "source".to_string(),
                record_id: "source_test".to_string(),
                locator: "chars:0-4".to_string(),
                context_excerpt: "synthetic private review context".to_string(),
                matched_text: "Synthetic".to_string(),
                normalized_text: "synthetic".to_string(),
                suggested_entity_type: "organization".to_string(),
                scores: crate::domain::models::ReviewScores {
                    extraction: 0.9,
                    type_score: 0.9,
                    best_identity_match: 0.0,
                },
                candidate_matches: Vec::new(),
                risk: risk.to_string(),
                status: "pending".to_string(),
                resolution: None,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            };
            write_json_atomic(
                &self
                    .path
                    .join("privacy/review-items")
                    .join(format!("{id}.json")),
                &item,
            )
            .expect("write review item");
        }
    }

    impl Drop for TestVault {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn entity(
        id: &str,
        entity_type: EntityType,
        canonical_name: &str,
        alias: &str,
        token: &str,
        description: Option<&str>,
        sensitivity: EntitySensitivity,
    ) -> PrivateEntity {
        PrivateEntity {
            schema_version: 1,
            entity_id: id.to_string(),
            entity_type,
            canonical_name: canonical_name.to_string(),
            public_token: token.to_string(),
            public_description: description.map(str::to_string),
            sensitivity,
            status: EntityStatus::Confirmed,
            aliases: vec![EntityAlias {
                value: alias.to_string(),
                normalized_value: alias
                    .chars()
                    .filter(|character| character.is_ascii_alphanumeric())
                    .flat_map(|character| character.to_lowercase())
                    .collect(),
                status: "confirmed".to_string(),
                source: "user".to_string(),
            }],
            relationships: Vec::new(),
            occurrences: Vec::new(),
            redirect_to_entity_id: None,
            retired_tokens: Vec::new(),
            notes: String::new(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            revision: 1,
        }
    }

    #[test]
    fn public_transform_uses_only_stored_descriptions_and_preserves_private_truth() {
        let vault = TestVault::create(
            vec![
                entity(
                    "entity_employer",
                    EntityType::Employer,
                    "Acme Corporation",
                    "Acme",
                    "[EMPLOYER_1]",
                    Some("a regulated services company"),
                    EntitySensitivity::Private,
                ),
                entity(
                    "entity_project",
                    EntityType::Project,
                    "Project Nightjar",
                    "Nightjar",
                    "[PROJECT_1]",
                    None,
                    EntitySensitivity::Private,
                ),
                entity(
                    "entity_public",
                    EntityType::Organization,
                    "Open Standards Agency",
                    "Open Standards Agency",
                    "[ORGANIZATION_1]",
                    None,
                    EntitySensitivity::Public,
                ),
            ],
            true,
            CloudIdentifierMode::Include,
        );
        let before = fs::read_to_string(vault.path.join("privacy/entity-registry.json"))
            .expect("read registry before transform");
        let input = "At Acme Corporation and Acme, I cut cycle time 42% and recovered $15M on Nightjar with Open Standards Agency.";

        let result = transform_for_public_use(
            &vault.path,
            ConfidentialityTransformRequest {
                text: input.to_string(),
            },
        )
        .expect("transform should succeed without a provider");

        assert_eq!(result.state, ConfidentialityState::Ready);
        assert!(!result.provider_used);
        assert_eq!(result.registry_revision, 7);
        assert_eq!(
            result.token_redacted_text,
            "At [EMPLOYER_1] and [EMPLOYER_1], I cut cycle time 42% and recovered $15M on [PROJECT_1] with Open Standards Agency."
        );
        assert_eq!(
            result.public_safe_text,
            "At a regulated services company and a regulated services company, I cut cycle time 42% and recovered $15M on [PROJECT_1] with Open Standards Agency."
        );
        assert_eq!(result.replacements.len(), 2);
        assert_eq!(result.replacements[0].entity_id, "entity_employer");
        assert_eq!(result.replacements[0].occurrence_count, 2);
        assert_eq!(
            result.replacements[0].representation_kind,
            PublicRepresentationKind::PublicDescription
        );
        assert_eq!(
            result.replacements[1].representation_kind,
            PublicRepresentationKind::StableToken
        );
        assert_eq!(result.replacements[1].replacement, "[PROJECT_1]");
        assert!(result.public_safe_text.contains("42%"));
        assert!(result.public_safe_text.contains("$15M"));
        assert!(result.public_safe_text.contains("Open Standards Agency"));
        assert!(!result.public_safe_text.contains("Acme"));
        assert!(!result.public_safe_text.contains("Nightjar"));

        let after = fs::read_to_string(vault.path.join("privacy/entity-registry.json"))
            .expect("read registry after transform");
        assert_eq!(before, after);
    }

    #[test]
    fn high_risk_pending_review_blocks_when_vault_policy_requires_it() {
        let vault = TestVault::create(Vec::new(), true, CloudIdentifierMode::Redact);
        vault.add_review("review_high", "high");
        vault.add_review("review_low", "low");

        let result = transform_for_public_use(
            &vault.path,
            ConfidentialityTransformRequest {
                text: "A generic statement with no private entity.".to_string(),
            },
        )
        .expect("transform should return review state instead of provider error");

        assert_eq!(result.state, ConfidentialityState::Blocked);
        assert_eq!(result.unresolved_risks.len(), 2);
        assert!(result
            .unresolved_risks
            .iter()
            .any(|risk| risk.review_item_id.as_deref() == Some("review_high")));
        assert!(result
            .unresolved_risks
            .iter()
            .any(|risk| risk.review_item_id.as_deref() == Some("review_low")));
    }

    #[test]
    fn lower_risk_pending_review_remains_visible_without_claiming_ready() {
        let vault = TestVault::create(Vec::new(), true, CloudIdentifierMode::Redact);
        vault.add_review("review_medium", "medium");

        let result = transform_for_public_use(
            &vault.path,
            ConfidentialityTransformRequest {
                text: "A generic statement with no private entity.".to_string(),
            },
        )
        .expect("transform should succeed");

        assert_eq!(result.state, ConfidentialityState::NeedsReview);
        assert_eq!(result.unresolved_risks.len(), 1);
        assert_eq!(
            result.unresolved_risks[0].source,
            ConfidentialityRiskSource::PendingRegistryReview
        );
    }

    #[test]
    fn unregistered_high_risk_text_is_blocked_without_mutating_the_registry() {
        let vault = TestVault::create(Vec::new(), false, CloudIdentifierMode::Include);
        let before = fs::read_to_string(vault.path.join("privacy/entity-registry.json"))
            .expect("read registry before transform");

        let result = transform_for_public_use(
            &vault.path,
            ConfidentialityTransformRequest {
                text: "Contact synthetic.person@example.test before publication.".to_string(),
            },
        )
        .expect("transient scan should succeed");

        assert_eq!(result.state, ConfidentialityState::Blocked);
        assert_eq!(result.public_safe_text, result.original_text);
        assert!(result.unresolved_risks.iter().any(|risk| {
            risk.source == ConfidentialityRiskSource::UnregisteredSensitiveText
                && risk.entity_type == Some(EntityType::Email)
                && risk.risk == "high"
        }));

        let after = fs::read_to_string(vault.path.join("privacy/entity-registry.json"))
            .expect("read registry after transform");
        assert_eq!(before, after);
    }

    #[test]
    fn never_send_to_cloud_stays_redacted_on_existing_external_path() {
        let vault = TestVault::create(
            vec![entity(
                "entity_secret",
                EntityType::Client,
                "Secret Client",
                "Client X",
                "[CLIENT_1]",
                Some("a private client"),
                EntitySensitivity::NeverSendToCloud,
            )],
            false,
            CloudIdentifierMode::Include,
        );

        let result = redaction_service::redact_for_external_use(
            &vault.path,
            "Secret Client and Client X remain protected.",
        )
        .expect("external redaction should succeed");

        assert_eq!(
            result.text,
            "[CLIENT_1] and [CLIENT_1] remain protected."
        );
        assert!(result.preflight.contains_never_send_entities);
        assert_eq!(result.preflight.replacement_count, 2);
    }
}
