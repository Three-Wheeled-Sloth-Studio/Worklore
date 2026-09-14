use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PostStatus {
    Working,
    FinalApproved,
}

impl PostStatus {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "working" => Ok(Self::Working),
            "final_approved" => Ok(Self::FinalApproved),
            _ => Err(format!("Unknown post status {value}.")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PostRevisionOrigin {
    User,
    Model,
}

impl PostRevisionOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Model => "model",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PostRevisionAuthorship {
    UserAuthored,
    UserEditedModel,
    ModelGenerated,
}

impl PostRevisionAuthorship {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UserAuthored => "user_authored",
            Self::UserEditedModel => "user_edited_model",
            Self::ModelGenerated => "model_generated",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "user_authored" => Ok(Self::UserAuthored),
            "user_edited_model" => Ok(Self::UserEditedModel),
            "model_generated" => Ok(Self::ModelGenerated),
            _ => Err(format!("Unknown post revision authorship state {value}.")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PostSupportRole {
    EvidenceSource,
    Evidence,
    Story,
    ProofPoint,
    Topic,
    Theme,
    Inspiration,
    TargetContext,
    VoiceEvidence,
}

impl PostSupportRole {
    pub fn relationship_type(self) -> &'static str {
        match self {
            Self::EvidenceSource => "post_evidence_source",
            Self::Evidence => "post_evidence",
            Self::Story => "post_story",
            Self::ProofPoint => "post_proof_point",
            Self::Topic => "post_topic",
            Self::Theme => "post_theme",
            Self::Inspiration => "post_inspiration",
            Self::TargetContext => "post_target_context",
            Self::VoiceEvidence => "post_voice_evidence",
        }
    }

    pub fn target_type(self) -> &'static str {
        match self {
            Self::EvidenceSource => "source",
            Self::Evidence => "evidence",
            Self::Story => "story",
            Self::ProofPoint => "proof_point",
            Self::Topic => "topic",
            Self::Theme => "theme",
            Self::Inspiration => "inspiration",
            Self::TargetContext => "target_context",
            Self::VoiceEvidence => "voice_evidence",
        }
    }

    pub fn parse_relationship(value: &str) -> Option<Self> {
        match value {
            "post_evidence_source" => Some(Self::EvidenceSource),
            "post_evidence" => Some(Self::Evidence),
            "post_story" => Some(Self::Story),
            "post_proof_point" => Some(Self::ProofPoint),
            "post_topic" => Some(Self::Topic),
            "post_theme" => Some(Self::Theme),
            "post_inspiration" => Some(Self::Inspiration),
            "post_target_context" => Some(Self::TargetContext),
            "post_voice_evidence" => Some(Self::VoiceEvidence),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostRequest {
    pub title: String,
    pub text: String,
    pub origin: PostRevisionOrigin,
    pub authorship_state: PostRevisionAuthorship,
    pub provider_run_id: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppendPostRevisionRequest {
    pub post_id: String,
    pub text: String,
    pub origin: PostRevisionOrigin,
    pub authorship_state: PostRevisionAuthorship,
    pub provider_run_id: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApprovePostRevisionRequest {
    pub post_id: String,
    pub revision_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LinkPostSupportingMaterialRequest {
    pub post_id: String,
    pub role: PostSupportRole,
    pub target_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PostRecordView {
    pub post_id: String,
    pub title: String,
    pub status: PostStatus,
    pub current_revision_id: String,
    pub final_approved_revision_id: Option<String>,
    pub approved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PostRevisionView {
    pub revision_id: String,
    pub post_id: String,
    pub sequence: u32,
    pub parent_revision_id: Option<String>,
    pub text: String,
    pub origin: PostRevisionOrigin,
    pub authorship_state: PostRevisionAuthorship,
    pub provider_run_id: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PostSupportingMaterialView {
    pub relationship_id: String,
    pub role: PostSupportRole,
    pub target_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PostLineageView {
    pub post: PostRecordView,
    pub revisions: Vec<PostRevisionView>,
    pub supporting_material: Vec<PostSupportingMaterialView>,
}
