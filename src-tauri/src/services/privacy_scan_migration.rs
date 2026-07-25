use std::{fs, path::Path};

use chrono::Utc;
use serde_json::json;

use crate::{
    domain::models::{EntityReviewItem, EntityStatus, PrivateEntityRegistry},
    error::ServiceResult,
    io_utils::{read_json, write_json_atomic},
    services::entity_scan::{
        is_generic_entity_term, is_public_domain_acronym, load_registry, save_registry,
    },
};

pub fn migrate_legacy_review_noise(vault_path: &Path) -> ServiceResult<usize> {
    let directory = vault_path.join("privacy/review-items");
    if !directory.exists() {
        return Ok(0);
    }

    let mut registry = load_registry(vault_path)?;
    let now = Utc::now().to_rfc3339();
    let mut changed = 0;
    let mut registry_changed = false;

    for entry in fs::read_dir(&directory)? {
        let entry = entry?;
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let mut item: EntityReviewItem = read_json(&entry.path())?;
        if item.status != "pending" {
            continue;
        }

        let Some(action) = legacy_review_action(&item) else {
            continue;
        };

        if let Some(index) = review_entity_index(&registry, &item) {
            match action {
                LegacyReviewAction::Confirm => {
                    registry.entities[index].status = EntityStatus::Confirmed;
                    for alias in &mut registry.entities[index].aliases {
                        if alias.normalized_value == item.normalized_text {
                            alias.status = "confirmed".to_string();
                            alias.source = "deterministic_scan_v2".to_string();
                        }
                    }
                    registry.entities[index].updated_at = now.clone();
                    registry.entities[index].revision += 1;
                    registry_changed = true;
                }
                LegacyReviewAction::DismissNoise => {
                    if registry.entities[index].status == EntityStatus::Provisional
                        && registry.entities[index].occurrences.iter().all(|occurrence| {
                            occurrence.record_id == item.record_id
                                && occurrence.locator == item.locator
                        })
                    {
                        registry.entities[index].status = EntityStatus::Archived;
                        registry.entities[index].updated_at = now.clone();
                        registry.entities[index].revision += 1;
                        registry_changed = true;
                    }
                }
            }
        }

        item.status = match action {
            LegacyReviewAction::Confirm => "auto_resolved_v2",
            LegacyReviewAction::DismissNoise => "dismissed_noise_v2",
        }
        .to_string();
        item.updated_at = now.clone();
        item.resolution = Some(json!({
            "action": item.status.clone(),
            "resolvedAt": now.clone(),
            "resolvedBy": "privacy_scan_v2_migration",
            "notes": "Legacy review item removed from the user queue by the tightened deterministic scanner."
        }));
        write_json_atomic(&entry.path(), &item)?;
        changed += 1;
    }

    if registry_changed {
        registry.updated_at = now;
        registry.revision += 1;
        save_registry(vault_path, &registry)?;
    }

    Ok(changed)
}

#[derive(Debug, Clone, Copy)]
enum LegacyReviewAction {
    Confirm,
    DismissNoise,
}

fn legacy_review_action(item: &EntityReviewItem) -> Option<LegacyReviewAction> {
    let suggested = item.suggested_entity_type.as_str();
    if suggested == "project" && is_generic_entity_term(&item.matched_text) {
        return Some(LegacyReviewAction::DismissNoise);
    }

    if matches!(suggested, "email" | "phone" | "url" | "organization") {
        return Some(LegacyReviewAction::Confirm);
    }

    if matches!(suggested, "userdefined" | "user_defined") {
        if is_public_domain_acronym(&item.matched_text) {
            return Some(LegacyReviewAction::DismissNoise);
        }
        if is_uppercase_acronym(&item.matched_text) {
            return Some(LegacyReviewAction::Confirm);
        }
    }

    None
}

fn review_entity_index(
    registry: &PrivateEntityRegistry,
    item: &EntityReviewItem,
) -> Option<usize> {
    for candidate in &item.candidate_matches {
        if let Some(index) = registry
            .entities
            .iter()
            .position(|entity| entity.entity_id == candidate.entity_id)
        {
            return Some(index);
        }
    }

    registry.entities.iter().position(|entity| {
        entity.occurrences.iter().any(|occurrence| {
            occurrence.record_id == item.record_id && occurrence.locator == item.locator
        })
    })
}

fn is_uppercase_acronym(value: &str) -> bool {
    let trimmed = value.trim();
    (3..=10).contains(&trimmed.len())
        && trimmed
            .chars()
            .all(|character| character.is_ascii_uppercase() || character.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{ReviewCandidateMatch, ReviewScores};

    fn review(matched_text: &str, suggested_entity_type: &str) -> EntityReviewItem {
        EntityReviewItem {
            schema_version: 1,
            review_item_id: "review_test".to_string(),
            record_type: "source".to_string(),
            record_id: "source_test".to_string(),
            locator: "chars:0-4".to_string(),
            context_excerpt: String::new(),
            matched_text: matched_text.to_string(),
            normalized_text: matched_text.to_ascii_lowercase(),
            suggested_entity_type: suggested_entity_type.to_string(),
            scores: ReviewScores {
                extraction: 0.8,
                type_score: 0.7,
                best_identity_match: 1.0,
            },
            candidate_matches: vec![ReviewCandidateMatch {
                entity_id: "entity_test".to_string(),
                score: 1.0,
                reasons: Vec::new(),
            }],
            risk: "medium".to_string(),
            status: "pending".to_string(),
            resolution: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn generic_project_words_are_dismissed() {
        assert!(matches!(
            legacy_review_action(&review("Leadership", "project")),
            Some(LegacyReviewAction::DismissNoise)
        ));
    }

    #[test]
    fn pii_is_auto_confirmed() {
        assert!(matches!(
            legacy_review_action(&review("person@example.com", "email")),
            Some(LegacyReviewAction::Confirm)
        ));
    }

    #[test]
    fn real_named_projects_remain_for_review() {
        assert!(legacy_review_action(&review("Kinections", "project")).is_none());
    }
}
