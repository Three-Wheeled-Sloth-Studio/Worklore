use std::collections::{BTreeSet, HashSet};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use quick_xml::{events::Event, Reader};
use serde::Deserialize;

use crate::{
    domain::discovery::{DiscoveryFreshness, DiscoverySourceView},
    error::{ServiceResult, WorkLoreError},
};

const USER_AGENT: &str = "WorkLore/0.1 (+https://github.com/Three-Wheeled-Sloth-Studio/Worklore)";
const HN_FRONT_PAGE_ENDPOINT: &str =
    "https://hn.algolia.com/api/v1/search?tags=front_page&hitsPerPage=30";

const RSS_SEEDS: [(&str, &str); 4] = [
    ("TechCrunch", "https://techcrunch.com/feed/"),
    (
        "BBC Business",
        "https://feeds.bbci.co.uk/news/business/rss.xml",
    ),
    ("BLS", "https://www.bls.gov/feed/bls_latest.rss"),
    (
        "Federal Reserve",
        "https://www.federalreserve.gov/feeds/press_all.xml",
    ),
];

#[derive(Debug)]
struct Candidate {
    source: DiscoverySourceView,
    published_at: Option<DateTime<Utc>>,
    relevance: usize,
}

#[derive(Debug, Deserialize)]
struct HackerNewsResponse {
    #[serde(default)]
    hits: Vec<HackerNewsHit>,
}

#[derive(Debug, Deserialize)]
struct HackerNewsHit {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(rename = "objectID")]
    object_id: String,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    points: Option<i64>,
    #[serde(default)]
    num_comments: Option<i64>,
}

#[derive(Debug, Default)]
struct RssItem {
    title: String,
    link: String,
    description: String,
    published: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RssField {
    Title,
    Link,
    Description,
    Published,
}

pub async fn fetch(
    focus: &str,
    freshness: DiscoveryFreshness,
    count: usize,
) -> ServiceResult<Vec<DiscoverySourceView>> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|_| {
            provider_error(
                "provider_unavailable",
                "WorkLore could not initialize public discovery.",
            )
        })?;

    let focus_tokens = tokens(focus);
    let mut buckets = Vec::new();
    let mut successful_sources = 0usize;

    if let Ok(items) = fetch_hacker_news(&client, freshness, &focus_tokens).await {
        successful_sources += 1;
        buckets.push(items);
    }

    for (label, endpoint) in RSS_SEEDS {
        if let Ok(items) = fetch_rss(&client, label, endpoint, freshness, &focus_tokens).await {
            successful_sources += 1;
            buckets.push(items);
        }
    }

    if successful_sources == 0 {
        return Err(provider_error(
            "provider_unavailable",
            "WorkLore could not reach any seeded public discovery sources. Check the network connection and try again.",
        ));
    }

    Ok(round_robin(buckets, count.clamp(1, 20)))
}

pub fn is_http_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

pub fn source_labels() -> Vec<&'static str> {
    let mut labels = vec!["Hacker News"];
    labels.extend(RSS_SEEDS.iter().map(|(label, _)| *label));
    labels
}

async fn fetch_hacker_news(
    client: &reqwest::Client,
    freshness: DiscoveryFreshness,
    focus_tokens: &BTreeSet<String>,
) -> ServiceResult<Vec<Candidate>> {
    let response = client
        .get(HN_FRONT_PAGE_ENDPOINT)
        .send()
        .await
        .map_err(|_| {
            provider_error(
                "source_unavailable",
                "Hacker News discovery is unavailable.",
            )
        })?;

    if !response.status().is_success() {
        return Err(provider_error(
            "source_unavailable",
            format!(
                "Hacker News discovery returned HTTP {}.",
                response.status().as_u16()
            ),
        ));
    }

    let payload: HackerNewsResponse = response.json().await.map_err(|_| {
        provider_error(
            "invalid_provider_response",
            "Hacker News returned an unreadable response.",
        )
    })?;

    let mut candidates = payload
        .hits
        .into_iter()
        .filter_map(|hit| {
            let title = hit.title?.trim().to_string();
            if title.is_empty() {
                return None;
            }
            let published_at = hit.created_at.as_deref().and_then(parse_published_at);
            if !within_freshness(published_at.as_ref(), freshness) {
                return None;
            }
            let url = hit
                .url
                .filter(|value| is_http_url(value))
                .unwrap_or_else(|| {
                    format!("https://news.ycombinator.com/item?id={}", hit.object_id)
                });
            let description = match (hit.points, hit.num_comments) {
                (Some(points), Some(comments)) => {
                    format!("Hacker News discussion: {points} points, {comments} comments.")
                }
                _ => "Hacker News discussion.".to_string(),
            };
            let relevance = relevance_score(&title, &description, focus_tokens);
            if !focus_tokens.is_empty() && relevance == 0 {
                return None;
            }
            Some(Candidate {
                source: DiscoverySourceView {
                    title,
                    domain: domain_from_url(&url),
                    url,
                    description,
                    age: published_at.as_ref().map(DateTime::to_rfc3339),
                },
                published_at,
                relevance,
            })
        })
        .collect::<Vec<_>>();
    sort_candidates(&mut candidates);
    Ok(candidates)
}

