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

pub const OLLAMA_PROVIDER_ID: &str = "ollama";
pub const ANALYZE_VOICE_EVIDENCE_OPERATION: &str = "analyze_voice_evidence";
pub const ANALYZE_VOICE_EVIDENCE_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSettingsView {
    pub selected_provider_id: Option<String>,
    pub ollama_base_url: String,
    pub ollama_model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviderSettingsRequest {
    pub selected_provider_id: Option<String>,
    pub ollama_base_url: String,
    pub ollama_model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelView {
    pub model_id: String,
    pub display_name: String,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConnectionView {
    pub provider_id: String,
    pub available: bool,
    pub model_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeVoiceEvidenceRequest {
    pub provider_id: String,
    pub model_id: String,
    pub voice_evidence_ids: Vec<String>,
    pub user_guidance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VoiceTraitProposal {
    pub proposal_id: String,
    pub name: String,
    pub value: String,
    pub evidence_ids: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WritingRuleProposal {
    pub proposal_id: String,
    pub name: String,
    pub instruction: String,
    pub evidence_ids: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VoiceAnalysisProposalSet {
    pub run_id: String,
    pub operation_id: String,
    pub operation_version: u32,
    pub provider_id: String,
    pub model_id: String,
    pub proposals: Vec<VoiceTraitProposal>,
    pub rule_proposals: Vec<WritingRuleProposal>,
}
