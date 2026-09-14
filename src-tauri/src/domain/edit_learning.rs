use serde::Serialize;

use crate::domain::posts::{PostRevisionAuthorship, PostRevisionOrigin};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EditChangeKind {
    Addition,
    Removal,
    Replacement,
}

impl EditChangeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Addition => "addition",
            Self::Removal => "removal",
            Self::Replacement => "replacement",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EditChangeView {
    pub kind: EditChangeKind,
    pub removed_text: Option<String>,
    pub added_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EditObservationView {
    pub post_id: String,
    pub parent_revision_id: String,
    pub child_revision_id: String,
    pub parent_origin: PostRevisionOrigin,
    pub parent_authorship_state: PostRevisionAuthorship,
    pub child_origin: PostRevisionOrigin,
    pub child_authorship_state: PostRevisionAuthorship,
    pub changes: Vec<EditChangeView>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EditPairProvenanceView {
    pub post_id: String,
    pub parent_revision_id: String,
    pub child_revision_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecurringEditPreferenceProposalView {
    pub proposal_key: String,
    pub kind: EditChangeKind,
    pub statement: String,
    pub support_count: usize,
    pub distinct_post_count: usize,
    pub supporting_pairs: Vec<EditPairProvenanceView>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EditLearningAnalysisView {
    pub observations: Vec<EditObservationView>,
    pub proposals: Vec<RecurringEditPreferenceProposalView>,
}
