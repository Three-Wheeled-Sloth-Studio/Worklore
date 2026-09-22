use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterviewSession {
    pub schema_version: u32,
    pub interview_id: String,
    pub candidate_ids: Vec<String>,
    pub story_id: Option<String>,
    pub status: InterviewStatus,
    pub question_round: u32,
    pub questions_in_current_round: u8,
    pub completeness: Vec<CompletenessItem>,
    pub turns: Vec<InterviewTurn>,
    pub active_question_turn_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InterviewStatus {
    NotStarted,
    Active,
    Paused,
    ReadyForSynthesis,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletenessItem {
    pub field: String,
    pub status: CompletenessStatus,
    pub confidence: f32,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompletenessStatus {
    Missing,
    Partial,
    Sufficient,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterviewTurn {
    pub turn_id: String,
    pub actor: TurnActor,
    pub turn_type: TurnType,
    pub text: String,
    #[serde(default)]
    pub target_fields: Vec<String>,
    pub answer_classification: Option<AnswerClassification>,
    pub provider_run_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TurnActor {
    Agent,
    User,
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TurnType {
    Question,
    Answer,
    Skip,
    DoNotRemember,
    MemoryPrompt,
    WhyItMatters,
    StateUpdate,
    Note,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnswerClassification {
    ConfirmedFact,
    UserEstimate,
    Uncertain,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InterviewResponseAction {
    Answer,
    Skip,
    DoNotRemember,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitInterviewResponseRequest {
    pub interview_id: String,
    pub action: InterviewResponseAction,
    pub text: String,
    pub classification: Option<AnswerClassification>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterviewSummary {
    pub interview_id: String,
    pub candidate_id: String,
    pub candidate_claim: String,
    pub status: InterviewStatus,
    pub current_question: Option<String>,
    pub current_target_field: Option<String>,
    pub completed_field_count: usize,
    pub total_field_count: usize,
    pub last_updated_at: String,
}
