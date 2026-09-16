use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleRecord {
    pub schema_version: u32,
    pub role_id: String,
    pub title: String,
    pub organization_entity_id: String,
    #[serde(default)]
    pub client_entity_ids: Vec<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_current: bool,
    pub summary: String,
    #[serde(default)]
    pub source_ids: Vec<String>,
    #[serde(default)]
    pub project_entity_ids: Vec<String>,
    #[serde(default)]
    pub story_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleSummary {
    pub role_id: String,
    pub title: String,
    pub organization_entity_id: String,
    pub organization_name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_current: bool,
    pub story_count: usize,
}

#[derive(Debug, Clone)]
pub struct ParsedRoleHeading {
    pub organization_name: String,
    pub title: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_current: bool,
}
