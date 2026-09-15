use std::{collections::HashSet, fs, path::Path};

use chrono::Utc;
use uuid::Uuid;

use crate::{
    domain::{
        candidates::{CandidateStatus, StoryCandidate},
        interviews::{
            CompletenessItem, CompletenessStatus, InterviewSession, InterviewStatus,
            InterviewSummary, InterviewTurn, SubmitInterviewResponseRequest, TurnActor, TurnType,
        },
    },
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, write_json_atomic},
    services::guided_development_rules,
};

pub fn start_interview(vault_path: &Path, candidate_id: &str) -> ServiceResult<InterviewSummary> {
    ensure_vault(vault_path)?;
    let mut candidate = read_candidate(vault_path, candidate_id)?;

    if let Some(existing) = find_interview_for_candidate(vault_path, candidate_id)? {
        return summarize(&existing, &candidate);
    }

    let now = Utc::now().to_rfc3339();
    let fields = guided_development_rules::normalized_fields(&candidate.missing_fields);
    let mut interview = InterviewSession {
        schema_version: 1,
        interview_id: format!("interview_{}", Uuid::now_v7()),
        candidate_ids: vec![candidate_id.to_string()],
        story_id: None,
        status: InterviewStatus::Active,
        question_round: 1,
        questions_in_current_round: 0,
        completeness: fields
            .iter()
            .map(|field| CompletenessItem {
                field: field.clone(),
                status: CompletenessStatus::Missing,
                confidence: 0.0,
                notes: String::new(),
            })
            .collect(),
        turns: Vec::new(),
        active_question_turn_id: None,
        created_at: now.clone(),
        updated_at: now,
        revision: 1,
    };
    add_next_question(&mut interview, &candidate.claim);
    write_interview(vault_path, &interview)?;

    candidate.status = CandidateStatus::Interviewing;
    candidate.updated_at = Utc::now().to_rfc3339();
    candidate.revision += 1;
    write_candidate(vault_path, &candidate)?;

    summarize(&interview, &candidate)
}

pub fn list_interviews(vault_path: &Path) -> ServiceResult<Vec<InterviewSummary>> {
    ensure_vault(vault_path)?;
    let mut sessions = read_interviews(vault_path)?;
    sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    let mut summaries = Vec::new();
    for session in sessions {
        let Some(candidate_id) = session.candidate_ids.first() else {
            continue;
        };
        let candidate = read_candidate(vault_path, candidate_id)?;
        summaries.push(summarize(&session, &candidate)?);
    }
    Ok(summaries)
}

pub fn submit_response(
    vault_path: &Path,
    request: SubmitInterviewResponseRequest,
) -> ServiceResult<InterviewSummary> {
    ensure_vault(vault_path)?;
    let path = vault_path
        .join("interviews")
        .join(format!("{}.json", request.interview_id));
    if !path.is_file() {
        return Err(WorkLoreError::InterviewNotFound);
    }

    let mut interview: InterviewSession = read_json(&path)?;
    if interview.status != InterviewStatus::Active {
        return Err(WorkLoreError::InvalidInterviewAction(
            "Only an active interview can accept a response.".to_string(),
        ));
    }

    let active_question_id = interview.active_question_turn_id.clone().ok_or_else(|| {
        WorkLoreError::InvalidInterviewAction("No active question exists.".to_string())
    })?;
    let target_field = interview
        .turns
        .iter()
        .find(|turn| turn.turn_id == active_question_id)
        .and_then(|turn| turn.target_fields.first())
        .cloned()
        .ok_or_else(|| {
            WorkLoreError::InvalidInterviewAction(
                "The active question does not identify a target field.".to_string(),
            )
        })?;

    let now = Utc::now().to_rfc3339();
    let (turn_type, classification, completeness_status, confidence, answer_text) =
        guided_development_rules::response_values(&request)?;
    interview.turns.push(InterviewTurn {
        turn_id: format!("turn_{}", Uuid::now_v7()),
        actor: TurnActor::User,
        turn_type,
        text: answer_text.clone(),
        target_fields: vec![target_field.clone()],
        answer_classification: classification,
        provider_run_id: None,
        created_at: now.clone(),
    });

    if let Some(item) = interview
        .completeness
        .iter_mut()
        .find(|item| item.field == target_field)
    {
        item.status = completeness_status;
        item.confidence = confidence;
        item.notes = answer_text;
    }
    interview.active_question_turn_id = None;
    interview.updated_at = now;
    interview.revision += 1;

    let candidate_id = interview
        .candidate_ids
        .first()
        .cloned()
        .ok_or(WorkLoreError::CandidateNotFound)?;
    let mut candidate = read_candidate(vault_path, &candidate_id)?;

    if next_unasked_field(&interview).is_some() {
        if interview.questions_in_current_round >= 3 {
            interview.question_round += 1;
            interview.questions_in_current_round = 0;
        }
        add_next_question(&mut interview, &candidate.claim);
    } else {
        interview.status = InterviewStatus::ReadyForSynthesis;
        candidate.status = CandidateStatus::Interviewing;
        candidate.updated_at = Utc::now().to_rfc3339();
        candidate.revision += 1;
        write_candidate(vault_path, &candidate)?;
    }

    write_interview(vault_path, &interview)?;
    summarize(&interview, &candidate)
}

