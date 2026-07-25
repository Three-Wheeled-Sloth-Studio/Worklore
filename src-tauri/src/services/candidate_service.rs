use std::{collections::HashSet, fs, path::Path};

use chrono::Utc;
use regex::Regex;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    domain::{
        candidates::{
            CandidateContext, CandidateSourceRef, CandidateStatus, CandidateSummary, CandidateType,
            ExtractCandidatesResult, StoryCandidate,
        },
        models::{ExtractionStatus, SourceDocument, SourceType},
    },
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, write_json_atomic},
};

pub fn extract_resume_candidates(
    vault_path: &Path,
    source_id: &str,
) -> ServiceResult<ExtractCandidatesResult> {
    ensure_vault(vault_path)?;
    let source = read_source(vault_path, source_id)?;
    if source.source_type != SourceType::Resume {
        return Err(WorkLoreError::SourceNotReady(
            "Only resume sources can use resume candidate extraction.".to_string(),
        ));
    }
    if source.extraction.status != ExtractionStatus::Complete {
        return Err(WorkLoreError::SourceNotReady(
            "Text extraction must complete before candidate extraction.".to_string(),
        ));
    }

    let extracted_path =
        source.extraction.text_path.as_deref().ok_or_else(|| {
            WorkLoreError::SourceNotReady("Extracted text is missing.".to_string())
        })?;
    let text = fs::read_to_string(vault_path.join(extracted_path))?;
    let existing = read_candidates(vault_path)?;
    let existing_fragments = existing
        .iter()
        .flat_map(|candidate| candidate.source_refs.iter())
        .filter(|reference| reference.source_id == source_id)
        .map(|reference| reference.fragment_id.clone())
        .collect::<HashSet<_>>();

    let now = Utc::now().to_rfc3339();
    let mut created = Vec::new();
    let parsed = parse_resume_bullets(&text);
    for bullet in parsed {
        let fragment_id = fragment_id(bullet.line_number, &bullet.claim);
        if existing_fragments.contains(&fragment_id) {
            continue;
        }

        let candidate = StoryCandidate {
            schema_version: 1,
            candidate_id: format!("candidate_{}", Uuid::now_v7()),
            source_refs: vec![CandidateSourceRef {
                source_id: source_id.to_string(),
                fragment_id,
                captured_text: bullet.claim.clone(),
            }],
            candidate_type: CandidateType::ResumeClaim,
            status: CandidateStatus::New,
            claim: bullet.claim.clone(),
            context: CandidateContext {
                role_ids: Vec::new(),
                entity_ids: Vec::new(),
                skills: infer_skills(&bullet.claim),
                tools: infer_tools(&bullet.claim),
                metrics: infer_metrics(&bullet.claim),
                surrounding_heading: bullet.heading,
            },
            missing_fields: infer_missing_fields(&bullet.claim),
            possible_existing_story_ids: Vec::new(),
            merged_into_id: None,
            split_into_ids: Vec::new(),
            created_at: now.clone(),
            updated_at: now.clone(),
            revision: 1,
        };
        write_candidate(vault_path, &candidate)?;
        created.push(candidate);
    }

    let candidates = read_candidates(vault_path)?
        .into_iter()
        .filter(|candidate| {
            candidate
                .source_refs
                .iter()
                .any(|reference| reference.source_id == source_id)
        })
        .map(|candidate| CandidateSummary::from(&candidate))
        .collect::<Vec<_>>();

    Ok(ExtractCandidatesResult {
        source_id: source_id.to_string(),
        created_count: created.len(),
        existing_count: candidates.len().saturating_sub(created.len()),
        candidates,
        message: if created.is_empty() {
            "No new work-history bullets were found. Titles, summaries, skills, and other resume sections were ignored."
                .to_string()
        } else {
            format!(
                "Created {} story candidate{} from work-history bullets.",
                created.len(),
                if created.len() == 1 { "" } else { "s" }
            )
        },
    })
}

pub fn list_candidates(vault_path: &Path) -> ServiceResult<Vec<CandidateSummary>> {
    ensure_vault(vault_path)?;
    let mut candidates = read_candidates(vault_path)?;
    candidates.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(candidates.iter().map(CandidateSummary::from).collect())
}

pub fn set_candidate_status(
    vault_path: &Path,
    candidate_id: &str,
    status: CandidateStatus,
) -> ServiceResult<CandidateSummary> {
    ensure_vault(vault_path)?;
    let path = vault_path
        .join("candidates")
        .join(format!("{candidate_id}.json"));
    if !path.is_file() {
        return Err(WorkLoreError::CandidateNotFound);
    }

    let mut candidate: StoryCandidate = read_json(&path)?;
    candidate.status = status;
    candidate.updated_at = Utc::now().to_rfc3339();
    candidate.revision += 1;
    write_json_atomic(&path, &candidate)?;
    Ok(CandidateSummary::from(&candidate))
}

