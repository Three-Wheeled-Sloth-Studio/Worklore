from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8", newline="\n")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


# HTTP client for provider adapters. No credential-storage dependency is introduced here.
path = "src-tauri/Cargo.toml"
text = read(path)
text = replace_once(
    text,
    'regex = "1"\n',
    'regex = "1"\nreqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }\n',
    "reqwest dependency",
)
write(path, text)


# Provider-neutral public domain contract. Preserve the existing manual-workspace types.
path = "src-tauri/src/domain/providers.rs"
text = read(path)
text += r'''

pub const OLLAMA_PROVIDER_ID: &str = "ollama";
pub const ANALYZE_VOICE_EVIDENCE_OPERATION: &str = "analyze_voice_evidence";
pub const ANALYZE_VOICE_EVIDENCE_VERSION: u32 = 1;

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
pub struct VoiceAnalysisProposalSet {
    pub run_id: String,
    pub operation_id: String,
    pub operation_version: u32,
    pub provider_id: String,
    pub model_id: String,
    pub proposals: Vec<VoiceTraitProposal>,
}
'''
write(path, text)


# Machine-local provider configuration. Existing preference files deserialize safely via defaults.
path = "src-tauri/src/services/app_preferences_service.rs"
text = read(path)
text = replace_once(
    text,
    '''use crate::{\n    error::{ServiceResult, WorkLoreError},\n    io_utils::{read_json, write_json_atomic},\n};''',
    '''use crate::{\n    domain::providers::ProviderSettingsView,\n    error::{ServiceResult, WorkLoreError},\n    io_utils::{read_json, write_json_atomic},\n};''',
    "provider settings import",
)
text = replace_once(
    text,
    '''    #[serde(default)]\n    last_import_directory: Option<String>,\n    updated_at: String,''',
    '''    #[serde(default)]\n    last_import_directory: Option<String>,\n    #[serde(default)]\n    selected_provider_id: Option<String>,\n    #[serde(default = "default_ollama_base_url")]\n    ollama_base_url: String,\n    #[serde(default)]\n    ollama_model_id: Option<String>,\n    updated_at: String,''',
    "provider preference fields",
)
text = replace_once(
    text,
    '''            last_vault_path: None,\n            last_import_directory: None,\n            updated_at: Utc::now().to_rfc3339(),''',
    '''            last_vault_path: None,\n            last_import_directory: None,\n            selected_provider_id: None,\n            ollama_base_url: default_ollama_base_url(),\n            ollama_model_id: None,\n            updated_at: Utc::now().to_rfc3339(),''',
    "provider preference defaults",
)
anchor = '''pub fn get_default_vault_root() -> ServiceResult<String> {\n    Ok(default_vault_root()?.to_string_lossy().to_string())\n}\n'''
addition = anchor + '''\npub fn get_provider_settings() -> ServiceResult<ProviderSettingsView> {\n    let preferences = read_preferences_or_default(&preferences_path()?)?;\n    Ok(ProviderSettingsView {\n        selected_provider_id: preferences.selected_provider_id,\n        ollama_base_url: preferences.ollama_base_url,\n        ollama_model_id: preferences.ollama_model_id,\n    })\n}\n\npub fn save_provider_settings(settings: ProviderSettingsView) -> ServiceResult<ProviderSettingsView> {\n    update_preferences(|preferences| {\n        preferences.selected_provider_id = settings.selected_provider_id.clone();\n        preferences.ollama_base_url = settings.ollama_base_url.clone();\n        preferences.ollama_model_id = settings.ollama_model_id.clone();\n    })?;\n    Ok(settings)\n}\n'''
text = replace_once(text, anchor, addition, "provider preference accessors")
text = replace_once(
    text,
    '''const fn default_schema_version() -> u32 {\n    1\n}\n''',
    '''const fn default_schema_version() -> u32 {\n    2\n}\n\nfn default_ollama_base_url() -> String {\n    "http://127.0.0.1:11434".to_string()\n}\n''',
    "preference schema version",
)
text = replace_once(
    text,
    '''        assert_eq!(preferences.schema_version, 1);\n        assert!(preferences.last_vault_path.is_none());\n        assert!(preferences.last_import_directory.is_none());''',
    '''        assert_eq!(preferences.schema_version, 2);\n        assert!(preferences.last_vault_path.is_none());\n        assert!(preferences.last_import_directory.is_none());\n        assert!(preferences.selected_provider_id.is_none());\n        assert_eq!(preferences.ollama_base_url, "http://127.0.0.1:11434");\n        assert!(preferences.ollama_model_id.is_none());''',
    "preference default test",
)
write(path, text)


# Normalized provider errors use architecture-level codes rather than leaking adapter errors.
path = "src-tauri/src/error.rs"
text = read(path)
text = replace_once(
    text,
    '''    #[error("The manual workspace could not be created: {0}")]\n    ManualWorkspace(String),\n''',
    '''    #[error("The manual workspace could not be created: {0}")]\n    ManualWorkspace(String),\n\n    #[error("Provider operation failed: {message}")]\n    ProviderOperation { code: &'static str, message: String },\n''',
    "provider error variant",
)
old_impl_start = '''impl From<WorkLoreError> for CommandError {\n    fn from(value: WorkLoreError) -> Self {\n        let code = match &value {'''
new_impl_start = '''impl From<WorkLoreError> for CommandError {\n    fn from(value: WorkLoreError) -> Self {\n        if let WorkLoreError::ProviderOperation { code, message } = &value {\n            return Self {\n                code: (*code).to_string(),\n                message: message.clone(),\n                detail: None,\n            };\n        }\n        let code = match &value {'''
text = replace_once(text, old_impl_start, new_impl_start, "provider command error mapping")
text = replace_once(
    text,
    '''            WorkLoreError::ManualWorkspace(_) => "manual_workspace_failed",\n            WorkLoreError::Sqlite(_) => "sqlite_error",''',
    '''            WorkLoreError::ManualWorkspace(_) => "manual_workspace_failed",\n            WorkLoreError::ProviderOperation { .. } => unreachable!("provider errors return above"),\n            WorkLoreError::Sqlite(_) => "sqlite_error",''',
    "provider error exhaustive match",
)
write(path, text)


