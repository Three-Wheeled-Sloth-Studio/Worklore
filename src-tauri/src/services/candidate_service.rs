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

    let extracted_path = source
        .extraction
        .text_path
        .as_deref()
        .ok_or_else(|| WorkLoreError::SourceNotReady("Extracted text is missing.".to_string()))?;
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
            "No new local work-history candidates were found. Titles, summaries, skills, and other resume sections were ignored."
                .to_string()
        } else {
            format!(
                "Created {} local story candidate{} from work-history content.",
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
    let mut seen_claims = HashSet::new();
    let mut section = ResumeSection::OutsideEmployment;
    let mut heading: Option<String> = None;

    for (index, raw_line) in text.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut employment_line = trimmed;
        if let Some((next_section, remainder)) = classify_resume_section_prefix(trimmed) {
            section = next_section;
            heading = None;
            employment_line = remainder;
            if section != ResumeSection::Employment || employment_line.is_empty() {
                continue;
            }
        }

        if section != ResumeSection::Employment {
            continue;
        }

        let line_number = index + 1;

        if let Some(claim) = strip_bullet_prefix(employment_line) {
            push_candidate(
                &mut results,
                &mut seen_claims,
                line_number,
                claim,
                heading.clone(),
            );
            continue;
        }

        if let Some((role_heading, claim_text)) = split_role_heading_and_claim(employment_line) {
            heading = Some(role_heading.to_string());
            for claim in split_candidate_sentences(claim_text) {
                push_candidate(
                    &mut results,
                    &mut seen_claims,
                    line_number,
                    claim,
                    heading.clone(),
                );
            }
            continue;
        }

        if looks_like_role_heading(employment_line) {
            heading = Some(employment_line.to_string());
            continue;
        }

        let candidate_sentences = split_candidate_sentences(employment_line);
        if !candidate_sentences.is_empty() {
            for claim in candidate_sentences {
                push_candidate(
                    &mut results,
                    &mut seen_claims,
                    line_number,
                    claim,
                    heading.clone(),
                );
            }
            continue;
        }

        append_wrapped_continuation(&mut results, line_number, employment_line);
    }

    results
}

fn push_candidate(
    results: &mut Vec<ParsedBullet>,
    seen_claims: &mut HashSet<String>,
    line_number: usize,
    claim: &str,
    heading: Option<String>,
) {
    let cleaned = claim
        .trim()
        .trim_start_matches(|character: char| {
            character.is_whitespace() || is_bullet_character(character)
        })
        .trim();
    if !looks_like_candidate_text(cleaned) {
        return;
    }

    let normalized = cleaned
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    if !seen_claims.insert(normalized) {
        return;
    }

    results.push(ParsedBullet {
        line_number,
        claim: cleaned.to_string(),
        heading,
    });
}

fn append_wrapped_continuation(results: &mut [ParsedBullet], line_number: usize, line: &str) {
    let Some(previous) = results.last_mut() else {
        return;
    };
    if previous.line_number + 1 != line_number
        || ends_sentence(&previous.claim)
        || looks_like_role_heading(line)
        || line.len() < 3
    {
        return;
    }

    previous.claim.push(' ');
    previous.claim.push_str(line.trim());
}

fn classify_resume_section_prefix(line: &str) -> Option<(ResumeSection, &str)> {
    const EMPLOYMENT_SECTIONS: &[&str] = &[
        "professional experience",
        "employment experience",
        "professional history",
        "employment history",
        "work experience",
        "work history",
        "career history",
    ];
    const NON_EMPLOYMENT_SECTIONS: &[&str] = &[
        "professional affiliations",
        "professional summary",
        "professional profile",
        "community involvement",
        "executive summary",
        "volunteer experience",
        "technical skills",
        "selected projects",
        "core qualifications",
        "core competencies",
        "career summary",
        "career profile",
        "certifications",
        "qualifications",
        "publications",
        "affiliations",
        "competencies",
        "certification",
        "references",
        "education",
        "objective",
        "projects",
        "licenses",
        "summary",
        "profile",
        "skills",
        "awards",
    ];

    if let Some(remainder) = match_section_prefix(line, EMPLOYMENT_SECTIONS) {
        if remainder.is_empty() || looks_like_inline_employment_content(remainder) {
            return Some((ResumeSection::Employment, remainder));
        }
    }

    match_section_prefix(line, NON_EMPLOYMENT_SECTIONS)
        .map(|remainder| (ResumeSection::OutsideEmployment, remainder))
}