pub fn resume_interview(vault_path: &Path, interview_id: &str) -> ServiceResult<InterviewSummary> {
    ensure_vault(vault_path)?;
    let path = vault_path
        .join("interviews")
        .join(format!("{interview_id}.json"));
    if !path.is_file() {
        return Err(WorkLoreError::InterviewNotFound);
    }
    let mut interview: InterviewSession = read_json(&path)?;
    if interview.status == InterviewStatus::Completed
        || interview.status == InterviewStatus::Abandoned
    {
        return Err(WorkLoreError::InvalidInterviewAction(
            "This interview cannot be resumed.".to_string(),
        ));
    }

    let candidate_id = interview
        .candidate_ids
        .first()
        .cloned()
        .ok_or(WorkLoreError::CandidateNotFound)?;
    let candidate = read_candidate(vault_path, &candidate_id)?;
    interview.status = InterviewStatus::Active;
    if interview.active_question_turn_id.is_none() && next_unasked_field(&interview).is_some() {
        add_next_question(&mut interview, &candidate.claim);
    }
    interview.updated_at = Utc::now().to_rfc3339();
    interview.revision += 1;
    write_interview(vault_path, &interview)?;
    summarize(&interview, &candidate)
}

fn add_next_question(interview: &mut InterviewSession, claim: &str) {
    let Some(field) = next_unasked_field(interview) else {
        return;
    };
    let question = guided_development_rules::question_for_field(&field, claim);
    let now = Utc::now().to_rfc3339();
    let turn_id = format!("turn_{}", Uuid::now_v7());
    interview.turns.push(InterviewTurn {
        turn_id: turn_id.clone(),
        actor: TurnActor::Agent,
        turn_type: TurnType::Question,
        text: question,
        target_fields: vec![field],
        answer_classification: None,
        provider_run_id: None,
        created_at: now.clone(),
    });
    interview.active_question_turn_id = Some(turn_id);
    interview.questions_in_current_round += 1;
    interview.updated_at = now;
}

fn next_unasked_field(interview: &InterviewSession) -> Option<String> {
    let asked = interview
        .turns
        .iter()
        .filter(|turn| turn.turn_type == TurnType::Question)
        .flat_map(|turn| turn.target_fields.iter().cloned())
        .collect::<HashSet<_>>();
    interview
        .completeness
        .iter()
        .find(|item| !asked.contains(&item.field))
        .map(|item| item.field.clone())
}

fn summarize(
    interview: &InterviewSession,
    candidate: &StoryCandidate,
) -> ServiceResult<InterviewSummary> {
    let current_question = interview
        .active_question_turn_id
        .as_ref()
        .and_then(|turn_id| interview.turns.iter().find(|turn| &turn.turn_id == turn_id));
    let completed_field_count = interview
        .completeness
        .iter()
        .filter(|item| {
            matches!(
                item.status,
                CompletenessStatus::Sufficient | CompletenessStatus::NotApplicable
            )
        })
        .count();

    Ok(InterviewSummary {
        interview_id: interview.interview_id.clone(),
        candidate_id: candidate.candidate_id.clone(),
        candidate_claim: candidate.claim.clone(),
        status: interview.status,
        current_question: current_question.map(|turn| turn.text.clone()),
        current_target_field: current_question.and_then(|turn| turn.target_fields.first().cloned()),
        completed_field_count,
        total_field_count: interview.completeness.len(),
        last_updated_at: interview.updated_at.clone(),
    })
}

fn find_interview_for_candidate(
    vault_path: &Path,
    candidate_id: &str,
) -> ServiceResult<Option<InterviewSession>> {
    Ok(read_interviews(vault_path)?.into_iter().find(|interview| {
        interview.candidate_ids.iter().any(|id| id == candidate_id)
            && interview.status != InterviewStatus::Abandoned
    }))
}

fn read_interviews(vault_path: &Path) -> ServiceResult<Vec<InterviewSession>> {
    let directory = vault_path.join("interviews");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut sessions = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
        {
            sessions.push(read_json(&entry.path())?);
        }
    }
    Ok(sessions)
}

fn read_candidate(vault_path: &Path, candidate_id: &str) -> ServiceResult<StoryCandidate> {
    let path = vault_path
        .join("candidates")
        .join(format!("{candidate_id}.json"));
    if !path.is_file() {
        return Err(WorkLoreError::CandidateNotFound);
    }
    read_json(&path)
}

fn write_candidate(vault_path: &Path, candidate: &StoryCandidate) -> ServiceResult<()> {
    write_json_atomic(
        &vault_path
            .join("candidates")
            .join(format!("{}.json", candidate.candidate_id)),
        candidate,
    )
}

fn write_interview(vault_path: &Path, interview: &InterviewSession) -> ServiceResult<()> {
    write_json_atomic(
        &vault_path
            .join("interviews")
            .join(format!("{}.json", interview.interview_id)),
        interview,
    )
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

    #[test]
    fn question_map_keeps_work_grounded_in_the_claim() {
        let question = guided_development_rules::question_for_field(
            "metrics_and_evidence",
            "Reduced complaint investigation time from months to hours.",
        );
        assert!(question.contains("Reduced complaint investigation time"));
        assert!(question.contains("exact metrics"));
    }

    #[test]
    fn normalized_fields_keep_required_interview_coverage() {
        let fields =
            guided_development_rules::normalized_fields(&["metrics_and_evidence".to_string()]);
        assert!(fields.contains(&"constraints".to_string()));
        assert_eq!(
            fields
                .iter()
                .filter(|field| field.as_str() == "metrics_and_evidence")
                .count(),
            1
        );
    }
}
