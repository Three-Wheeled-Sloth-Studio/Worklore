use std::{fs, path::Path};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::models::{
        EntityAlias, EntityOccurrence, EntityRelationship, EntityReviewItem, EntityStatus,
        EntityType, IgnoredTerm, PrivateEntity, PrivateEntityRegistry, PrivacyScanStatus,
        SourceDocument, TokenRedirect,
    },
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, write_json_atomic},
    services::entity_scan::{count_pending_review_items, load_registry, save_registry},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityCandidateView {
    pub entity_id: String,
    pub canonical_name: String,
    pub public_token: String,
    pub entity_type: EntityType,
    pub sensitivity: String,
    pub score: f32,
    pub reasons: Vec<String>,
    pub provisional: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityReviewView {
    pub review_item_id: String,
    pub record_type: String,
    pub record_id: String,
    pub locator: String,
    pub context_excerpt: String,
    pub matched_text: String,
    pub suggested_entity_type: String,
    pub extraction_confidence: f32,
    pub type_confidence: f32,
    pub identity_match_confidence: f32,
    pub risk: String,
    pub status: String,
    pub question: String,
    pub provisional_entity_id: Option<String>,
    pub candidates: Vec<EntityCandidateView>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewResolutionAction {
    SameEntity,
    RelatedEntity,
    NewEntity,
    IgnoreTerm,
    Dismiss,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveEntityReviewRequest {
    pub review_item_id: String,
    pub action: ReviewResolutionAction,
    pub target_entity_id: Option<String>,
    pub canonical_name: Option<String>,
    pub entity_type: Option<EntityType>,
    pub relationship_label: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveEntityReviewResult {
    pub review_item_id: String,
    pub status: String,
    pub affected_entity_id: Option<String>,
    pub pending_review_count: usize,
    pub message: String,
}

pub fn list_reviews(vault_path: &Path, pending_only: bool) -> ServiceResult<Vec<EntityReviewView>> {
    ensure_vault(vault_path)?;
    let registry = load_registry(vault_path)?;
    let directory = vault_path.join("privacy/review-items");
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut views = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("json")
        {
            continue;
        }

        let item: EntityReviewItem = read_json(&entry.path())?;
        if pending_only && item.status != "pending" {
            continue;
        }
        views.push(to_view(&item, &registry));
    }

    views.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(views)
}

pub fn resolve_review(
    vault_path: &Path,
    request: ResolveEntityReviewRequest,
) -> ServiceResult<ResolveEntityReviewResult> {
    ensure_vault(vault_path)?;
    let item_path = vault_path
        .join("privacy/review-items")
        .join(format!("{}.json", request.review_item_id));
    if !item_path.is_file() {
        return Err(WorkLoreError::ReviewItemNotFound);
    }

    let mut item: EntityReviewItem = read_json(&item_path)?;
    if item.status != "pending" {
        return Err(WorkLoreError::InvalidReviewResolution(
            "This review item has already been resolved.".to_string(),
        ));
    }

    let mut registry = load_registry(vault_path)?;
    let now = Utc::now().to_rfc3339();
    let provisional_index = find_provisional_index(&registry, &item);

    let (status, affected_entity_id, message) = match request.action {
        ReviewResolutionAction::SameEntity => {
            let target_id = required_target(&request)?;
            let target_index = entity_index(&registry, &target_id)?;
            let surviving_id = if let Some(provisional_index) = provisional_index {
                if provisional_index == target_index {
                    confirm_entity(&mut registry.entities[target_index], &item, &request, &now);
                    registry.entities[target_index].entity_id.clone()
                } else {
                    merge_provisional(
                        &mut registry,
                        provisional_index,
                        target_index,
                        &item,
                        &request,
                        &now,
                    )?
                }
            } else {
                confirm_entity(&mut registry.entities[target_index], &item, &request, &now);
                registry.entities[target_index].entity_id.clone()
            };
            (
                "resolved_same_entity",
                Some(surviving_id),
                "Entity linked and its durable alias mapping was updated.".to_string(),
            )
        }
        ReviewResolutionAction::RelatedEntity => {
            let target_id = required_target(&request)?;
            let target_index = entity_index(&registry, &target_id)?;
            let provisional_index = match provisional_index {
                Some(index) => index,
                None => create_entity_from_review(&mut registry, &item, &request, &now),
            };
            if provisional_index == target_index {
                return Err(WorkLoreError::InvalidReviewResolution(
                    "An entity cannot be related to itself. Choose Same entity instead."
                        .to_string(),
                ));
            }
            confirm_entity(
                &mut registry.entities[provisional_index],
                &item,
                &request,
                &now,
            );
            add_related_pair(
                &mut registry,
                provisional_index,
                target_index,
                request.relationship_label.as_deref(),
                &now,
            );
            (
                "resolved_related_entity",
                Some(registry.entities[provisional_index].entity_id.clone()),
                "A separate entity was confirmed and linked to the selected entity.".to_string(),
            )
        }
        ReviewResolutionAction::NewEntity => {
            let index = match provisional_index {
                Some(index) => index,
                None => create_entity_from_review(&mut registry, &item, &request, &now),
            };
            apply_type_change_if_needed(&mut registry, index, request.entity_type, &now);
            confirm_entity(&mut registry.entities[index], &item, &request, &now);
            (
                "resolved_new_entity",
                Some(registry.entities[index].entity_id.clone()),
                "A new durable entity mapping was confirmed.".to_string(),
            )
        }
        ReviewResolutionAction::IgnoreTerm => {
            if !registry
                .ignored_terms
                .iter()
                .any(|term| term.normalized_value == item.normalized_text && term.scope == "vault")
            {
                registry.ignored_terms.push(IgnoredTerm {
                    normalized_value: item.normalized_text.clone(),
                    scope: "vault".to_string(),
                    record_id: None,
                    created_at: now.clone(),
                });
            }
            if let Some(index) = provisional_index {
                registry.entities[index].status = EntityStatus::Archived;
                registry.entities[index].updated_at = now.clone();
                registry.entities[index].revision += 1;
            }
            (
                "ignored",
                None,
                "This term will be ignored in future vault scans.".to_string(),
            )
        }
        ReviewResolutionAction::Dismiss => (
            "dismissed",
            provisional_index.map(|index| registry.entities[index].entity_id.clone()),
            "This occurrence was dismissed. No alias or sensitivity rule was changed."
                .to_string(),
        ),
    };

    item.status = status.to_string();
    item.updated_at = now.clone();
    item.resolution = Some(serde_json::json!({
        "action": action_name(request.action),
        "entityId": affected_entity_id,
        "targetEntityId": request.target_entity_id,
        "canonicalName": request.canonical_name,
        "resolvedAt": now,
        "resolvedBy": "user",
        "notes": request.notes.unwrap_or_default()
    }));

    registry.updated_at = Utc::now().to_rfc3339();
    registry.revision += 1;
    save_registry(vault_path, &registry)?;
    write_json_atomic(&item_path, &item)?;
    reconcile_record_scan(vault_path, &item)?;

    Ok(ResolveEntityReviewResult {
        review_item_id: item.review_item_id,
        status: item.status,
        affected_entity_id,
        pending_review_count: count_pending_review_items(vault_path)?,
        message,
    })
}

fn to_view(item: &EntityReviewItem, registry: &PrivateEntityRegistry) -> EntityReviewView {
    let provisional_entity_id = find_provisional_index(registry, item)
        .map(|index| registry.entities[index].entity_id.clone());
    let candidates = item
        .candidate_matches
        .iter()
        .filter_map(|candidate| {
            registry
                .entities
                .iter()
                .find(|entity| entity.entity_id == candidate.entity_id)
                .map(|entity| EntityCandidateView {
                    entity_id: entity.entity_id.clone(),
                    canonical_name: entity.canonical_name.clone(),
                    public_token: entity.public_token.clone(),
                    entity_type: entity.entity_type,
                    sensitivity: format!("{:?}", entity.sensitivity).to_lowercase(),
                    score: candidate.score,
                    reasons: candidate.reasons.clone(),
                    provisional: entity.status == EntityStatus::Provisional,
                })
        })
        .collect::<Vec<_>>();

    let existing = candidates
        .iter()
        .find(|candidate| Some(&candidate.entity_id) != provisional_entity_id.as_ref());
    let question = match existing {
        Some(candidate) => format!(
            "Is \"{}\" the same {} as \"{}\"?",
            item.matched_text,
            entity_type_label(candidate.entity_type),
            candidate.canonical_name
        ),
        None => format!("How should WorkLore classify \"{}\"?", item.matched_text),
    };

    EntityReviewView {
        review_item_id: item.review_item_id.clone(),
        record_type: item.record_type.clone(),
        record_id: item.record_id.clone(),
        locator: item.locator.clone(),
        context_excerpt: item.context_excerpt.clone(),
        matched_text: item.matched_text.clone(),
        suggested_entity_type: item.suggested_entity_type.clone(),
        extraction_confidence: item.scores.extraction,
        type_confidence: item.scores.type_score,
        identity_match_confidence: item.scores.best_identity_match,
        risk: item.risk.clone(),
        status: item.status.clone(),
        question,
        provisional_entity_id,
        candidates,
        created_at: item.created_at.clone(),
    }
}

fn required_target(request: &ResolveEntityReviewRequest) -> ServiceResult<String> {
    request.target_entity_id.clone().ok_or_else(|| {
        WorkLoreError::InvalidReviewResolution(
            "Select the existing entity this term should use.".to_string(),
        )
    })
}

fn entity_index(registry: &PrivateEntityRegistry, entity_id: &str) -> ServiceResult<usize> {
    registry
        .entities
        .iter()
        .position(|entity| entity.entity_id == entity_id)
        .ok_or(WorkLoreError::EntityNotFound)
}

fn find_provisional_index(
    registry: &PrivateEntityRegistry,
    item: &EntityReviewItem,
) -> Option<usize> {
    registry.entities.iter().position(|entity| {
        entity.status == EntityStatus::Provisional
            && entity.occurrences.iter().any(|occurrence| {
                occurrence.record_id == item.record_id && occurrence.locator == item.locator
            })
    })
}

fn create_entity_from_review(
    registry: &mut PrivateEntityRegistry,
    item: &EntityReviewItem,
    request: &ResolveEntityReviewRequest,
    now: &str,
) -> usize {
    let entity_type = request.entity_type.unwrap_or(EntityType::UserDefined);
    let entity_id = format!("entity_{}", Uuid::now_v7());
    let canonical_name = request
        .canonical_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&item.matched_text)
        .to_string();
    let token = allocate_token(registry, entity_type);

    registry.entities.push(PrivateEntity {
        schema_version: 1,
        entity_id,
        entity_type,
        canonical_name: canonical_name.clone(),
        public_token: token,
        public_description: None,
        sensitivity: crate::domain::models::EntitySensitivity::Private,
        status: EntityStatus::Confirmed,
        aliases: vec![EntityAlias {
            value: item.matched_text.clone(),
            normalized_value: item.normalized_text.clone(),
            status: "confirmed".to_string(),
            source: "user".to_string(),
        }],
        relationships: Vec::new(),
        occurrences: vec![EntityOccurrence {
            occurrence_id: format!("occurrence_{}", Uuid::now_v7()),
            record_type: item.record_type.clone(),
            record_id: item.record_id.clone(),
            locator: item.locator.clone(),
            matched_text: item.matched_text.clone(),
            extraction_confidence: item.scores.extraction,
            type_confidence: item.scores.type_score,
            identity_match_confidence: 1.0,
            first_seen_at: now.to_string(),
            last_seen_at: now.to_string(),
        }],
        redirect_to_entity_id: None,
        retired_tokens: Vec::new(),
        notes: request.notes.clone().unwrap_or_default(),
        created_at: now.to_string(),
        updated_at: now.to_string(),
        revision: 1,
    });
    registry.entities.len() - 1
}

fn confirm_entity(
    entity: &mut PrivateEntity,
    item: &EntityReviewItem,
    request: &ResolveEntityReviewRequest,
    now: &str,
) {
    if let Some(name) = request
        .canonical_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        entity.canonical_name = name.to_string();
    }
    if !entity
        .aliases
        .iter()
        .any(|alias| alias.normalized_value == item.normalized_text && alias.status != "rejected")
    {
        entity.aliases.push(EntityAlias {
            value: item.matched_text.clone(),
            normalized_value: item.normalized_text.clone(),
            status: "confirmed".to_string(),
            source: "user".to_string(),
        });
    } else {
        for alias in &mut entity.aliases {
            if alias.normalized_value == item.normalized_text {
                alias.status = "confirmed".to_string();
                alias.source = "user".to_string();
            }
        }
    }
    entity.status = EntityStatus::Confirmed;
    entity.updated_at = now.to_string();
    entity.revision += 1;
}

fn merge_provisional(
    registry: &mut PrivateEntityRegistry,
    provisional_index: usize,
    target_index: usize,
    item: &EntityReviewItem,
    request: &ResolveEntityReviewRequest,
    now: &str,
) -> ServiceResult<String> {
    let (provisional, target) = two_entities_mut(&mut registry.entities, provisional_index, target_index)?;
    let retired_token = provisional.public_token.clone();
    let provisional_id = provisional.entity_id.clone();
    let target_id = target.entity_id.clone();

    confirm_entity(target, item, request, now);
    for alias in provisional.aliases.clone() {
        if !target
            .aliases
            .iter()
            .any(|existing| existing.normalized_value == alias.normalized_value)
        {
            target.aliases.push(EntityAlias {
                status: "confirmed".to_string(),
                source: "merge".to_string(),
                ..alias
            });
        }
    }
    for occurrence in provisional.occurrences.clone() {
        if !target.occurrences.iter().any(|existing| {
            existing.record_id == occurrence.record_id && existing.locator == occurrence.locator
        }) {
            target.occurrences.push(occurrence);
        }
    }

    provisional.status = EntityStatus::Merged;
    provisional.redirect_to_entity_id = Some(target_id.clone());
    provisional.updated_at = now.to_string();
    provisional.revision += 1;

    registry.token_redirects.push(TokenRedirect {
        retired_token,
        surviving_entity_id: target_id.clone(),
        created_at: now.to_string(),
        reason: "merge".to_string(),
    });
    if !target.retired_tokens.contains(&provisional.public_token) {
        target.retired_tokens.push(provisional.public_token.clone());
    }

    if provisional_id == target_id {
        return Err(WorkLoreError::InvalidReviewResolution(
            "The provisional and surviving entities cannot be the same record.".to_string(),
        ));
    }
    Ok(target_id)
}

fn add_related_pair(
    registry: &mut PrivateEntityRegistry,
    left_index: usize,
    right_index: usize,
    label: Option<&str>,
    now: &str,
) {
    let left_id = registry.entities[left_index].entity_id.clone();
    let right_id = registry.entities[right_index].entity_id.clone();
    let label = label.unwrap_or("").trim().to_string();

    let (left, right) = if left_index < right_index {
        let (head, tail) = registry.entities.split_at_mut(right_index);
        (&mut head[left_index], &mut tail[0])
    } else {
        let (head, tail) = registry.entities.split_at_mut(left_index);
        (&mut tail[0], &mut head[right_index])
    };

    if !left
        .relationships
        .iter()
        .any(|relationship| relationship.target_entity_id == right_id)
    {
        left.relationships.push(EntityRelationship {
            relationship_id: format!("relationship_{}", Uuid::now_v7()),
            relationship_type: "related_to".to_string(),
            target_entity_id: right_id,
            status: "confirmed".to_string(),
            label: label.clone(),
        });
    }
    if !right
        .relationships
        .iter()
        .any(|relationship| relationship.target_entity_id == left_id)
    {
        right.relationships.push(EntityRelationship {
            relationship_id: format!("relationship_{}", Uuid::now_v7()),
            relationship_type: "related_to".to_string(),
            target_entity_id: left_id,
            status: "confirmed".to_string(),
            label,
        });
    }
    left.updated_at = now.to_string();
    right.updated_at = now.to_string();
    left.revision += 1;
    right.revision += 1;
}

fn apply_type_change_if_needed(
    registry: &mut PrivateEntityRegistry,
    index: usize,
    requested_type: Option<EntityType>,
    now: &str,
) {
    let Some(requested_type) = requested_type else {
        return;
    };
    if registry.entities[index].entity_type == requested_type {
        return;
    }

    let retired_token = registry.entities[index].public_token.clone();
    let entity_id = registry.entities[index].entity_id.clone();
    let new_token = allocate_token(registry, requested_type);
    registry.entities[index].entity_type = requested_type;
    registry.entities[index].public_token = new_token;
    registry.entities[index].retired_tokens.push(retired_token.clone());
    registry.entities[index].updated_at = now.to_string();
    registry.entities[index].revision += 1;
    registry.token_redirects.push(TokenRedirect {
        retired_token,
        surviving_entity_id: entity_id,
        created_at: now.to_string(),
        reason: "manual_correction".to_string(),
    });
}

fn allocate_token(registry: &mut PrivateEntityRegistry, entity_type: EntityType) -> String {
    let family = entity_type.token_family().to_string();
    let next = registry.token_counters.entry(family.clone()).or_insert(0);
    *next += 1;
    format!("[{family}_{next}]")
}

fn two_entities_mut(
    entities: &mut [PrivateEntity],
    first: usize,
    second: usize,
) -> ServiceResult<(&mut PrivateEntity, &mut PrivateEntity)> {
    if first == second {
        return Err(WorkLoreError::InvalidReviewResolution(
            "Choose a different surviving entity.".to_string(),
        ));
    }
    if first < second {
        let (head, tail) = entities.split_at_mut(second);
        Ok((&mut head[first], &mut tail[0]))
    } else {
        let (head, tail) = entities.split_at_mut(first);
        Ok((&mut tail[0], &mut head[second]))
    }
}

fn reconcile_record_scan(vault_path: &Path, item: &EntityReviewItem) -> ServiceResult<()> {
    if item.record_type != "source" {
        return Ok(());
    }
    let path = vault_path
        .join("sources/metadata")
        .join(format!("{}.json", item.record_id));
    if !path.is_file() {
        return Ok(());
    }

    let mut source: SourceDocument = read_json(&path)?;
    let still_pending = source.privacy_scan.review_item_ids.iter().any(|review_id| {
        let review_path = vault_path
            .join("privacy/review-items")
            .join(format!("{review_id}.json"));
        read_json::<EntityReviewItem>(&review_path)
            .map(|review| review.status == "pending")
            .unwrap_or(true)
    });
    if !still_pending {
        source.privacy_scan.status = PrivacyScanStatus::Complete;
        source.privacy_scan.scanned_at = Some(Utc::now().to_rfc3339());
        source.updated_at = Utc::now().to_rfc3339();
        write_json_atomic(&path, &source)?;
    }
    Ok(())
}

fn ensure_vault(vault_path: &Path) -> ServiceResult<()> {
    if !vault_path.join("vault.json").is_file() {
        return Err(WorkLoreError::NotAVault);
    }
    Ok(())
}

fn action_name(action: ReviewResolutionAction) -> &'static str {
    match action {
        ReviewResolutionAction::SameEntity => "same_entity",
        ReviewResolutionAction::RelatedEntity => "related_entity",
        ReviewResolutionAction::NewEntity => "new_entity",
        ReviewResolutionAction::IgnoreTerm => "ignore_term",
        ReviewResolutionAction::Dismiss => "dismiss",
    }
}

fn entity_type_label(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Employer => "employer",
        EntityType::Client => "client",
        EntityType::Project => "project",
        EntityType::Product => "product",
        EntityType::System => "system",
        EntityType::Repository => "repository",
        EntityType::Person => "person",
        EntityType::Location => "location",
        EntityType::Email => "email address",
        EntityType::Phone => "phone number",
        EntityType::Url => "URL",
        EntityType::Account => "account",
        EntityType::Identifier => "identifier",
        EntityType::Organization => "organization",
        EntityType::UserDefined => "entity",
    }
}
