use std::{fs, path::Path};

use chrono::Utc;
use regex::Regex;
use uuid::Uuid;

use crate::{
    domain::{
        candidates::StoryCandidate,
        models::{
            EntityAlias, EntityOccurrence, EntityReviewItem, EntitySensitivity, EntityStatus,
            EntityType, PrivateEntity, PrivateEntityRegistry, ReviewCandidateMatch, ReviewScores,
        },
        roles::{ParsedRoleHeading, RoleRecord, RoleSummary},
    },
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, write_json_atomic},
    services::entity_scan::{load_registry, save_registry},
};

pub fn ensure_role_for_candidate(
    vault_path: &Path,
    candidate: &StoryCandidate,
) -> ServiceResult<RoleRecord> {
    for role_id in &candidate.context.role_ids {
        let path = role_path(vault_path, role_id);
        if path.is_file() {
            return read_json(&path);
        }
    }

    let heading = candidate
        .context
        .surrounding_heading
        .as_deref()
        .ok_or_else(|| {
            WorkLoreError::RoleInferenceFailed(
                "the work-history bullet has no nearby role heading".to_string(),
            )
        })?;
    let mut registry = load_registry(vault_path)?;
    let parsed = parse_role_heading(heading, &registry)?;
    let organization_entity_id = ensure_employer_entity(
        vault_path,
        &mut registry,
        candidate,
        heading,
        &parsed.organization_name,
    )?;
    save_registry(vault_path, &registry)?;

    if let Some(existing) = read_roles(vault_path)?.into_iter().find(|role| {
        role.organization_entity_id == organization_entity_id
            && normalize(&role.title) == normalize(&parsed.title)
            && role.start_date == parsed.start_date
            && role.end_date == parsed.end_date
    }) {
        return Ok(existing);
    }

    let now = Utc::now().to_rfc3339();
    let role = RoleRecord {
        schema_version: 1,
        role_id: format!("role_{}", Uuid::now_v7()),
        title: parsed.title,
        organization_entity_id,
        client_entity_ids: Vec::new(),
        start_date: parsed.start_date,
        end_date: parsed.end_date,
        is_current: parsed.is_current,
        summary: String::new(),
        source_ids: candidate
            .source_refs
            .iter()
            .map(|reference| reference.source_id.clone())
            .collect(),
        project_entity_ids: candidate.context.entity_ids.clone(),
        story_ids: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
        revision: 1,
    };
    write_json_atomic(&role_path(vault_path, &role.role_id), &role)?;
    Ok(role)
}

pub fn read_role(vault_path: &Path, role_id: &str) -> ServiceResult<RoleRecord> {
    let path = role_path(vault_path, role_id);
    if !path.is_file() {
        return Err(WorkLoreError::RoleNotFound);
    }
    read_json(&path)
}

pub fn save_role(vault_path: &Path, role: &RoleRecord) -> ServiceResult<()> {
    write_json_atomic(&role_path(vault_path, &role.role_id), role)
}

pub fn list_role_summaries(vault_path: &Path) -> ServiceResult<Vec<RoleSummary>> {
    let registry = load_registry(vault_path)?;
    let mut summaries = read_roles(vault_path)?
        .into_iter()
        .map(|role| RoleSummary {
            organization_name: registry
                .entities
                .iter()
                .find(|entity| entity.entity_id == role.organization_entity_id)
                .map(|entity| entity.canonical_name.clone())
                .unwrap_or_else(|| "Unknown organization".to_string()),
            role_id: role.role_id,
            title: role.title,
            organization_entity_id: role.organization_entity_id,
            start_date: role.start_date,
            end_date: role.end_date,
            is_current: role.is_current,
            story_count: role.story_ids.len(),
        })
        .collect::<Vec<_>>();
    summaries.sort_by(|left, right| right.start_date.cmp(&left.start_date));
    Ok(summaries)
}

