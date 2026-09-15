use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{
    domain::{
        candidates::{CandidateStatus, StoryCandidate},
        interviews::{AnswerClassification, InterviewSession, InterviewStatus, TurnActor},
        models::{EntityReviewItem, PrivacyScanStatus},
        stories::{
            ClaimClassification, EvidenceType, ImportStoryResponseResult, StoryClaim, StoryContent,
            StoryDisclosure, StoryEvidence, StoryPrivacyScan, StoryRecord, StoryStatus,
            StorySummary, StorySynthesisResponse, StoryType, SynthesisEvidenceLevel,
        },
    },
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, write_json_atomic},
    services::{
        contextual_entity_scan::scan_named_projects,
        entity_scan::{load_registry, scan_text},
        performance_service::OperationSession,
        role_service,
    },
};

pub fn import_synthesis_response(
    vault_path: &Path,
    interview_id: &str,
    response_path: &Path,
) -> ServiceResult<ImportStoryResponseResult> {
    ensure_vault(vault_path)?;
    let mut metadata = Map::new();
    metadata.insert(
        "interviewId".to_string(),
        Value::String(interview_id.to_string()),
    );
    metadata.insert(
        "responseExtension".to_string(),
        Value::String(
            response_path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase(),
        ),
    );
    let mut operation = OperationSession::start(vault_path, "import_story_response", metadata)?;

    let result = (|| {
        operation.set_phase("reading_response")?;
        if !response_path.is_file() {
            return Err(WorkLoreError::InvalidStoryResponse(
                "the selected response file does not exist".to_string(),
            ));
        }
        let response: StorySynthesisResponse = read_json(response_path).map_err(|error| {
            WorkLoreError::InvalidStoryResponse(format!(
                "the response did not match the required JSON contract: {error}"
            ))
        })?;
        validate_response(&response)?;

        operation.set_phase("loading_evidence")?;
        let mut interview = read_interview(vault_path, interview_id)?;
        if !matches!(
            interview.status,
            InterviewStatus::ReadyForSynthesis | InterviewStatus::Completed
        ) {
            return Err(WorkLoreError::InvalidStoryResponse(
                "complete the guided interview pass before importing a story response".to_string(),
            ));
        }
        let candidate_id = interview
            .candidate_ids
            .first()
            .cloned()
            .ok_or(WorkLoreError::CandidateNotFound)?;
        let mut candidate = read_candidate(vault_path, &candidate_id)?;

        operation.set_phase("resolving_role")?;
        let mut role = role_service::ensure_role_for_candidate(vault_path, &candidate)?;
        if !candidate.context.role_ids.contains(&role.role_id) {
            candidate.context.role_ids.push(role.role_id.clone());
        }

        operation.set_phase("building_story")?;
        let now = Utc::now().to_rfc3339();
        let existing = interview
            .story_id
            .as_deref()
            .and_then(|story_id| read_story(vault_path, story_id).ok());
        let story_id = existing
            .as_ref()
            .map(|story| story.story_id.clone())
            .unwrap_or_else(|| format!("story_{}", Uuid::now_v7()));
        let created = existing.is_none();
        let revision = existing.as_ref().map_or(1, |story| story.revision + 1);
        let created_at = existing
            .as_ref()
            .map(|story| story.created_at.clone())
            .unwrap_or_else(|| now.clone());

        let (evidence, field_evidence) = build_evidence(&candidate, &interview);
        let claims = build_claims(&response, &field_evidence);
        let mut entity_ids = candidate.context.entity_ids.clone();
        entity_ids.push(role.organization_entity_id.clone());
        entity_ids.sort();
        entity_ids.dedup();

        let mut actions_and_decisions = response.actions.clone();
        actions_and_decisions.extend(response.decisions.clone());
        normalize_strings(&mut actions_and_decisions);
        let mut metrics = response
            .metrics
            .iter()
            .map(|metric| metric.statement.clone())
            .collect::<Vec<_>>();
        normalize_strings(&mut metrics);

        let content = StoryContent {
            summary: response.summary.trim().to_string(),
            situation: response.situation.trim().to_string(),
            problem_or_opportunity: response.problem.trim().to_string(),
            responsibilities: cleaned(response.responsibilities.clone()),
            constraints: cleaned(response.constraints.clone()),
            actions_and_decisions,
            alternatives_considered: Vec::new(),
            tools_and_systems: cleaned(response.tools_and_systems.clone()),
            stakeholders: cleaned(response.stakeholders.clone()),
            outcomes: cleaned(response.outcomes.clone()),
            metrics,
            lessons_learned: cleaned(response.lessons_learned.clone()),
            operating_philosophy: cleaned(response.operating_philosophy.clone()),
            reusable_themes: candidate.context.skills.clone(),
            skills_demonstrated: candidate.context.skills.clone(),
        };

        let mut story = StoryRecord {
            schema_version: 1,
            story_id: story_id.clone(),
            title: response.title.trim().to_string(),
            status: StoryStatus::ReadyForReview,
            story_type: infer_story_type(&response),
            role_ids: vec![role.role_id.clone()],
            entity_ids,
            related_story_ids: existing
                .as_ref()
                .map(|story| story.related_story_ids.clone())
                .unwrap_or_default(),
            job_requirement_ids: existing
                .as_ref()
                .map(|story| story.job_requirement_ids.clone())
                .unwrap_or_default(),
            content,
            claims,
            evidence,
            disclosure: existing
                .as_ref()
                .map(|story| story.disclosure.clone())
                .unwrap_or(StoryDisclosure {
                    default_public_mode: "stable_tokens".to_string(),
                    review_required: true,
                    notes: String::new(),
                }),
            privacy_scan: StoryPrivacyScan {
                status: PrivacyScanStatus::Pending,
                content_revision: revision,
                scanned_at: None,
                review_item_ids: Vec::new(),
            },
            tags: existing
                .as_ref()
                .map(|story| story.tags.clone())
                .unwrap_or_default(),
            created_at,
            updated_at: now.clone(),
            revision,
        };

        operation.set_phase("scanning_story_privacy")?;
        let scan_text_value = story_text_for_scan(&story);
        let base_scan = scan_text(vault_path, "story", &story_id, &scan_text_value)?;
        let contextual_scan =
            scan_named_projects(vault_path, "story", &story_id, &scan_text_value)?;
        let mut review_item_ids = base_scan.review_item_ids;
        review_item_ids.extend(contextual_scan.review_item_ids);
        review_item_ids.extend(pending_story_reviews(vault_path, &story_id)?);
        review_item_ids.sort();
        review_item_ids.dedup();
        story.privacy_scan = StoryPrivacyScan {
            status: if review_item_ids.is_empty() {
                PrivacyScanStatus::Complete
            } else {
                PrivacyScanStatus::NeedsReview
            },
            content_revision: revision,
            scanned_at: Some(contextual_scan.scanned_at),
            review_item_ids,
        };

        operation.set_phase("writing_story_pair")?;
        write_story_pair(vault_path, &story)?;

        operation.set_phase("linking_records")?;
        if !role.story_ids.contains(&story_id) {
            role.story_ids.push(story_id.clone());
            role.updated_at = now.clone();
            role.revision += 1;
            role_service::save_role(vault_path, &role)?;
        }
        candidate.status = CandidateStatus::ConvertedToStory;
        candidate.updated_at = now.clone();
        candidate.revision += 1;
        write_json_atomic(
            &candidate_path(vault_path, &candidate.candidate_id),
            &candidate,
        )?;

        interview.story_id = Some(story_id);
        interview.status = InterviewStatus::Completed;
        interview.updated_at = now;
        interview.revision += 1;
        write_json_atomic(
            &interview_path(vault_path, &interview.interview_id),
            &interview,
        )?;

        let summary = summarize_story(vault_path, &story)?;
        Ok(ImportStoryResponseResult {
            story: summary,
            created,
            message: if created {
                "Story response validated and saved as a reviewable WorkLore story.".to_string()
            } else {
                "Story response validated and the existing WorkLore story was revised.".to_string()
            },
        })
    })();

    operation.finish(result)
}

