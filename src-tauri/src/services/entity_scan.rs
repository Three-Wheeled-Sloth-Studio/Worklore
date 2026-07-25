use std::{collections::HashSet, fs, path::Path};

use chrono::Utc;
use regex::Regex;
use uuid::Uuid;

use crate::{
    domain::models::{
        EntityAlias, EntityOccurrence, EntityReviewItem, EntitySensitivity, EntityStatus,
        EntityType, PrivateEntity, PrivateEntityRegistry, PrivacyScanStatus,
        ReviewCandidateMatch, ReviewScores,
    },
    error::ServiceResult,
    io_utils::{read_json, write_json_atomic},
};

const SCAN_VERSION: &str = "deterministic-entity-scan-v1";

#[derive(Debug)]
pub struct ScanOutcome {
    pub status: PrivacyScanStatus,
    pub review_item_ids: Vec<String>,
    pub scan_version: String,
    pub scanned_at: String,
}

#[derive(Debug, Clone)]
struct Detection {
    text: String,
    normalized: String,
    entity_type: EntityType,
    extraction_confidence: f32,
    type_confidence: f32,
    risk: &'static str,
    start: usize,
    end: usize,
}

pub fn initialize_registry(vault_id: &str) -> PrivateEntityRegistry {
    let now = Utc::now().to_rfc3339();
    PrivateEntityRegistry {
        schema_version: 1,
        registry_id: format!("registry_{}", Uuid::now_v7()),
        vault_id: vault_id.to_string(),
        token_counters: Default::default(),
        entities: Vec::new(),
        token_redirects: Vec::new(),
        ignored_terms: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
        revision: 1,
    }
}

pub fn load_registry(vault_path: &Path) -> ServiceResult<PrivateEntityRegistry> {
    read_json(&vault_path.join("privacy/entity-registry.json"))
}

pub fn save_registry(vault_path: &Path, registry: &PrivateEntityRegistry) -> ServiceResult<()> {
    write_json_atomic(&vault_path.join("privacy/entity-registry.json"), registry)
}

