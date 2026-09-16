use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    domain::providers::{GEMINI_PROVIDER_ID, OPENAI_PROVIDER_ID},
    error::{ServiceResult, WorkLoreError},
    services::structured_provider::{StructuredProviderRequest, StructuredProviderResponse},
};

const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ApiModel>,
}

#[derive(Debug, Deserialize)]
struct ApiModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    #[serde(default)]
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

pub fn provider_base_url(provider_id: &str) -> ServiceResult<&'static str> {
    match provider_id {
        OPENAI_PROVIDER_ID => Ok(OPENAI_BASE_URL),
        GEMINI_PROVIDER_ID => Ok(GEMINI_BASE_URL),
        _ => Err(provider_error(
            "not_configured",
            "The selected BYOK provider is not registered.",
        )),
    }
}

pub fn provider_label(provider_id: &str) -> &'static str {
    match provider_id {
        OPENAI_PROVIDER_ID => "OpenAI",
        GEMINI_PROVIDER_ID => "Gemini",
        _ => "BYOK provider",
    }
}

pub async fn list_models(
    provider_id: &str,
    api_key: &str,
) -> ServiceResult<Vec<crate::domain::providers::ProviderModelView>> {
    let base_url = provider_base_url(provider_id)?;
    let api_key = require_api_key(api_key)?;
    let response = client()?
        .get(format!("{base_url}/models"))
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|error| map_transport_error(error, provider_id))?;
    if !response.status().is_success() {
        return Err(map_status(response.status(), provider_id, false));
    }
    let payload: ModelsResponse = response.json().await.map_err(|_| {
        provider_error(
            "invalid_structured_output",
            format!(
                "{} returned an unreadable model-list response.",
                provider_label(provider_id)
            ),
        )
    })?;
    let mut models = payload
        .data
        .into_iter()
        .filter_map(|model| {
            let id = model.id.trim();
            if id.is_empty() {
                None
            } else {
                Some(crate::domain::providers::ProviderModelView {
                    model_id: id.to_string(),
                    display_name: id.to_string(),
                    parameter_size: None,
                    quantization_level: None,
                })
            }
        })
        .collect::<Vec<_>>();
    models.sort_by(|left, right| left.model_id.cmp(&right.model_id));
    models.dedup_by(|left, right| left.model_id == right.model_id);
    Ok(models)
}

pub async fn run_structured(
    provider_id: &str,
    api_key: &str,
    request: StructuredProviderRequest,
) -> ServiceResult<StructuredProviderResponse> {
    let base_url = provider_base_url(provider_id)?;
    let api_key = require_api_key(api_key)?;
    let StructuredProviderRequest {
        model_id,
        system_prompt,
        user_prompt,
        response_schema,
    } = request;
    let response_schema = schema_for_provider(provider_id, response_schema);
    let response = client()?
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth(api_key)
        .json(&json!({
            "model": &model_id,
            "messages": [
                {"role": "system", "content": &system_prompt},
                {"role": "user", "content": &user_prompt}
            ],
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "worklore_structured_output",
                    "strict": true,
                    "schema": response_schema
                }
            }
        }))
        .send()
        .await
        .map_err(|error| map_transport_error(error, provider_id))?;
    if !response.status().is_success() {
        return Err(map_status(response.status(), provider_id, true));
    }
    let payload: ChatResponse = response.json().await.map_err(|_| {
        provider_error(
            "invalid_structured_output",
            format!(
                "{} returned an unreadable structured response.",
                provider_label(provider_id)
            ),
        )
    })?;
    let content = payload
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .filter(|content| !content.trim().is_empty())
        .ok_or_else(|| {
            provider_error(
                "invalid_structured_output",
                format!(
                    "{} returned no structured response content.",
                    provider_label(provider_id)
                ),
            )
        })?;
    let value = serde_json::from_str(&content).map_err(|_| {
        provider_error(
            "invalid_structured_output",
            format!(
                "{} did not return JSON matching the requested structured-output contract.",
                provider_label(provider_id)
            ),
        )
    })?;
    Ok(StructuredProviderResponse { model_id, value })
}

