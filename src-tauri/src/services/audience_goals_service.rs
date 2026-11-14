use std::{collections::BTreeMap, path::Path, time::Duration};

use rusqlite::{Connection, OptionalExtension};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, write_json_atomic},
};

const AUDIENCE_GOALS_FILE: &str = "data/audience-goals.json";
const POST_AUDIENCE_INTENTS_FILE: &str = "data/post-audience-intents.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AudienceGoals {
    #[serde(default)]
    pub primary_audience: String,
    #[serde(default)]
    pub secondary_audiences: Vec<String>,
    #[serde(default)]
    pub target_roles: Vec<String>,
    #[serde(default)]
    pub target_industries: Vec<String>,
    #[serde(default)]
    pub positioning: String,
    #[serde(default)]
    pub job_search_disclosure: JobSearchDisclosure,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobSearchDisclosure {
    #[default]
    DoNotMention,
    MayMention,
}

fn goals_path(vault_path: &Path) -> ServiceResult<std::path::PathBuf> {
    // Never create records in arbitrary paths just because the frontend passes a string.
    if !vault_path.join("vault.json").is_file() {
        return Err(WorkLoreError::NotAVault);
    }
    Ok(vault_path.join(AUDIENCE_GOALS_FILE))
}

pub fn get(vault_path: &Path) -> ServiceResult<AudienceGoals> {
    let path = goals_path(vault_path)?;
    if !path.exists() {
        return Ok(AudienceGoals::default());
    }
    read_json(&path)
}

pub fn save(vault_path: &Path, mut goals: AudienceGoals) -> ServiceResult<AudienceGoals> {
    let path = goals_path(vault_path)?;
    goals.primary_audience = limit_text(&goals.primary_audience, 200, "Primary audience")?;
    goals.positioning = limit_text(&goals.positioning, 2000, "Positioning")?;
    goals.secondary_audiences = clean_list(goals.secondary_audiences)?;
    goals.target_roles = clean_list(goals.target_roles)?;
    goals.target_industries = clean_list(goals.target_industries)?;
    goals.updated_at = Utc::now().to_rfc3339();
    write_json_atomic(&path, &goals)?;
    Ok(goals)
}

fn limit_text(value: &str, max_chars: usize, label: &str) -> ServiceResult<String> {
    let value = value.trim();
    if value.chars().count() > max_chars {
        return Err(WorkLoreError::InvalidVault(format!(
            "{label} exceeds {max_chars} characters."
        )));
    }
    Ok(value.to_string())
}

