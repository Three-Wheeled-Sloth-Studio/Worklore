use serde::{Deserialize, Serialize};

use crate::domain::models::PrivacyScanStatus;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StoryStatus {
    Draft,
    Interviewing,
    ReadyForReview,
    Validated,
    Finalized,
    Archived,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StoryType {
    Accomplishment,
    Decision,
    Failure,
    Lesson,
    Conflict,
    Leadership,
    TechnicalDelivery,
    ProcessChange,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimClassification {
    ConfirmedFact,
    UserEstimate,
    ModelInference,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryClaim {
    pub claim_id: String,
    pub text: String,
    pub classification: ClaimClassification,
    pub confidence: f32,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    pub included_by_default: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    SourceFragment,
    InterviewAnswer,
    GitCommit,
    GitDiffSummary,
    UserConfirmation,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryEvidence {
    pub evidence_id: String,
    pub evidence_type: EvidenceType,
    pub source_id: Option<String>,
    pub locator: String,
    pub captured_text: String,
    pub captured_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryContent {
    pub summary: String,
    pub situation: String,
    pub problem_or_opportunity: String,
    #[serde(default)]
    pub responsibilities: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub actions_and_decisions: Vec<String>,
    #[serde(default)]
    pub alternatives_considered: Vec<String>,
    #[serde(default)]
    pub tools_and_systems: Vec<String>,
    #[serde(default)]
    pub stakeholders: Vec<String>,
    #[serde(default)]
    pub outcomes: Vec<String>,
    #[serde(default)]
    pub metrics: Vec<String>,
    #[serde(default)]
    pub lessons_learned: Vec<String>,
    #[serde(default)]
    pub operating_philosophy: Vec<String>,
    #[serde(default)]
    pub reusable_themes: Vec<String>,
    #[serde(default)]
    pub skills_demonstrated: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryDisclosure {
    pub default_public_mode: String,
    pub review_required: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryPrivacyScan {
    pub status: PrivacyScanStatus,
    pub content_revision: u32,
    pub scanned_at: Option<String>,
    #[serde(default)]
    pub review_item_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryRecord {
    pub schema_version: u32,
    pub story_id: String,
    pub title: String,
    pub status: StoryStatus,
    pub story_type: StoryType,
    #[serde(default)]
    pub role_ids: Vec<String>,
    #[serde(default)]
    pub entity_ids: Vec<String>,
    #[serde(default)]
    pub related_story_ids: Vec<String>,
    #[serde(default)]
    pub job_requirement_ids: Vec<String>,
    pub content: StoryContent,
    #[serde(default)]
    pub claims: Vec<StoryClaim>,
    #[serde(default)]
    pub evidence: Vec<StoryEvidence>,
    pub disclosure: StoryDisclosure,
    pub privacy_scan: StoryPrivacyScan,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySummary {
    pub story_id: String,
    pub title: String,
    pub status: StoryStatus,
    pub story_type: StoryType,
    pub summary: String,
    pub role_id: Option<String>,
    pub role_title: Option<String>,
    pub organization_name: Option<String>,
    pub metrics: Vec<String>,
    pub outcomes: Vec<String>,
    pub privacy_scan_status: PrivacyScanStatus,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportStoryResponseRequest {
    pub interview_id: String,
    pub response_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportStoryResponseResult {
    pub story: StorySummary,
    pub created: bool,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorySynthesisResponse {
    pub title: String,
    pub summary: String,
    pub situation: String,
    pub problem: String,
    pub constraints: Vec<String>,
    pub responsibilities: Vec<String>,
    pub actions: Vec<String>,
    pub decisions: Vec<String>,
    pub tools_and_systems: Vec<String>,
    pub stakeholders: Vec<String>,
    pub outcomes: Vec<String>,
    pub metrics: Vec<SynthesisMetric>,
    pub lessons_learned: Vec<String>,
    pub operating_philosophy: Vec<String>,
    pub claims: Vec<SynthesisClaim>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SynthesisMetric {
    pub statement: String,
    pub evidence_level: SynthesisEvidenceLevel,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SynthesisClaim {
    pub statement: String,
    pub evidence_level: SynthesisEvidenceLevel,
    pub supporting_fields: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SynthesisEvidenceLevel {
    ConfirmedFact,
    UserEstimate,
    Uncertain,
    Unsupported,
}