fn read_source(vault_path: &Path, source_id: &str) -> ServiceResult<SourceDocument> {
    let path = vault_path
        .join("sources/metadata")
        .join(format!("{source_id}.json"));
    if !path.is_file() {
        return Err(WorkLoreError::SourceNotFound);
    }
    read_json(&path)
}

fn read_candidates(vault_path: &Path) -> ServiceResult<Vec<StoryCandidate>> {
    let directory = vault_path.join("candidates");
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
        {
            candidates.push(read_json(&entry.path())?);
        }
    }
    Ok(candidates)
}

fn write_candidate(vault_path: &Path, candidate: &StoryCandidate) -> ServiceResult<()> {
    write_json_atomic(
        &vault_path
            .join("candidates")
            .join(format!("{}.json", candidate.candidate_id)),
        candidate,
    )
}

#[derive(Debug)]
struct ParsedBullet {
    line_number: usize,
    claim: String,
    heading: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResumeSection {
    OutsideEmployment,
    Employment,
}

fn parse_resume_bullets(text: &str) -> Vec<ParsedBullet> {
    let mut results = Vec::new();
    let mut section = ResumeSection::OutsideEmployment;
    let mut heading: Option<String> = None;

    for (index, raw_line) in text.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(next_section) = classify_resume_section(trimmed) {
            section = next_section;
            heading = None;
            continue;
        }

        if section != ResumeSection::Employment {
            continue;
        }

        if let Some(claim) = strip_bullet_prefix(trimmed) {
            if claim.len() >= 12 {
                results.push(ParsedBullet {
                    line_number: index + 1,
                    claim: claim.to_string(),
                    heading: heading.clone(),
                });
            }
            continue;
        }

        if looks_like_heading(trimmed) {
            heading = Some(trimmed.to_string());
        }
    }

    results
}

fn classify_resume_section(line: &str) -> Option<ResumeSection> {
    let normalized = normalize_section_heading(line);

    const EMPLOYMENT_SECTIONS: &[&str] = &[
        "work history",
        "employment history",
        "work experience",
        "professional experience",
        "employment experience",
        "career history",
        "professional history",
    ];

    const NON_EMPLOYMENT_SECTIONS: &[&str] = &[
        "summary",
        "career summary",
        "professional summary",
        "executive summary",
        "profile",
        "career profile",
        "professional profile",
        "objective",
        "qualifications",
        "core qualifications",
        "skills",
        "technical skills",
        "core competencies",
        "competencies",
        "education",
        "certification",
        "certifications",
        "licenses",
        "projects",
        "selected projects",
        "publications",
        "awards",
        "volunteer experience",
        "community involvement",
        "professional affiliations",
        "affiliations",
        "references",
    ];

    if EMPLOYMENT_SECTIONS.contains(&normalized.as_str()) {
        Some(ResumeSection::Employment)
    } else if NON_EMPLOYMENT_SECTIONS.contains(&normalized.as_str()) {
        Some(ResumeSection::OutsideEmployment)
    } else {
        None
    }
}

fn normalize_section_heading(line: &str) -> String {
    line.trim()
        .trim_matches(|character: char| matches!(character, ':' | '-' | '_' | '=' | '#'))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn strip_bullet_prefix(line: &str) -> Option<&str> {
    let prefixes = ["- ", "* ", "+ ", "• ", "▪ ", "– "];
    prefixes
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))
        .map(str::trim)
}

fn looks_like_heading(line: &str) -> bool {
    if line.len() > 180 || line.ends_with('.') || line.ends_with(';') {
        return false;
    }
    let separators = [" | ", " - ", " at ", " @ "];
    separators.iter().any(|separator| line.contains(separator))
        || (line.chars().any(char::is_alphabetic)
            && line
                .chars()
                .filter(|character| character.is_alphabetic())
                .all(char::is_uppercase))
}

fn fragment_id(line_number: usize, claim: &str) -> String {
    let digest = Sha256::digest(claim.as_bytes());
    format!("line:{line_number}:{}", hex::encode(&digest[..6]))
}

fn infer_metrics(claim: &str) -> Vec<String> {
    let regex = Regex::new(
        r"(?i)\b(?:\$\s*)?\d+(?:[,.]\d+)*(?:\.\d+)?\s*(?:%|percent|hours?|days?|weeks?|months?|years?|teams?|users?|programs?|states?|million|billion|thousand|k|m)?\b",
    )
    .expect("metric regex");
    regex
        .find_iter(claim)
        .map(|matched| matched.as_str().trim().to_string())
        .collect()
}

