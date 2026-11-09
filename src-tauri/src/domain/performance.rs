use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationOutcome {
    Succeeded,
    Failed,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationMetric {
    pub schema_version: u32,
    pub run_id: String,
    pub parent_run_id: Option<String>,
    pub operation: String,
    pub phase: String,
    pub started_at: String,
    pub completed_at: String,
    pub duration_ms: u64,
    pub outcome: OperationOutcome,
    pub error_code: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveOperation {
    pub schema_version: u32,
    pub run_id: String,
    pub operation: String,
    pub phase: String,
    pub started_at: String,
    pub updated_at: String,
    pub elapsed_ms: u64,
    #[serde(default)]
    pub process_id: u32,
    pub progress_current: Option<u64>,
    pub progress_total: Option<u64>,
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSnapshot {
    pub active_operations: Vec<ActiveOperation>,
    pub recent_metrics: Vec<OperationMetric>,
}
