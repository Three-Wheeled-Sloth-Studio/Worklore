use std::{fs, path::Path};

use chrono::Utc;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{
    domain::{
        candidates::StoryCandidate,
        interviews::{InterviewSession, InterviewStatus, TurnActor},
        providers::{
            CreateManualWorkspaceRequest, ManualWorkspaceResult, ManualWorkspaceTarget,
        },
    },
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, write_json_atomic},
    services::{
        performance_service::OperationSession, redaction_service::redact_for_external_use,
    },
};

pub fn create_manual_workspace(
    vault_path: &Path,
    request: CreateManualWorkspaceRequest,
) -> ServiceResult<ManualWorkspaceResult> {
    ensure_vault(vault_path)?;
    let mut metadata = Map::new();
    metadata.insert(
        "interviewId".to_string(),
        Value::String(request.interview_id.clone()),
    );
    metadata.insert(
        "target".to_string(),
        Value::String(target_slug(request.target).to_string()),
    );
    let mut operation = OperationSession::start(vault_path, "create_manual_workspace", metadata)?;

    let result = (|| {
        operation.set_phase("loading_interview")?;
        let interview = read_interview(vault_path, &request.interview_id)?;
        if interview.status != InterviewStatus::ReadyForSynthesis
            && interview.status != InterviewStatus::Completed
        {
            return Err(WorkLoreError::ManualWorkspace(
                "Complete the current interview pass before exporting a synthesis workspace."
                    .to_string(),
            ));
        }
        let candidate_id = interview
            .candidate_ids
            .first()
            .ok_or(WorkLoreError::CandidateNotFound)?;
        let candidate = read_candidate(vault_path, candidate_id)?;

        operation.set_phase("assembling_context")?;
        let exact_context = build_context(&candidate, &interview);

        operation.set_phase("privacy_preflight")?;
        let redacted = redact_for_external_use(vault_path, &exact_context)?;
        let privacy_mode = redacted.preflight.mode.clone();
        let replacement_count = redacted.preflight.replacement_count;
        let warning_count = redacted.preflight.warning_review_item_ids.len();

        operation.set_phase("creating_workspace")?;
        let output_root = Path::new(&request.output_directory);
        fs::create_dir_all(output_root)?;
        let workspace_id = format!("workspace_{}", Uuid::now_v7());
        let stamp = Utc::now().format("%Y%m%d-%H%M%S");
        let workspace_path = output_root.join(format!(
            "worklore-{}-{stamp}-{}",
            target_slug(request.target),
            short_id(&workspace_id)
        ));
        if workspace_path.exists() {
            return Err(WorkLoreError::ManualWorkspace(
                "The generated workspace folder already exists.".to_string(),
            ));
        }
        fs::create_dir_all(&workspace_path)?;

        operation.set_phase("writing_workspace")?;
        fs::write(
            workspace_path.join("README.md"),
            workspace_readme(request.target),
        )?;
        fs::write(
            workspace_path.join("task.md"),
            task_instructions(request.target),
        )?;
        fs::write(
            workspace_path.join("selected-context.md"),
            redacted.text,
        )?;
        write_json_atomic(
            &workspace_path.join("structured-input.json"),
            &json!({
                "schemaVersion": 1,
                "workspaceId": workspace_id.clone(),
                "task": "synthesize_story",
                "candidateId": candidate.candidate_id,
                "interviewId": interview.interview_id,
                "candidateClaim": candidate.claim,
                "privacyMode": privacy_mode.clone(),
                "evidenceRules": [
                    "Do not invent missing facts, metrics, tools, employers, or outcomes.",
                    "Preserve stable private tokens exactly.",
                    "Label user estimates and uncertain memories.",
                    "Return JSON matching response-schema.json."
                ]
            }),
        )?;
        write_json_atomic(
            &workspace_path.join("response-schema.json"),
            &story_response_schema(),
        )?;
        write_json_atomic(
            &workspace_path.join("privacy-summary.json"),
            &redacted.preflight,
        )?;

        Ok(ManualWorkspaceResult {
            workspace_id,
            workspace_path: workspace_path.to_string_lossy().to_string(),
            target: request.target,
            privacy_mode,
            replacement_count,
            warning_count,
            message: format!(
                "Manual {} workspace created with {} private-name replacement{}.",
                target_label(request.target),
                replacement_count,
                if replacement_count == 1 { "" } else { "s" }
            ),
        })
    })();

    operation.finish(result)
}

fn build_context(candidate: &StoryCandidate, interview: &InterviewSession) -> String {
    let mut output = String::new();
    output.push_str("# Story Candidate\n\n");
    output.push_str(&candidate.claim);
    output.push_str("\n\n# Source Context\n\n");
    if let Some(heading) = &candidate.context.surrounding_heading {
        output.push_str("Role heading: ");
        output.push_str(heading);
        output.push('\n');
    }
    if !candidate.context.metrics.is_empty() {
        output.push_str("Detected metrics: ");
        output.push_str(&candidate.context.metrics.join(", "));
        output.push('\n');
    }
    if !candidate.context.tools.is_empty() {
        output.push_str("Detected tools: ");
        output.push_str(&candidate.context.tools.join(", "));
        output.push('\n');
    }

    output.push_str("\n# Interview Evidence\n\n");
    for turn in interview
        .turns
        .iter()
        .filter(|turn| turn.actor == TurnActor::User)
    {
        let fields = if turn.target_fields.is_empty() {
            "general".to_string()
        } else {
            turn.target_fields.join(", ")
        };
        output.push_str("## ");
        output.push_str(&fields.replace('_', " "));
        output.push_str("\n\nEvidence classification: ");
        output.push_str(
            &turn
                .answer_classification
                .map(|classification| format!("{classification:?}").to_ascii_lowercase())
                .unwrap_or_else(|| "not_provided".to_string()),
        );
        output.push_str("\n\n");
        output.push_str(&turn.text);
        output.push_str("\n\n");
    }
    output
}