# Expose full governed snapshots only to the local analysis service. These are not frontend views.
path = "src-tauri/src/services/voice_evidence_service.rs"
text = read(path)
anchor = '''#[derive(Debug, Clone)]\nstruct SourceRecord {\n    source_id: String,\n    source_type: String,\n    display_name: String,\n    source_origin: String,\n    captured_text: String,\n    extraction_json: String,\n}\n'''
addition = anchor + '''\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct EligibleVoiceEvidenceMaterial {\n    pub voice_evidence_id: String,\n    pub source_id: String,\n    pub text_snapshot: String,\n}\n'''
text = replace_once(text, anchor, addition, "eligible evidence material type")
function_anchor = '''pub fn create_voice_evidence_from_source(\n    vault_path: &Path,\n    source_id: &str,\n) -> ServiceResult<CreateVoiceEvidenceResult> {'''
new_function = '''pub fn load_eligible_voice_evidence_material(\n    vault_path: &Path,\n    requested_ids: &[String],\n) -> ServiceResult<Vec<EligibleVoiceEvidenceMaterial>> {\n    canonical_store::initialize(vault_path)?;\n    let requested = requested_ids\n        .iter()\n        .map(|value| value.trim())\n        .filter(|value| !value.is_empty())\n        .collect::<std::collections::BTreeSet<_>>();\n    if requested.is_empty() {\n        return Err(WorkLoreError::SourceNotReady(\n            "Select at least one eligible Voice Evidence record for analysis.".to_string(),\n        ));\n    }\n    if requested.len() > 24 {\n        return Err(WorkLoreError::ProviderOperation {\n            code: "request_too_large",\n            message: "Voice analysis is limited to 24 evidence records per run.".to_string(),\n        });\n    }\n\n    let connection = open_connection(vault_path)?;\n    let mut material = Vec::with_capacity(requested.len());\n    for voice_evidence_id in requested {\n        let row = connection\n            .query_row(\n                "SELECT source_id,text_snapshot,status FROM voice_evidence WHERE voice_evidence_id=?1",\n                [voice_evidence_id],\n                |row| {\n                    Ok((\n                        row.get::<_, String>(0)?,\n                        row.get::<_, String>(1)?,\n                        row.get::<_, String>(2)?,\n                    ))\n                },\n            )\n            .optional()?\n            .ok_or_else(|| {\n                WorkLoreError::InvalidVault(format!(\n                    "Voice Evidence {voice_evidence_id} was not found."\n                ))\n            })?;\n        if row.2 != "eligible" {\n            return Err(WorkLoreError::SourceNotReady(format!(\n                "Voice Evidence {voice_evidence_id} is {} and cannot be sent as canonical voice evidence.",\n                row.2\n            )));\n        }\n        material.push(EligibleVoiceEvidenceMaterial {\n            voice_evidence_id: voice_evidence_id.to_string(),\n            source_id: row.0,\n            text_snapshot: row.1,\n        });\n    }\n    Ok(material)\n}\n\n''' + function_anchor
text = replace_once(text, function_anchor, new_function, "eligible evidence material loader")
write(path, text)


write(
    "src-tauri/src/services/ollama_provider.rs",
    r'''use std::time::Duration;

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
    if !matches!(host, "localhost" | "127.0.0.1" | "::1") {
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
        .map_err(|_| provider_error("unknown", "WorkLore could not initialize the Ollama client."))
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
''',
)


write(
    "src-tauri/src/services/provider_registry.rs",
    r'''use crate::{
    domain::providers::{
        ProviderConnectionView, ProviderModelView, ProviderSettingsView, UpdateProviderSettingsRequest,
        OLLAMA_PROVIDER_ID,
    },
    error::{ServiceResult, WorkLoreError},
    services::{app_preferences_service, ollama_provider},
};

pub fn get_settings() -> ServiceResult<ProviderSettingsView> {
    let settings = app_preferences_service::get_provider_settings()?;
    Ok(sanitize_loaded_settings(settings))
}

pub fn update_settings(request: UpdateProviderSettingsRequest) -> ServiceResult<ProviderSettingsView> {
    let selected_provider_id = normalize_provider_id(request.selected_provider_id.as_deref())?;
    let ollama_base_url = ollama_provider::normalize_base_url(&request.ollama_base_url)?;
    let ollama_model_id = normalize_optional_text(request.ollama_model_id.as_deref());
    app_preferences_service::save_provider_settings(ProviderSettingsView {
        selected_provider_id,
        ollama_base_url,
        ollama_model_id,
    })
}

pub async fn list_models(provider_id: &str) -> ServiceResult<Vec<ProviderModelView>> {
    let settings = require_selected_provider(provider_id)?;
    match provider_id {
        OLLAMA_PROVIDER_ID => ollama_provider::list_models(&settings.ollama_base_url).await,
        _ => Err(provider_not_configured("The selected provider is not registered.")),
    }
}

pub async fn test_connection(provider_id: &str) -> ServiceResult<ProviderConnectionView> {
    let models = list_models(provider_id).await?;
    Ok(ProviderConnectionView {
        provider_id: provider_id.to_string(),
        available: true,
        model_count: models.len(),
        message: if models.is_empty() {
            "Ollama is reachable, but it reports no installed local models.".to_string()
        } else {
            format!("Ollama is reachable with {} installed model(s).", models.len())
        },
    })
}

pub fn require_selected_provider(provider_id: &str) -> ServiceResult<ProviderSettingsView> {
    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        return Err(provider_not_configured("Select a provider before running this operation."));
    }
    let settings = get_settings()?;
    if settings.selected_provider_id.as_deref() != Some(provider_id) {
        return Err(provider_not_configured(
            "The requested provider is not the explicitly selected WorkLore provider.",
        ));
    }
    if provider_id == OLLAMA_PROVIDER_ID && settings.ollama_model_id.is_none() {
        return Err(provider_not_configured(
            "Select and save an Ollama model before running voice analysis.",
        ));
    }
    Ok(settings)
}

pub fn validate_requested_model(settings: &ProviderSettingsView, model_id: &str) -> ServiceResult<String> {
    let requested = model_id.trim();
    if requested.is_empty() {
        return Err(provider_not_configured("Select a provider model before running this operation."));
    }
    if settings.ollama_model_id.as_deref() != Some(requested) {
        return Err(provider_not_configured(
            "The requested model is not the explicitly configured model for this provider.",
        ));
    }
    Ok(requested.to_string())
}

fn sanitize_loaded_settings(mut settings: ProviderSettingsView) -> ProviderSettingsView {
    if settings.selected_provider_id.as_deref() != Some(OLLAMA_PROVIDER_ID) {
        settings.selected_provider_id = None;
    }
    settings.ollama_model_id = normalize_optional_text(settings.ollama_model_id.as_deref());
    settings
}

fn normalize_provider_id(value: Option<&str>) -> ServiceResult<Option<String>> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(None),
        Some(OLLAMA_PROVIDER_ID) => Ok(Some(OLLAMA_PROVIDER_ID.to_string())),
        Some(_) => Err(provider_not_configured(
            "Only the local Ollama provider is executable in this WorkLore slice.",
        )),
    }
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn provider_not_configured(message: impl Into<String>) -> WorkLoreError {
    WorkLoreError::ProviderOperation {
        code: "not_configured",
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_selection_is_explicit_and_does_not_fallback() {
        assert_eq!(normalize_provider_id(None).unwrap(), None);
        assert_eq!(
            normalize_provider_id(Some("ollama")).unwrap(),
            Some("ollama".to_string())
        );
        assert!(normalize_provider_id(Some("gemini")).is_err());
    }

    #[test]
    fn requested_model_must_match_explicit_configuration() {
        let settings = ProviderSettingsView {
            selected_provider_id: Some("ollama".into()),
            ollama_base_url: "http://127.0.0.1:11434".into(),
            ollama_model_id: Some("qwen3".into()),
        };
        assert_eq!(validate_requested_model(&settings, "qwen3").unwrap(), "qwen3");
        assert!(validate_requested_model(&settings, "gemma3").is_err());
    }
}
''',
)