fn schema_for_provider(provider_id: &str, mut schema: Value) -> Value {
    if provider_id == GEMINI_PROVIDER_ID {
        strip_gemini_unsupported_schema_keywords(&mut schema);
    }
    schema
}

fn strip_gemini_unsupported_schema_keywords(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("$schema");
            map.remove("minLength");
            map.remove("maxLength");
            map.remove("uniqueItems");
            for child in map.values_mut() {
                strip_gemini_unsupported_schema_keywords(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                strip_gemini_unsupported_schema_keywords(child);
            }
        }
        _ => {}
    }
}

fn require_api_key(api_key: &str) -> ServiceResult<&str> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(provider_error(
            "not_configured",
            "Save an API key for the selected BYOK provider before using it.",
        ));
    }
    Ok(api_key)
}

fn client() -> ServiceResult<Client> {
    Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|_| provider_error("unknown", "WorkLore could not initialize the BYOK client."))
}

fn map_transport_error(error: reqwest::Error, provider_id: &str) -> WorkLoreError {
    if error.is_timeout() {
        return provider_error(
            "timeout",
            format!("The {} request timed out.", provider_label(provider_id)),
        );
    }
    if error.is_connect() {
        return provider_error(
            "provider_unavailable",
            format!("{} is not reachable.", provider_label(provider_id)),
        );
    }
    provider_error(
        "unknown",
        format!(
            "The {} request failed before completion.",
            provider_label(provider_id)
        ),
    )
}

fn map_status(status: StatusCode, provider_id: &str, model_request: bool) -> WorkLoreError {
    let label = provider_label(provider_id);
    match status.as_u16() {
        400 => provider_error(
            "provider_rejected_request",
            format!("{label} rejected the structured request."),
        ),
        401 | 403 => provider_error(
            "authentication_failed",
            format!("{label} rejected the saved API key."),
        ),
        404 if model_request => provider_error(
            "model_unavailable",
            format!("The selected {label} model is not available."),
        ),
        408 => provider_error("timeout", format!("The {label} request timed out.")),
        413 => provider_error(
            "request_too_large",
            format!("The request is too large for {label}."),
        ),
        429 => provider_error(
            "rate_limited",
            format!("{label} temporarily refused the request because it is rate limited."),
        ),
        _ => provider_error(
            "provider_unavailable",
            format!("{label} returned an unsuccessful HTTP response."),
        ),
    }
}

fn provider_error(code: &'static str, message: impl Into<String>) -> WorkLoreError {
    WorkLoreError::ProviderOperation {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_byok_endpoints_are_fixed_and_explicit() {
        assert_eq!(
            provider_base_url(OPENAI_PROVIDER_ID).unwrap(),
            OPENAI_BASE_URL
        );
        assert_eq!(
            provider_base_url(GEMINI_PROVIDER_ID).unwrap(),
            GEMINI_BASE_URL
        );
        assert!(provider_base_url("custom").is_err());
    }

    #[test]
    fn gemini_schema_removes_keywords_outside_its_compatibility_subset() {
        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {
                "name": {"type": "string", "minLength": 1, "maxLength": 40},
                "ids": {"type": "array", "uniqueItems": true, "items": {"type": "string"}}
            }
        });
        let cleaned = schema_for_provider(GEMINI_PROVIDER_ID, schema);
        let encoded = serde_json::to_string(&cleaned).unwrap();
        assert!(!encoded.contains("$schema"));
        assert!(!encoded.contains("minLength"));
        assert!(!encoded.contains("maxLength"));
        assert!(!encoded.contains("uniqueItems"));
    }

    #[test]
    fn normalized_statuses_do_not_expose_provider_response_bodies() {
        let error = map_status(StatusCode::UNAUTHORIZED, OPENAI_PROVIDER_ID, false);
        match error {
            WorkLoreError::ProviderOperation { code, message } => {
                assert_eq!(code, "authentication_failed");
                assert_eq!(message, "OpenAI rejected the saved API key.");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