fn match_section_prefix<'a>(line: &'a str, headings: &[&str]) -> Option<&'a str> {
    headings.iter().find_map(|heading| {
        if line.len() < heading.len() {
            return None;
        }
        let prefix = line.get(..heading.len())?;
        if !prefix.eq_ignore_ascii_case(heading) {
            return None;
        }
        let remainder = line.get(heading.len()..)?;
        let boundary_ok = remainder
            .chars()
            .next()
            .is_none_or(|character| !character.is_ascii_alphabetic());
        if !boundary_ok {
            return None;
        }
        Some(trim_section_separator(remainder))
    })
}

fn trim_section_separator(value: &str) -> &str {
    value.trim_start_matches(|character: char| {
        character.is_whitespace()
            || matches!(character, ':' | '-' | '–' | '—' | '_' | '=' | '#')
    })
}

fn looks_like_inline_employment_content(value: &str) -> bool {
    contains_date_range(value)
        || value.contains(" | ")
        || value
            .chars()
            .find(|character| character.is_alphabetic())
            .is_some_and(char::is_uppercase)
}

fn strip_bullet_prefix(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if let Some(first) = trimmed.chars().next() {
        if is_bullet_character(first) {
            return Some(trimmed[first.len_utf8()..].trim_start());
        }
    }

    let numbered = Regex::new(r"^\d{1,2}[.)]\s+").expect("numbered bullet regex");
    numbered
        .find(trimmed)
        .map(|matched| trimmed[matched.end()..].trim_start())
}

fn is_bullet_character(character: char) -> bool {
    matches!(
        character,
        '-' | '*' | '+' | '•' | '▪' | '▫' | '■' | '□' | '●' | '○' | '◦' | '‣' | '►'
            | '–' | '—' | '·' | ''
    )
}

fn split_role_heading_and_claim(line: &str) -> Option<(&str, &str)> {
    let date_end = date_range_match(line)?.end();
    let action_start = action_verb_match(line)?.start();
    if action_start <= date_end {
        return None;
    }

    let heading = line[..action_start]
        .trim()
        .trim_end_matches(|character: char| matches!(character, ':' | '-' | '–' | '—' | '|'))
        .trim();
    let claim = line[action_start..].trim();
    if heading.len() < 4 || !looks_like_candidate_text(claim) {
        return None;
    }
    Some((heading, claim))
}

fn looks_like_role_heading(line: &str) -> bool {
    if line.len() > 220 || ends_sentence(line) {
        return false;
    }
    if contains_date_range(line) {
        return true;
    }

    let separators = [" | ", " - ", " at ", " @ "];
    separators.iter().any(|separator| line.contains(separator))
        || (line.chars().any(char::is_alphabetic)
            && line
                .chars()
                .filter(|character| character.is_alphabetic())
                .all(char::is_uppercase))
}

fn split_candidate_sentences(line: &str) -> Vec<&str> {
    let mut results = Vec::new();
    let mut segment_start = 0;
    let bytes = line.as_bytes();

    for (index, character) in line.char_indices() {
        if !matches!(character, '.' | '!' | '?' | ';') {
            continue;
        }
        let end = index + character.len_utf8();
        let next_is_boundary = end == bytes.len()
            || bytes
                .get(end)
                .is_some_and(|byte| byte.is_ascii_whitespace());
        if !next_is_boundary {
            continue;
        }

        let segment = line[segment_start..end].trim();
        if looks_like_candidate_text(segment) {
            results.push(segment);
        }
        segment_start = end;
        while bytes
            .get(segment_start)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            segment_start += 1;
        }
    }

    if segment_start < line.len() {
        let tail = line[segment_start..].trim();
        if looks_like_candidate_text(tail) {
            results.push(tail);
        }
    }

    if results.is_empty() && looks_like_candidate_text(line) {
        results.push(line.trim());
    }
    results
}

fn looks_like_candidate_text(line: &str) -> bool {
    let word_count = line.split_whitespace().count();
    if line.len() < 24 || word_count < 4 || looks_like_role_heading(line) {
        return false;
    }

    starts_with_action_verb(line)
        || (contains_action_verb(line) && !infer_metrics(line).is_empty())
        || contains_outcome_signal(line)
}