write(
    "src-tauri/src/services/voice_analysis_service.rs",
    r'''use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
    time::Duration,
};

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    domain::providers::{
        AnalyzeVoiceEvidenceRequest, VoiceAnalysisProposalSet, VoiceTraitProposal,
        ANALYZE_VOICE_EVIDENCE_OPERATION, ANALYZE_VOICE_EVIDENCE_VERSION, OLLAMA_PROVIDER_ID,
    },
    error::{ServiceResult, WorkLoreError},
    services::{
        canonical_store, ollama_provider::{self, StructuredProviderRequest}, provider_registry,
        voice_evidence_service::{self, EligibleVoiceEvidenceMaterial},
    },
};

const MAX_ANALYSIS_CHARACTERS: usize = 80_000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawVoiceAnalysis {
    #[serde(default)]
    proposals: Vec<RawVoiceTraitProposal>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawVoiceTraitProposal {
    name: String,
    value: String,
    evidence_ids: Vec<String>,
    rationale: String,
}

pub async fn analyze_voice_evidence(
    vault_path: &Path,
    request: AnalyzeVoiceEvidenceRequest,
) -> ServiceResult<VoiceAnalysisProposalSet> {
    canonical_store::initialize(vault_path)?;
    let run_id = format!("provider_run_{}", Uuid::now_v7());
    let provider_id = request.provider_id.trim().to_string();
    let selected_ids = request
        .voice_evidence_ids
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();

    let result = analyze_inner(vault_path, &request, &selected_ids, &run_id).await;
    let (outcome, proposal_count, error_code) = match &result {
        Ok(value) => ("succeeded", value.proposals.len(), None),
        Err(WorkLoreError::ProviderOperation { code, .. }) => ("failed", 0, Some(*code)),
        Err(_) => ("failed", 0, Some("input_validation_failed")),
    };
    let _ = audit_provider_run(
        vault_path,
        &run_id,
        &provider_id,
        request.model_id.trim(),
        selected_ids.len(),
        proposal_count,
        outcome,
        error_code,
    );
    result
}

async fn analyze_inner(
    vault_path: &Path,
    request: &AnalyzeVoiceEvidenceRequest,
    selected_ids: &BTreeSet<String>,
    run_id: &str,
) -> ServiceResult<VoiceAnalysisProposalSet> {
    if request.provider_id.trim() != OLLAMA_PROVIDER_ID {
        return Err(provider_error(
            "not_configured",
            "Only the explicitly selected local Ollama provider is executable in this slice.",
        ));
    }
    let settings = provider_registry::require_selected_provider(&request.provider_id)?;
    let model_id = provider_registry::validate_requested_model(&settings, &request.model_id)?;
    let material = voice_evidence_service::load_eligible_voice_evidence_material(
        vault_path,
        &request.voice_evidence_ids,
    )?;
    let total_chars = material
        .iter()
        .map(|item| item.text_snapshot.chars().count())
        .sum::<usize>();
    if total_chars > MAX_ANALYSIS_CHARACTERS {
        return Err(provider_error(
            "request_too_large",
            "The selected Voice Evidence exceeds the 80,000-character analysis limit. Use a smaller evidence set.",
        ));
    }

    let structured_input = material
        .iter()
        .map(|item| {
            json!({
                "voiceEvidenceId": item.voice_evidence_id,
                "sourceId": item.source_id,
                "text": item.text_snapshot
            })
        })
        .collect::<Vec<_>>();
    let guidance = request
        .user_guidance
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("No additional user guidance was provided.");
    let user_prompt = format!(
        "Analyze the writing samples below for stable, observable authorial voice traits. Content topics, employers, products, factual claims, and subject-matter expertise are not voice traits. Only propose traits directly supported by the supplied samples. Every proposal must cite one or more voiceEvidenceId values from the supplied set. If support is weak or inconsistent, return fewer proposals, including zero. Do not produce confidence percentages or human-vs-AI probability scores.\n\nExplicit user guidance (context only, not evidence):\n{guidance}\n\nVoice Evidence JSON:\n{}\n\nReturn only JSON matching the supplied schema.",
        serde_json::to_string_pretty(&structured_input)?
    );
    let raw_value = ollama_provider::run_structured(
        &settings.ollama_base_url,
        StructuredProviderRequest {
            model_id: model_id.clone(),
            system_prompt: "You are WorkLore's bounded voice-analysis operation. Observe style only. Never invent identity traits, never infer authorship probability, never treat provider output as authoritative, and preserve supplied evidence identifiers exactly.".to_string(),
            user_prompt,
            response_schema: response_schema(),
        },
    )
    .await?;
    let proposals = validate_analysis_output(selected_ids, raw_value)?;
    Ok(VoiceAnalysisProposalSet {
        run_id: run_id.to_string(),
        operation_id: ANALYZE_VOICE_EVIDENCE_OPERATION.to_string(),
        operation_version: ANALYZE_VOICE_EVIDENCE_VERSION,
        provider_id: OLLAMA_PROVIDER_ID.to_string(),
        model_id,
        proposals,
    })
}

fn response_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "required": ["proposals"],
        "properties": {
            "proposals": {
                "type": "array",
                "maxItems": 12,
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["name", "value", "evidenceIds", "rationale"],
                    "properties": {
                        "name": {"type": "string", "minLength": 1, "maxLength": 80},
                        "value": {"type": "string", "minLength": 1, "maxLength": 500},
                        "evidenceIds": {
                            "type": "array",
                            "minItems": 1,
                            "uniqueItems": true,
                            "items": {"type": "string", "minLength": 1}
                        },
                        "rationale": {"type": "string", "minLength": 1, "maxLength": 800}
                    }
                }
            }
        }
    })
}

fn validate_analysis_output(
    selected_ids: &BTreeSet<String>,
    value: Value,
) -> ServiceResult<Vec<VoiceTraitProposal>> {
    let raw: RawVoiceAnalysis = serde_json::from_value(value).map_err(|_| {
        provider_error(
            "invalid_structured_output",
            "The provider response does not match the WorkLore voice-analysis contract.",
        )
    })?;
    if raw.proposals.len() > 12 {
        return Err(provider_error(
            "invalid_structured_output",
            "The provider returned more voice proposals than the operation contract allows.",
        ));
    }
    let mut seen_names = HashSet::new();
    raw.proposals
        .into_iter()
        .map(|proposal| {
            let name = required_trimmed(proposal.name, "proposal name")?;
            let value = required_trimmed(proposal.value, "proposal value")?;
            let rationale = required_trimmed(proposal.rationale, "proposal rationale")?;
            let normalized_name = name.to_ascii_lowercase();
            if !seen_names.insert(normalized_name) {
                return Err(provider_error(
                    "invalid_structured_output",
                    "The provider returned duplicate voice-trait proposals.",
                ));
            }
            let evidence_ids = proposal
                .evidence_ids
                .into_iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect::<BTreeSet<_>>();
            if evidence_ids.is_empty() {
                return Err(provider_error(
                    "invalid_structured_output",
                    "Every voice proposal must cite at least one eligible Voice Evidence record.",
                ));
            }
            if let Some(unknown) = evidence_ids.iter().find(|id| !selected_ids.contains(*id)) {
                return Err(provider_error(
                    "invalid_structured_output",
                    format!(
                        "The provider cited unknown or unselected Voice Evidence {unknown}."
                    ),
                ));
            }
            Ok(VoiceTraitProposal {
                proposal_id: format!("voice_proposal_{}", Uuid::now_v7()),
                name,
                value,
                evidence_ids: evidence_ids.into_iter().collect(),
                rationale,
            })
        })
        .collect()
}

fn required_trimmed(value: String, label: &str) -> ServiceResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(provider_error(
            "invalid_structured_output",
            format!("The provider returned an empty {label}."),
        ));
    }
    Ok(value.to_string())
}

fn provider_error(code: &'static str, message: impl Into<String>) -> WorkLoreError {
    WorkLoreError::ProviderOperation {
        code,
        message: message.into(),
    }
}

fn audit_provider_run(
    vault_path: &Path,
    run_id: &str,
    provider_id: &str,
    model_id: &str,
    evidence_count: usize,
    proposal_count: usize,
    outcome: &str,
    error_code: Option<&str>,
) -> ServiceResult<()> {
    let connection = Connection::open(vault_path.join(canonical_store::DATABASE_RELATIVE_PATH))?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)\n         VALUES (?1,'provider_operation','provider_run',?2,'system',?3,?4)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            run_id,
            json!({
                "providerId": provider_id,
                "modelId": model_id,
                "operationId": ANALYZE_VOICE_EVIDENCE_OPERATION,
                "operationVersion": ANALYZE_VOICE_EVIDENCE_VERSION,
                "outcome": outcome,
                "evidenceCount": evidence_count,
                "proposalCount": proposal_count,
                "errorCode": error_code
            })
            .to_string(),
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{vault_service, voice_profile_service};

    fn vault() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-voice-analysis-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Voice Analysis Test").expect("create vault");
        path
    }

    fn selected(ids: &[&str]) -> BTreeSet<String> {
        ids.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn structured_output_rejects_unknown_evidence_ids() {
        let result = validate_analysis_output(
            &selected(&["voice_evidence_allowed"]),
            json!({
                "proposals": [{
                    "name": "Direct",
                    "value": "Leads with the useful point.",
                    "evidenceIds": ["voice_evidence_unknown"],
                    "rationale": "Observed in the sample."
                }]
            }),
        );
        assert!(result.is_err());
    }

    #[test]
    fn structured_output_rejects_missing_evidence_and_duplicates() {
        assert!(validate_analysis_output(
            &selected(&["voice_evidence_a"]),
            json!({"proposals":[{
                "name":"Direct","value":"Concise opening.","evidenceIds":[],"rationale":"Observed."
            }]})
        )
        .is_err());
        assert!(validate_analysis_output(
            &selected(&["voice_evidence_a"]),
            json!({"proposals":[
                {"name":"Direct","value":"Concise opening.","evidenceIds":["voice_evidence_a"],"rationale":"Observed."},
                {"name":"direct","value":"Again.","evidenceIds":["voice_evidence_a"],"rationale":"Observed."}
            ]})
        )
        .is_err());
    }

    #[test]
    fn valid_proposals_are_transient_and_do_not_mutate_core_voice() {
        let path = vault();
        let before = voice_profile_service::list_core_voices(&path).unwrap();
        assert!(before.is_empty());
        let proposals = validate_analysis_output(
            &selected(&["voice_evidence_a"]),
            json!({"proposals":[{
                "name":"Direct",
                "value":"Leads with the useful point.",
                "evidenceIds":["voice_evidence_a"],
                "rationale":"The sample reaches the claim before background detail."
            }]})
        )
        .unwrap();
        assert_eq!(proposals.len(), 1);
        assert!(voice_profile_service::list_core_voices(&path).unwrap().is_empty());
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn provider_audit_metadata_excludes_prompt_and_response_bodies() {
        let path = vault();
        audit_provider_run(
            &path,
            "provider_run_test",
            "ollama",
            "qwen3",
            2,
            1,
            "succeeded",
            None,
        )
        .unwrap();
        let connection = Connection::open(path.join(canonical_store::DATABASE_RELATIVE_PATH)).unwrap();
        let details: String = connection
            .query_row(
                "SELECT details_json FROM audit_events WHERE record_id='provider_run_test'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(details.contains("qwen3"));
        assert!(!details.contains("prompt"));
        assert!(!details.contains("response"));
        assert!(!details.contains("guidance"));
        std::fs::remove_dir_all(path).unwrap();
    }
}
''',
)


