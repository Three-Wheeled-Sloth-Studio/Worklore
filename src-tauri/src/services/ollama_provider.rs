use std::time::Duration;

use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    domain::providers::ProviderModelView,
    error::{ServiceResult, WorkLoreError},
};

#[derive(Debug, Clone)]
pub struct StructuredProviderRequest {
    pub model_id: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub response_schema: Value,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}

#[derive(Debug, Deserialize)]
struct TagModel {
    name: String,
    #[serde(default)]
    details: TagDetails,
}

#[derive(Debug, Default, Deserialize)]
struct TagDetails {
    parameter_size: Option<String>,
    quantization_level: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: String,
}

pub fn normalize_base_url(value: &str) -> ServiceResult<String> {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(provider_error(
            "not_configured",
            "Configure the local Ollama base URL before using the provider.",
        ));
    }
    let parsed = Url::parse(trimmed).map_err(|_| {
        provider_error(
            "not_configured",
            "The Ollama base URL is not a valid HTTP URL.",
        )
    })?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(provider_error(
            "not_configured",
            "The Ollama base URL must use http or https.",
        ));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(provider_error(
            "not_configured",
            "Do not embed credentials in the Ollama base URL.",
        ));
    }
    let host = parsed.host_str().unwrap_or_default();
    if !matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]") {
        return Err(provider_error(
            "not_configured",
            "This Ollama adapter is local-only in the current slice. Use localhost, 127.0.0.1, or ::1.",
        ));
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(provider_error(
            "not_configured",
            "The Ollama base URL cannot contain a query string or fragment.",
        ));
    }
    if !matches!(parsed.path(), "" | "/") {
        return Err(provider_error(
            "not_configured",
            "The Ollama base URL must point to the server root, not an API subpath.",
        ));
    }
    Ok(trimmed.to_string())
}

pub async fn list_models(base_url: &str) -> ServiceResult<Vec<ProviderModelView>> {
    let base_url = normalize_base_url(base_url)?;
    let response = client()?
        .get(format!("{base_url}/api/tags"))
        .send()
        .await
        .map_err(map_transport_error)?;
    if !response.status().is_success() {
        return Err(map_status(response.status(), false));
    }
    let payload: TagsResponse = response.json().await.map_err(|_| {
        provider_error(
            "invalid_structured_output",
            "Ollama returned an unreadable model-list response.",
        )
    })?;
    let mut models = payload
        .models
        .into_iter()
        .map(|model| ProviderModelView {
            display_name: model.name.clone(),
            model_id: model.name,
            parameter_size: model.details.parameter_size,
            quantization_level: model.details.quantization_level,
        })
        .collect::<Vec<_>>();
    models.sort_by(|left, right| left.model_id.cmp(&right.model_id));
    Ok(models)
}

pub async fn run_structured(
    base_url: &str,
    request: StructuredProviderRequest,
) -> ServiceResult<Value> {
    let base_url = normalize_base_url(base_url)?;
    let response = client()?
        .post(format!("{base_url}/api/chat"))
        .json(&json!({
            "model": request.model_id,
            "messages": [
                {"role": "system", "content": request.system_prompt},
                {"role": "user", "content": request.user_prompt}
            ],
            "stream": false,
            "format": request.response_schema,
            "options": {"temperature": 0}
        }))
        .send()
        .await
        .map_err(map_transport_error)?;
    if !response.status().is_success() {
        return Err(map_status(response.status(), true));
    }
    let payload: ChatResponse = response.json().await.map_err(|_| {
        provider_error(
            "invalid_structured_output",
            "Ollama returned an unreadable chat response.",
        )
    })?;
    serde_json::from_str(&payload.message.content).map_err(|_| {
        provider_error(
            "invalid_structured_output",
            "Ollama did not return JSON matching the requested structured-output contract.",
        )
    })
}

fn client() -> ServiceResult<Client> {
    Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|_| {
            provider_error(
                "unknown",
                "WorkLore could not initialize the Ollama client.",
            )
        })
}

fn map_transport_error(error: reqwest::Error) -> WorkLoreError {
    if error.is_timeout() {
        return provider_error("timeout", "The Ollama request timed out.");
    }
    if error.is_connect() {
        return provider_error(
            "provider_unavailable",
            "Ollama is not reachable at the configured local address.",
        );
    }
    provider_error("unknown", "The Ollama request failed before completion.")
}

fn map_status(status: StatusCode, model_request: bool) -> WorkLoreError {
    match status.as_u16() {
        401 | 403 => provider_error(
            "authentication_failed",
            "Ollama rejected the request as unauthorized.",
        ),
        404 if model_request => provider_error(
            "model_unavailable",
            "The selected Ollama model is not available.",
        ),
        408 => provider_error("timeout", "The Ollama request timed out."),
        413 => provider_error(
            "request_too_large",
            "The selected Voice Evidence is too large for the provider request.",
        ),
        429 => provider_error(
            "rate_limited",
            "Ollama temporarily refused the request because it is rate limited.",
        ),
        _ => provider_error(
            "provider_unavailable",
            "Ollama returned an unsuccessful HTTP response.",
        ),
    }
}

pub fn provider_error(code: &'static str, message: impl Into<String>) -> WorkLoreError {
    WorkLoreError::ProviderOperation {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_ollama_urls_are_normalized_and_remote_hosts_are_rejected() {
        assert_eq!(
            normalize_base_url("http://localhost:11434/").unwrap(),
            "http://localhost:11434"
        );
        assert_eq!(
            normalize_base_url("http://127.0.0.1:11434").unwrap(),
            "http://127.0.0.1:11434"
        );
        assert!(normalize_base_url("https://example.com:11434").is_err());
        assert!(normalize_base_url("http://user:secret@localhost:11434").is_err());
        assert!(normalize_base_url("http://localhost:11434/api").is_err());
    }

    #[test]
    fn http_statuses_map_to_normalized_provider_errors() {
        let error = map_status(StatusCode::NOT_FOUND, true);
        match error {
            WorkLoreError::ProviderOperation { code, .. } => assert_eq!(code, "model_unavailable"),
            other => panic!("unexpected error: {other:?}"),
        }
        let error = map_status(StatusCode::TOO_MANY_REQUESTS, true);
        match error {
            WorkLoreError::ProviderOperation { code, .. } => assert_eq!(code, "rate_limited"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