fn parse_role_heading(
    heading: &str,
    registry: &PrivateEntityRegistry,
) -> ServiceResult<ParsedRoleHeading> {
    let trimmed = heading.trim();
    if trimmed.is_empty() {
        return Err(WorkLoreError::RoleInferenceFailed(
            "the role heading is empty".to_string(),
        ));
    }

    let (start_date, end_date, is_current) = parse_dates(trimmed);
    let mut segments = split_heading(trimmed)
        .into_iter()
        .filter(|segment| !looks_like_date_segment(segment))
        .collect::<Vec<_>>();

    if segments.len() < 2 {
        if let Some((title, organization)) = split_title_at_organization(trimmed) {
            segments = vec![organization.to_string(), title.to_string()];
        }
    }
    if segments.len() < 2 {
        return Err(WorkLoreError::RoleInferenceFailed(format!(
            "\"{trimmed}\" does not clearly identify both an organization and a role title"
        )));
    }

    let organization_index = segments
        .iter()
        .position(|segment| entity_name_exists(registry, segment))
        .unwrap_or(0);
    let organization_name = segments.remove(organization_index);
    let title = segments
        .into_iter()
        .find(|segment| !segment.trim().is_empty())
        .ok_or_else(|| {
            WorkLoreError::RoleInferenceFailed(
                "the role heading does not contain a usable title".to_string(),
            )
        })?;

    Ok(ParsedRoleHeading {
        organization_name: organization_name.trim().to_string(),
        title: title.trim().to_string(),
        start_date,
        end_date,
        is_current,
    })
}

fn split_heading(heading: &str) -> Vec<String> {
    for separator in [" | ", " — ", " – ", " - "] {
        let values = heading
            .split(separator)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        if values.len() >= 2 {
            return values;
        }
    }
    vec![heading.to_string()]
}

fn split_title_at_organization(heading: &str) -> Option<(&str, &str)> {
    let lower = heading.to_ascii_lowercase();
    let index = lower.find(" at ")?;
    let title = heading.get(..index)?.trim();
    let organization = heading.get(index + 4..)?.trim();
    (!title.is_empty() && !organization.is_empty()).then_some((title, organization))
}

fn parse_dates(value: &str) -> (Option<String>, Option<String>, bool) {
    let year_regex = Regex::new(r"\b(?:19|20)\d{2}\b").expect("year regex");
    let years = year_regex
        .find_iter(value)
        .map(|matched| matched.as_str().to_string())
        .collect::<Vec<_>>();
    let lower = value.to_ascii_lowercase();
    let is_current = lower.contains("present") || lower.contains("current");
    let start_date = years.first().cloned();
    let end_date = if is_current {
        None
    } else {
        years.get(1).cloned()
    };
    (start_date, end_date, is_current)
}

fn looks_like_date_segment(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("present")
        || lower.contains("current")
        || Regex::new(r"\b(?:19|20)\d{2}\b")
            .expect("year regex")
            .is_match(value)
}

fn entity_name_exists(registry: &PrivateEntityRegistry, value: &str) -> bool {
    let normalized = normalize(value);
    registry.entities.iter().any(|entity| {
        !matches!(entity.status, EntityStatus::Merged | EntityStatus::Archived)
            && (normalize(&entity.canonical_name) == normalized
                || entity.aliases.iter().any(|alias| {
                    alias.status != "rejected" && alias.normalized_value == normalized
                }))
    })
}