async fn fetch_rss(
    client: &reqwest::Client,
    label: &str,
    endpoint: &str,
    freshness: DiscoveryFreshness,
    focus_tokens: &BTreeSet<String>,
) -> ServiceResult<Vec<Candidate>> {
    let response = client.get(endpoint).send().await.map_err(|_| {
        provider_error(
            "source_unavailable",
            format!("{label} discovery is unavailable."),
        )
    })?;

    if !response.status().is_success() {
        return Err(provider_error(
            "source_unavailable",
            format!(
                "{label} discovery returned HTTP {}.",
                response.status().as_u16()
            ),
        ));
    }

    let body = response.text().await.map_err(|_| {
        provider_error(
            "invalid_provider_response",
            format!("{label} returned unreadable feed data."),
        )
    })?;
    let mut candidates = parse_rss_items(&body)
        .into_iter()
        .filter_map(|item| {
            let title = plain_text(&item.title);
            let url = item.link.trim().to_string();
            if title.is_empty() || !is_http_url(&url) {
                return None;
            }
            let description = plain_text(&item.description);
            let published_at = parse_published_at(&item.published);
            if !within_freshness(published_at.as_ref(), freshness) {
                return None;
            }
            let relevance = relevance_score(&title, &description, focus_tokens);
            if !focus_tokens.is_empty() && relevance == 0 {
                return None;
            }
            Some(Candidate {
                source: DiscoverySourceView {
                    title,
                    domain: domain_from_url(&url),
                    url,
                    description,
                    age: published_at.as_ref().map(DateTime::to_rfc3339),
                },
                published_at,
                relevance,
            })
        })
        .collect::<Vec<_>>();
    sort_candidates(&mut candidates);
    Ok(candidates)
}

fn parse_rss_items(xml: &str) -> Vec<RssItem> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut items = Vec::new();
    let mut current: Option<RssItem> = None;
    let mut field: Option<RssField> = None;

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"item" => current = Some(RssItem::default()),
                b"title" if current.is_some() => field = Some(RssField::Title),
                b"link" if current.is_some() => field = Some(RssField::Link),
                b"description" if current.is_some() => field = Some(RssField::Description),
                b"pubDate" | b"published" | b"updated" if current.is_some() => {
                    field = Some(RssField::Published)
                }
                _ => {}
            },
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"item" => {
                    if let Some(item) = current.take() {
                        items.push(item);
                    }
                    field = None;
                }
                b"title" | b"link" | b"description" | b"pubDate" | b"published" | b"updated" => {
                    field = None;
                }
                _ => {}
            },
            Ok(Event::Text(text)) => {
                if let (Some(item), Some(active_field)) = (current.as_mut(), field) {
                    if let Ok(value) = text.decode() {
                        append_field(item, active_field, &value);
                    }
                }
            }
            Ok(Event::CData(text)) => {
                if let (Some(item), Some(active_field)) = (current.as_mut(), field) {
                    append_field(item, active_field, &String::from_utf8_lossy(text.as_ref()));
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buffer.clear();
    }

    items
}

fn append_field(item: &mut RssItem, field: RssField, value: &str) {
    let target = match field {
        RssField::Title => &mut item.title,
        RssField::Link => &mut item.link,
        RssField::Description => &mut item.description,
        RssField::Published => &mut item.published,
    };
    if !target.is_empty() {
        target.push(' ');
    }
    target.push_str(value.trim());
}

fn sort_candidates(candidates: &mut [Candidate]) {
    candidates.sort_by(|left, right| {
        right
            .relevance
            .cmp(&left.relevance)
            .then_with(|| right.published_at.cmp(&left.published_at))
    });
}