# Register services.
path = "src-tauri/src/services/mod.rs"
text = read(path)
text = replace_once(
    text,
    "pub mod manual_workspace_service;\n",
    "pub mod manual_workspace_service;\npub mod ollama_provider;\npub mod provider_registry;\n",
    "provider service modules",
)
text = replace_once(
    text,
    "pub mod voice_evidence_service;\n",
    "pub mod voice_analysis_service;\npub mod voice_evidence_service;\n",
    "voice analysis service module",
)
write(path, text)


# Expand provider commands while preserving the manual workspace bridge.
write(
    "src-tauri/src/commands/providers.rs",
    r'''use std::path::PathBuf;

use crate::{
    domain::providers::{
        AnalyzeVoiceEvidenceRequest, CreateManualWorkspaceRequest, ManualWorkspaceResult,
        ProviderConnectionView, ProviderModelView, ProviderSettingsView, UpdateProviderSettingsRequest,
        VoiceAnalysisProposalSet,
    },
    error::{CommandError, CommandResult},
    services::{
        app_preferences_service, manual_workspace_service, provider_registry, voice_analysis_service,
    },
};

#[tauri::command]
pub fn create_manual_workspace(
    vault_path: String,
    request: CreateManualWorkspaceRequest,
) -> CommandResult<ManualWorkspaceResult> {
    manual_workspace_service::create_manual_workspace(&PathBuf::from(vault_path), request)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_provider_settings() -> CommandResult<ProviderSettingsView> {
    provider_registry::get_settings().map_err(CommandError::from)
}

#[tauri::command]
pub fn update_provider_settings(
    request: UpdateProviderSettingsRequest,
) -> CommandResult<ProviderSettingsView> {
    provider_registry::update_settings(request).map_err(CommandError::from)
}

#[tauri::command]
pub async fn list_provider_models(provider_id: String) -> CommandResult<Vec<ProviderModelView>> {
    provider_registry::list_models(&provider_id)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn test_provider_connection(
    provider_id: String,
) -> CommandResult<ProviderConnectionView> {
    provider_registry::test_connection(&provider_id)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn analyze_voice_evidence(
    vault_path: String,
    request: AnalyzeVoiceEvidenceRequest,
) -> CommandResult<VoiceAnalysisProposalSet> {
    voice_analysis_service::analyze_voice_evidence(&PathBuf::from(vault_path), request)
        .await
        .map_err(CommandError::from)
}

#[allow(dead_code)]
fn _preference_boundary_is_machine_local() {
    let _ = app_preferences_service::get_provider_settings;
}
''',
)