fn clean_list(values: Vec<String>) -> ServiceResult<Vec<String>> {
    if values.len() > 30 {
        return Err(WorkLoreError::InvalidVault("Too many audience items.".to_string()));
    }
    let mut result = Vec::new();
    for value in values {
        let normalized = limit_text(&value, 200, "Audience item")?;
        if !normalized.is_empty() && !result.iter().any(|existing: &String| existing.eq_ignore_ascii_case(&normalized)) {
            result.push(normalized);
        }
    }
    Ok(result)
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PostAudienceIntent {
    pub post_id: String,
    pub primary_audience: String,
    pub desired_signal: String,
    #[serde(default)]
    pub updated_at: String,
}

fn require_existing_post(vault_path: &Path, post_id: &str) -> ServiceResult<()> {
    let _ = goals_path(vault_path)?;
    let connection = Connection::open(crate::services::canonical_store::database_path(vault_path))?;
    connection.busy_timeout(Duration::from_secs(5))?;
    let found: Option<String> = connection
        .query_row("SELECT post_id FROM posts WHERE post_id=?1", [post_id], |row| row.get(0))
        .optional()?;
    if found.is_none() {
        return Err(WorkLoreError::InvalidVault("Selected post does not exist in this vault.".into()));
    }
    Ok(())
}

fn intent_file(vault_path: &Path) -> ServiceResult<std::path::PathBuf> {
    let _ = goals_path(vault_path)?;
    Ok(vault_path.join(POST_AUDIENCE_INTENTS_FILE))
}

fn read_intents(vault_path: &Path) -> ServiceResult<BTreeMap<String, PostAudienceIntent>> {
    let path = intent_file(vault_path)?;
    if path.exists() {
        read_json(&path)
    } else {
        Ok(BTreeMap::new())
    }
}

pub fn get_post_intent(vault_path: &Path, post_id: &str) -> ServiceResult<Option<PostAudienceIntent>> {
    require_existing_post(vault_path, post_id)?;
    Ok(read_intents(vault_path)?.remove(post_id))
}

pub fn save_post_intent(
    vault_path: &Path,
    mut intent: PostAudienceIntent,
) -> ServiceResult<PostAudienceIntent> {
    require_existing_post(vault_path, &intent.post_id)?;
    intent.primary_audience = limit_text(&intent.primary_audience, 200, "Post audience")?;
    intent.desired_signal = limit_text(&intent.desired_signal, 1000, "Desired reader signal")?;
    if intent.primary_audience.is_empty() {
        return Err(WorkLoreError::InvalidVault("Choose a primary post audience.".into()));
    }
    let mut intents = read_intents(vault_path)?;
    intent.updated_at = Utc::now().to_rfc3339();
    intents.insert(intent.post_id.clone(), intent.clone());
    write_json_atomic(&intent_file(vault_path)?, &intents)?;
    Ok(intent)
}

pub fn clear_post_intent(vault_path: &Path, post_id: &str) -> ServiceResult<()> {
    require_existing_post(vault_path, post_id)?;
    let mut intents = read_intents(vault_path)?;
    if intents.remove(post_id).is_some() {
        write_json_atomic(&intent_file(vault_path)?, &intents)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::vault_service;
    use uuid::Uuid;

    #[test]
    fn defaults_and_round_trip_are_vault_local() {
        let path = std::env::temp_dir().join(format!("worklore-audiences-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Audience QA").unwrap();
        assert_eq!(get(&path).unwrap(), AudienceGoals::default());
        let saved = save(&path, AudienceGoals {
            primary_audience: " Hiring managers ".into(),
            target_roles: vec!["Product Director".into(), "product director".into()],
            positioning: "Show how I lead change".into(),
            job_search_disclosure: JobSearchDisclosure::MayMention,
            ..AudienceGoals::default()
        }).unwrap();
        assert_eq!(saved.primary_audience, "Hiring managers");
        assert_eq!(saved.target_roles, vec!["Product Director"]);
        assert_eq!(get(&path).unwrap(), saved);
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejects_non_vault_and_oversized_input() {
        let path = std::env::temp_dir().join(format!("worklore-missing-audience-{}", Uuid::now_v7()));
        assert!(get(&path).is_err());
        assert!(save(&path, AudienceGoals::default()).is_err());
        let path = std::env::temp_dir().join(format!("worklore-audiences-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Audience Limits").unwrap();
        assert!(save(&path, AudienceGoals { primary_audience: "x".repeat(201), ..AudienceGoals::default() }).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn post_override_round_trip_and_clear_do_not_change_vault_goals() {
        use crate::domain::posts::{CreatePostRequest, PostRevisionAuthorship, PostRevisionOrigin};
        use crate::services::post_lineage_service;
        let path = std::env::temp_dir().join(format!("worklore-post-audience-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Post Audience").unwrap();
        save(&path, AudienceGoals { primary_audience: "Peers".into(), ..AudienceGoals::default() }).unwrap();
        let post = post_lineage_service::create_post(&path, CreatePostRequest {
            title: "Example".into(), text: "Synthetic sample post".into(),
            origin: PostRevisionOrigin::User,
            authorship_state: PostRevisionAuthorship::UserAuthored,
            provider_run_id: None, provider_id: None, model_id: None,
        }).unwrap();
        let post_id = post.post.post_id;
        assert!(get_post_intent(&path, &post_id).unwrap().is_none());
        let stored = save_post_intent(&path, PostAudienceIntent {
            post_id: post_id.clone(), primary_audience: "Executives".into(),
            desired_signal: "Sound judgment".into(), updated_at: String::new(),
        }).unwrap();
        assert_eq!(get_post_intent(&path, &post_id).unwrap(), Some(stored));
        clear_post_intent(&path, &post_id).unwrap();
        assert!(get_post_intent(&path, &post_id).unwrap().is_none());
        assert_eq!(get(&path).unwrap().primary_audience, "Peers");
        std::fs::remove_dir_all(path).unwrap();
    }

}