fn workspace_readme(target: ManualWorkspaceTarget) -> String {
    format!(
        "# WorkLore Manual AI Workspace\n\nThis package contains one story candidate, its interview evidence, a privacy summary, and a structured response contract.\n\n## Use\n\n1. Open {}.\n2. Attach or paste `task.md`, `selected-context.md`, `structured-input.json`, and `response-schema.json`.\n3. Ask the AI to complete the task and return only the requested JSON.\n4. Save the JSON response. WorkLore validates it before updating the vault.\n\nDo not add private names that are not already present in the package. Stable tokens such as `[EMPLOYER_1]` must remain unchanged.\n",
        target_label(target)
    )
}

fn task_instructions(target: ManualWorkspaceTarget) -> String {
    format!(
        "# Task: Synthesize a Career Story\n\nYou are helping create a private, evidence-backed career story in {}.\n\nRead all attached files. Build one discrete professional story from the candidate claim and interview evidence. Preserve exact metrics, tools, stable redaction tokens, and evidence labels. Do not invent missing details. An incomplete honest field is better than a polished fabrication.\n\nReturn only JSON matching `response-schema.json`.\n",
        target_label(target)
    )
}

fn story_response_schema() -> serde_json::Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "WorkLore Story Synthesis Response",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "title",
            "summary",
            "situation",
            "problem",
            "constraints",
            "responsibilities",
            "actions",
            "decisions",
            "toolsAndSystems",
            "stakeholders",
            "outcomes",
            "metrics",
            "lessonsLearned",
            "operatingPhilosophy",
            "claims"
        ],
        "properties": {
            "title": { "type": "string", "minLength": 1 },
            "summary": { "type": "string" },
            "situation": { "type": "string" },
            "problem": { "type": "string" },
            "constraints": { "type": "array", "items": { "type": "string" } },
            "responsibilities": { "type": "array", "items": { "type": "string" } },
            "actions": { "type": "array", "items": { "type": "string" } },
            "decisions": { "type": "array", "items": { "type": "string" } },
            "toolsAndSystems": { "type": "array", "items": { "type": "string" } },
            "stakeholders": { "type": "array", "items": { "type": "string" } },
            "outcomes": { "type": "array", "items": { "type": "string" } },
            "metrics": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["statement", "evidenceLevel"],
                    "properties": {
                        "statement": { "type": "string" },
                        "evidenceLevel": {
                            "type": "string",
                            "enum": ["confirmed_fact", "user_estimate", "uncertain", "unsupported"]
                        }
                    }
                }
            },
            "lessonsLearned": { "type": "array", "items": { "type": "string" } },
            "operatingPhilosophy": { "type": "array", "items": { "type": "string" } },
            "claims": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["statement", "evidenceLevel", "supportingFields"],
                    "properties": {
                        "statement": { "type": "string" },
                        "evidenceLevel": {
                            "type": "string",
                            "enum": ["confirmed_fact", "user_estimate", "uncertain", "unsupported"]
                        },
                        "supportingFields": { "type": "array", "items": { "type": "string" } }
                    }
                }
            }
        }
    })
}

fn read_interview(vault_path: &Path, interview_id: &str) -> ServiceResult<InterviewSession> {
    let path = vault_path
        .join("interviews")
        .join(format!("{interview_id}.json"));
    if !path.is_file() {
        return Err(WorkLoreError::InterviewNotFound);
    }
    read_json(&path)
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

fn ensure_vault(vault_path: &Path) -> ServiceResult<()> {
    if !vault_path.join("vault.json").is_file() {
        return Err(WorkLoreError::NotAVault);
    }
    Ok(())
}

fn short_id(value: &str) -> &str {
    value.rsplit('_').next().unwrap_or(value).get(..8).unwrap_or(value)
}

fn target_slug(target: ManualWorkspaceTarget) -> &'static str {
    match target {
        ManualWorkspaceTarget::Chatgpt => "chatgpt",
        ManualWorkspaceTarget::Claude => "claude",
        ManualWorkspaceTarget::Gemini => "gemini",
        ManualWorkspaceTarget::Generic => "generic",
    }
}

fn target_label(target: ManualWorkspaceTarget) -> &'static str {
    match target {
        ManualWorkspaceTarget::Chatgpt => "ChatGPT",
        ManualWorkspaceTarget::Claude => "Claude",
        ManualWorkspaceTarget::Gemini => "Gemini",
        ManualWorkspaceTarget::Generic => "your preferred AI workspace",
    }
}
