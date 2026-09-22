const MAX_DISCOVERY_QUERIES: usize = 4;

const BROAD_EXPLORATION_QUERIES: [&str; 4] = [
    "workplace leadership management trends",
    "technology digital work trends",
    "business operations customer experience trends",
    "professional practice skills career trends",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryQueryPlan {
    queries: Vec<String>,
    explicit_focus: bool,
}

impl DiscoveryQueryPlan {
    pub fn queries(&self) -> &[String] {
        &self.queries
    }

    pub fn is_explicit_focus(&self) -> bool {
        self.explicit_focus
    }
}

pub fn build_query_plan(
    focus: &str,
    theme_labels: &[String],
    target_context_labels: &[String],
    good_candidate_titles: &[String],
) -> DiscoveryQueryPlan {
    let focus = normalize_query(focus);
    if !focus.is_empty() {
        return DiscoveryQueryPlan {
            queries: vec![focus],
            explicit_focus: true,
        };
    }

    let mut queries = Vec::with_capacity(MAX_DISCOVERY_QUERIES);
    push_first_unique(&mut queries, theme_labels);
    push_first_unique(&mut queries, target_context_labels);
    push_first_unique(&mut queries, good_candidate_titles);

    for fallback in BROAD_EXPLORATION_QUERIES {
        push_unique(&mut queries, fallback);
        if queries.len() == MAX_DISCOVERY_QUERIES {
            break;
        }
    }

    DiscoveryQueryPlan {
        queries,
        explicit_focus: false,
    }
}

fn push_first_unique(queries: &mut Vec<String>, candidates: &[String]) {
    if queries.len() >= MAX_DISCOVERY_QUERIES.saturating_sub(1) {
        return;
    }
    if let Some(candidate) = candidates
        .iter()
        .map(|value| normalize_query(value))
        .find(|value| !value.is_empty() && !contains_query(queries, value))
    {
        queries.push(candidate);
    }
}

fn push_unique(queries: &mut Vec<String>, value: &str) {
    if queries.len() >= MAX_DISCOVERY_QUERIES {
        return;
    }
    let normalized = normalize_query(value);
    if !normalized.is_empty() && !contains_query(queries, &normalized) {
        queries.push(normalized);
    }
}

fn contains_query(queries: &[String], candidate: &str) -> bool {
    queries
        .iter()
        .any(|value| value.eq_ignore_ascii_case(candidate))
}

fn normalize_query(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_focus_succeeds_without_themes_or_other_personal_signals() {
        let plan = build_query_plan("", &[], &[], &[]);

        assert_eq!(plan.queries().len(), MAX_DISCOVERY_QUERIES);
        assert!(plan.queries().iter().all(|query| !query.is_empty()));
    }

    #[test]
    fn blank_focus_is_bounded_and_diverse() {
        let plan = build_query_plan(
            "   ",
            &["AI product work".into(), "Decision quality".into()],
            &["Director of Product".into(), "Customer experience".into()],
            &["New agent workflow release".into()],
        );

        assert_eq!(plan.queries().len(), MAX_DISCOVERY_QUERIES);
        let unique = plan
            .queries()
            .iter()
            .map(|query| query.to_ascii_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), plan.queries().len());
    }

    #[test]
    fn explicit_focus_narrows_discovery_to_one_query() {
        let plan = build_query_plan(
            "  agentic workflow management  ",
            &["Ignored theme".into()],
            &["Ignored target".into()],
            &["Ignored feedback".into()],
        );

        assert!(plan.is_explicit_focus());
        assert_eq!(
            plan.queries(),
            &[String::from("agentic workflow management")]
        );
    }

    #[test]
    fn active_theme_can_inform_but_is_not_required() {
        let with_theme = build_query_plan("", &["Evidence-led product work".into()], &[], &[]);
        let without_theme = build_query_plan("", &[], &[], &[]);

        assert_eq!(with_theme.queries()[0], "Evidence-led product work");
        assert!(!without_theme.queries().is_empty());
    }

    #[test]
    fn planner_has_no_story_or_proof_point_input_surface() {
        let private_story = "Project Nightjar reduced internal cycle time by 83 percent";
        let private_proof = "Recovered confidential amount from Private Client";
        let plan = build_query_plan(
            "",
            &["Public theme".into()],
            &["Public target context".into()],
            &["Public external discussion".into()],
        );

        let rendered = plan.queries().join(" ");
        assert!(!rendered.contains(private_story));
        assert!(!rendered.contains(private_proof));
    }
}
