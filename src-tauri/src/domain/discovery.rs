use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryFreshness {
    Day,
    Week,
    Month,
}

impl DiscoveryFreshness {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        }
    }

    pub fn brave_value(self) -> &'static str {
        match self {
            Self::Day => "pd",
            Self::Week => "pw",
            Self::Month => "pm",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryOpportunityStatus {
    Candidate,
    InspirationSaved,
    TopicCreated,
    Dismissed,
}

impl DiscoveryOpportunityStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::InspirationSaved => "inspiration_saved",
            Self::TopicCreated => "topic_created",
            Self::Dismissed => "dismissed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "candidate" => Some(Self::Candidate),
            "inspiration_saved" => Some(Self::InspirationSaved),
            "topic_created" => Some(Self::TopicCreated),
            "dismissed" => Some(Self::Dismissed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryFeedbackVerdict {
    GoodCandidate,
    NotForMe,
    NotNow,
}

impl DiscoveryFeedbackVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GoodCandidate => "good_candidate",
            Self::NotForMe => "not_for_me",
            Self::NotNow => "not_now",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverySourceView {
    pub title: String,
    pub url: String,
    pub description: String,
    pub age: Option<String>,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryMatchView {
    pub kind: String,
    pub record_id: String,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryOpportunityView {
    pub opportunity_id: String,
    pub run_id: String,
    pub title: String,
    pub summary: String,
    pub sources: Vec<DiscoverySourceView>,
    pub theme_matches: Vec<DiscoveryMatchView>,
    pub standing_matches: Vec<DiscoveryMatchView>,
    pub audience_matches: Vec<DiscoveryMatchView>,
    pub why_now: String,
    pub possible_angle: String,
    pub concerns: Vec<String>,
    pub feedback_adjustment: Option<String>,
    pub status: DiscoveryOpportunityStatus,
    pub topic_id: Option<String>,
    pub inspiration_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanDiscoveryRequest {
    pub focus: String,
    pub freshness: DiscoveryFreshness,
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryScanResult {
    pub run_id: String,
    pub external_query: String,
    pub freshness: DiscoveryFreshness,
    pub feedback_examples_used: usize,
    pub opportunities: Vec<DiscoveryOpportunityView>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDiscoveryFeedbackRequest {
    pub opportunity_id: String,
    pub verdict: DiscoveryFeedbackVerdict,
    pub reasons: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryFeedbackView {
    pub feedback_id: String,
    pub opportunity_id: String,
    pub verdict: DiscoveryFeedbackVerdict,
    pub reasons: Vec<String>,
    pub note: String,
    pub normalized_signals: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevelopDiscoveryTopicRequest {
    pub opportunity_id: String,
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevelopDiscoveryTopicResult {
    pub topic_id: String,
    pub inspiration_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDiscoveryInspirationResult {
    pub inspiration_id: String,
}