fn starts_with_action_verb(line: &str) -> bool {
    let first_word = line
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|character: char| !character.is_ascii_alphabetic())
        .to_ascii_lowercase();
    action_verbs().contains(&first_word.as_str())
}

fn contains_action_verb(line: &str) -> bool {
    line.split_whitespace().any(|word| {
        let normalized = word
            .trim_matches(|character: char| !character.is_ascii_alphabetic())
            .to_ascii_lowercase();
        action_verbs().contains(&normalized.as_str())
    })
}

fn action_verb_match(line: &str) -> Option<regex::Match<'_>> {
    let pattern = format!(r"(?i)\b(?:{})\b", action_verbs().join("|"));
    Regex::new(&pattern).expect("action verb regex").find(line)
}

fn action_verbs() -> &'static [&'static str] {
    &[
        "accelerated",
        "achieved",
        "automated",
        "built",
        "created",
        "cut",
        "defined",
        "delivered",
        "designed",
        "developed",
        "drove",
        "enabled",
        "established",
        "expanded",
        "guided",
        "implemented",
        "improved",
        "increased",
        "introduced",
        "launched",
        "led",
        "managed",
        "modernized",
        "owned",
        "recovered",
        "reduced",
        "redesigned",
        "resolved",
        "scaled",
        "shortened",
        "simplified",
        "streamlined",
        "supported",
        "translated",
        "transformed",
    ]
}

fn contains_outcome_signal(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "resulted in",
        "leading to",
        "reduced ",
        "increased ",
        "improved ",
        "saved ",
        "recovered ",
        "from months to",
        "from weeks to",
        "from days to",
        "cycle time",
        "delivery time",
        "supporting modernization",
    ]
    .iter()
    .any(|signal| lower.contains(signal))
}

fn contains_date_range(line: &str) -> bool {
    date_range_match(line).is_some()
}

fn date_range_match(line: &str) -> Option<regex::Match<'_>> {
    Regex::new(
        r"(?i)\b(?:19|20)\d{2}\s*(?:-|–|—|to)\s*(?:present|current|(?:19|20)\d{2})\b",
    )
    .expect("date range regex")
    .find(line)
}

fn ends_sentence(line: &str) -> bool {
    line.trim_end()
        .chars()
        .next_back()
        .is_some_and(|character| matches!(character, '.' | '!' | '?' | ';'))
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
    fn finds_bulletless_action_lines_inside_employment_only() {
        let text = "PROFESSIONAL SUMMARY\nReduced review time by 80 percent.\nPROFESSIONAL EXPERIENCE\nAcme Cooperative | Product Manager | 2022 to 2026\nReduced review time by 80 percent.\nLed an LLM intake tool for 14 teams.\nEDUCATION\nBuilt a capstone project.";
        let bullets = parse_resume_bullets(text);
        assert_eq!(bullets.len(), 2);
        assert!(bullets.iter().all(|bullet| {
            bullet
                .heading
                .as_deref()
                .is_some_and(|heading| heading.contains("Acme Cooperative"))
        }));
    }

    #[test]
    fn handles_section_heading_role_and_claim_flattened_on_one_line() {
        let text = "PROFESSIONAL SUMMARY\nProduct leader.\nPROFESSIONAL EXPERIENCE ATI Government Solutions (2025–Present) Data Analytics Product Lead Translated complex requirements into scalable workflows, supporting modernization for 30,000 users. Defined a new reporting model for 22 service centers.\nEDUCATION Bachelor of Science";
        let bullets = parse_resume_bullets(text);
        assert_eq!(bullets.len(), 2);
        assert!(bullets[0].claim.starts_with("Translated complex requirements"));
        assert!(bullets[1].claim.starts_with("Defined a new reporting model"));
        assert!(bullets
            .iter()
            .all(|bullet| !bullet.claim.contains("Product leader")));
    }

    #[test]
    fn supports_pdf_bullet_glyphs() {
        let text = "WORK EXPERIENCE\nAcme | Product Manager | 2022 to 2026\n Transformed intake for 14 teams.\n● Reduced cycle time by 80 percent.";
        let bullets = parse_resume_bullets(text);
        assert_eq!(bullets.len(), 2);
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
