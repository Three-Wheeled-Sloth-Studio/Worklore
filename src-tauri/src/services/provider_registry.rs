use std::path::Path;

use crate::{
    domain::providers::{
        ProviderConnectionView, ProviderModelView, ProviderSettingsView,
        UpdateProviderSettingsRequest, GEMINI_PROVIDER_ID, OLLAMA_PROVIDER_ID, OPENAI_PROVIDER_ID,
    },
    error::{ServiceResult, WorkLoreError},
    services::{
        app_preferences_service::{self, ProviderSecretUpdate},
        ollama_provider, openai_compatible_provider, redaction_service,
        structured_provider::{StructuredProviderRequest, StructuredProviderResponse},
    },
};

pub fn get_settings() -> ServiceResult<ProviderSettingsView> {
    let settings = app_preferences_service::get_provider_settings()?;
    Ok(sanitize_loaded_settings(settings))
}

pub fn update_settings(
    request: UpdateProviderSettingsRequest,
) -> ServiceResult<ProviderSettingsView> {
    let selected_provider_id = normalize_provider_id(request.selected_provider_id.as_deref())?;
    let ollama_base_url = if selected_provider_id.as_deref() == Some(OLLAMA_PROVIDER_ID) {
        ollama_provider::normalize_base_url(&request.ollama_base_url)?
    } else {
        ollama_provider::normalize_base_url(&request.ollama_base_url)
            .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string())
    };
    let settings = ProviderSettingsView {
        selected_provider_id,
        ollama_base_url,
        ollama_model_id: normalize_optional_text(request.ollama_model_id.as_deref()),
        openai_model_id: normalize_optional_text(request.openai_model_id.as_deref()),
        openai_api_key_configured: false,
        gemini_model_id: normalize_optional_text(request.gemini_model_id.as_deref()),
        gemini_api_key_configured: false,
        brave_search_api_key_configured: false,
    };
    app_preferences_service::save_provider_settings(
        settings,
        ProviderSecretUpdate {
            openai_api_key: normalize_optional_secret(request.openai_api_key.as_deref()),
            clear_openai_api_key: request.clear_openai_api_key,
            gemini_api_key: normalize_optional_secret(request.gemini_api_key.as_deref()),
            clear_gemini_api_key: request.clear_gemini_api_key,
            brave_search_api_key: normalize_optional_secret(
                request.brave_search_api_key.as_deref(),
            ),
            clear_brave_search_api_key: request.clear_brave_search_api_key,
        },
    )?;
    get_settings()
}

pub async fn list_models(provider_id: &str) -> ServiceResult<Vec<ProviderModelView>> {
    let settings = require_selected_provider(provider_id)?;
    match provider_id {
        OLLAMA_PROVIDER_ID => ollama_provider::list_models(&settings.ollama_base_url).await,
        OPENAI_PROVIDER_ID | GEMINI_PROVIDER_ID => {
            let api_key = require_api_key(provider_id)?;
            openai_compatible_provider::list_models(provider_id, &api_key).await
        }
        _ => Err(provider_not_configured(
            "The selected provider is not registered.",
        )),
    }
}

pub async fn test_connection(provider_id: &str) -> ServiceResult<ProviderConnectionView> {
    let models = list_models(provider_id).await?;
    let message = if provider_id == OLLAMA_PROVIDER_ID {
        if models.is_empty() {
            "Ollama is reachable, but it reports no installed local models.".to_string()
        } else {
            format!(
                "Ollama is reachable with {} installed model(s).",
                models.len()
            )
        }
    } else {
        let label = openai_compatible_provider::provider_label(provider_id);
        if models.is_empty() {
            format!("{label} accepted the saved API key, but no models were listed.")
        } else {
            format!(
                "{label} accepted the saved API key and listed {} model(s).",
                models.len()
            )
        }
    };
    Ok(ProviderConnectionView {
        provider_id: provider_id.to_string(),
        available: true,
        model_count: models.len(),
        message,
    })
}

pub async fn run_structured(
    vault_path: &Path,
    provider_id: &str,
    mut request: StructuredProviderRequest,
) -> ServiceResult<StructuredProviderResponse> {
    let settings = require_selected_provider(provider_id)?;
    request.model_id = validate_requested_model(&settings, &request.model_id)?;
    match provider_id {
        OLLAMA_PROVIDER_ID => {
            ollama_provider::run_structured(&settings.ollama_base_url, request).await
        }
        OPENAI_PROVIDER_ID | GEMINI_PROVIDER_ID => {
            let api_key = require_api_key(provider_id)?;
            let redacted =
                redaction_service::redact_for_external_use(vault_path, &request.user_prompt)?;
            request.user_prompt = redacted.text;
            openai_compatible_provider::run_structured(provider_id, &api_key, request).await
        }
        _ => Err(provider_not_configured(
            "The selected provider is not registered.",
        )),
    }
}

