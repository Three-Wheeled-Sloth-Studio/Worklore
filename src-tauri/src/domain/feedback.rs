use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MarkPostPublishedRequest {
    pub post_id: String,
    pub revision_id: String,
    pub platform: String,
    pub published_at: String,
    pub publication_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManualPublicationView {
    pub publication_id: String,
    pub post_id: String,
    pub post_title: String,
    pub revision_id: String,
    pub platform: String,
    pub published_at: String,
    pub publication_url: Option<String>,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceMetrics {
    pub impressions: u64,
    pub reactions: u64,
    pub comments: u64,
    pub reposts: u64,
    pub saves: u64,
    pub profile_views: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordPostPerformanceRequest {
    pub publication_id: String,
    pub metrics: PerformanceMetrics,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceRecordView {
    pub performance_id: String,
    pub publication_id: String,
    pub metrics: PerformanceMetrics,
    pub notes: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackInsightView {
    pub kind: String,
    pub statement: String,
    pub sample_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackSnapshotView {
    pub publications: Vec<ManualPublicationView>,
    pub latest_performance: Vec<PerformanceRecordView>,
    pub insights: Vec<FeedbackInsightView>,
}
