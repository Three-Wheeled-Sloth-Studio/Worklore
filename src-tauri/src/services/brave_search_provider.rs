use serde::Deserialize;

use crate::{
    domain::discovery::{DiscoveryFreshness, DiscoverySourceView},
    error::{ServiceResult, WorkLoreError},
};

const BRAVE_SEARCH_ENDPOINT: &str = "https://api.search.brave.com/res/v1/web/search";
const MAX_QUERY_CHARACTERS: usize = 400;

#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    web: Option<BraveWebResults>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResults {
    #[serde(default)]
    results: Vec<BraveWebResult>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResult {
    title: String,
    url: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    age: Option<String>,
    #[serde(default)]
    page_age: Option<String>,
}

pub async fn search(
    api_key: &str,
    query: &str,
    freshness: DiscoveryFreshness,
    count: usize,
) -> ServiceResult<Vec<DiscoverySourceView>> {
    let query = query.trim();
    if query.is_empty() {
        return Err(provider_error(
            "invalid_request",
            "Enter a discovery focus or create an active Theme before scanning.",
        ));
    }
    if query.chars().count() > MAX_QUERY_CHARACTERS {
        return Err(provider_error(
            "request_too_large",
            "The discovery search query is too long. Narrow the focus and try again.",
        ));
    }
    if api_key.trim().is_empty() {
        return Err(provider_error(
            "not_configured",
            "Save a Brave Search API key in Settings before scanning for timely topics.",
        ));
    }

    let response = reqwest::Client::new()
        .get(BRAVE_SEARCH_ENDPOINT)
        .header("Accept", "application/json")
        .header("X-Subscription-Token", api_key)
        .query(&[
            ("q", query.to_string()),
            ("count", count.clamp(1, 20).to_string()),
            ("freshness", freshness.brave_value().to_string()),
        ])
        .send()
        .await
        .map_err(|_| {
            provider_error(
                "provider_unavailable",
                "WorkLore could not reach Brave Search. Check the network connection and try again.",
            )
        })?;

    let status = response.status();
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(provider_error(
            "authentication_failed",
            "Brave Search rejected the saved API key. Update it in Settings and try again.",
        ));
    }
    if status.as_u16() == 429 {
        return Err(provider_error(
            "rate_limited",
            "Brave Search rate-limited this request. Wait before scanning again.",
        ));
    }
    if !status.is_success() {
        return Err(provider_error(
            "provider_unavailable",
            format!("Brave Search returned HTTP {}.", status.as_u16()),
        ));
    }

    let payload: BraveSearchResponse = response.json().await.map_err(|_| {
        provider_error(
            "invalid_provider_response",
            "Brave Search returned a response WorkLore could not read.",
        )
    })?;

    Ok(payload
        .web
        .map(|web| web.results)
        .unwrap_or_default()
        .into_iter()
        .filter(|result| is_http_url(&result.url))
        .map(|result| DiscoverySourceView {
            domain: domain_from_url(&result.url),
            title: result.title.trim().to_string(),
            url: result.url,
            description: result.description.trim().to_string(),
            age: result.age.or(result.page_age),
        })
        .filter(|result| !result.title.is_empty())
        .collect())
}

pub fn is_http_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

fn domain_from_url(url: &str) -> String {
    url.split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .trim_start_matches("www.")
        .to_string()
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
    fn freshness_values_match_brave_contract() {
        assert_eq!(DiscoveryFreshness::Day.brave_value(), "pd");
        assert_eq!(DiscoveryFreshness::Week.brave_value(), "pw");
        assert_eq!(DiscoveryFreshness::Month.brave_value(), "pm");
    }

    #[test]
    fn only_http_sources_are_openable() {
        assert!(is_http_url("https://example.com/a"));
        assert!(is_http_url("http://example.com/a"));
        assert!(!is_http_url("javascript:alert(1)"));
        assert!(!is_http_url("file:///tmp/a"));
    }

    #[test]
    fn domain_extraction_is_display_only() {
        assert_eq!(domain_from_url("https://www.example.com/path"), "example.com");
        assert_eq!(domain_from_url("http://sub.example.com"), "sub.example.com");
    }
}
