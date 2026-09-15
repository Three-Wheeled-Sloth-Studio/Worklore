use crate::{
    domain::providers::{
        ProviderConnectionView, ProviderModelView, ProviderSettingsView,
        UpdateProviderSettingsRequest, OLLAMA_PROVIDER_ID,
    },
    error::{ServiceResult, WorkLoreError},
    services::{app_preferences_service, ollama_provider},
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
        _ => Err(provider_not_configured(
            "The selected provider is not registered.",
        )),
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
            format!(
                "Ollama is reachable with {} installed model(s).",
                models.len()
            )
        },
    })
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
    fn selected_provider_can_be_discovered_before_a_model_is_selected() {
        let settings = ProviderSettingsView {
            selected_provider_id: Some("ollama".into()),
            ollama_base_url: "http://127.0.0.1:11434".into(),
            ollama_model_id: None,
        };
        assert_eq!(settings.selected_provider_id.as_deref(), Some("ollama"));
        assert!(validate_requested_model(&settings, "qwen3").is_err());
    }

    #[test]
    fn requested_model_must_match_explicit_configuration() {
        let settings = ProviderSettingsView {
            selected_provider_id: Some("ollama".into()),
            ollama_base_url: "http://127.0.0.1:11434".into(),
            ollama_model_id: Some("qwen3".into()),
        };
        assert_eq!(
            validate_requested_model(&settings, "qwen3").unwrap(),
            "qwen3"
        );
        assert!(validate_requested_model(&settings, "gemma3").is_err());
    }
}