pub fn require_selected_provider(provider_id: &str) -> ServiceResult<ProviderSettingsView> {
    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        return Err(provider_not_configured(
            "Select a provider before running this operation.",
        ));
    }
    let settings = get_settings()?;
    if settings.selected_provider_id.as_deref() != Some(provider_id) {
        return Err(provider_not_configured(
            "The requested provider is not the explicitly selected WorkLore provider.",
        ));
    }
    Ok(settings)
}

pub fn validate_requested_model(
    settings: &ProviderSettingsView,
    model_id: &str,
) -> ServiceResult<String> {
    let requested = model_id.trim();
    if requested.is_empty() {
        return Err(provider_not_configured(
            "Select a provider model before running this operation.",
        ));
    }
    let configured = match settings.selected_provider_id.as_deref() {
        Some(OLLAMA_PROVIDER_ID) => settings.ollama_model_id.as_deref(),
        Some(OPENAI_PROVIDER_ID) => settings.openai_model_id.as_deref(),
        Some(GEMINI_PROVIDER_ID) => settings.gemini_model_id.as_deref(),
        _ => None,
    };
    if configured != Some(requested) {
        return Err(provider_not_configured(
            "The requested model is not the explicitly configured model for this provider.",
        ));
    }
    Ok(requested.to_string())
}

fn require_api_key(provider_id: &str) -> ServiceResult<String> {
    app_preferences_service::get_provider_api_key(provider_id)?.ok_or_else(|| {
        provider_not_configured("Save an API key for the selected BYOK provider before using it.")
    })
}

fn sanitize_loaded_settings(mut settings: ProviderSettingsView) -> ProviderSettingsView {
    if !matches!(
        settings.selected_provider_id.as_deref(),
        Some(OLLAMA_PROVIDER_ID | OPENAI_PROVIDER_ID | GEMINI_PROVIDER_ID)
    ) {
        settings.selected_provider_id = None;
    }
    settings.ollama_model_id = normalize_optional_text(settings.ollama_model_id.as_deref());
    settings.openai_model_id = normalize_optional_text(settings.openai_model_id.as_deref());
    settings.gemini_model_id = normalize_optional_text(settings.gemini_model_id.as_deref());
    settings
}

fn normalize_provider_id(value: Option<&str>) -> ServiceResult<Option<String>> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(None),
        Some(OLLAMA_PROVIDER_ID) => Ok(Some(OLLAMA_PROVIDER_ID.to_string())),
        Some(OPENAI_PROVIDER_ID) => Ok(Some(OPENAI_PROVIDER_ID.to_string())),
        Some(GEMINI_PROVIDER_ID) => Ok(Some(GEMINI_PROVIDER_ID.to_string())),
        Some(_) => Err(provider_not_configured(
            "Choose one of the providers registered in WorkLore Settings.",
        )),
    }
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn normalize_optional_secret(value: Option<&str>) -> Option<String> {
    normalize_optional_text(value)
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

    fn settings(provider_id: &str, model_id: &str) -> ProviderSettingsView {
        ProviderSettingsView {
            selected_provider_id: Some(provider_id.to_string()),
            ollama_base_url: "http://127.0.0.1:11434".into(),
            ollama_model_id: (provider_id == OLLAMA_PROVIDER_ID).then(|| model_id.to_string()),
            openai_model_id: (provider_id == OPENAI_PROVIDER_ID).then(|| model_id.to_string()),
            openai_api_key_configured: provider_id == OPENAI_PROVIDER_ID,
            gemini_model_id: (provider_id == GEMINI_PROVIDER_ID).then(|| model_id.to_string()),
            gemini_api_key_configured: provider_id == GEMINI_PROVIDER_ID,
            brave_search_api_key_configured: false,
        }
    }

    #[test]
    fn provider_selection_is_explicit_and_does_not_fallback() {
        assert_eq!(normalize_provider_id(None).unwrap(), None);
        assert_eq!(
            normalize_provider_id(Some("ollama")).unwrap(),
            Some("ollama".to_string())
        );
        assert_eq!(
            normalize_provider_id(Some("openai")).unwrap(),
            Some("openai".to_string())
        );
        assert_eq!(
            normalize_provider_id(Some("gemini")).unwrap(),
            Some("gemini".to_string())
        );
        assert!(normalize_provider_id(Some("other-cloud")).is_err());
    }

    #[test]
    fn requested_model_must_match_the_selected_provider_configuration() {
        for provider_id in [OLLAMA_PROVIDER_ID, OPENAI_PROVIDER_ID, GEMINI_PROVIDER_ID] {
            let configured = settings(provider_id, "model-a");
            assert_eq!(
                validate_requested_model(&configured, "model-a").unwrap(),
                "model-a"
            );
            assert!(validate_requested_model(&configured, "model-b").is_err());
        }
    }

    #[test]
    fn selected_provider_can_be_saved_before_a_model_is_selected() {
        let mut configured = settings(OPENAI_PROVIDER_ID, "model-a");
        configured.openai_model_id = None;
        assert_eq!(
            configured.selected_provider_id.as_deref(),
            Some(OPENAI_PROVIDER_ID)
        );
        assert!(validate_requested_model(&configured, "model-a").is_err());
    }
}
