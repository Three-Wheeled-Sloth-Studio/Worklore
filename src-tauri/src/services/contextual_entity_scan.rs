use std::{collections::HashSet, fs, path::Path};

use chrono::Utc;
use regex::Regex;
use uuid::Uuid;

use crate::{
    domain::models::{
        EntityAlias, EntityOccurrence, EntityReviewItem, EntitySensitivity, EntityStatus,
        EntityType, PrivateEntity, PrivacyScanStatus, ReviewCandidateMatch, ReviewScores,
    },
    error::ServiceResult,
    io_utils::{read_json, write_json_atomic},
    services::entity_scan::{load_registry, save_registry, ScanOutcome},
};

const SCAN_VERSION: &str = "contextual-project-scan-v1";
const MIN_CANDIDATE_SCORE: f32 = 0.65;

#[derive(Debug)]
struct NamedDetection {
    text: String,
    normalized: String,
    start: usize,
    end: usize,
    context_start: usize,
    context_end: usize,
}

pub fn scan_named_projects(
    vault_path: &Path,
    record_type: &str,
    record_id: &str,
    text: &str,
) -> ServiceResult<ScanOutcome> {
    let mut registry = load_registry(vault_path)?;
    let now = Utc::now().to_rfc3339();
    let mut review_item_ids = Vec::new();
    let existing_review_locators = existing_review_locators(vault_path, record_id)?;

    for detection in detect_named_projects(text) {
        let locator = format!("chars:{}-{}", detection.start, detection.end);
        if existing_review_locators.contains(&locator)
            || occurrence_already_exists(&registry.entities, record_id, &locator)
        {
            continue;
        }

        if let Some(index) = exact_entity_index(&registry.entities, &detection.normalized) {
            add_occurrence(
                &mut registry.entities[index],
                record_type,
                record_id,
                &locator,
                &detection.text,
                &now,
                1.0,
            );
            continue;
        }

        let context = text
            .get(detection.context_start..detection.context_end)
            .unwrap_or_default();
        let candidates = ranked_candidates(&registry.entities, context);
        let entity_id = create_provisional_project(
            &mut registry,
            record_type,
            record_id,
            &locator,
            &detection,
            &now,
        );
        let review_item = EntityReviewItem {
            schema_version: 1,
            review_item_id: format!("review_{}", Uuid::now_v7()),
            record_type: record_type.to_string(),
            record_id: record_id.to_string(),
            locator,
            context_excerpt: context.to_string(),
            matched_text: detection.text.clone(),
            normalized_text: detection.normalized.clone(),
            suggested_entity_type: "project".to_string(),
            scores: ReviewScores {
                extraction: 0.84,
                type_score: 0.72,
                best_identity_match: candidates
                    .iter()
                    .map(|candidate| candidate.score)
                    .fold(0.0_f32, f32::max),
            },
            candidate_matches: candidates,
            risk: "medium".to_string(),
            status: "pending".to_string(),
            resolution: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        review_item_ids.push(review_item.review_item_id.clone());
        write_json_atomic(
            &vault_path
                .join("privacy/review-items")
                .join(format!("{}.json", review_item.review_item_id)),
            &review_item,
        )?;

        if let Some(entity) = registry
            .entities
            .iter_mut()
            .find(|entity| entity.entity_id == entity_id)
        {
            entity.notes = if review_item.candidate_matches.is_empty() {
                "Detected from a project-name cue and awaiting confirmation.".to_string()
            } else {
                "Detected from a project-name cue with possible existing matches."
                    .to_string()
            };
        }
    }

    registry.updated_at = now.clone();
    registry.revision += 1;
    save_registry(vault_path, &registry)?;

    Ok(ScanOutcome {
        status: if review_item_ids.is_empty() {
            PrivacyScanStatus::Complete
        } else {
            PrivacyScanStatus::NeedsReview
        },
        review_item_ids,
        scan_version: SCAN_VERSION.to_string(),
        scanned_at: now,
    })
}

fn detect_named_projects(text: &str) -> Vec<NamedDetection> {
    let patterns = [
        Regex::new(
            r"(?i:(?:development|implementation|launch|creation|rollout) of)\s+(?:the\s+)?(?P<name>[A-Z][A-Za-z0-9&.'-]{2,})",
        )
        .expect("development cue regex"),
        Regex::new(
            r"(?i:(?:built|created|launched|introduced|implemented|named|called))\s+(?:a\s+|an\s+|the\s+)?(?P<name>[A-Z][A-Za-z0-9&.'-]{2,})",
        )
        .expect("verb cue regex"),
        Regex::new(
            r"(?i:(?:project|product|platform|system|tool))\s+(?:(?:called|named)\s+)?(?P<name>[A-Z][A-Za-z0-9&.'-]{2,})",
        )
        .expect("noun cue regex"),
    ];

    let mut results = Vec::new();
    let mut seen = HashSet::new();
    for pattern in patterns {
        for captures in pattern.captures_iter(text) {
            let Some(matched) = captures.name("name") else {
                continue;
            };
            if is_generic_name(matched.as_str()) {
                continue;
            }
            let key = (matched.start(), matched.end());
            if !seen.insert(key) {
                continue;
            }
            results.push(NamedDetection {
                text: matched.as_str().to_string(),
                normalized: normalize(matched.as_str()),
                start: matched.start(),
                end: matched.end(),
                context_start: matched.start().saturating_sub(140),
                context_end: (matched.end() + 180).min(text.len()),
            });
        }
    }
    results.sort_by_key(|item| item.start);
    results
}

fn ranked_candidates(entities: &[PrivateEntity], context: &str) -> Vec<ReviewCandidateMatch> {
    let context_tokens = meaningful_tokens(context);
    let mut candidates = entities
        .iter()
        .filter(|entity| {
            !matches!(entity.status, EntityStatus::Merged | EntityStatus::Archived)
                && matches!(
                    entity.entity_type,
                    EntityType::Project
                        | EntityType::Product
                        | EntityType::System
                        | EntityType::Repository
                        | EntityType::UserDefined
                )
        })
        .filter_map(|entity| {
            let mut entity_text = entity.canonical_name.clone();
            if let Some(description) = &entity.public_description {
                entity_text.push(' ');
                entity_text.push_str(description);
            }
            for alias in &entity.aliases {
                entity_text.push(' ');
                entity_text.push_str(&alias.value);
            }
            let entity_tokens = meaningful_tokens(&entity_text);
            if entity_tokens.is_empty() {
                return None;
            }
            let overlap = entity_tokens.intersection(&context_tokens).count();
            let coverage = overlap as f32 / entity_tokens.len() as f32;
            if overlap < 2 || coverage < 0.35 {
                return None;
            }

            let score = (0.52 + coverage * 0.35 + (overlap.min(5) as f32 * 0.02)).min(0.89);
            if score < MIN_CANDIDATE_SCORE {
                return None;
            }
            Some(ReviewCandidateMatch {
                entity_id: entity.entity_id.clone(),
                score,
                reasons: vec![format!(
                    "The surrounding text shares {overlap} meaningful terms with this entity."
                )],
            })
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(4);
    candidates
}

fn meaningful_tokens(value: &str) -> HashSet<String> {
    const STOP_WORDS: &[&str] = &[
        "the", "and", "for", "with", "from", "into", "used", "use", "that", "this",
        "was", "were", "are", "an", "a", "of", "to", "in", "on", "by", "before",
        "after", "project", "product", "system",
    ];

    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(str::to_ascii_lowercase)
        .filter(|token| token.len() >= 3 && !STOP_WORDS.contains(&token.as_str()))
        .collect()
}

fn create_provisional_project(
    registry: &mut crate::domain::models::PrivateEntityRegistry,
    record_type: &str,
    record_id: &str,
    locator: &str,
    detection: &NamedDetection,
    now: &str,
) -> String {
    let entity_id = format!("entity_{}", Uuid::now_v7());
    let family = EntityType::Project.token_family().to_string();
    let counter = registry.token_counters.entry(family.clone()).or_insert(0);
    *counter += 1;

    registry.entities.push(PrivateEntity {
        schema_version: 1,
        entity_id: entity_id.clone(),
        entity_type: EntityType::Project,
        canonical_name: detection.text.clone(),
        public_token: format!("[{family}_{counter}]"),
        public_description: None,
        sensitivity: EntitySensitivity::Private,
        status: EntityStatus::Provisional,
        aliases: vec![EntityAlias {
            value: detection.text.clone(),
            normalized_value: detection.normalized.clone(),
            status: "inferred".to_string(),
            source: "scan".to_string(),
        }],
        relationships: Vec::new(),
        occurrences: vec![EntityOccurrence {
            occurrence_id: format!("occurrence_{}", Uuid::now_v7()),
            record_type: record_type.to_string(),
            record_id: record_id.to_string(),
            locator: locator.to_string(),
            matched_text: detection.text.clone(),
            extraction_confidence: 0.84,
            type_confidence: 0.72,
            identity_match_confidence: 0.0,
            first_seen_at: now.to_string(),
            last_seen_at: now.to_string(),
        }],
        redirect_to_entity_id: None,
        retired_tokens: Vec::new(),
        notes: String::new(),
        created_at: now.to_string(),
        updated_at: now.to_string(),
        revision: 1,
    });
    entity_id
}

fn add_occurrence(
    entity: &mut PrivateEntity,
    record_type: &str,
    record_id: &str,
    locator: &str,
    matched_text: &str,
    now: &str,
    identity_match_confidence: f32,
) {
    entity.occurrences.push(EntityOccurrence {
        occurrence_id: format!("occurrence_{}", Uuid::now_v7()),
        record_type: record_type.to_string(),
        record_id: record_id.to_string(),
        locator: locator.to_string(),
        matched_text: matched_text.to_string(),
        extraction_confidence: 0.95,
        type_confidence: 0.90,
        identity_match_confidence,
        first_seen_at: now.to_string(),
        last_seen_at: now.to_string(),
    });
    entity.updated_at = now.to_string();
    entity.revision += 1;
}

fn exact_entity_index(entities: &[PrivateEntity], normalized: &str) -> Option<usize> {
    entities.iter().position(|entity| {
        !matches!(entity.status, EntityStatus::Merged | EntityStatus::Archived)
            && (normalize(&entity.canonical_name) == normalized
                || entity.aliases.iter().any(|alias| {
                    alias.status != "rejected" && alias.normalized_value == normalized
                }))
    })
}

fn occurrence_already_exists(
    entities: &[PrivateEntity],
    record_id: &str,
    locator: &str,
) -> bool {
    entities.iter().any(|entity| {
        entity
            .occurrences
            .iter()
            .any(|occurrence| occurrence.record_id == record_id && occurrence.locator == locator)
    })
}

fn existing_review_locators(vault_path: &Path, record_id: &str) -> ServiceResult<HashSet<String>> {
    let directory = vault_path.join("privacy/review-items");
    if !directory.exists() {
        return Ok(HashSet::new());
    }

    let mut locators = HashSet::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let item: EntityReviewItem = read_json(&entry.path())?;
        if item.record_id == record_id {
            locators.insert(item.locator);
        }
    }
    Ok(locators)
}

fn is_generic_name(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "the" | "this" | "that" | "new" | "existing" | "current" | "team" | "user"
    )
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_named_project_after_development_cue() {
        let text = "Led development of Kinections, an LLM-assisted tool used to pre-screen investor complaints.";
        let detections = detect_named_projects(text);
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].text, "Kinections");
    }

    #[test]
    fn lexical_candidate_score_requires_real_overlap() {
        let tokens = meaningful_tokens("LLM-assisted tool to pre-screen investor complaints");
        assert!(tokens.contains("llm"));
        assert!(tokens.contains("complaints"));
        assert!(!tokens.contains("the"));
    }
}
