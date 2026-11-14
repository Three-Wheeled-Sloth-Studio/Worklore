use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryCandidate {
    pub schema_version: u32,
    pub candidate_id: String,
    pub source_refs: Vec<CandidateSourceRef>,
    pub candidate_type: CandidateType,
    pub status: CandidateStatus,
    pub claim: String,
    pub context: CandidateContext,
    pub missing_fields: Vec<String>,
    #[serde(default)]
    pub possible_existing_story_ids: Vec<String>,
    pub merged_into_id: Option<String>,
    #[serde(default)]
    pub split_into_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateSourceRef {
    pub source_id: String,
    pub fragment_id: String,
    pub captured_text: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateType {
    ResumeClaim,
    JobGap,
    GitCluster,
    ManualNote,
    InterviewFollowup,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateStatus {
    New,
    SavedForLater,
    ReadyToInterview,
    Interviewing,
    Merged,
    Split,
    Ignored,
    Unsupported,
    ConvertedToStory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateContext {
    #[serde(default)]
    pub role_ids: Vec<String>,
    #[serde(default)]
    pub entity_ids: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub metrics: Vec<String>,
    pub surrounding_heading: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateSummary {
    pub candidate_id: String,
    pub source_id: String,
    pub status: CandidateStatus,
    pub claim: String,
    pub surrounding_heading: Option<String>,
    pub missing_fields: Vec<String>,
    pub metrics: Vec<String>,
    pub created_at: String,
}

impl From<&StoryCandidate> for CandidateSummary {
    fn from(value: &StoryCandidate) -> Self {
        Self {
            candidate_id: value.candidate_id.clone(),
            source_id: value
                .source_refs
                .first()
                .map(|reference| reference.source_id.clone())
                .unwrap_or_default(),
            status: value.status,
            claim: value.claim.clone(),
            surrounding_heading: value.context.surrounding_heading.clone(),
            missing_fields: value.missing_fields.clone(),
            metrics: value.context.metrics.clone(),
            created_at: value.created_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractCandidatesResult {
    pub source_id: String,
    pub created_count: usize,
    pub existing_count: usize,
    pub candidates: Vec<CandidateSummary>,
    pub message: String,
}