pub fn list_stories(vault_path: &Path) -> ServiceResult<Vec<StorySummary>> {
    ensure_vault(vault_path)?;
    let mut stories = read_stories(vault_path)?;
    stories.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    stories
        .iter()
        .map(|story| summarize_story(vault_path, story))
        .collect()
}

pub fn set_story_status(
    vault_path: &Path,
    story_id: &str,
    status: StoryStatus,
) -> ServiceResult<StorySummary> {
    let mut story = read_story(vault_path, story_id)?;
    story.status = status;
    story.updated_at = Utc::now().to_rfc3339();
    story.revision += 1;
    write_story_pair(vault_path, &story)?;
    summarize_story(vault_path, &story)
}

pub fn read_story(vault_path: &Path, story_id: &str) -> ServiceResult<StoryRecord> {
    let path = story_json_path(vault_path, story_id);
    if !path.is_file() {
        return Err(WorkLoreError::StoryNotFound);
    }
    read_json(&path)
}

fn validate_response(response: &StorySynthesisResponse) -> ServiceResult<()> {
    if response.title.trim().is_empty() {
        return Err(WorkLoreError::InvalidStoryResponse(
            "title cannot be empty".to_string(),
        ));
    }
    if response.title.chars().count() > 240 {
        return Err(WorkLoreError::InvalidStoryResponse(
            "title cannot exceed 240 characters".to_string(),
        ));
    }
    if response
        .claims
        .iter()
        .any(|claim| claim.statement.trim().is_empty())
    {
        return Err(WorkLoreError::InvalidStoryResponse(
            "claim statements cannot be empty".to_string(),
        ));
    }
    if response
        .metrics
        .iter()
        .any(|metric| metric.statement.trim().is_empty())
    {
        return Err(WorkLoreError::InvalidStoryResponse(
            "metric statements cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn build_evidence(
    candidate: &StoryCandidate,
    interview: &InterviewSession,
) -> (Vec<StoryEvidence>, HashMap<String, Vec<String>>) {
    let mut evidence = Vec::new();
    let mut field_evidence: HashMap<String, Vec<String>> = HashMap::new();

    for reference in &candidate.source_refs {
        let evidence_id = format!("evidence_{}", Uuid::now_v7());
        evidence.push(StoryEvidence {
            evidence_id: evidence_id.clone(),
            evidence_type: EvidenceType::SourceFragment,
            source_id: Some(reference.source_id.clone()),
            locator: reference.fragment_id.clone(),
            captured_text: reference.captured_text.clone(),
            captured_at: None,
        });
        field_evidence
            .entry("source_claim".to_string())
            .or_default()
            .push(evidence_id);
    }

    for turn in interview
        .turns
        .iter()
        .filter(|turn| turn.actor == TurnActor::User)
    {
        let evidence_id = format!("evidence_{}", Uuid::now_v7());
        let evidence_type =
            if turn.answer_classification == Some(AnswerClassification::ConfirmedFact) {
                EvidenceType::UserConfirmation
            } else {
                EvidenceType::InterviewAnswer
            };
        evidence.push(StoryEvidence {
            evidence_id: evidence_id.clone(),
            evidence_type,
            source_id: None,
            locator: format!("interview:{}/turn:{}", interview.interview_id, turn.turn_id),
            captured_text: turn.text.clone(),
            captured_at: Some(turn.created_at.clone()),
        });
        for field in &turn.target_fields {
            field_evidence
                .entry(field.clone())
                .or_default()
                .push(evidence_id.clone());
        }
    }

    (evidence, field_evidence)
}

fn build_claims(
    response: &StorySynthesisResponse,
    field_evidence: &HashMap<String, Vec<String>>,
) -> Vec<StoryClaim> {
    let mut source_claims = response.claims.clone();
    for metric in &response.metrics {
        if !source_claims
            .iter()
            .any(|claim| normalize(&claim.statement) == normalize(&metric.statement))
        {
            source_claims.push(crate::domain::stories::SynthesisClaim {
                statement: metric.statement.clone(),
                evidence_level: metric.evidence_level,
                supporting_fields: vec!["metrics_and_evidence".to_string()],
            });
        }
    }

    source_claims
        .into_iter()
        .map(|claim| {
            let mut evidence_ids = claim
                .supporting_fields
                .iter()
                .flat_map(|field| field_evidence.get(field).cloned().unwrap_or_default())
                .collect::<Vec<_>>();
            evidence_ids.sort();
            evidence_ids.dedup();
            let (mut classification, mut confidence, mut notes) =
                claim_classification(claim.evidence_level);
            if evidence_ids.is_empty() {
                classification = ClaimClassification::Unsupported;
                confidence = 0.0;
                notes = "No matching local evidence field was found for this claim.".to_string();
            }
            StoryClaim {
                claim_id: format!("claim_{}", Uuid::now_v7()),
                text: claim.statement.trim().to_string(),
                classification,
                confidence,
                evidence_ids,
                included_by_default: classification != ClaimClassification::Unsupported,
                notes,
            }
        })
        .collect()
}

fn claim_classification(level: SynthesisEvidenceLevel) -> (ClaimClassification, f32, String) {
    match level {
        SynthesisEvidenceLevel::ConfirmedFact => {
            (ClaimClassification::ConfirmedFact, 1.0, String::new())
        }
        SynthesisEvidenceLevel::UserEstimate => (
            ClaimClassification::UserEstimate,
            0.75,
            "The user identified this as a reasonable estimate.".to_string(),
        ),
        SynthesisEvidenceLevel::Uncertain => (
            ClaimClassification::Unsupported,
            0.35,
            "The user or synthesis response identified this as uncertain memory.".to_string(),
        ),
        SynthesisEvidenceLevel::Unsupported => (
            ClaimClassification::Unsupported,
            0.0,
            "The synthesis response did not provide supporting evidence.".to_string(),
        ),
    }
}

fn infer_story_type(response: &StorySynthesisResponse) -> StoryType {
    let text = format!(
        "{} {} {}",
        response.title, response.summary, response.problem
    )
    .to_ascii_lowercase();
    if text.contains("failure") || text.contains("failed") {
        StoryType::Failure
    } else if text.contains("lesson") || text.contains("learned") {
        StoryType::Lesson
    } else if text.contains("conflict") || text.contains("resistance") {
        StoryType::Conflict
    } else if text.contains("decision") || !response.decisions.is_empty() {
        StoryType::Decision
    } else if text.contains("process") || text.contains("workflow") {
        StoryType::ProcessChange
    } else if text.contains("technical") || text.contains("system") {
        StoryType::TechnicalDelivery
    } else if text.contains("leadership") || text.contains("led ") {
        StoryType::Leadership
    } else {
        StoryType::Accomplishment
    }
}

fn story_text_for_scan(story: &StoryRecord) -> String {
    let mut sections = vec![
        story.title.clone(),
        story.content.summary.clone(),
        story.content.situation.clone(),
        story.content.problem_or_opportunity.clone(),
    ];
    sections.extend(story.content.responsibilities.clone());
    sections.extend(story.content.constraints.clone());
    sections.extend(story.content.actions_and_decisions.clone());
    sections.extend(story.content.tools_and_systems.clone());
    sections.extend(story.content.stakeholders.clone());
    sections.extend(story.content.outcomes.clone());
    sections.extend(story.content.metrics.clone());
    sections.extend(story.content.lessons_learned.clone());
    sections.extend(story.content.operating_philosophy.clone());
    sections.extend(story.claims.iter().map(|claim| claim.text.clone()));
    sections.join("\n")
}

fn summarize_story(vault_path: &Path, story: &StoryRecord) -> ServiceResult<StorySummary> {
    let role = story
        .role_ids
        .first()
        .and_then(|role_id| role_service::read_role(vault_path, role_id).ok());
    let registry = load_registry(vault_path)?;
    let organization_name = role.as_ref().and_then(|role| {
        registry
            .entities
            .iter()
            .find(|entity| entity.entity_id == role.organization_entity_id)
            .map(|entity| entity.canonical_name.clone())
    });
    Ok(StorySummary {
        story_id: story.story_id.clone(),
        title: story.title.clone(),
        status: story.status,
        story_type: story.story_type,
        summary: story.content.summary.clone(),
        role_id: role.as_ref().map(|role| role.role_id.clone()),
        role_title: role.as_ref().map(|role| role.title.clone()),
        organization_name,
        metrics: story.content.metrics.clone(),
        outcomes: story.content.outcomes.clone(),
        privacy_scan_status: story.privacy_scan.status,
        updated_at: story.updated_at.clone(),
        revision: story.revision,
    })
}

fn pending_story_reviews(vault_path: &Path, story_id: &str) -> ServiceResult<Vec<String>> {
    let directory = vault_path.join("privacy/review-items");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut ids = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let item: EntityReviewItem = read_json(&entry.path())?;
        if item.record_id == story_id && item.status == "pending" {
            ids.push(item.review_item_id);
        }
    }
    Ok(ids)
}

fn write_story_pair(vault_path: &Path, story: &StoryRecord) -> ServiceResult<()> {
    let json_path = story_json_path(vault_path, &story.story_id);
    let markdown_path = story_markdown_path(vault_path, &story.story_id);
    let directory = json_path.parent().ok_or(WorkLoreError::InvalidPath)?;
    fs::create_dir_all(directory)?;

    let transaction_id = Uuid::now_v7();
    let json_temp = directory.join(format!(".{}.{}.json.tmp", story.story_id, transaction_id));
    let markdown_temp = directory.join(format!(".{}.{}.md.tmp", story.story_id, transaction_id));
    let json_backup = directory.join(format!(".{}.json.previous", story.story_id));
    let markdown_backup = directory.join(format!(".{}.md.previous", story.story_id));

    write_synced(&json_temp, &serde_json::to_vec_pretty(story)?)?;
    write_synced(&markdown_temp, render_markdown(story).as_bytes())?;

    replace_with_backup(&json_path, &json_backup)?;
    if let Err(error) = replace_with_backup(&markdown_path, &markdown_backup) {
        restore_backup(&json_path, &json_backup);
        let _ = fs::remove_file(&json_temp);
        let _ = fs::remove_file(&markdown_temp);
        return Err(error);
    }

    if let Err(error) = fs::rename(&json_temp, &json_path) {
        restore_backup(&json_path, &json_backup);
        restore_backup(&markdown_path, &markdown_backup);
        let _ = fs::remove_file(&markdown_temp);
        return Err(error.into());
    }
    if let Err(error) = fs::rename(&markdown_temp, &markdown_path) {
        let _ = fs::remove_file(&json_path);
        restore_backup(&json_path, &json_backup);
        restore_backup(&markdown_path, &markdown_backup);
        return Err(error.into());
    }

    remove_if_exists(&json_backup)?;
    remove_if_exists(&markdown_backup)?;
    Ok(())
}

fn render_markdown(story: &StoryRecord) -> String {
    let title = serde_json::to_string(&story.title).unwrap_or_else(|_| "\"Untitled\"".to_string());
    let roles = story
        .role_ids
        .iter()
        .map(|role_id| format!("  - {role_id}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "---\nstoryId: {}\ntitle: {}\nstatus: {}\nstoryType: {}\nrevision: {}\nroleIds:\n{}\n---\n\n# {}\n\n{}\n\n## Situation\n\n{}\n\n## Problem or Opportunity\n\n{}\n\n## Responsibilities\n\n{}\n\n## Constraints\n\n{}\n\n## Actions and Decisions\n\n{}\n\n## Tools and Systems\n\n{}\n\n## Stakeholders\n\n{}\n\n## Outcomes\n\n{}\n\n## Metrics\n\n{}\n\n## Lessons Learned\n\n{}\n\n## Operating Philosophy\n\n{}\n\n## Claims\n\n{}\n",
        story.story_id,
        title,
        enum_label(story.status),
        enum_label(story.story_type),
        story.revision,
        roles,
        story.title,
        story.content.summary,
        story.content.situation,
        story.content.problem_or_opportunity,
        markdown_list(&story.content.responsibilities),
        markdown_list(&story.content.constraints),
        markdown_list(&story.content.actions_and_decisions),
        markdown_list(&story.content.tools_and_systems),
        markdown_list(&story.content.stakeholders),
        markdown_list(&story.content.outcomes),
        markdown_list(&story.content.metrics),
        markdown_list(&story.content.lessons_learned),
        markdown_list(&story.content.operating_philosophy),
        story
            .claims
            .iter()
            .map(|claim| {
                format!(
                    "- [{} | {:.0}%] {}",
                    enum_label(claim.classification),
                    claim.confidence * 100.0,
                    claim.text
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn enum_label<T: std::fmt::Debug>(value: T) -> String {
    format!("{value:?}").to_ascii_lowercase()
}

fn markdown_list(values: &[String]) -> String {
    if values.is_empty() {
        "_Not captured._".to_string()
    } else {
        values
            .iter()
            .map(|value| format!("- {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn write_synced(path: &Path, bytes: &[u8]) -> ServiceResult<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn replace_with_backup(path: &Path, backup: &Path) -> ServiceResult<()> {
    remove_if_exists(backup)?;
    if path.exists() {
        fs::rename(path, backup)?;
    }
    Ok(())
}

fn restore_backup(path: &Path, backup: &Path) {
    if backup.exists() {
        let _ = fs::remove_file(path);
        let _ = fs::rename(backup, path);
    }
}

fn remove_if_exists(path: &Path) -> ServiceResult<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn read_stories(vault_path: &Path) -> ServiceResult<Vec<StoryRecord>> {
    let directory = vault_path.join("stories");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut stories = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json")
            && !path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with('.'))
        {
            stories.push(read_json(&path)?);
        }
    }
    Ok(stories)
}

fn read_interview(vault_path: &Path, interview_id: &str) -> ServiceResult<InterviewSession> {
    let path = interview_path(vault_path, interview_id);
    if !path.is_file() {
        return Err(WorkLoreError::InterviewNotFound);
    }
    read_json(&path)
}

fn read_candidate(vault_path: &Path, candidate_id: &str) -> ServiceResult<StoryCandidate> {
    let path = candidate_path(vault_path, candidate_id);
    if !path.is_file() {
        return Err(WorkLoreError::CandidateNotFound);
    }
    read_json(&path)
}

fn story_json_path(vault_path: &Path, story_id: &str) -> PathBuf {
    vault_path.join("stories").join(format!("{story_id}.json"))
}

fn story_markdown_path(vault_path: &Path, story_id: &str) -> PathBuf {
    vault_path.join("stories").join(format!("{story_id}.md"))
}

fn interview_path(vault_path: &Path, interview_id: &str) -> PathBuf {
    vault_path
        .join("interviews")
        .join(format!("{interview_id}.json"))
}

fn candidate_path(vault_path: &Path, candidate_id: &str) -> PathBuf {
    vault_path
        .join("candidates")
        .join(format!("{candidate_id}.json"))
}

fn cleaned(values: Vec<String>) -> Vec<String> {
    let mut values = values;
    normalize_strings(&mut values);
    values
}

fn normalize_strings(values: &mut Vec<String>) {
    for value in values.iter_mut() {
        *value = value.trim().to_string();
    }
    values.retain(|value| !value.is_empty());
    let mut seen = HashSet::new();
    values.retain(|value| seen.insert(normalize(value)));
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn ensure_vault(vault_path: &Path) -> ServiceResult<()> {
    if !vault_path.join("vault.json").is_file() {
        return Err(WorkLoreError::NotAVault);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::stories::{SynthesisClaim, SynthesisMetric};

    fn response() -> StorySynthesisResponse {
        StorySynthesisResponse {
            title: "Reduced investigation time".to_string(),
            summary: "A workflow redesign.".to_string(),
            situation: "Investigations were slow.".to_string(),
            problem: "Analysts lacked a reliable intake path.".to_string(),
            constraints: vec!["Regulated environment".to_string()],
            responsibilities: vec!["Owned product direction".to_string()],
            actions: vec!["Mapped the workflow".to_string()],
            decisions: vec!["Added human review".to_string()],
            tools_and_systems: vec!["LLM".to_string()],
            stakeholders: vec!["Analysts".to_string()],
            outcomes: vec!["Faster review".to_string()],
            metrics: vec![SynthesisMetric {
                statement: "Months to hours".to_string(),
                evidence_level: SynthesisEvidenceLevel::ConfirmedFact,
            }],
            lessons_learned: vec!["Fix workflow logic first".to_string()],
            operating_philosophy: vec!["Analytics should expose broken logic".to_string()],
            claims: vec![SynthesisClaim {
                statement: "Reduced investigation time from months to hours".to_string(),
                evidence_level: SynthesisEvidenceLevel::ConfirmedFact,
                supporting_fields: vec!["metrics_and_evidence".to_string()],
            }],
        }
    }

    #[test]
    fn validates_required_story_title() {
        let mut value = response();
        value.title.clear();
        assert!(validate_response(&value).is_err());
    }

    #[test]
    fn decision_evidence_classifies_story_as_decision() {
        assert_eq!(infer_story_type(&response()), StoryType::Decision);
    }

    #[test]
    fn normalization_removes_empty_and_duplicate_values() {
        let mut values = vec![" SQL ".to_string(), "sql".to_string(), "".to_string()];
        normalize_strings(&mut values);
        assert_eq!(values, vec!["SQL"]);
    }
}
