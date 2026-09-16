use std::{collections::HashMap, fs, path::Path};

use crate::{
    domain::{
        models::{
            CloudIdentifierMode, EntityReviewItem, EntitySensitivity, EntityStatus, PrivateEntity,
            VaultDocument,
        },
        providers::{PrivacyPreflightSummary, RedactionReplacement},
    },
    error::{ServiceResult, WorkLoreError},
    io_utils::read_json,
    services::entity_scan::load_registry,
};

#[derive(Debug, Clone)]
pub struct RedactedText {
    pub text: String,
    pub preflight: PrivacyPreflightSummary,
}

pub fn redact_for_external_use(vault_path: &Path, input: &str) -> ServiceResult<RedactedText> {
    let vault: VaultDocument = read_json(&vault_path.join("vault.json"))?;
    let registry = load_registry(vault_path)?;
    let review_items = pending_reviews(vault_path)?;
    let blocked_review_item_ids = review_items
        .iter()
        .filter(|item| matches!(item.risk.as_str(), "high" | "critical"))
        .map(|item| item.review_item_id.clone())
        .collect::<Vec<_>>();
    let warning_review_item_ids = review_items
        .iter()
        .filter(|item| !matches!(item.risk.as_str(), "high" | "critical"))
        .map(|item| item.review_item_id.clone())
        .collect::<Vec<_>>();

    if !blocked_review_item_ids.is_empty()
        && vault.privacy.block_cloud_when_high_risk_review_pending
    {
        return Err(WorkLoreError::ProviderPreflightBlocked(format!(
            "Resolve {} high-risk privacy review item{} before exporting this content.",
            blocked_review_item_ids.len(),
            if blocked_review_item_ids.len() == 1 {
                ""
            } else {
                "s"
            }
        )));
    }

    let mut output = input.to_string();
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut replacements =
        replacement_rules(&registry.entities, vault.privacy.cloud_identifier_mode);
    replacements.sort_by_key(|rule| std::cmp::Reverse(rule.alias.len()));

    for rule in replacements {
        if rule.alias.is_empty() || rule.alias == rule.token {
            continue;
        }
        let (replaced, count) = replace_alias(&output, &rule.alias, &rule.token);
        if count == 0 {
            continue;
        }
        output = replaced;
        *counts.entry(rule.entity_id.clone()).or_insert(0) += count;
    }

    let mut replacement_summaries = registry
        .entities
        .iter()
        .filter_map(|entity| {
            counts
                .get(&entity.entity_id)
                .map(|count| RedactionReplacement {
                    entity_id: entity.entity_id.clone(),
                    entity_type: format!("{:?}", entity.entity_type).to_ascii_lowercase(),
                    replacement: entity.public_token.clone(),
                    occurrence_count: *count,
                })
        })
        .collect::<Vec<_>>();
    replacement_summaries.sort_by(|left, right| left.replacement.cmp(&right.replacement));

    let contains_never_send_entities = registry.entities.iter().any(|entity| {
        entity.sensitivity == EntitySensitivity::NeverSendToCloud
            && counts.contains_key(&entity.entity_id)
    });
    let replacement_count = replacement_summaries
        .iter()
        .map(|replacement| replacement.occurrence_count)
        .sum();

    Ok(RedactedText {
        text: output,
        preflight: PrivacyPreflightSummary {
            mode: match vault.privacy.cloud_identifier_mode {
                CloudIdentifierMode::Redact => "redact",
                CloudIdentifierMode::Include => "include",
            }
            .to_string(),
            replacement_count,
            replacements: replacement_summaries,
            warning_review_item_ids,
            blocked_review_item_ids,
            contains_never_send_entities,
        },
    })
}

#[derive(Debug)]
struct ReplacementRule {
    entity_id: String,
    alias: String,
    token: String,
}

fn replacement_rules(
    entities: &[PrivateEntity],
    mode: CloudIdentifierMode,
) -> Vec<ReplacementRule> {
    let mut rules = Vec::new();
    for entity in entities {
        if matches!(entity.status, EntityStatus::Merged | EntityStatus::Archived) {
            continue;
        }
        let should_redact = match entity.sensitivity {
            EntitySensitivity::NeverSendToCloud => true,
            EntitySensitivity::Public => false,
            EntitySensitivity::Private | EntitySensitivity::AskBeforeCloud => {
                mode == CloudIdentifierMode::Redact
            }
        };
        if !should_redact {
            continue;
        }

        rules.push(ReplacementRule {
            entity_id: entity.entity_id.clone(),
            alias: entity.canonical_name.clone(),
            token: entity.public_token.clone(),
        });
        for alias in entity
            .aliases
            .iter()
            .filter(|alias| alias.status != "rejected")
        {
            rules.push(ReplacementRule {
                entity_id: entity.entity_id.clone(),
                alias: alias.value.clone(),
                token: entity.public_token.clone(),
            });
        }
    }
    rules
}