# Register commands in the Tauri handler.
path = "src-tauri/src/lib.rs"
text = read(path)
text = replace_once(
    text,
    "    providers::create_manual_workspace,\n",
    "    providers::{\n        analyze_voice_evidence, create_manual_workspace, get_provider_settings,\n        list_provider_models, test_provider_connection, update_provider_settings,\n    },\n",
    "provider command imports",
)
text = replace_once(
    text,
    '''            get_performance_snapshot,\n            create_manual_workspace,\n            list_roles,''',
    '''            get_performance_snapshot,\n            create_manual_workspace,\n            get_provider_settings,\n            update_provider_settings,\n            list_provider_models,\n            test_provider_connection,\n            analyze_voice_evidence,\n            list_roles,''',
    "provider handler registration",
)
write(path, text)


# Frontend provider and analysis contracts.
path = "src/domain/types.ts"
text = read(path)
anchor = '''export interface UpdateWritingRuleRequest extends CreateWritingRuleRequest {\n  ruleId: string;\n  status: WritingRuleStatus;\n}\n'''
addition = anchor + '''\nexport interface ProviderSettings {\n  selectedProviderId: string | null;\n  ollamaBaseUrl: string;\n  ollamaModelId: string | null;\n}\n\nexport interface UpdateProviderSettingsRequest extends ProviderSettings {}\n\nexport interface ProviderModel {\n  modelId: string;\n  displayName: string;\n  parameterSize: string | null;\n  quantizationLevel: string | null;\n}\n\nexport interface ProviderConnection {\n  providerId: string;\n  available: boolean;\n  modelCount: number;\n  message: string;\n}\n\nexport interface AnalyzeVoiceEvidenceRequest {\n  providerId: string;\n  modelId: string;\n  voiceEvidenceIds: string[];\n  userGuidance?: string | null;\n}\n\nexport interface VoiceTraitProposal {\n  proposalId: string;\n  name: string;\n  value: string;\n  evidenceIds: string[];\n  rationale: string;\n}\n\nexport interface VoiceAnalysisProposalSet {\n  runId: string;\n  operationId: string;\n  operationVersion: number;\n  providerId: string;\n  modelId: string;\n  proposals: VoiceTraitProposal[];\n}\n'''
text = replace_once(text, anchor, addition, "frontend provider types")
write(path, text)