fn round_robin(mut buckets: Vec<Vec<Candidate>>, count: usize) -> Vec<DiscoverySourceView> {
    let mut output = Vec::new();
    let mut seen_urls = HashSet::new();
    let mut index = 0usize;

    while output.len() < count {
        let mut added = false;
        for bucket in &mut buckets {
            if let Some(candidate) = bucket.get(index) {
                if seen_urls.insert(candidate.source.url.clone()) {
                    output.push(candidate.source.clone());
                    added = true;
                    if output.len() == count {
                        break;
                    }
                }
            }
        }
        if !added && buckets.iter().all(|bucket| bucket.len() <= index) {
            break;
        }
        index += 1;
    }

    output
}

fn within_freshness(published_at: Option<&DateTime<Utc>>, freshness: DiscoveryFreshness) -> bool {
    let Some(published_at) = published_at else {
        return true;
    };
    let window = match freshness {
        DiscoveryFreshness::Day => ChronoDuration::hours(24),
        DiscoveryFreshness::Week => ChronoDuration::days(7),
        DiscoveryFreshness::Month => ChronoDuration::days(31),
    };
    *published_at >= Utc::now() - window
}

fn parse_published_at(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc2822(value)
        .or_else(|_| DateTime::parse_from_rfc3339(value))
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn relevance_score(title: &str, description: &str, focus_tokens: &BTreeSet<String>) -> usize {
    if focus_tokens.is_empty() {
        return 0;
    }
    let candidate_tokens = tokens(&format!("{title} {description}"));
    focus_tokens.intersection(&candidate_tokens).count()
}

fn tokens(value: &str) -> BTreeSet<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .map(str::trim)
        .filter(|token| token.len() >= 3)
        .map(str::to_ascii_lowercase)
        .collect()
}

fn plain_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
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
    fn seeded_endpoints_never_interpolate_user_focus() {
        let sensitive_focus = "Private Client Project Nightjar";
        assert!(!HN_FRONT_PAGE_ENDPOINT.contains(sensitive_focus));
        assert!(RSS_SEEDS
            .iter()
            .all(|(_, endpoint)| !endpoint.contains(sensitive_focus)));
    }

    #[test]
    fn source_mix_is_bounded_and_diverse() {
        assert_eq!(
            source_labels(),
            vec![
                "Hacker News",
                "TechCrunch",
                "BBC Business",
                "BLS",
                "Federal Reserve"
            ]
        );
    }

    #[test]
    fn parses_rss_items_without_network_access() {
        let xml = r#"<rss><channel><item>
            <title>AI changes product operations</title>
            <link>https://example.com/ai-product</link>
            <description><![CDATA[Teams are changing how they work.]]></description>
            <pubDate>Mon, 21 Sep 2026 12:00:00 GMT</pubDate>
        </item></channel></rss>"#;
        let items = parse_rss_items(xml);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "AI changes product operations");
        assert_eq!(items[0].link, "https://example.com/ai-product");
        assert!(items[0].description.contains("Teams are changing"));
    }

    #[test]
    fn focus_matching_happens_locally() {
        let focus = tokens("agentic product management");
        assert!(
            relevance_score(
                "Agentic workflows reshape product teams",
                "New management patterns are emerging.",
                &focus
            ) > 0
        );
        assert_eq!(
            relevance_score("Retail sales update", "Quarterly demand changed.", &focus),
            0
        );
    }

    #[test]
    fn round_robin_prevents_one_source_from_consuming_the_scan() {
        fn candidate(url: &str) -> Candidate {
            Candidate {
                source: DiscoverySourceView {
                    title: url.to_string(),
                    url: url.to_string(),
                    description: String::new(),
                    age: None,
                    domain: "example.com".to_string(),
                },
                published_at: None,
                relevance: 0,
            }
        }
        let output = round_robin(
            vec![
                vec![
                    candidate("https://example.com/a1"),
                    candidate("https://example.com/a2"),
                ],
                vec![
                    candidate("https://example.org/b1"),
                    candidate("https://example.org/b2"),
                ],
            ],
            3,
        );
        assert_eq!(
            output
                .iter()
                .map(|item| item.url.as_str())
                .collect::<Vec<_>>(),
            vec![
                "https://example.com/a1",
                "https://example.org/b1",
                "https://example.com/a2"
            ]
        );
    }
}