fn replace_alias(input: &str, alias: &str, token: &str) -> (String, usize) {
    if alias.is_empty() {
        return (input.to_string(), 0);
    }
    if input.is_ascii() && alias.is_ascii() {
        replace_ascii_case_insensitive(input, alias, token)
    } else {
        replace_case_sensitive(input, alias, token)
    }
}

fn replace_ascii_case_insensitive(input: &str, alias: &str, token: &str) -> (String, usize) {
    let haystack = input.to_ascii_lowercase();
    let needle = alias.to_ascii_lowercase();
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    let mut count = 0;

    while let Some(relative_start) = haystack[cursor..].find(&needle) {
        let start = cursor + relative_start;
        let end = start + needle.len();
        if is_token_boundary(input.as_bytes(), start, end) {
            output.push_str(&input[cursor..start]);
            output.push_str(token);
            cursor = end;
            count += 1;
        } else {
            let advance = end.max(start + 1);
            output.push_str(&input[cursor..advance]);
            cursor = advance;
        }
    }
    output.push_str(&input[cursor..]);
    (output, count)
}

fn replace_case_sensitive(input: &str, alias: &str, token: &str) -> (String, usize) {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    let mut count = 0;

    while let Some(relative_start) = input[cursor..].find(alias) {
        let start = cursor + relative_start;
        let end = start + alias.len();
        if is_token_boundary(input.as_bytes(), start, end) {
            output.push_str(&input[cursor..start]);
            output.push_str(token);
            cursor = end;
            count += 1;
        } else {
            let advance = end.max(start + 1);
            output.push_str(&input[cursor..advance]);
            cursor = advance;
        }
    }
    output.push_str(&input[cursor..]);
    (output, count)
}

fn is_token_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before_is_word = start
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .is_some_and(u8::is_ascii_alphanumeric);
    let after_is_word = bytes.get(end).is_some_and(u8::is_ascii_alphanumeric);
    !before_is_word && !after_is_word
}

fn pending_reviews(vault_path: &Path) -> ServiceResult<Vec<EntityReviewItem>> {
    let directory = vault_path.join("privacy/review-items");
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
        {
            let item: EntityReviewItem = read_json(&entry.path())?;
            if item.status == "pending" {
                items.push(item);
            }
        }
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{EntityAlias, EntityType};

    fn entity(name: &str, alias: &str, sensitivity: EntitySensitivity) -> PrivateEntity {
        PrivateEntity {
            schema_version: 1,
            entity_id: "entity_test".to_string(),
            entity_type: EntityType::Employer,
            canonical_name: name.to_string(),
            public_token: "[EMPLOYER_1]".to_string(),
            public_description: None,
            sensitivity,
            status: EntityStatus::Confirmed,
            aliases: vec![EntityAlias {
                value: alias.to_string(),
                normalized_value: alias.to_ascii_lowercase(),
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
    fn replacement_rules_respect_include_mode_except_never_send() {
        let private = entity("Acme", "Acme Corp", EntitySensitivity::Private);
        let never = entity(
            "Secret Client",
            "Client X",
            EntitySensitivity::NeverSendToCloud,
        );
        assert!(replacement_rules(&[private], CloudIdentifierMode::Include).is_empty());
        assert_eq!(
            replacement_rules(&[never], CloudIdentifierMode::Include).len(),
            2
        );
    }

    #[test]
    fn replacement_does_not_change_substrings_inside_words() {
        let (text, count) = replace_alias("At FINRA, not FINRATED.", "FINRA", "[EMPLOYER_1]");
        assert_eq!(text, "At [EMPLOYER_1], not FINRATED.");
        assert_eq!(count, 1);
    }

    #[test]
    fn replacement_is_ascii_case_insensitive() {
        let (text, count) = replace_alias("finra and FINRA", "FINRA", "[EMPLOYER_1]");
        assert_eq!(text, "[EMPLOYER_1] and [EMPLOYER_1]");
        assert_eq!(count, 2);
    }
}