fn infer_tools(claim: &str) -> Vec<String> {
    const TOOLS: &[&str] = &[
        "SQL",
        "GitHub",
        "Git",
        "Jira",
        "Figma",
        "Salesforce",
        "Python",
        "React",
        "Tauri",
        "Ollama",
        "Gemini",
        "Tableau",
        "Power BI",
        "Excel",
        "WordPress",
        "Firebase",
        "LLM",
    ];

    TOOLS
        .iter()
        .filter(|tool| contains_term(claim, tool))
        .map(|tool| (*tool).to_string())
        .collect()
}

fn infer_skills(claim: &str) -> Vec<String> {
    const SKILL_RULES: &[(&str, &[&str])] = &[
        ("workflow redesign", &["workflow", "process"]),
        ("product leadership", &["led", "guided", "owned"]),
        ("data analytics", &["analytics", "reporting", "data"]),
        ("automation", &["automation", "automated"]),
        (
            "stakeholder management",
            &["stakeholder", "teams", "cross-functional"],
        ),
        ("AI-assisted delivery", &["llm", "ai", "coding agents"]),
    ];

    let lower = claim.to_ascii_lowercase();
    SKILL_RULES
        .iter()
        .filter(|(_, needles)| needles.iter().any(|needle| lower.contains(needle)))
        .map(|(skill, _)| (*skill).to_string())
        .collect()
}

fn infer_missing_fields(claim: &str) -> Vec<String> {
    let lower = claim.to_ascii_lowercase();
    let mut missing = Vec::new();

    if !lower.contains("because") && !lower.contains("to address") && !lower.contains("when") {
        missing.push("problem_or_opportunity".to_string());
    }
    if !lower.contains("by ") && !lower.contains("through ") && !lower.contains("using ") {
        missing.push("actions_and_decisions".to_string());
    }
    if infer_metrics(claim).is_empty() {
        missing.push("metrics_and_evidence".to_string());
    }
    if !lower.contains("team")
        && !lower.contains("user")
        && !lower.contains("client")
        && !lower.contains("stakeholder")
        && !lower.contains("analyst")
    {
        missing.push("stakeholders".to_string());
    }
    if infer_tools(claim).is_empty() {
        missing.push("tools_and_systems".to_string());
    }
    missing.push("constraints".to_string());
    missing.push("lessons_learned".to_string());
    missing.sort();
    missing.dedup();
    missing
}

fn contains_term(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
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
    fn parses_only_bullets_inside_work_history() {
        let text = "PRODUCT MANAGER\n\nPROFESSIONAL SUMMARY\n- Product leader with 15 years of experience.\n\nWORK HISTORY\n\nAcme Cooperative | Product Manager | 2022 to 2026\n\n- Reduced review time by 80 percent.\n- Led an LLM intake tool for 14 teams.\n\nSKILLS\n- Product strategy\n- SQL";
        let bullets = parse_resume_bullets(text);
        assert_eq!(bullets.len(), 2);
        assert_eq!(
            bullets[0].heading.as_deref(),
            Some("Acme Cooperative | Product Manager | 2022 to 2026")
        );
        assert!(bullets
            .iter()
            .all(|bullet| !bullet.claim.contains("Product leader")));
    }

    #[test]
    fn ignores_top_level_title_and_summary_when_no_employment_section_exists() {
        let text = "SENIOR PRODUCT MANAGER\n\nCAREER SUMMARY\n- Reduced delivery cycle time by 80 percent.\n- Led 14 teams.";
        assert!(parse_resume_bullets(text).is_empty());
    }

    #[test]
    fn supports_employment_history_and_stops_at_education() {
        let text = "EMPLOYMENT HISTORY\nNorthwind Health - Product Manager\n- Automated intake for 20 programs.\n\nEDUCATION\n- Bachelor of Science, Computer Science";
        let bullets = parse_resume_bullets(text);
        assert_eq!(bullets.len(), 1);
        assert_eq!(bullets[0].claim, "Automated intake for 20 programs.");
    }

    #[test]
    fn detects_metrics_and_tools() {
        let claim = "Led an LLM workflow that reduced review time by 80 percent for 14 teams.";
        assert!(infer_tools(claim).contains(&"LLM".to_string()));
        assert!(infer_metrics(claim).len() >= 2);
    }

    #[test]
    fn fragment_ids_are_stable() {
        assert_eq!(
            fragment_id(7, "Built a tool"),
            fragment_id(7, "Built a tool")
        );
    }
}
