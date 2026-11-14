use crate::{
    domain::interviews::{
        AnswerClassification, CompletenessStatus, InterviewResponseAction,
        SubmitInterviewResponseRequest, TurnType,
    },
    error::{ServiceResult, WorkLoreError},
};

const DEFAULT_FIELDS: &[&str] = &[
    "problem_or_opportunity",
    "actions_and_decisions",
    "metrics_and_evidence",
    "stakeholders",
    "tools_and_systems",
    "constraints",
    "lessons_learned",
];

pub fn response_values(
    request: &SubmitInterviewResponseRequest,
) -> ServiceResult<(
    TurnType,
    Option<AnswerClassification>,
    CompletenessStatus,
    f32,
    String,
)> {
    match request.action {
        InterviewResponseAction::Answer => {
            let text = request.text.trim();
            if text.is_empty() {
                return Err(WorkLoreError::InvalidInterviewAction(
                    "Answer text cannot be empty.".to_string(),
                ));
            }
            let classification = request
                .classification
                .unwrap_or(AnswerClassification::ConfirmedFact);
            let confidence = match classification {
                AnswerClassification::ConfirmedFact => 1.0,
                AnswerClassification::UserEstimate => 0.75,
                AnswerClassification::Uncertain => 0.45,
                AnswerClassification::NotApplicable => 1.0,
            };
            let status = if classification == AnswerClassification::NotApplicable {
                CompletenessStatus::NotApplicable
            } else if text.len() < 20 {
                CompletenessStatus::Partial
            } else {
                CompletenessStatus::Sufficient
            };
            Ok((
                TurnType::Answer,
                Some(classification),
                status,
                confidence,
                text.to_string(),
            ))
        }
        InterviewResponseAction::Skip => Ok((
            TurnType::Skip,
            None,
            CompletenessStatus::Partial,
            0.0,
            "Skipped for now.".to_string(),
        )),
        InterviewResponseAction::DoNotRemember => Ok((
            TurnType::DoNotRemember,
            Some(AnswerClassification::Uncertain),
            CompletenessStatus::Partial,
            0.0,
            "User does not remember.".to_string(),
        )),
    }
}

pub fn question_for_field(field: &str, claim: &str) -> String {
    let claim_reference = compact_claim(claim);
    match field {
        "problem_or_opportunity" => format!(
            "Before {claim_reference}, what was broken, inefficient, risky, or newly possible?"
        ),
        "actions_and_decisions" => format!(
            "For {claim_reference}, what did you personally change or decide? Include the important sequence, not just the team result."
        ),
        "metrics_and_evidence" => format!(
            "How do you know {claim_reference} worked? Share exact metrics, reasonable estimates, or concrete evidence."
        ),
        "stakeholders" => format!(
            "Who used, approved, resisted, or benefited from {claim_reference}?"
        ),
        "tools_and_systems" => format!(
            "Which tools, systems, data sources, or technical constraints shaped {claim_reference}?"
        ),
        "constraints" => format!(
            "What constraints made {claim_reference} difficult or limited your options?"
        ),
        "lessons_learned" => format!(
            "What did {claim_reference} teach you, or what would you do differently now?"
        ),
        _ => format!("What important detail is missing from {claim_reference}?"),
    }
}

pub fn normalized_fields(candidate_fields: &[String]) -> Vec<String> {
    let mut fields = candidate_fields
        .iter()
        .filter(|field| !field.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    for default_field in DEFAULT_FIELDS {
        if !fields.iter().any(|field| field == default_field) {
            fields.push((*default_field).to_string());
        }
    }
    fields.sort();
    fields.dedup();
    fields
}

fn compact_claim(claim: &str) -> String {
    let trimmed = claim.trim().trim_end_matches('.');
    if trimmed.chars().count() <= 90 {
        format!("\"{trimmed}\"")
    } else {
        let shortened = trimmed.chars().take(87).collect::<String>();
        format!("\"{shortened}...\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn question_map_keeps_work_grounded_in_the_claim() {
        let question = question_for_field(
            "metrics_and_evidence",
            "Reduced complaint investigation time from months to hours.",
        );
        assert!(question.contains("Reduced complaint investigation time"));
        assert!(question.contains("exact metrics"));
    }

    #[test]
    fn normalized_fields_keep_required_interview_coverage() {
        let fields = normalized_fields(&["metrics_and_evidence".to_string()]);
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
