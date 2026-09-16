use serde_json::Value;

#[derive(Debug, Clone)]
pub struct StructuredProviderRequest {
    pub model_id: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub response_schema: Value,
}

#[derive(Debug)]
pub struct StructuredProviderResponse {
    pub model_id: String,
    pub value: Value,
}