pub fn scan_text(
    vault_path: &Path,
    record_type: &str,
    record_id: &str,
    text: &str,
) -> ServiceResult<ScanOutcome> {
    let mut registry = load_registry(vault_path)?;
    let now = Utc::now().to_rfc3339();
    let mut review_item_ids = Vec::new();
    let mut processed = HashSet::new();

    for entity in &mut registry.entities {
        if matches!(entity.status, EntityStatus::Merged | EntityStatus::Archived) {
            continue;
        }

        let aliases = entity
            .aliases
            .iter()
            .filter(|alias| alias.status != "rejected")
            .map(|alias| alias.value.clone())
            .chain(std::iter::once(entity.canonical_name.clone()))
            .collect::<Vec<_>>();

        for alias in aliases {
            for (start, end) in find_case_insensitive(text, &alias) {
                let key = format!("{}:{}:{}", entity.entity_id, start, end);
                if processed.insert(key) {
                    add_occurrence(
                        entity,
                        record_type,
                        record_id,
                        text,
                        start,
                        end,
                        1.0,
                        1.0,
                        1.0,
                        &now,
                    );
                }
            }
        }
    }

    for detection in collect_detections(text)? {
        let detection_key = format!("{}:{}:{}", detection.normalized, detection.start, detection.end);
        if !processed.insert(detection_key) {
            continue;
        }

        if registry
            .ignored_terms
            .iter()
            .any(|ignored| ignored.normalized_value == detection.normalized)
        {
            continue;
        }

        let matching_indexes = registry
            .entities
            .iter()
            .enumerate()
            .filter(|(_, entity)| entity_matches(entity, &detection.normalized))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        match matching_indexes.as_slice() {
            [index] => {
                let entity = &mut registry.entities[*index];
                add_occurrence(
                    entity,
                    record_type,
                    record_id,
                    text,
                    detection.start,
                    detection.end,
                    detection.extraction_confidence,
                    detection.type_confidence,
                    1.0,
                    &now,
                );
            }
            [] => {
                let entity_id = format!("entity_{}", Uuid::now_v7());
                let token = allocate_token(&mut registry, detection.entity_type);
                let occurrence = occurrence_for(
                    record_type,
                    record_id,
                    text,
                    detection.start,
                    detection.end,
                    detection.extraction_confidence,
                    detection.type_confidence,
                    0.0,
                    &now,
                );

                registry.entities.push(PrivateEntity {
                    schema_version: 1,
                    entity_id: entity_id.clone(),
                    entity_type: detection.entity_type,
                    canonical_name: detection.text.clone(),
                    public_token: token,
                    public_description: None,
                    sensitivity: default_sensitivity(detection.entity_type),
                    status: EntityStatus::Provisional,
                    aliases: vec![EntityAlias {
                        value: detection.text.clone(),
                        normalized_value: detection.normalized.clone(),
                        status: "inferred".to_string(),
                        source: "scan".to_string(),
                    }],
                    relationships: Vec::new(),
                    occurrences: vec![occurrence],
                    redirect_to_entity_id: None,
                    retired_tokens: Vec::new(),
                    notes: String::new(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                    revision: 1,
                });

                if detection.entity_type != EntityType::Url {
                    let review_item = new_review_item(
                        record_type,
                        record_id,
                        text,
                        &detection,
                        vec![ReviewCandidateMatch {
                            entity_id,
                            score: 1.0,
                            reasons: vec!["WorkLore created this as a provisional new entity.".to_string()],
                        }],
                        &now,
                    );
                    review_item_ids.push(review_item.review_item_id.clone());
                    write_review_item(vault_path, &review_item)?;
                }
            }
            indexes => {
                let candidates = indexes
                    .iter()
                    .map(|index| ReviewCandidateMatch {
                        entity_id: registry.entities[*index].entity_id.clone(),
                        score: 0.75,
                        reasons: vec!["Multiple existing entities share this normalized alias.".to_string()],
                    })
                    .collect();
                let review_item = new_review_item(
                    record_type,
                    record_id,
                    text,
                    &detection,
                    candidates,
                    &now,
                );
                review_item_ids.push(review_item.review_item_id.clone());
                write_review_item(vault_path, &review_item)?;
            }
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

fn collect_detections(text: &str) -> ServiceResult<Vec<Detection>> {
    let mut detections = Vec::new();

    let patterns: Vec<(Regex, EntityType, f32, f32, &'static str)> = vec![
        (
            Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b").expect("email regex"),
            EntityType::Email,
            0.99,
            0.99,
            "high",
        ),
        (
            Regex::new(r"(?i)\bhttps?://[^\s<>()\[\]{}]+" ).expect("url regex"),
            EntityType::Url,
            0.99,
            0.99,
            "medium",
        ),
        (
            Regex::new(r"(?x)\b(?:\+?1[-.\s]?)?(?:\(?\d{3}\)?[-.\s]?)\d{3}[-.\s]?\d{4}\b")
                .expect("phone regex"),
            EntityType::Phone,
            0.96,
            0.98,
            "high",
        ),
        (
            Regex::new(
                r"\b(?:[A-Z][A-Za-z0-9&.'-]+\s+){0,5}(?:Inc|LLC|Ltd|Corporation|Corp|Company|Association|Authority|Agency|Department|University|Bank|Group)\b\.?","
            )
            .expect("organization suffix regex"),
            EntityType::Organization,
            0.86,
            0.8,
            "medium",
        ),
        (
            Regex::new(r"\b[A-Z][A-Z0-9]{2,9}\b").expect("acronym regex"),
            EntityType::UserDefined,
            0.68,
            0.45,
            "medium",
        ),
    ];

    for (regex, entity_type, extraction, type_confidence, risk) in patterns {
        for matched in regex.find_iter(text) {
            let value = trim_terminal_punctuation(matched.as_str());
            if value.len() < 3 || is_common_acronym(value) {
                continue;
            }
            detections.push(Detection {
                text: value.to_string(),
                normalized: normalize(value),
                entity_type,
                extraction_confidence: extraction,
                type_confidence,
                risk,
                start: matched.start(),
                end: matched.start() + value.len(),
            });
        }
    }

    detections.sort_by_key(|detection| (detection.start, std::cmp::Reverse(detection.end)));
    detections.dedup_by(|left, right| {
        left.start == right.start && left.end == right.end && left.normalized == right.normalized
    });
    Ok(detections)
}

fn is_common_acronym(value: &str) -> bool {
    matches!(
        value,
        "AND" | "THE" | "WITH" | "FOR" | "FROM" | "THIS" | "THAT" | "STAR" | "PDF" | "DOCX"
    )
}

fn trim_terminal_punctuation(value: &str) -> &str {
    value.trim_end_matches(|character: char| matches!(character, '.' | ',' | ';' | ':' | ')' | ']'))
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

fn entity_matches(entity: &PrivateEntity, normalized: &str) -> bool {
    normalize(&entity.canonical_name) == normalized
        || entity
            .aliases
            .iter()
            .filter(|alias| alias.status != "rejected")
            .any(|alias| alias.normalized_value == normalized)
}

fn find_case_insensitive(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    if needle.trim().is_empty() {
        return Vec::new();
    }

    let lower_haystack = haystack.to_lowercase();
    let lower_needle = needle.to_lowercase();
    lower_haystack
        .match_indices(&lower_needle)
        .map(|(start, _)| (start, start + lower_needle.len()))
        .collect()
}

fn allocate_token(registry: &mut PrivateEntityRegistry, entity_type: EntityType) -> String {
    let family = entity_type.token_family().to_string();
    let next = registry.token_counters.entry(family.clone()).or_insert(0);
    *next += 1;
    format!("[{family}_{next}]")
}

fn default_sensitivity(entity_type: EntityType) -> EntitySensitivity {
    match entity_type {
        EntityType::Email
        | EntityType::Phone
        | EntityType::Account
        | EntityType::Identifier
        | EntityType::Person => EntitySensitivity::Private,
        EntityType::Url => EntitySensitivity::AskBeforeCloud,
        _ => EntitySensitivity::Private,
    }
}

#[allow(clippy::too_many_arguments)]
fn add_occurrence(
    entity: &mut PrivateEntity,
    record_type: &str,
    record_id: &str,
    text: &str,
    start: usize,
    end: usize,
    extraction_confidence: f32,
    type_confidence: f32,
    identity_match_confidence: f32,
    now: &str,
) {
    let locator = format!("chars:{start}-{end}");
    if let Some(existing) = entity.occurrences.iter_mut().find(|occurrence| {
        occurrence.record_id == record_id && occurrence.locator == locator
    }) {
        existing.last_seen_at = now.to_string();
        return;
    }

    entity.occurrences.push(occurrence_for(
        record_type,
        record_id,
        text,
        start,
        end,
        extraction_confidence,
        type_confidence,
        identity_match_confidence,
        now,
    ));
    entity.updated_at = now.to_string();
    entity.revision += 1;
}

#[allow(clippy::too_many_arguments)]
fn occurrence_for(
    record_type: &str,
    record_id: &str,
    text: &str,
    start: usize,
    end: usize,
    extraction_confidence: f32,
    type_confidence: f32,
    identity_match_confidence: f32,
    now: &str,
) -> EntityOccurrence {
    EntityOccurrence {
        occurrence_id: format!("occurrence_{}", Uuid::now_v7()),
        record_type: record_type.to_string(),
        record_id: record_id.to_string(),
        locator: format!("chars:{start}-{end}"),
        matched_text: text.get(start..end).unwrap_or_default().to_string(),
        extraction_confidence,
        type_confidence,
        identity_match_confidence,
        first_seen_at: now.to_string(),
        last_seen_at: now.to_string(),
    }
}

fn new_review_item(
    record_type: &str,
    record_id: &str,
    text: &str,
    detection: &Detection,
    candidate_matches: Vec<ReviewCandidateMatch>,
    now: &str,
) -> EntityReviewItem {
    let context_start = detection.start.saturating_sub(100);
    let context_end = (detection.end + 100).min(text.len());
    EntityReviewItem {
        schema_version: 1,
        review_item_id: format!("review_{}", Uuid::now_v7()),
        record_type: record_type.to_string(),
        record_id: record_id.to_string(),
        locator: format!("chars:{}-{}", detection.start, detection.end),
        context_excerpt: text
            .get(context_start..context_end)
            .unwrap_or_default()
            .to_string(),
        matched_text: detection.text.clone(),
        normalized_text: detection.normalized.clone(),
        suggested_entity_type: format!("{:?}", detection.entity_type).to_lowercase(),
        scores: ReviewScores {
            extraction: detection.extraction_confidence,
            type_score: detection.type_confidence,
            best_identity_match: candidate_matches
                .iter()
                .map(|candidate| candidate.score)
                .fold(0.0_f32, f32::max),
        },
        candidate_matches,
        risk: detection.risk.to_string(),
        status: "pending".to_string(),
        resolution: None,
        created_at: now.to_string(),
        updated_at: now.to_string(),
    }
}

fn write_review_item(vault_path: &Path, item: &EntityReviewItem) -> ServiceResult<()> {
    let path = vault_path
        .join("privacy/review-items")
        .join(format!("{}.json", item.review_item_id));
    write_json_atomic(&path, item)
}

pub fn count_pending_review_items(vault_path: &Path) -> ServiceResult<usize> {
    let directory = vault_path.join("privacy/review-items");
    if !directory.exists() {
        return Ok(0);
    }

    let mut count = 0;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let item: EntityReviewItem = read_json(&entry.path())?;
        if item.status == "pending" {
            count += 1;
        }
    }
    Ok(count)
}
