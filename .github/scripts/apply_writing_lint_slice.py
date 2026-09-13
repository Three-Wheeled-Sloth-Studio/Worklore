from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise RuntimeError(f"Expected anchor not found in {path}: {old[:120]!r}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


writing_lint_service = r'''use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    error::ServiceResult,
    services::voice_profile_service::{self, WritingRuleRecordView, WritingRuleStatus},
};

const BUILTIN_HASHTAG_LIMIT: usize = 5;
const LIST_MIN_NONEMPTY_LINES: usize = 5;
const LIST_MIN_LIST_LINES: usize = 4;
const LIST_DOMINANCE_PERCENT: usize = 60;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LintDraftRequest {
    pub text: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LintSeverity {
    Advisory,
    Warning,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LintCategory {
    OpeningPattern,
    EngagementBait,
    Hashtags,
    Structure,
    WritingRule,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LintSourceKind {
    BuiltIn,
    WritingRule,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LintFinding {
    pub rule_id: String,
    pub category: LintCategory,
    pub severity: LintSeverity,
    pub reason: String,
    pub remediation: Option<String>,
    pub matched_text: Option<String>,
    pub start_offset: Option<usize>,
    pub end_offset: Option<usize>,
    pub source_kind: LintSourceKind,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UnsupportedWritingRuleView {
    pub rule_id: String,
    pub name: String,
    pub instruction: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LintDraftResult {
    pub findings: Vec<LintFinding>,
    pub active_writing_rule_count: usize,
    pub enforceable_writing_rule_count: usize,
    pub unsupported_writing_rules: Vec<UnsupportedWritingRuleView>,
}

struct FindingSpec<'a> {
    rule_id: &'a str,
    category: LintCategory,
    severity: LintSeverity,
    reason: String,
    remediation: Option<String>,
    byte_range: Option<(usize, usize)>,
    source_kind: LintSourceKind,
    source_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeterministicWritingRule {
    BanPhrase(String),
    BanWord(String),
    ForbidPunctuation(PunctuationKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PunctuationKind {
    EmDash,
    EnDash,
    Semicolon,
    ExclamationMark,
    Ellipsis,
}

impl PunctuationKind {
    fn label(self) -> &'static str {
        match self {
            Self::EmDash => "em dash",
            Self::EnDash => "en dash",
            Self::Semicolon => "semicolon",
            Self::ExclamationMark => "exclamation mark",
            Self::Ellipsis => "ellipsis",
        }
    }

    fn patterns(self) -> &'static [&'static str] {
        match self {
            Self::EmDash => &["—"],
            Self::EnDash => &["–"],
            Self::Semicolon => &[";"],
            Self::ExclamationMark => &["!"],
            Self::Ellipsis => &["…", "..."],
        }
    }
}

pub fn lint_draft(vault_path: &Path, request: LintDraftRequest) -> ServiceResult<LintDraftResult> {
    let rules = voice_profile_service::list_writing_rules(vault_path)?;
    Ok(lint_text_with_rules(&request.text, &rules))
}

fn lint_text_with_rules(text: &str, rules: &[WritingRuleRecordView]) -> LintDraftResult {
    let mut findings = Vec::new();

    if let Some(finding) = lint_question_opening(text) {
        findings.push(finding);
    }
    if let Some(finding) = lint_engagement_bait_closing(text) {
        findings.push(finding);
    }
    if let Some(finding) = lint_engagement_bait_phrase(text) {
        findings.push(finding);
    }
    if let Some(finding) = lint_excessive_hashtags(text) {
        findings.push(finding);
    }
    if let Some(finding) = lint_list_dominance(text) {
        findings.push(finding);
    }

    let active_rules = rules
        .iter()
        .filter(|rule| rule.status == WritingRuleStatus::Active)
        .collect::<Vec<_>>();
    let active_writing_rule_count = active_rules.len();
    let mut enforceable_writing_rule_count = 0;
    let mut unsupported_writing_rules = Vec::new();

    for rule in active_rules {
        match parse_deterministic_rule(&rule.instruction) {
            Ok(matcher) => {
                enforceable_writing_rule_count += 1;
                if let Some(finding) = lint_writing_rule(text, rule, matcher) {
                    findings.push(finding);
                }
            }
            Err(reason) => unsupported_writing_rules.push(UnsupportedWritingRuleView {
                rule_id: rule.rule_id.clone(),
                name: rule.name.clone(),
                instruction: rule.instruction.clone(),
                reason,
            }),
        }
    }

    LintDraftResult {
        findings,
        active_writing_rule_count,
        enforceable_writing_rule_count,
        unsupported_writing_rules,
    }
}

fn lint_question_opening(text: &str) -> Option<LintFinding> {
    let start = text.char_indices().find(|(_, ch)| !ch.is_whitespace())?.0;
    for (relative, ch) in text[start..].char_indices() {
        let byte_index = start + relative;
        match ch {
            '?' => {
                let end = byte_index + ch.len_utf8();
                return Some(make_finding(
                    text,
                    FindingSpec {
                        rule_id: "builtin.question_opening",
                        category: LintCategory::OpeningPattern,
                        severity: LintSeverity::Advisory,
                        reason: "The draft opens with a question. Review whether the question is deliberate rather than a default rhetorical hook.".to_string(),
                        remediation: Some("If the question is only a hook, consider opening with the concrete observation, tension, or claim instead.".to_string()),
                        byte_range: Some((start, end)),
                        source_kind: LintSourceKind::BuiltIn,
                        source_id: None,
                    },
                ));
            }
            '.' | '!' | '\n' => return None,
            _ => {}
        }
    }
    None
}

fn lint_engagement_bait_closing(text: &str) -> Option<LintFinding> {
    let trimmed = text.trim_end();
    if !trimmed.ends_with('?') {
        return None;
    }

    let question_mark_index = trimmed.len() - 1;
    let sentence_start = trimmed[..question_mark_index]
        .char_indices()
        .rev()
        .find(|(_, ch)| matches!(ch, '.' | '!' | '?' | '\n'))
        .map_or(0, |(index, ch)| index + ch.len_utf8());
    let sentence = &trimmed[sentence_start..];
    let leading = sentence.len() - sentence.trim_start().len();
    let start = sentence_start + leading;
    let sentence = &trimmed[start..];
    let normalized = sentence.to_ascii_lowercase();
    const STEMS: &[&str] = &[
        "what do you think",
        "do you agree",
        "what's your take",
        "what is your take",
        "how about you",
        "any thoughts",
        "would you do the same",
    ];

    if STEMS.iter().any(|stem| normalized.contains(stem)) {
        Some(make_finding(
            text,
            FindingSpec {
                rule_id: "builtin.engagement_bait_closing",
                category: LintCategory::EngagementBait,
                severity: LintSeverity::Advisory,
                reason: "The final question matches a common engagement-bait closing pattern. This does not mean every closing question is bad; this finding is limited to explicit bait-style stems.".to_string(),
                remediation: Some("Keep the question only if you genuinely need the reader's answer; otherwise end on the useful point.".to_string()),
                byte_range: Some((start, trimmed.len())),
                source_kind: LintSourceKind::BuiltIn,
                source_id: None,
            },
        ))
    } else {
        None
    }
}

fn lint_engagement_bait_phrase(text: &str) -> Option<LintFinding> {
    const PHRASES: &[&str] = &[
        "let me know in the comments",
        "drop your thoughts in the comments",
        "comment below",
        "sound off in the comments",
        "share your thoughts below",
    ];
    let mut earliest: Option<(usize, &str)> = None;
    for phrase in PHRASES {
        if let Some(index) = find_case_insensitive(text, phrase) {
            if earliest.is_none_or(|(best, _)| index < best) {
                earliest = Some((index, phrase));
            }
        }
    }
    let (start, phrase) = earliest?;
    Some(make_finding(
        text,
        FindingSpec {
            rule_id: "builtin.engagement_bait_phrase",
            category: LintCategory::EngagementBait,
            severity: LintSeverity::Warning,
            reason: "The draft contains an explicit engagement-bait phrase rather than a content-bearing close.".to_string(),
            remediation: Some("Remove the engagement prompt unless asking for comments is itself necessary to the post.".to_string()),
            byte_range: Some((start, start + phrase.len())),
            source_kind: LintSourceKind::BuiltIn,
            source_id: None,
        },
    ))
}

fn lint_excessive_hashtags(text: &str) -> Option<LintFinding> {
    let hashtags = hashtag_ranges(text);
    if hashtags.len() <= BUILTIN_HASHTAG_LIMIT {
        return None;
    }
    let range = hashtags[BUILTIN_HASHTAG_LIMIT];
    Some(make_finding(
        text,
        FindingSpec {
            rule_id: "builtin.excessive_hashtags",
            category: LintCategory::Hashtags,
            severity: LintSeverity::Advisory,
            reason: format!(
                "The draft contains {} hashtags; the deterministic review threshold is more than {}.",
                hashtags.len(),
                BUILTIN_HASHTAG_LIMIT
            ),
            remediation: Some(format!(
                "Consider keeping at most {BUILTIN_HASHTAG_LIMIT} hashtags unless the extra tags have a specific purpose."
            )),
            byte_range: Some(range),
            source_kind: LintSourceKind::BuiltIn,
            source_id: None,
        },
    ))
}

fn lint_list_dominance(text: &str) -> Option<LintFinding> {
    let mut cursor = 0;
    let mut nonempty = 0;
    let mut list_lines = Vec::new();

    for segment in text.split_inclusive('\n') {
        let line_with_cr = segment.strip_suffix('\n').unwrap_or(segment);
        let line = line_with_cr.strip_suffix('\r').unwrap_or(line_with_cr);
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            nonempty += 1;
            if is_list_line(trimmed) {
                let leading = line.len() - line.trim_start().len();
                list_lines.push((cursor + leading, cursor + line.len()));
            }
        }
        cursor += segment.len();
    }

    if nonempty < LIST_MIN_NONEMPTY_LINES || list_lines.len() < LIST_MIN_LIST_LINES {
        return None;
    }
    if list_lines.len() * 100 < nonempty * LIST_DOMINANCE_PERCENT {
        return None;
    }

    Some(make_finding(
        text,
        FindingSpec {
            rule_id: "builtin.list_dominance",
            category: LintCategory::Structure,
            severity: LintSeverity::Advisory,
            reason: format!(
                "{} of {} non-empty lines are list items. The deterministic review threshold is at least {} list lines and {}% of non-empty lines.",
                list_lines.len(),
                nonempty,
                LIST_MIN_LIST_LINES,
                LIST_DOMINANCE_PERCENT
            ),
            remediation: Some("Check whether the list is genuinely the clearest structure or whether the draft has collapsed into a default list template.".to_string()),
            byte_range: list_lines.first().copied(),
            source_kind: LintSourceKind::BuiltIn,
            source_id: None,
        },
    ))
}

fn lint_writing_rule(
    text: &str,
    rule: &WritingRuleRecordView,
    matcher: DeterministicWritingRule,
) -> Option<LintFinding> {
    let (range, reason, remediation) = match matcher {
        DeterministicWritingRule::BanPhrase(phrase) => {
            let start = find_case_insensitive(text, &phrase)?;
            (
                (start, start + phrase.len()),
                format!("Active Writing Rule '{}' bans the phrase '{}'.", rule.name, phrase),
                Some(format!("Remove or rewrite the banned phrase required by '{}'.", rule.name)),
            )
        }
        DeterministicWritingRule::BanWord(word) => {
            let start = find_word_case_insensitive(text, &word)?;
            (
                (start, start + word.len()),
                format!("Active Writing Rule '{}' bans the word '{}'.", rule.name, word),
                Some(format!("Replace the banned word required by '{}'.", rule.name)),
            )
        }
        DeterministicWritingRule::ForbidPunctuation(kind) => {
            let (start, matched) = earliest_pattern(text, kind.patterns())?;
            (
                (start, start + matched.len()),
                format!(
                    "Active Writing Rule '{}' forbids {} punctuation.",
                    rule.name,
                    kind.label()
                ),
                Some(format!("Remove or replace the {} required by '{}'.", kind.label(), rule.name)),
            )
        }
    };

    Some(make_finding(
        text,
        FindingSpec {
            rule_id: &format!("writing_rule:{}", rule.rule_id),
            category: LintCategory::WritingRule,
            severity: LintSeverity::Warning,
            reason,
            remediation,
            byte_range: Some(range),
            source_kind: LintSourceKind::WritingRule,
            source_id: Some(rule.rule_id.clone()),
        },
    ))
}

fn parse_deterministic_rule(instruction: &str) -> Result<DeterministicWritingRule, String> {
    let instruction = instruction.trim();
    if let Some(value) = strip_prefix_ascii_case(instruction, "ban phrase:") {
        let value = value.trim();
        if !value.is_empty() {
            return Ok(DeterministicWritingRule::BanPhrase(value.to_string()));
        }
    }
    if let Some(value) = strip_prefix_ascii_case(instruction, "ban word:") {
        let value = value.trim();
        if !value.is_empty() && !value.chars().any(char::is_whitespace) {
            return Ok(DeterministicWritingRule::BanWord(value.to_string()));
        }
    }
    if let Some(value) = strip_prefix_ascii_case(instruction, "forbid punctuation:") {
        let value = value.trim();
        if let Some(kind) = parse_punctuation_kind(value) {
            return Ok(DeterministicWritingRule::ForbidPunctuation(kind));
        }
    }
    Err("This active rule is advisory only in the deterministic linter. Machine-enforceable forms are 'ban phrase: ...', 'ban word: ...', or 'forbid punctuation: em dash|en dash|semicolon|exclamation mark|ellipsis'.".to_string())
}

fn parse_punctuation_kind(value: &str) -> Option<PunctuationKind> {
    match value.to_ascii_lowercase().as_str() {
        "em dash" | "—" => Some(PunctuationKind::EmDash),
        "en dash" | "–" => Some(PunctuationKind::EnDash),
        "semicolon" | ";" => Some(PunctuationKind::Semicolon),
        "exclamation mark" | "!" => Some(PunctuationKind::ExclamationMark),
        "ellipsis" | "…" | "..." => Some(PunctuationKind::Ellipsis),
        _ => None,
    }
}

fn strip_prefix_ascii_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let candidate = value.get(..prefix.len())?;
    if candidate.eq_ignore_ascii_case(prefix) {
        value.get(prefix.len()..)
    } else {
        None
    }
}

fn hashtag_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    for (index, ch) in text.char_indices() {
        if ch != '#' {
            continue;
        }
        let previous = text[..index].chars().next_back();
        if previous.is_some_and(|value| value.is_alphanumeric() || value == '_') {
            continue;
        }

        let mut end = index + ch.len_utf8();
        let mut has_body = false;
        for (relative, value) in text[end..].char_indices() {
            if value.is_alphanumeric() || value == '_' {
                has_body = true;
                end = index + ch.len_utf8() + relative + value.len_utf8();
            } else {
                break;
            }
        }
        if has_body {
            ranges.push((index, end));
        }
    }
    ranges
}

fn is_list_line(value: &str) -> bool {
    if value.starts_with("- ") || value.starts_with("* ") || value.starts_with("• ") {
        return true;
    }
    let mut chars = value.chars().peekable();
    let mut saw_digit = false;
    while chars.peek().is_some_and(|ch| ch.is_ascii_digit()) {
        saw_digit = true;
        chars.next();
    }
    if !saw_digit {
        return false;
    }
    matches!(chars.next(), Some('.' | ')')) && chars.next().is_some_and(char::is_whitespace)
}

fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_ascii() {
        haystack
            .to_ascii_lowercase()
            .find(&needle.to_ascii_lowercase())
    } else {
        haystack.find(needle)
    }
}

fn find_word_case_insensitive(text: &str, word: &str) -> Option<usize> {
    let mut search_start = 0;
    while search_start < text.len() {
        let relative = find_case_insensitive(&text[search_start..], word)?;
        let start = search_start + relative;
        let end = start + word.len();
        if is_word_boundary(text, start, end) {
            return Some(start);
        }
        let advance = text[start..].chars().next()?.len_utf8();
        search_start = start + advance;
    }
    None
}

fn is_word_boundary(text: &str, start: usize, end: usize) -> bool {
    let before_ok = text[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| !ch.is_alphanumeric() && ch != '_');
    let after_ok = text[end..]
        .chars()
        .next()
        .is_none_or(|ch| !ch.is_alphanumeric() && ch != '_');
    before_ok && after_ok
}

fn earliest_pattern<'a>(text: &str, patterns: &'a [&'a str]) -> Option<(usize, &'a str)> {
    patterns
        .iter()
        .filter_map(|pattern| text.find(pattern).map(|index| (index, *pattern)))
        .min_by_key(|(index, _)| *index)
}

fn make_finding(text: &str, spec: FindingSpec<'_>) -> LintFinding {
    let (matched_text, start_offset, end_offset) = match spec.byte_range {
        Some((start, end)) => (
            Some(text[start..end].to_string()),
            Some(utf16_offset(text, start)),
            Some(utf16_offset(text, end)),
        ),
        None => (None, None, None),
    };
    LintFinding {
        rule_id: spec.rule_id.to_string(),
        category: spec.category,
        severity: spec.severity,
        reason: spec.reason,
        remediation: spec.remediation,
        matched_text,
        start_offset,
        end_offset,
        source_kind: spec.source_kind,
        source_id: spec.source_id,
    }
}

fn utf16_offset(text: &str, byte_index: usize) -> usize {
    text[..byte_index].encode_utf16().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{canonical_store, vault_service, voice_profile_service};
    use uuid::Uuid;

    fn rule(id: &str, instruction: &str, status: WritingRuleStatus) -> WritingRuleRecordView {
        WritingRuleRecordView {
            rule_id: id.to_string(),
            name: format!("Rule {id}"),
            instruction: instruction.to_string(),
            status,
            created_at: "2026-09-13T00:00:00Z".to_string(),
            updated_at: "2026-09-13T00:00:00Z".to_string(),
            revision: 1,
        }
    }

    #[test]
    fn clean_ordinary_prose_can_return_zero_findings() {
        let result = lint_text_with_rules(
            "A production incident exposed a weak handoff between two teams. We changed the ownership boundary and documented the new escalation path.",
            &[],
        );
        assert!(result.findings.is_empty());
        assert_eq!(result.active_writing_rule_count, 0);
    }

    #[test]
    fn rhetorical_question_opening_is_deterministic_and_located() {
        let result = lint_text_with_rules("What happens when the handoff is unclear? The queue grows.", &[]);
        let finding = result
            .findings
            .iter()
            .find(|item| item.rule_id == "builtin.question_opening")
            .expect("question opening finding");
        assert_eq!(finding.start_offset, Some(0));
        assert_eq!(
            finding.matched_text.as_deref(),
            Some("What happens when the handoff is unclear?")
        );
    }

    #[test]
    fn bait_style_closing_is_detected_but_other_questions_are_not() {
        let bait = lint_text_with_rules(
            "We changed the handoff and measured the queue again. What do you think?",
            &[],
        );
        assert!(bait
            .findings
            .iter()
            .any(|item| item.rule_id == "builtin.engagement_bait_closing"));

        let genuine = lint_text_with_rules(
            "The deployment failed after the certificate rotated. Which certificate chain did the service actually load?",
            &[],
        );
        assert!(!genuine
            .findings
            .iter()
            .any(|item| item.rule_id == "builtin.engagement_bait_closing"));
    }

    #[test]
    fn hashtag_threshold_is_boundary_tested() {
        let five = lint_text_with_rules("#one #two #three #four #five", &[]);
        assert!(!five
            .findings
            .iter()
            .any(|item| item.rule_id == "builtin.excessive_hashtags"));

        let six = lint_text_with_rules("#one #two #three #four #five #six", &[]);
        let finding = six
            .findings
            .iter()
            .find(|item| item.rule_id == "builtin.excessive_hashtags")
            .expect("hashtag finding");
        assert_eq!(finding.matched_text.as_deref(), Some("#six"));
    }

    #[test]
    fn list_dominance_uses_transparent_thresholds() {
        let text = "Context\n- one\n- two\n- three\n- four\nClosing";
        let result = lint_text_with_rules(text, &[]);
        assert!(result
            .findings
            .iter()
            .any(|item| item.rule_id == "builtin.list_dominance"));
    }

    #[test]
    fn only_active_explicit_writing_rules_are_enforced() {
        let rules = vec![
            rule("active", "ban phrase: game changer", WritingRuleStatus::Active),
            rule("proposed", "ban word: synergy", WritingRuleStatus::Proposed),
            rule("disabled", "ban word: leverage", WritingRuleStatus::Disabled),
            rule("retired", "forbid punctuation: em dash", WritingRuleStatus::Retired),
        ];
        let result = lint_text_with_rules(
            "This game changer gives us synergy and leverage — but only one active rule should fire.",
            &rules,
        );
        let writing_findings = result
            .findings
            .iter()
            .filter(|item| item.source_kind == LintSourceKind::WritingRule)
            .collect::<Vec<_>>();
        assert_eq!(result.active_writing_rule_count, 1);
        assert_eq!(result.enforceable_writing_rule_count, 1);
        assert_eq!(writing_findings.len(), 1);
        assert_eq!(writing_findings[0].rule_id, "writing_rule:active");
        assert_eq!(writing_findings[0].matched_text.as_deref(), Some("game changer"));
    }

    #[test]
    fn unsupported_active_rule_is_reported_without_hidden_inference() {
        let rules = vec![rule(
            "advisory",
            "Write with more warmth and less corporate energy.",
            WritingRuleStatus::Active,
        )];
        let result = lint_text_with_rules("A plain draft.", &rules);
        assert_eq!(result.active_writing_rule_count, 1);
        assert_eq!(result.enforceable_writing_rule_count, 0);
        assert_eq!(result.unsupported_writing_rules.len(), 1);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn punctuation_rule_is_enforced_with_browser_compatible_offsets() {
        let rules = vec![rule(
            "dash",
            "forbid punctuation: em dash",
            WritingRuleStatus::Active,
        )];
        let result = lint_text_with_rules("🙂 Clear point — no detour.", &rules);
        let finding = result
            .findings
            .iter()
            .find(|item| item.rule_id == "writing_rule:dash")
            .expect("punctuation finding");
        assert_eq!(finding.matched_text.as_deref(), Some("—"));
        assert_eq!(finding.start_offset, Some(15));
        assert_eq!(finding.end_offset, Some(16));
    }

    #[test]
    fn lint_command_reads_rules_without_mutating_canonical_voice_state() {
        let path = std::env::temp_dir().join(format!("worklore-lint-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Lint Test").expect("create vault");
        let created = voice_profile_service::create_writing_rule(
            &path,
            voice_profile_service::CreateWritingRuleRequest {
                name: "No hype".to_string(),
                instruction: "ban word: revolutionary".to_string(),
            },
        )
        .expect("create rule");
        voice_profile_service::update_writing_rule(
            &path,
            voice_profile_service::UpdateWritingRuleRequest {
                rule_id: created.rule_id.clone(),
                name: created.name,
                instruction: created.instruction,
                status: WritingRuleStatus::Active,
            },
        )
        .expect("activate rule");

        let before_rules = voice_profile_service::list_writing_rules(&path).unwrap();
        let before_voices = voice_profile_service::list_core_voices(&path).unwrap();
        let before_tones = voice_profile_service::list_tone_modes(&path).unwrap();
        let before_directions = voice_profile_service::list_voice_directions(&path).unwrap();

        let result = lint_draft(
            &path,
            LintDraftRequest {
                text: "A revolutionary claim.".to_string(),
            },
        )
        .expect("lint draft");
        assert!(result
            .findings
            .iter()
            .any(|item| item.rule_id == format!("writing_rule:{}", created.rule_id)));

        assert_eq!(before_rules, voice_profile_service::list_writing_rules(&path).unwrap());
        assert_eq!(before_voices, voice_profile_service::list_core_voices(&path).unwrap());
        assert_eq!(before_tones, voice_profile_service::list_tone_modes(&path).unwrap());
        assert_eq!(
            before_directions,
            voice_profile_service::list_voice_directions(&path).unwrap()
        );
        canonical_store::initialize(&path).unwrap();
        std::fs::remove_dir_all(path).unwrap();
    }
}
'''

quality_command = r'''use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::writing_lint_service::{self, LintDraftRequest, LintDraftResult},
};

#[tauri::command]
pub fn lint_draft(vault_path: String, request: LintDraftRequest) -> CommandResult<LintDraftResult> {
    writing_lint_service::lint_draft(&PathBuf::from(vault_path), request).map_err(CommandError::from)
}
'''

(ROOT / "src-tauri/src/services/writing_lint_service.rs").write_text(
    writing_lint_service, encoding="utf-8"
)
(ROOT / "src-tauri/src/commands/quality.rs").write_text(quality_command, encoding="utf-8")

replace_once(
    ROOT / "src-tauri/src/services/mod.rs",
    "pub mod voice_profile_service;\n",
    "pub mod voice_profile_service;\npub mod writing_lint_service;\n",
)
replace_once(
    ROOT / "src-tauri/src/commands/mod.rs",
    "pub mod providers;\n",
    "pub mod providers;\npub mod quality;\n",
)
replace_once(
    ROOT / "src-tauri/src/lib.rs",
    "    providers::{\n        analyze_voice_evidence, create_manual_workspace, get_provider_settings,\n        list_provider_models, test_provider_connection, update_provider_settings,\n    },\n    roles::list_roles,\n",
    "    providers::{\n        analyze_voice_evidence, create_manual_workspace, get_provider_settings,\n        list_provider_models, test_provider_connection, update_provider_settings,\n    },\n    quality::lint_draft,\n    roles::list_roles,\n",
)
replace_once(
    ROOT / "src-tauri/src/lib.rs",
    "            analyze_voice_evidence,\n            list_roles,\n",
    "            analyze_voice_evidence,\n            lint_draft,\n            list_roles,\n",
)

replace_once(
    ROOT / "src/domain/types.ts",
    "export interface UpdateWritingRuleRequest extends CreateWritingRuleRequest {\n  ruleId: string;\n  status: WritingRuleStatus;\n}\n\nexport interface ProviderSettings {",
    '''export interface UpdateWritingRuleRequest extends CreateWritingRuleRequest {\n  ruleId: string;\n  status: WritingRuleStatus;\n}\n\nexport type LintSeverity = "advisory" | "warning";\nexport type LintCategory =\n  | "opening_pattern"\n  | "engagement_bait"\n  | "hashtags"\n  | "structure"\n  | "writing_rule";\nexport type LintSourceKind = "built_in" | "writing_rule";\n\nexport interface LintDraftRequest {\n  text: string;\n}\n\nexport interface LintFinding {\n  ruleId: string;\n  category: LintCategory;\n  severity: LintSeverity;\n  reason: string;\n  remediation: string | null;\n  matchedText: string | null;\n  startOffset: number | null;\n  endOffset: number | null;\n  sourceKind: LintSourceKind;\n  sourceId: string | null;\n}\n\nexport interface UnsupportedWritingRule {\n  ruleId: string;\n  name: string;\n  instruction: string;\n  reason: string;\n}\n\nexport interface LintDraftResult {\n  findings: LintFinding[];\n  activeWritingRuleCount: number;\n  enforceableWritingRuleCount: number;\n  unsupportedWritingRules: UnsupportedWritingRule[];\n}\n\nexport interface ProviderSettings {''',
)

replace_once(
    ROOT / "src/lib/workloreApi.ts",
    "  UpdateWritingRuleRequest,\n  ProviderSettings,\n",
    "  UpdateWritingRuleRequest,\n  LintDraftRequest,\n  LintDraftResult,\n  ProviderSettings,\n",
)
replace_once(
    ROOT / "src/lib/workloreApi.ts",
    "export async function getProviderSettings(): Promise<ProviderSettings> {\n",
    '''export async function lintDraft(\n  vaultPath: string,\n  request: LintDraftRequest,\n): Promise<LintDraftResult> {\n  return invoke<LintDraftResult>("lint_draft", { vaultPath, request });\n}\n\nexport async function getProviderSettings(): Promise<ProviderSettings> {\n''',
)

replace_once(
    ROOT / "src/components/VoiceWorkspace.tsx",
    "  WritingRuleRecord,\n  WritingRuleStatus,\n} from \"../domain/types\";",
    "  WritingRuleRecord,\n  WritingRuleStatus,\n  LintDraftResult,\n} from \"../domain/types\";",
)
replace_once(
    ROOT / "src/components/VoiceWorkspace.tsx",
    "  listWritingRules,\n  reviewVoiceEvidence,\n",
    "  listWritingRules,\n  lintDraft,\n  reviewVoiceEvidence,\n",
)
replace_once(
    ROOT / "src/components/VoiceWorkspace.tsx",
    "  const [ruleInstruction, setRuleInstruction] = useState(\"\");\n  const [providerSettings, setProviderSettings] = useState<ProviderSettings | null>(null);",
    "  const [ruleInstruction, setRuleInstruction] = useState(\"\");\n  const [lintText, setLintText] = useState(\"\");\n  const [lintResult, setLintResult] = useState<LintDraftResult | null>(null);\n  const [providerSettings, setProviderSettings] = useState<ProviderSettings | null>(null);",
)
replace_once(
    ROOT / "src/components/VoiceWorkspace.tsx",
    "  async function addRule() {\n",
    '''  async function runDraftLint() {\n    if (!lintText.trim()) return;\n    setBusy("Checking draft patterns deterministically");\n    setNotice(null);\n    setError(null);\n    try {\n      const result = await lintDraft(vaultPath, { text: lintText });\n      setLintResult(result);\n      setNotice(\n        result.findings.length > 0\n          ? `${result.findings.length} explainable pattern finding${result.findings.length === 1 ? "" : "s"} returned. No score was calculated and nothing was changed.`\n          : "No deterministic pattern findings for this draft. No score was calculated and nothing was changed.",\n      );\n    } catch (caught) {\n      setError(errorMessage(caught));\n    } finally {\n      setBusy(null);\n    }\n  }\n\n  async function addRule() {\n''',
)
replace_once(
    ROOT / "src/components/VoiceWorkspace.tsx",
    '''      <section className="workspace-panel" aria-labelledby="rules-heading">\n        <div className="panel-heading-row"><div><p className="eyebrow">Behavioral constraints</p><h2 id="rules-heading">Writing Rules</h2></div><span className="status-pill">{rules.length}</span></div>\n        <div className="voice-form-grid"><input value={ruleName} onChange={(event) => setRuleName(event.target.value)} placeholder="Rule name" /><textarea value={ruleInstruction} onChange={(event) => setRuleInstruction(event.target.value)} placeholder="Explicit instruction" rows={2} /><button className="secondary-button compact" disabled={busy !== null || !ruleName.trim() || !ruleInstruction.trim()} onClick={() => void addRule()}>Add proposed rule</button></div>\n        <div className="voice-card-list">{rules.map((item) => <article className="voice-model-card" key={item.ruleId}><div className="voice-card-heading"><div><h3>{item.name}</h3><p className="voice-preview">{item.instruction}</p></div><span className="status-pill">{item.status}</span></div><div className="support-actions">{item.status === "proposed" ? <button className="primary-button compact" onClick={() => void setRuleStatus(item, "active")}>Activate</button> : null}{item.status === "active" ? <button className="quiet-button compact" onClick={() => void setRuleStatus(item, "disabled")}>Disable</button> : null}{item.status === "disabled" ? <button className="secondary-button compact" onClick={() => void setRuleStatus(item, "active")}>Enable</button> : null}{item.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setRuleStatus(item, "retired")}>Retire</button> : null}</div></article>)}</div>\n      </section>\n\n      {busy ?''',
    '''      <section className="workspace-panel" aria-labelledby="rules-heading">\n        <div className="panel-heading-row"><div><p className="eyebrow">Behavioral constraints</p><h2 id="rules-heading">Writing Rules</h2></div><span className="status-pill">{rules.length}</span></div>\n        <div className="voice-form-grid"><input value={ruleName} onChange={(event) => setRuleName(event.target.value)} placeholder="Rule name" /><textarea value={ruleInstruction} onChange={(event) => setRuleInstruction(event.target.value)} placeholder="Explicit instruction" rows={2} /><button className="secondary-button compact" disabled={busy !== null || !ruleName.trim() || !ruleInstruction.trim()} onClick={() => void addRule()}>Add proposed rule</button></div>\n        <p className="voice-rule">Deterministic enforcement currently understands active rules written as <code>ban phrase: ...</code>, <code>ban word: ...</code>, or <code>forbid punctuation: em dash|en dash|semicolon|exclamation mark|ellipsis</code>. Other active rules remain advisory rather than becoming hidden regexes.</p>\n        <div className="voice-card-list">{rules.map((item) => <article className="voice-model-card" key={item.ruleId}><div className="voice-card-heading"><div><h3>{item.name}</h3><p className="voice-preview">{item.instruction}</p></div><span className="status-pill">{item.status}</span></div><div className="support-actions">{item.status === "proposed" ? <button className="primary-button compact" onClick={() => void setRuleStatus(item, "active")}>Activate</button> : null}{item.status === "active" ? <button className="quiet-button compact" onClick={() => void setRuleStatus(item, "disabled")}>Disable</button> : null}{item.status === "disabled" ? <button className="secondary-button compact" onClick={() => void setRuleStatus(item, "active")}>Enable</button> : null}{item.status !== "retired" ? <button className="quiet-button compact" onClick={() => void setRuleStatus(item, "retired")}>Retire</button> : null}</div></article>)}</div>\n      </section>\n\n      <section className="workspace-panel" aria-labelledby="draft-lint-heading">\n        <div className="panel-heading-row">\n          <div><p className="eyebrow">Provider-free challenge</p><h2 id="draft-lint-heading">Draft Pattern Check</h2></div>\n          <span className="status-pill">deterministic</span>\n        </div>\n        <p>Paste a draft for a transient check against explainable single-draft patterns and active machine-enforceable Writing Rules. This does not call a provider, calculate an AI probability, create a quality score, or save the draft.</p>\n        <textarea value={lintText} onChange={(event) => setLintText(event.target.value)} rows={8} placeholder="Paste a draft to challenge. The text stays transient in this view." />\n        <div className="support-actions"><button className="secondary-button compact" disabled={busy !== null || !lintText.trim()} onClick={() => void runDraftLint()}>Check draft patterns</button></div>\n        {lintResult ? (\n          <div className="voice-analysis-results">\n            <p className="voice-meta">Active Writing Rules: {lintResult.activeWritingRuleCount} | machine-enforceable: {lintResult.enforceableWritingRuleCount} | findings: {lintResult.findings.length}</p>\n            {lintResult.findings.length === 0 ? <p className="voice-rule">No deterministic findings. This is not a claim that the draft is perfect or human-written.</p> : (\n              <div className="voice-card-list">\n                {lintResult.findings.map((finding, index) => (\n                  <article className="voice-model-card" key={`${finding.ruleId}-${finding.startOffset ?? "global"}-${index}`}>\n                    <div className="voice-card-heading"><div><h3>{finding.category.replaceAll("_", " ")}</h3><p className="voice-meta">{finding.ruleId} | {finding.sourceKind}</p></div><span className={`status-pill ${finding.severity === "warning" ? "attention" : ""}`}>{finding.severity}</span></div>\n                    <p>{finding.reason}</p>\n                    {finding.matchedText ? <p className="voice-preview">Matched: {finding.matchedText}</p> : null}\n                    {finding.startOffset !== null && finding.endOffset !== null ? <p className="voice-meta">UTF-16 offsets {finding.startOffset}-{finding.endOffset}</p> : null}\n                    {finding.remediation ? <p className="voice-rule">Challenge: {finding.remediation}</p> : null}\n                  </article>\n                ))}\n              </div>\n            )}\n            {lintResult.unsupportedWritingRules.length > 0 ? (\n              <div className="voice-card-list">\n                {lintResult.unsupportedWritingRules.map((rule) => (\n                  <article className="voice-model-card" key={rule.ruleId}>\n                    <div className="voice-card-heading"><div><h3>{rule.name}</h3><p className="voice-meta">Active rule is advisory only</p></div><span className="status-pill attention">not auto-enforced</span></div>\n                    <p className="voice-preview">{rule.instruction}</p><p className="voice-rule">{rule.reason}</p>\n                  </article>\n                ))}\n              </div>\n            ) : null}\n          </div>\n        ) : null}\n      </section>\n\n      {busy ?''',
)
