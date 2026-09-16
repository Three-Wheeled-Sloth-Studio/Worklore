use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultDocument {
    pub schema_version: u32,
    pub vault_id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub privacy: VaultPrivacy,
    pub features: VaultFeatures,
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultPrivacy {
    pub cloud_identifier_mode: CloudIdentifierMode,
    pub block_cloud_when_high_risk_review_pending: bool,
    pub retain_provider_bodies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultFeatures {
    pub entity_registry: bool,
    pub ollama: bool,
    pub gemini: bool,
    pub manual_workspace: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CloudIdentifierMode {
    Redact,
    Include,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultSummary {
    pub schema_version: u32,
    pub vault_id: String,
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
    pub source_count: usize,
    pub story_count: usize,
    pub pending_privacy_review_count: usize,
    pub cloud_identifier_mode: CloudIdentifierMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Resume,
    JobDescription,
    WritingSample,
    InterviewTranscript,
    GitSnapshot,
    Other,
}

impl SourceType {
    pub fn folder_name(self) -> &'static str {
        match self {
            Self::Resume => "resumes",
            Self::JobDescription => "job-descriptions",
            Self::WritingSample => "writing-samples",
            Self::InterviewTranscript => "interview-transcripts",
            Self::GitSnapshot => "git-snapshots",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentHash {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionState {
    pub status: ExtractionStatus,
    pub extractor_version: Option<String>,
    pub text_path: Option<String>,
    pub character_count: usize,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionStatus {
    Pending,
    Complete,
    Partial,
    Unsupported,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyScanState {
    pub status: PrivacyScanStatus,
    pub scan_version: Option<String>,
    pub scanned_at: Option<String>,
    #[serde(default)]
    pub review_item_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyScanStatus {
    Pending,
    Complete,
    NeedsReview,
    Failed,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceProvenance {
    pub import_method: String,
    pub parent_source_id: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDocument {
    pub schema_version: u32,
    pub source_id: String,
    pub source_type: SourceType,
    pub display_name: String,
    pub stored_path: String,
    pub original_file_name: String,
    pub original_path_hint: Option<String>,
    pub media_type: Option<String>,
    pub byte_size: u64,
    pub content_hash: ContentHash,
    pub imported_at: String,
    pub updated_at: String,
    pub extraction: ExtractionState,
    pub privacy_scan: PrivacyScanState,
    pub provenance: SourceProvenance,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSummary {
    pub source_id: String,
    pub source_type: SourceType,
    pub display_name: String,
    pub stored_path: String,
    pub original_file_name: String,
    pub imported_at: String,
    pub extraction_status: ExtractionStatus,
    pub privacy_scan_status: PrivacyScanStatus,
    pub duplicate_of_source_id: Option<String>,
}

impl From<&SourceDocument> for SourceSummary {
    fn from(value: &SourceDocument) -> Self {
        Self {
            source_id: value.source_id.clone(),
            source_type: value.source_type,
            display_name: value.display_name.clone(),
            stored_path: value.stored_path.clone(),
            original_file_name: value.original_file_name.clone(),
            imported_at: value.imported_at.clone(),
            extraction_status: value.extraction.status,
            privacy_scan_status: value.privacy_scan.status,
            duplicate_of_source_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSourceResult {
    pub source: SourceSummary,
    pub created: bool,
    pub duplicate_detected: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateEntityRegistry {
    pub schema_version: u32,
    pub registry_id: String,
    pub vault_id: String,
    #[serde(default)]
    pub token_counters: std::collections::BTreeMap<String, u32>,
    #[serde(default)]
    pub entities: Vec<PrivateEntity>,
    #[serde(default)]
    pub token_redirects: Vec<TokenRedirect>,
    #[serde(default)]
    pub ignored_terms: Vec<IgnoredTerm>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateEntity {
    pub schema_version: u32,
    pub entity_id: String,
    pub entity_type: EntityType,
    pub canonical_name: String,
    pub public_token: String,
    pub public_description: Option<String>,
    pub sensitivity: EntitySensitivity,
    pub status: EntityStatus,
    #[serde(default)]
    pub aliases: Vec<EntityAlias>,
    #[serde(default)]
    pub relationships: Vec<EntityRelationship>,
    #[serde(default)]
    pub occurrences: Vec<EntityOccurrence>,
    pub redirect_to_entity_id: Option<String>,
    #[serde(default)]
    pub retired_tokens: Vec<String>,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Employer,
    Client,
    Project,
    Product,
    System,
    Repository,
    Person,
    Location,
    Email,
    Phone,
    Url,
    Account,
    Identifier,
    Organization,
    UserDefined,
}

impl EntityType {
    pub fn token_family(self) -> &'static str {
        match self {
            Self::Employer => "EMPLOYER",
            Self::Client => "CLIENT",
            Self::Project => "PROJECT",
            Self::Product => "PRODUCT",
            Self::System => "SYSTEM",
            Self::Repository => "REPOSITORY",
            Self::Person => "PERSON",
            Self::Location => "LOCATION",
            Self::Email => "EMAIL",
            Self::Phone => "PHONE",
            Self::Url => "URL",
            Self::Account => "ACCOUNT",
            Self::Identifier => "IDENTIFIER",
            Self::Organization => "ORGANIZATION",
            Self::UserDefined => "ENTITY",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntitySensitivity {
    Public,
    Private,
    AskBeforeCloud,
    NeverSendToCloud,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityStatus {
    Confirmed,
    Provisional,
    Merged,
    Split,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityAlias {
    pub value: String,
    pub normalized_value: String,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRelationship {
    pub relationship_id: String,
    pub relationship_type: String,
    pub target_entity_id: String,
    pub status: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityOccurrence {
    pub occurrence_id: String,
    pub record_type: String,
    pub record_id: String,
    pub locator: String,
    pub matched_text: String,
    pub extraction_confidence: f32,
    pub type_confidence: f32,
    pub identity_match_confidence: f32,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRedirect {
    pub retired_token: String,
    pub surviving_entity_id: String,
    pub created_at: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgnoredTerm {
    pub normalized_value: String,
    pub scope: String,
    pub record_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityReviewItem {
    pub schema_version: u32,
    pub review_item_id: String,
    pub record_type: String,
    pub record_id: String,
    pub locator: String,
    pub context_excerpt: String,
    pub matched_text: String,
    pub normalized_text: String,
    pub suggested_entity_type: String,
    pub scores: ReviewScores,
    #[serde(default)]
    pub candidate_matches: Vec<ReviewCandidateMatch>,
    pub risk: String,
    pub status: String,
    pub resolution: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewScores {
    pub extraction: f32,
    #[serde(rename = "type")]
    pub type_score: f32,
    pub best_identity_match: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCandidateMatch {
    pub entity_id: String,
    pub score: f32,
    pub reasons: Vec<String>,
}
