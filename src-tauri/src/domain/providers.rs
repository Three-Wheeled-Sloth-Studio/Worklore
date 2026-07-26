use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManualWorkspaceTarget {
    Chatgpt,
    Claude,
    Gemini,
    Generic,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateManualWorkspaceRequest {
    pub interview_id: String,
    pub output_directory: String,
    pub target: ManualWorkspaceTarget,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualWorkspaceResult {
    pub workspace_id: String,
    pub workspace_path: String,
    pub target: ManualWorkspaceTarget,
    pub privacy_mode: String,
    pub replacement_count: usize,
    pub warning_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyPreflightSummary {
    pub mode: String,
    pub replacement_count: usize,
    pub replacements: Vec<RedactionReplacement>,
    pub warning_review_item_ids: Vec<String>,
    pub blocked_review_item_ids: Vec<String>,
    pub contains_never_send_entities: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedactionReplacement {
    pub entity_id: String,
    pub entity_type: String,
    pub replacement: String,
    pub occurrence_count: usize,
}