fn ensure_employer_entity(
    vault_path: &Path,
    registry: &mut PrivateEntityRegistry,
    candidate: &StoryCandidate,
    heading: &str,
    organization_name: &str,
) -> ServiceResult<String> {
    let normalized = normalize(organization_name);
    if let Some(entity) = registry.entities.iter().find(|entity| {
        !matches!(entity.status, EntityStatus::Merged | EntityStatus::Archived)
            && (normalize(&entity.canonical_name) == normalized
                || entity.aliases.iter().any(|alias| {
                    alias.status != "rejected" && alias.normalized_value == normalized
                }))
    }) {
        return Ok(entity.entity_id.clone());
    }

    let now = Utc::now().to_rfc3339();
    let entity_id = format!("entity_{}", Uuid::now_v7());
    let family = EntityType::Employer.token_family().to_string();
    let counter = registry.token_counters.entry(family.clone()).or_insert(0);
    *counter += 1;
    let public_token = format!("[{family}_{counter}]");
    registry.entities.push(PrivateEntity {
        schema_version: 1,
        entity_id: entity_id.clone(),
        entity_type: EntityType::Employer,
        canonical_name: organization_name.to_string(),
        public_token,
        public_description: None,
        sensitivity: EntitySensitivity::Private,
        status: EntityStatus::Provisional,
        aliases: vec![EntityAlias {
            value: organization_name.to_string(),
            normalized_value: normalized.clone(),
            status: "inferred".to_string(),
            source: "employment_heading".to_string(),
        }],
        relationships: Vec::new(),
        occurrences: vec![EntityOccurrence {
            occurrence_id: format!("occurrence_{}", Uuid::now_v7()),
            record_type: "candidate".to_string(),
            record_id: candidate.candidate_id.clone(),
            locator: "context.surroundingHeading".to_string(),
            matched_text: organization_name.to_string(),
            extraction_confidence: 0.92,
            type_confidence: 0.82,
            identity_match_confidence: 0.0,
            first_seen_at: now.clone(),
            last_seen_at: now.clone(),
        }],
        redirect_to_entity_id: None,
        retired_tokens: Vec::new(),
        notes: "Inferred from a recognized employment-history role heading.".to_string(),
        created_at: now.clone(),
        updated_at: now.clone(),
        revision: 1,
    });
    registry.updated_at = now.clone();
    registry.revision += 1;

    let review = EntityReviewItem {
        schema_version: 1,
        review_item_id: format!("review_{}", Uuid::now_v7()),
        record_type: "candidate".to_string(),
        record_id: candidate.candidate_id.clone(),
        locator: "context.surroundingHeading".to_string(),
        context_excerpt: heading.to_string(),
        matched_text: organization_name.to_string(),
        normalized_text: normalized,
        suggested_entity_type: "employer".to_string(),
        scores: ReviewScores {
            extraction: 0.92,
            type_score: 0.82,
            best_identity_match: 0.0,
        },
        candidate_matches: vec![ReviewCandidateMatch {
            entity_id: entity_id.clone(),
            score: 1.0,
            reasons: vec![
                "WorkLore inferred this employer from a recognized employment-history heading."
                    .to_string(),
            ],
        }],
        risk: "medium".to_string(),
        status: "pending".to_string(),
        resolution: None,
        created_at: now.clone(),
        updated_at: now,
    };
    write_json_atomic(
        &vault_path
            .join("privacy/review-items")
            .join(format!("{}.json", review.review_item_id)),
        &review,
    )?;
    Ok(entity_id)
}

fn read_roles(vault_path: &Path) -> ServiceResult<Vec<RoleRecord>> {
    let directory = vault_path.join("roles");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut roles = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
        {
            roles.push(read_json(&entry.path())?);
        }
    }
    Ok(roles)
}

fn role_path(vault_path: &Path, role_id: &str) -> std::path::PathBuf {
    vault_path.join("roles").join(format!("{role_id}.json"))
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::entity_scan::initialize_registry;

    #[test]
    fn parses_common_employment_heading() {
        let registry = initialize_registry("vault_test");
        let parsed = parse_role_heading(
            "FINRA | Lead Product Manager | 2022 - Present",
            &registry,
        )
        .expect("heading should parse");
        assert_eq!(parsed.organization_name, "FINRA");
        assert_eq!(parsed.title, "Lead Product Manager");
        assert_eq!(parsed.start_date.as_deref(), Some("2022"));
        assert!(parsed.is_current);
    }

    #[test]
    fn rejects_headings_without_organization_and_title() {
        let registry = initialize_registry("vault_test");
        assert!(parse_role_heading("Lead Product Manager", &registry).is_err());
    }
}