# Frontend API wrappers.
path = "src/lib/workloreApi.ts"
text = read(path)
text = replace_once(
    text,
    '''  UpdateWritingRuleRequest,\n} from "../domain/types";''',
    '''  UpdateWritingRuleRequest,\n  ProviderSettings,\n  UpdateProviderSettingsRequest,\n  ProviderModel,\n  ProviderConnection,\n  AnalyzeVoiceEvidenceRequest,\n  VoiceAnalysisProposalSet,\n} from "../domain/types";''',
    "frontend provider API type imports",
)
anchor = '''export async function rememberLastImportFile(filePath: string): Promise<string> {\n  return invoke<string>("remember_last_import_file", { filePath });\n}\n'''
addition = anchor + '''\nexport async function getProviderSettings(): Promise<ProviderSettings> {\n  return invoke<ProviderSettings>("get_provider_settings");\n}\n\nexport async function updateProviderSettings(\n  request: UpdateProviderSettingsRequest,\n): Promise<ProviderSettings> {\n  return invoke<ProviderSettings>("update_provider_settings", { request });\n}\n\nexport async function listProviderModels(providerId: string): Promise<ProviderModel[]> {\n  return invoke<ProviderModel[]>("list_provider_models", { providerId });\n}\n\nexport async function testProviderConnection(\n  providerId: string,\n): Promise<ProviderConnection> {\n  return invoke<ProviderConnection>("test_provider_connection", { providerId });\n}\n\nexport async function analyzeVoiceEvidence(\n  vaultPath: string,\n  request: AnalyzeVoiceEvidenceRequest,\n): Promise<VoiceAnalysisProposalSet> {\n  return invoke<VoiceAnalysisProposalSet>("analyze_voice_evidence", { vaultPath, request });\n}\n'''
text = replace_once(text, anchor, addition, "frontend provider API wrappers")
write(path, text)


write(
    "src/components/ProviderSettingsPanel.tsx",
    r'''import { useEffect, useState } from "react";
import type { ProviderModel, ProviderSettings } from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  getProviderSettings,
  listProviderModels,
  testProviderConnection,
  updateProviderSettings,
} from "../lib/workloreApi";
import "../provider-settings.css";

const DEFAULT_OLLAMA_URL = "http://127.0.0.1:11434";

export function ProviderSettingsPanel() {
  const [settings, setSettings] = useState<ProviderSettings>({
    selectedProviderId: null,
    ollamaBaseUrl: DEFAULT_OLLAMA_URL,
    ollamaModelId: null,
  });
  const [models, setModels] = useState<ProviderModel[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      try {
        setSettings(await getProviderSettings());
      } catch (caught) {
        setError(errorMessage(caught));
      }
    })();
  }, []);

  async function save(): Promise<ProviderSettings | null> {
    setBusy("Saving provider settings");
    setNotice(null);
    setError(null);
    try {
      const saved = await updateProviderSettings(settings);
      setSettings(saved);
      setNotice(
        saved.selectedProviderId === "ollama"
          ? "Local Ollama configuration saved. WorkLore will not fall back to a cloud provider."
          : "AI execution is disabled. Provider-free WorkLore workflows remain available.",
      );
      return saved;
    } catch (caught) {
      setError(errorMessage(caught));
      return null;
    } finally {
      setBusy(null);
    }
  }

  async function refreshModels() {
    const saved = await save();
    if (!saved || saved.selectedProviderId !== "ollama") return;
    setBusy("Reading local Ollama models");
    setNotice(null);
    setError(null);
    try {
      const result = await listProviderModels("ollama");
      setModels(result);
      setNotice(
        result.length > 0
          ? `Ollama reported ${result.length} installed model${result.length === 1 ? "" : "s"}.`
          : "Ollama is reachable but no installed local models were reported.",
      );
    } catch (caught) {
      setModels([]);
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function testConnection() {
    const saved = await save();
    if (!saved || saved.selectedProviderId !== "ollama") return;
    setBusy("Testing local Ollama");
    setNotice(null);
    setError(null);
    try {
      const result = await testProviderConnection("ollama");
      setNotice(result.message);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  return (
    <section className="workspace-panel provider-settings" aria-labelledby="provider-settings-heading">
      <p className="eyebrow">AI Providers</p>
      <h2 id="provider-settings-heading">Provider execution</h2>
      <p>
        Provider use is explicit. This slice supports local Ollama only. No provider is required for
        professional-memory or manual Voice workflows, and WorkLore never silently falls back to cloud execution.
      </p>

      <label className="field-label" htmlFor="provider-selection">Selected provider</label>
      <select
        id="provider-selection"
        value={settings.selectedProviderId ?? "none"}
        disabled={busy !== null}
        onChange={(event) =>
          setSettings((current) => ({
            ...current,
            selectedProviderId: event.target.value === "ollama" ? "ollama" : null,
          }))
        }
      >
        <option value="none">None - provider execution disabled</option>
        <option value="ollama">Ollama - local only</option>
      </select>

      {settings.selectedProviderId === "ollama" ? (
        <div className="provider-config-grid">
          <label className="field-label" htmlFor="ollama-base-url">Ollama base URL</label>
          <input
            id="ollama-base-url"
            value={settings.ollamaBaseUrl}
            disabled={busy !== null}
            onChange={(event) =>
              setSettings((current) => ({ ...current, ollamaBaseUrl: event.target.value }))
            }
          />
          <p className="provider-note">Current adapter is intentionally loopback-only: localhost, 127.0.0.1, or ::1.</p>

          <label className="field-label" htmlFor="ollama-model">Selected model</label>
          <select
            id="ollama-model"
            value={settings.ollamaModelId ?? ""}
            disabled={busy !== null}
            onChange={(event) =>
              setSettings((current) => ({
                ...current,
                ollamaModelId: event.target.value || null,
              }))
            }
          >
            <option value="">Select a model</option>
            {settings.ollamaModelId && !models.some((model) => model.modelId === settings.ollamaModelId) ? (
              <option value={settings.ollamaModelId}>{settings.ollamaModelId} (saved)</option>
            ) : null}
            {models.map((model) => (
              <option key={model.modelId} value={model.modelId}>
                {model.displayName}
                {model.parameterSize ? ` - ${model.parameterSize}` : ""}
              </option>
            ))}
          </select>

          <div className="support-actions">
            <button className="secondary-button compact" disabled={busy !== null} onClick={() => void refreshModels()}>
              Refresh installed models
            </button>
            <button className="quiet-button compact" disabled={busy !== null} onClick={() => void testConnection()}>
              Test Ollama
            </button>
          </div>
        </div>
      ) : null}

      <div className="support-actions">
        <button className="primary-button compact" disabled={busy !== null} onClick={() => void save()}>
          Save provider settings
        </button>
      </div>
      {busy ? <p className="provider-note">{busy}</p> : null}
      {notice ? <div className="feedback notice" role="status">{notice}</div> : null}
      {error ? <div className="feedback error" role="alert">{error}</div> : null}
    </section>
  );
}
''',
)

write(
    "src/provider-settings.css",
    r'''.provider-settings {
  display: grid;
  gap: 0.75rem;
}

.provider-config-grid {
  display: grid;
  gap: 0.65rem;
  margin-top: 0.35rem;
}

.provider-note {
  color: var(--text-muted, #666);
  font-size: 0.9rem;
  margin: 0;
}
''',
)


# Put real provider controls in Settings without changing the rest of the shell.
path = "src/App.tsx"
text = read(path)
text = replace_once(
    text,
    'import { PerformancePanel } from "./components/PerformancePanel";\n',
    'import { PerformancePanel } from "./components/PerformancePanel";\nimport { ProviderSettingsPanel } from "./components/ProviderSettingsPanel";\n',
    "provider settings component import",
)
old_card = '''              <div className="next-step-card">\n                <h3>AI Providers</h3>\n                <p>Ollama and provider-neutral BYOK configuration begin in Phase 2. No provider is required for current professional-memory workflows, and WorkLore will not silently fall back to cloud execution.</p>\n              </div>'''
text = replace_once(text, old_card, "              <ProviderSettingsPanel />", "settings provider card")
write(path, text)


# Add provider-assisted proposal review to Voice without making proposals authoritative.
path = "src/components/VoiceWorkspace.tsx"
text = read(path)
text = replace_once(
    text,
    '''  CoreVoiceTrait,\n  ToneModeRecord,''',
    '''  CoreVoiceTrait,\n  ProviderSettings,\n  VoiceAnalysisProposalSet,\n  VoiceTraitProposal,\n  ToneModeRecord,''',
    "voice provider type imports",
)
text = replace_once(
    text,
    '''  activateCoreVoice,\n  createCoreVoice,''',
    '''  activateCoreVoice,\n  analyzeVoiceEvidence,\n  createCoreVoice,''',
    "voice analyze API import",
)
text = replace_once(
    text,
    '''  deleteCoreVoiceTrait,\n  listCoreVoices,''',
    '''  deleteCoreVoiceTrait,\n  getProviderSettings,\n  listCoreVoices,''',
    "voice provider settings API import",
)
state_anchor = '''  const [ruleInstruction, setRuleInstruction] = useState("");\n  const [busy, setBusy] = useState<string | null>(null);'''
state_new = '''  const [ruleInstruction, setRuleInstruction] = useState("");\n  const [providerSettings, setProviderSettings] = useState<ProviderSettings | null>(null);\n  const [analysisEvidenceIds, setAnalysisEvidenceIds] = useState<string[]>([]);\n  const [analysisGuidance, setAnalysisGuidance] = useState("");\n  const [analysisResult, setAnalysisResult] = useState<VoiceAnalysisProposalSet | null>(null);\n  const [proposalVoiceId, setProposalVoiceId] = useState("");\n  const [busy, setBusy] = useState<string | null>(null);'''
text = replace_once(text, state_anchor, state_new, "voice analysis state")
memo_anchor = '''  const eligibleEvidence = useMemo(\n    () => evidence.filter((item) => item.status === "eligible"),\n    [evidence],\n  );\n'''
memo_new = memo_anchor + '''\n  const proposedVoices = useMemo(\n    () => voices.filter((voice) => voice.status === "proposed"),\n    [voices],\n  );\n'''
text = replace_once(text, memo_anchor, memo_new, "proposed voices memo")
effect_anchor = '''  useEffect(() => {\n    setNotice(null);\n    setError(null);\n    void refresh();\n  }, [vaultPath]);\n'''
effect_new = effect_anchor + '''\n  useEffect(() => {\n    void getProviderSettings()\n      .then(setProviderSettings)\n      .catch(() => setProviderSettings(null));\n  }, [vaultPath]);\n\n  useEffect(() => {\n    const eligibleIds = new Set(eligibleEvidence.map((item) => item.voiceEvidenceId));\n    setAnalysisEvidenceIds((current) => current.filter((id) => eligibleIds.has(id)));\n  }, [eligibleEvidence]);\n\n  useEffect(() => {\n    if (proposalVoiceId && proposedVoices.some((voice) => voice.voiceId === proposalVoiceId)) return;\n    setProposalVoiceId(proposedVoices[0]?.voiceId ?? "");\n  }, [proposalVoiceId, proposedVoices]);\n'''
text = replace_once(text, effect_anchor, effect_new, "voice provider effects")
function_anchor = '''  async function addTone() {'''
functions = r'''  async function runVoiceAnalysis() {
    if (
      !providerSettings?.selectedProviderId ||
      !providerSettings.ollamaModelId ||
      analysisEvidenceIds.length === 0
    ) return;
    setBusy("Analyzing eligible Voice Evidence locally");
    setNotice(null);
    setError(null);
    try {
      const result = await analyzeVoiceEvidence(vaultPath, {
        providerId: providerSettings.selectedProviderId,
        modelId: providerSettings.ollamaModelId,
        voiceEvidenceIds: analysisEvidenceIds,
        userGuidance: analysisGuidance || null,
      });
      setAnalysisResult(result);
      setNotice(
        result.proposals.length > 0
          ? `${result.proposals.length} review-only voice proposal${result.proposals.length === 1 ? "" : "s"} returned. Nothing has changed in Core Voice.`
          : "The selected samples did not support a stable voice proposal. Core Voice remains unchanged.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function acceptVoiceProposal(proposal: VoiceTraitProposal) {
    if (!proposalVoiceId) return;
    setBusy("Accepting reviewed voice proposal");
    setNotice(null);
    setError(null);
    try {
      await saveCoreVoiceTrait(vaultPath, {
        voiceId: proposalVoiceId,
        name: proposal.name,
        value: proposal.value,
        userGuidance: null,
        voiceEvidenceIds: proposal.evidenceIds,
      });
      setAnalysisResult((current) =>
        current
          ? { ...current, proposals: current.proposals.filter((item) => item.proposalId !== proposal.proposalId) }
          : null,
      );
      setNotice("Proposal accepted into the selected proposed Core Voice version with its eligible evidence links preserved.");
      await refresh();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  function discardVoiceProposal(proposalId: string) {
    setAnalysisResult((current) =>
      current
        ? { ...current, proposals: current.proposals.filter((item) => item.proposalId !== proposalId) }
        : null,
    );
  }

''' + function_anchor
text = replace_once(text, function_anchor, functions, "voice analysis functions")
section_anchor = '''      <section className="workspace-panel" aria-labelledby="core-voice-heading">'''
analysis_section = r'''      <section className="workspace-panel" aria-labelledby="voice-analysis-heading">
        <div className="panel-heading-row">
          <div><p className="eyebrow">Provider-assisted, review-only</p><h2 id="voice-analysis-heading">Analyze Voice Evidence</h2></div>
          <span className="status-pill">v1</span>
        </div>
        <p>
          Ask the explicitly configured local provider for attributable observations. Provider output is temporary review material and cannot change Core Voice until you accept a proposal.
        </p>
        {!providerSettings?.selectedProviderId || !providerSettings.ollamaModelId ? (
          <div className="empty-state compact-empty">
            <h3>No executable provider configured</h3>
            <p>Configure local Ollama and an explicit model in Settings. Manual Voice management remains fully available without a provider.</p>
          </div>
        ) : eligibleEvidence.length === 0 ? (
          <div className="empty-state compact-empty">
            <h3>No eligible evidence to analyze</h3>
            <p>Only explicitly approved Voice Evidence can enter this operation.</p>
          </div>
        ) : (
          <div className="voice-analysis-panel">
            <p className="voice-meta">Provider: {providerSettings.selectedProviderId} | model: {providerSettings.ollamaModelId}</p>
            <fieldset className="voice-evidence-picker">
              <legend>Select eligible Voice Evidence</legend>
              {eligibleEvidence.map((item) => (
                <label key={item.voiceEvidenceId}>
                  <input
                    type="checkbox"
                    checked={analysisEvidenceIds.includes(item.voiceEvidenceId)}
                    disabled={busy !== null}
                    onChange={(event) =>
                      setAnalysisEvidenceIds((current) =>
                        event.target.checked
                          ? [...current, item.voiceEvidenceId]
                          : current.filter((id) => id !== item.voiceEvidenceId),
                      )
                    }
                  />
                  {item.sourceDisplayName}
                </label>
              ))}
            </fieldset>
            <textarea
              value={analysisGuidance}
              onChange={(event) => setAnalysisGuidance(event.target.value)}
              rows={3}
              placeholder="Optional guidance for this analysis, such as traits you want checked. Guidance is context, not evidence."
            />
            <div className="support-actions">
              <button
                className="primary-button compact"
                disabled={busy !== null || analysisEvidenceIds.length === 0}
                onClick={() => void runVoiceAnalysis()}
              >
                Analyze selected evidence
              </button>
            </div>
          </div>
        )}

        {analysisResult ? (
          <div className="voice-analysis-results">
            <div className="voice-card-heading">
              <div>
                <h3>Review proposals</h3>
                <p className="voice-meta">{analysisResult.operationId} v{analysisResult.operationVersion} | {analysisResult.runId}</p>
              </div>
              <span className="status-pill attention">Not canonical</span>
            </div>
            {analysisResult.proposals.length === 0 ? (
              <p className="voice-rule">No pending proposals. Core Voice was not changed.</p>
            ) : (
              <>
                <label className="field-label" htmlFor="proposal-voice-version">Accept into proposed Core Voice version</label>
                <select
                  id="proposal-voice-version"
                  value={proposalVoiceId}
                  disabled={busy !== null || proposedVoices.length === 0}
                  onChange={(event) => setProposalVoiceId(event.target.value)}
                >
                  {proposedVoices.length === 0 ? <option value="">Create a proposed Core Voice version first</option> : null}
                  {proposedVoices.map((voice) => <option key={voice.voiceId} value={voice.voiceId}>v{voice.versionNumber}: {voice.label}</option>)}
                </select>
                <div className="voice-card-list">
                  {analysisResult.proposals.map((proposal) => (
                    <article className="voice-model-card" key={proposal.proposalId}>
                      <h4>{proposal.name}</h4>
                      <p>{proposal.value}</p>
                      <p className="voice-meta">Evidence: {proposal.evidenceIds.join(", ")}</p>
                      <p className="voice-rule">Provider rationale: {proposal.rationale}</p>
                      <div className="support-actions">
                        <button
                          className="primary-button compact"
                          disabled={busy !== null || !proposalVoiceId}
                          onClick={() => void acceptVoiceProposal(proposal)}
                        >
                          Accept trait
                        </button>
                        <button className="quiet-button compact" disabled={busy !== null} onClick={() => discardVoiceProposal(proposal.proposalId)}>
                          Discard
                        </button>
                      </div>
                    </article>
                  ))}
                </div>
              </>
            )}
          </div>
        ) : null}
      </section>

''' + section_anchor
text = replace_once(text, section_anchor, analysis_section, "voice analysis UI section")
write(path, text)


# Lightweight layout support for analysis results.
path = "src/voice.css"
text = read(path)
if ".voice-analysis-panel" not in text:
    text += r'''

.voice-analysis-panel,
.voice-analysis-results {
  display: grid;
  gap: 0.8rem;
  margin-top: 1rem;
}

.voice-analysis-results {
  border-top: 1px solid var(--border-color, #d8d8d8);
  padding-top: 1rem;
}
'''
write(path, text)
