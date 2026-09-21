use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet},
    path::Path,
    time::Duration,
};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    domain::discovery::{
        DevelopDiscoveryTopicRequest, DevelopDiscoveryTopicResult, DiscoveryFeedbackView,
        DiscoveryMatchView, DiscoveryOpportunityStatus, DiscoveryOpportunityView,
        DiscoveryScanResult, DiscoverySourceView, RecordDiscoveryFeedbackRequest,
        SaveDiscoveryInspirationResult, ScanDiscoveryRequest,
    },
    error::{ServiceResult, WorkLoreError},
    services::{
        app_preferences_service, brave_search_provider, canonical_store, inspiration_service,
        redaction_service, topic_service,
    },
};

const DEFAULT_SEARCH_RESULTS: usize = 12;
const MAX_STORED_OPPORTUNITIES_PER_SCAN: usize = 10;
const MAX_FEEDBACK_EXAMPLES: usize = 40;
const RECENT_TOPIC_LIMIT: usize = 30;

#[derive(Debug, Clone)]
struct SignalRecord {
    kind: &'static str,
    record_id: String,
    label: String,
    detail: String,
    tokens: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct PriorFeedback {
    feedback_id: String,
    verdict: String,
    tokens: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct RankedOpportunity {
    view: DiscoveryOpportunityView,
    feature_tokens: BTreeSet<String>,
    provider_position: usize,
    feedback_bias: i8,
    recent_overlap: bool,
}

pub async fn scan(
    vault_path: &Path,
    request: ScanDiscoveryRequest,
) -> ServiceResult<DiscoveryScanResult> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let raw_query = discovery_query(&connection, &request.focus)?;
    drop(connection);

    let redacted = redaction_service::redact_for_external_use(vault_path, &raw_query)?;
    let external_query = redacted.text.trim().to_string();
    if external_query.is_empty() {
        return Err(discovery_error(
            "invalid_request",
            "The discovery focus was empty after privacy preflight. Enter a broader public-safe focus.",
        ));
    }

    let api_key = app_preferences_service::get_brave_search_api_key()?.ok_or_else(|| {
        discovery_error(
            "not_configured",
            "Save a Brave Search API key in Settings before scanning for timely topics.",
        )
    })?;
    let requested_count = request
        .max_results
        .unwrap_or(DEFAULT_SEARCH_RESULTS)
        .clamp(5, 20);
    let results = brave_search_provider::search(
        &api_key,
        &external_query,
        request.freshness,
        requested_count,
    )
    .await?;

    let connection = open_connection(vault_path)?;
    let themes = load_theme_signals(&connection)?;
    let standing = load_standing_signals(&connection)?;
    let audience = load_audience_signals(&connection)?;
    let recent_topics = load_recent_topic_signals(&connection)?;
    let prior_feedback = load_prior_feedback(&connection)?;

    let clusters = cluster_sources(results);
    let run_id = format!("discovery_run_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    let mut used_feedback_ids = HashSet::new();
    let mut ranked = clusters
        .into_iter()
        .enumerate()
        .map(|(position, sources)| {
            qualify_cluster(
                &run_id,
                position,
                sources,
                &themes,
                &standing,
                &audience,
                &recent_topics,
                &prior_feedback,
                &mut used_feedback_ids,
                &now,
            )
        })
        .collect::<Vec<_>>();

    ranked.sort_by(compare_ranked);
    ranked.truncate(MAX_STORED_OPPORTUNITIES_PER_SCAN);

    connection.execute(
        "INSERT INTO discovery_runs(
           run_id,provider_id,focus_text,external_query,freshness,requested_count,
           feedback_examples_used,created_at)
         VALUES (?1,'brave_search',?2,?3,?4,?5,?6,?7)",
        params![
            &run_id,
            request.focus.trim(),
            &external_query,
            request.freshness.as_str(),
            i64::try_from(requested_count).unwrap_or(20),
            i64::try_from(used_feedback_ids.len()).unwrap_or(0),
            &now
        ],
    )?;

    for (rank_position, opportunity) in ranked.iter().enumerate() {
        persist_opportunity(&connection, opportunity, rank_position)?;
    }

    let opportunities = ranked.into_iter().map(|item| item.view).collect();
    Ok(DiscoveryScanResult {
        run_id,
        external_query,
        freshness: request.freshness,
        feedback_examples_used: used_feedback_ids.len(),
        opportunities,
    })
}

pub fn list_opportunities(
    vault_path: &Path,
    limit: usize,
) -> ServiceResult<Vec<DiscoveryOpportunityView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut statement = connection.prepare(
        "SELECT opportunity_id FROM discovery_opportunities
         ORDER BY created_at DESC,rank_position ASC LIMIT ?1",
    )?;
    let limit = i64::try_from(limit.clamp(1, 100)).unwrap_or(20);
    let ids = statement
        .query_map([limit], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|id| load_opportunity(&connection, &id))
        .collect()
}

pub fn record_feedback(
    vault_path: &Path,
    request: RecordDiscoveryFeedbackRequest,
) -> ServiceResult<DiscoveryFeedbackView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let opportunity = load_opportunity(&connection, &request.opportunity_id)?;
    let reasons = normalize_reasons(request.reasons);
    let note = request.note.trim().to_string();
    let opportunity_tokens = tokens(&format!("{} {}", opportunity.title, opportunity.summary));
    let note_tokens = tokens(&note);
    let normalized_signals = json!({
        "verdict": request.verdict.as_str(),
        "reasons": reasons.clone(),
        "opportunityTokens": opportunity_tokens.iter().cloned().collect::<Vec<_>>(),
        "noteTokens": note_tokens.iter().cloned().collect::<Vec<_>>()
    });
    let feedback_id = format!("discovery_feedback_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();

    connection.execute(
        "INSERT INTO discovery_feedback(
           feedback_id,opportunity_id,verdict,reasons_json,user_note,normalized_signals_json,created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            &feedback_id,
            &request.opportunity_id,
            request.verdict.as_str(),
            serde_json::to_string(&reasons)?,
            &note,
            normalized_signals.to_string(),
            &now
        ],
    )?;
    audit(
        &connection,
        "discovery_feedback_recorded",
        "discovery_opportunity",
        &request.opportunity_id,
        json!({
            "feedbackId": feedback_id,
            "verdict": request.verdict.as_str(),
            "reasons": reasons
        }),
    )?;

    Ok(DiscoveryFeedbackView {
        feedback_id,
        opportunity_id: request.opportunity_id,
        verdict: request.verdict,
        reasons,
        note,
        normalized_signals,
        created_at: now,
    })
}

pub fn save_as_inspiration(
    vault_path: &Path,
    opportunity_id: &str,
) -> ServiceResult<SaveDiscoveryInspirationResult> {
    let inspiration_id = ensure_inspiration(vault_path, opportunity_id)?;
    Ok(SaveDiscoveryInspirationResult { inspiration_id })
}

pub fn develop_topic(
    vault_path: &Path,
    request: DevelopDiscoveryTopicRequest,
) -> ServiceResult<DevelopDiscoveryTopicResult> {
    canonical_store::initialize(vault_path)?;
    let title = required_text(&request.title, "Topic title")?;
    let summary = required_text(&request.summary, "Topic summary")?;
    let connection = open_connection(vault_path)?;
    let opportunity = load_opportunity(&connection, &request.opportunity_id)?;
    if let Some(topic_id) = opportunity.topic_id {
        let inspiration_id = opportunity.inspiration_id.ok_or_else(|| {
            WorkLoreError::InvalidVault(
                "A discovery-created Topic is missing its Inspiration lineage.".to_string(),
            )
        })?;
        return Ok(DevelopDiscoveryTopicResult {
            topic_id,
            inspiration_id,
        });
    }
    drop(connection);

    let inspiration_id = ensure_inspiration(vault_path, &request.opportunity_id)?;
    let opportunity = {
        let connection = open_connection(vault_path)?;
        load_opportunity(&connection, &request.opportunity_id)?
    };
    let topic = topic_service::create_topic(
        vault_path,
        topic_service::CreateTopicRequest {
            title,
            summary,
            timing_class: topic_service::TopicTimingClass::Timely,
            relevant_until: None,
            timely_note: Some(
                "Current external discussion captured through WorkLore discovery.".to_string(),
            ),
        },
    )?;

    topic_service::add_topic_relationship(
        vault_path,
        &topic.topic_id,
        topic_service::TopicRelationKind::Inspiration,
        &inspiration_id,
    )?;
    for item in opportunity
        .standing_matches
        .iter()
        .chain(opportunity.theme_matches.iter())
        .chain(opportunity.audience_matches.iter())
    {
        let relation = match item.kind.as_str() {
            "story" => Some(topic_service::TopicRelationKind::Story),
            "proof_point" => Some(topic_service::TopicRelationKind::ProofPoint),
            "theme" => Some(topic_service::TopicRelationKind::Theme),
            "target_context" => Some(topic_service::TopicRelationKind::TargetContext),
            _ => None,
        };
        if let Some(relation) = relation {
            topic_service::add_topic_relationship(
                vault_path,
                &topic.topic_id,
                relation,
                &item.record_id,
            )?;
        }
    }

    let connection = open_connection(vault_path)?;
    connection.execute(
        "UPDATE discovery_opportunities
         SET status='topic_created',topic_id=?2,inspiration_id=?3,updated_at=?4
         WHERE opportunity_id=?1",
        params![
            &request.opportunity_id,
            &topic.topic_id,
            &inspiration_id,
            Utc::now().to_rfc3339()
        ],
    )?;
    audit(
        &connection,
        "discovery_topic_created",
        "discovery_opportunity",
        &request.opportunity_id,
        json!({
            "topicId": topic.topic_id,
            "inspirationId": inspiration_id,
            "summaryWasExplicitlyConfirmed": true
        }),
    )?;

    Ok(DevelopDiscoveryTopicResult {
        topic_id: topic.topic_id,
        inspiration_id,
    })
}

pub fn dismiss_opportunity(
    vault_path: &Path,
    opportunity_id: &str,
) -> ServiceResult<DiscoveryOpportunityView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    load_opportunity(&connection, opportunity_id)?;
    connection.execute(
        "UPDATE discovery_opportunities SET status='dismissed',updated_at=?2
         WHERE opportunity_id=?1",
        params![opportunity_id, Utc::now().to_rfc3339()],
    )?;
    audit(
        &connection,
        "discovery_opportunity_dismissed",
        "discovery_opportunity",
        opportunity_id,
        json!({}),
    )?;
    load_opportunity(&connection, opportunity_id)
}

pub fn validate_source_url(
    vault_path: &Path,
    opportunity_id: &str,
    requested_url: &str,
) -> ServiceResult<String> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let opportunity = load_opportunity(&connection, opportunity_id)?;
    if !brave_search_provider::is_http_url(requested_url) {
        return Err(discovery_error(
            "invalid_external_url",
            "WorkLore only opens HTTP or HTTPS discovery sources.",
        ));
    }
    opportunity
        .sources
        .iter()
        .find(|source| source.url == requested_url)
        .map(|source| source.url.clone())
        .ok_or_else(|| {
            discovery_error(
                "invalid_external_url",
                "That URL is not one of the persisted sources for this discovery opportunity.",
            )
        })
}

fn ensure_inspiration(vault_path: &Path, opportunity_id: &str) -> ServiceResult<String> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let opportunity = load_opportunity(&connection, opportunity_id)?;
    if let Some(inspiration_id) = opportunity.inspiration_id {
        return Ok(inspiration_id);
    }
    let primary_source = opportunity.sources.first().ok_or_else(|| {
        WorkLoreError::InvalidVault(
            "A discovery opportunity needs at least one external source.".to_string(),
        )
    })?;
    let source_id = format!("source_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    let captured_text = format!(
        "{}\n\n{}\n\n{}",
        primary_source.title, primary_source.description, primary_source.url
    );
    let byte_size = i64::try_from(captured_text.len()).map_err(|_| {
        WorkLoreError::InvalidVault("Discovery source text is too large.".to_string())
    })?;
    let content_hash = hex::encode(Sha256::digest(captured_text.as_bytes()));
    let extraction = json!({
        "status": "complete",
        "extractorVersion": "discovery-search-v1",
        "textPath": null,
        "characterCount": captured_text.chars().count(),
        "warnings": [],
        "error": null
    });
    let privacy = json!({
        "status": "unavailable",
        "scanVersion": null,
        "scannedAt": null,
        "reviewItemIds": []
    });
    let provenance = json!({
        "creationActor": "system",
        "importMethod": "discovery_search",
        "sourceUrl": primary_source.url,
        "discoveryOpportunityId": opportunity_id
    });
    connection.execute(
        "INSERT INTO sources(
           source_id,source_type,display_name,stored_path,original_file_name,media_type,byte_size,
           content_hash,lifecycle_status,imported_at,updated_at,extraction_json,privacy_json,
           provenance_json,tags_json,revision,source_origin,captured_text)
         VALUES (?1,'other',?2,'','','text/plain',?3,?4,'active',?5,?5,?6,?7,?8,'[]',1,
                 'discovery_external',?9)",
        params![
            &source_id,
            &opportunity.title,
            byte_size,
            content_hash,
            &now,
            extraction.to_string(),
            privacy.to_string(),
            provenance.to_string(),
            &captured_text
        ],
    )?;
    drop(connection);

    let created = inspiration_service::create_inspiration_from_source(vault_path, &source_id)?;
    let concepts = opportunity
        .theme_matches
        .iter()
        .map(|item| item.label.clone())
        .collect::<Vec<_>>();
    let updated = inspiration_service::update_inspiration(
        vault_path,
        inspiration_service::UpdateInspirationRequest {
            inspiration_id: created.inspiration.inspiration_id.clone(),
            title: opportunity.title.clone(),
            lifecycle: inspiration_service::InspirationLifecycle::Saved,
            source_url: Some(primary_source.url.clone()),
            source_title: Some(primary_source.title.clone()),
            source_author: None,
            source_published_at: primary_source.age.clone(),
            summary: opportunity.summary.clone(),
            takeaways: Vec::new(),
            excerpts: Vec::new(),
            why_interesting: opportunity.why_now.clone(),
            user_reaction: String::new(),
            concepts,
            questions: Vec::new(),
            counterpoints: opportunity.concerns.clone(),
            notes: format!(
                "Saved from discovery opportunity {}. External material remains context, not evidence.",
                opportunity_id
            ),
        },
    )?;

    let connection = open_connection(vault_path)?;
    connection.execute(
        "UPDATE discovery_opportunities SET inspiration_id=?2,
         status=CASE WHEN status='topic_created' THEN status ELSE 'inspiration_saved' END,
         updated_at=?3 WHERE opportunity_id=?1",
        params![
            opportunity_id,
            &updated.inspiration_id,
            Utc::now().to_rfc3339()
        ],
    )?;
    audit(
        &connection,
        "discovery_inspiration_saved",
        "discovery_opportunity",
        opportunity_id,
        json!({"inspirationId": updated.inspiration_id, "sourceId": source_id}),
    )?;
    Ok(updated.inspiration_id)
}

fn qualify_cluster(
    run_id: &str,
    provider_position: usize,
    sources: Vec<DiscoverySourceView>,
    themes: &[SignalRecord],
    standing: &[SignalRecord],
    audience: &[SignalRecord],
    recent_topics: &[SignalRecord],
    prior_feedback: &[PriorFeedback],
    used_feedback_ids: &mut HashSet<String>,
    now: &str,
) -> RankedOpportunity {
    let title = sources
        .first()
        .map(|source| source.title.clone())
        .unwrap_or_else(|| "Untitled current discussion".to_string());
    let summary = sources
        .iter()
        .map(|source| source.description.trim())
        .find(|description| !description.is_empty())
        .unwrap_or("")
        .to_string();
    let feature_tokens = tokens(
        &sources
            .iter()
            .map(|source| format!("{} {}", source.title, source.description))
            .collect::<Vec<_>>()
            .join(" "),
    );
    let theme_matches = matching_signals(themes, &feature_tokens);
    let standing_matches = matching_signals(standing, &feature_tokens);
    let audience_matches = matching_signals(audience, &feature_tokens);
    let recent_matches = matching_signals(recent_topics, &feature_tokens);
    let recent_overlap = !recent_matches.is_empty();

    let mut positive = 0usize;
    let mut negative = 0usize;
    let mut not_now = 0usize;
    for feedback in prior_feedback {
        if token_similarity(&feature_tokens, &feedback.tokens) {
            used_feedback_ids.insert(feedback.feedback_id.clone());
            match feedback.verdict.as_str() {
                "good_candidate" => positive += 1,
                "not_for_me" => negative += 1,
                "not_now" => not_now += 1,
                _ => {}
            }
        }
    }
    let feedback_bias = match positive.cmp(&negative) {
        Ordering::Greater => 1,
        Ordering::Equal => 0,
        Ordering::Less => -1,
    };
    let feedback_adjustment = if positive > negative {
        Some(format!(
            "Similar to {} prior opportunit{} you marked as a good candidate.",
            positive,
            if positive == 1 { "y" } else { "ies" }
        ))
    } else if negative > positive {
        Some(format!(
            "Similar to {} prior opportunit{} you marked as not for you.",
            negative,
            if negative == 1 { "y" } else { "ies" }
        ))
    } else if not_now > 0 {
        Some(format!(
            "Similar to {} prior opportunit{} you marked as not timely then; WorkLore is not treating that as a topic rejection.",
            not_now,
            if not_now == 1 { "y" } else { "ies" }
        ))
    } else {
        None
    };

    let why_now = if sources.len() > 1 {
        format!(
            "{} current sources in this scan are discussing substantially similar material.",
            sources.len()
        )
    } else if let Some(age) = sources.first().and_then(|source| source.age.as_deref()) {
        format!("A current source surfaced this discussion ({age}).")
    } else {
        "A current source surfaced this discussion within the selected freshness window."
            .to_string()
    };

    let possible_angle = if let Some(item) = standing_matches.first() {
        if let Some(theme) = theme_matches.first() {
            format!(
                "Use {} as concrete standing and {} as the framing lens. Add your own point of view before drafting.",
                item.label, theme.label
            )
        } else {
            format!(
                "Use {} as concrete standing. Add the specific lesson or point of view you want the post to make.",
                item.label
            )
        }
    } else if let Some(theme) = theme_matches.first() {
        format!(
            "This intersects {}. WorkLore found no matching Story or Proof Point, so establish your own angle before treating it as a post candidate.",
            theme.label
        )
    } else {
        "The external discussion is current, but WorkLore found no existing Theme or standing connection. Treat it as exploration, not a ready-made post.".to_string()
    };

    let mut concerns = Vec::new();
    if standing_matches.is_empty() {
        concerns.push("No matching Story or Proof Point standing found.".to_string());
    }
    if theme_matches.is_empty() {
        concerns.push("No active or emerging Theme matched this opportunity.".to_string());
    }
    if audience_matches.is_empty() {
        concerns.push("No active Target Context matched this opportunity.".to_string());
    }
    if recent_overlap {
        concerns.push(format!(
            "This resembles a recent Topic: {}.",
            recent_matches[0].label
        ));
    }
    if negative > positive {
        concerns
            .push("Prior discovery feedback suggests this may be a poor personal fit.".to_string());
    }

    let view = DiscoveryOpportunityView {
        opportunity_id: format!("discovery_opportunity_{}", Uuid::now_v7()),
        run_id: run_id.to_string(),
        title,
        summary,
        sources,
        theme_matches,
        standing_matches,
        audience_matches,
        why_now,
        possible_angle,
        concerns,
        feedback_adjustment,
        status: DiscoveryOpportunityStatus::Candidate,
        topic_id: None,
        inspiration_id: None,
        created_at: now.to_string(),
    };
    RankedOpportunity {
        view,
        feature_tokens,
        provider_position,
        feedback_bias,
        recent_overlap,
    }
}

fn compare_ranked(left: &RankedOpportunity, right: &RankedOpportunity) -> Ordering {
    right
        .feedback_bias
        .cmp(&left.feedback_bias)
        .then_with(|| {
            right
                .view
                .standing_matches
                .len()
                .cmp(&left.view.standing_matches.len())
        })
        .then_with(|| {
            right
                .view
                .theme_matches
                .len()
                .cmp(&left.view.theme_matches.len())
        })
        .then_with(|| {
            right
                .view
                .audience_matches
                .len()
                .cmp(&left.view.audience_matches.len())
        })
        .then_with(|| left.recent_overlap.cmp(&right.recent_overlap))
        .then_with(|| right.view.sources.len().cmp(&left.view.sources.len()))
        .then_with(|| left.provider_position.cmp(&right.provider_position))
}

fn persist_opportunity(
    connection: &Connection,
    item: &RankedOpportunity,
    rank_position: usize,
) -> ServiceResult<()> {
    connection.execute(
        "INSERT INTO discovery_opportunities(
           opportunity_id,run_id,title,summary,sources_json,theme_matches_json,standing_matches_json,
           audience_matches_json,why_now,possible_angle,concerns_json,feedback_adjustment,status,
           feature_tokens_json,rank_position,topic_id,inspiration_id,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,NULL,NULL,?16,?16)",
        params![
            &item.view.opportunity_id,
            &item.view.run_id,
            &item.view.title,
            &item.view.summary,
            serde_json::to_string(&item.view.sources)?,
            serde_json::to_string(&item.view.theme_matches)?,
            serde_json::to_string(&item.view.standing_matches)?,
            serde_json::to_string(&item.view.audience_matches)?,
            &item.view.why_now,
            &item.view.possible_angle,
            serde_json::to_string(&item.view.concerns)?,
            &item.view.feedback_adjustment,
            item.view.status.as_str(),
            serde_json::to_string(
                &item.feature_tokens.iter().cloned().collect::<Vec<_>>()
            )?,
            i64::try_from(rank_position).unwrap_or(0),
            &item.view.created_at
        ],
    )?;
    Ok(())
}

fn load_opportunity(
    connection: &Connection,
    opportunity_id: &str,
) -> ServiceResult<DiscoveryOpportunityView> {
    let raw = connection
        .query_row(
            "SELECT opportunity_id,run_id,title,summary,sources_json,theme_matches_json,
                    standing_matches_json,audience_matches_json,why_now,possible_angle,concerns_json,
                    feedback_adjustment,status,topic_id,inspiration_id,created_at
             FROM discovery_opportunities WHERE opportunity_id=?1",
            [opportunity_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, Option<String>>(13)?,
                    row.get::<_, Option<String>>(14)?,
                    row.get::<_, String>(15)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| {
            WorkLoreError::InvalidVault(format!(
                "Discovery opportunity {opportunity_id} was not found."
            ))
        })?;
    Ok(DiscoveryOpportunityView {
        opportunity_id: raw.0,
        run_id: raw.1,
        title: raw.2,
        summary: raw.3,
        sources: serde_json::from_str(&raw.4)?,
        theme_matches: serde_json::from_str(&raw.5)?,
        standing_matches: serde_json::from_str(&raw.6)?,
        audience_matches: serde_json::from_str(&raw.7)?,
        why_now: raw.8,
        possible_angle: raw.9,
        concerns: serde_json::from_str(&raw.10)?,
        feedback_adjustment: raw.11,
        status: DiscoveryOpportunityStatus::parse(&raw.12).ok_or_else(|| {
            WorkLoreError::InvalidVault(format!("Unknown discovery opportunity status {}.", raw.12))
        })?,
        topic_id: raw.13,
        inspiration_id: raw.14,
        created_at: raw.15,
    })
}

fn discovery_query(connection: &Connection, focus: &str) -> ServiceResult<String> {
    let focus = focus.trim();
    if !focus.is_empty() {
        return Ok(focus.to_string());
    }
    let mut statement = connection.prepare(
        "SELECT name FROM themes WHERE status!='retired'
         ORDER BY CASE status WHEN 'active' THEN 0 ELSE 1 END,updated_at DESC LIMIT 3",
    )?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    if names.is_empty() {
        return Err(discovery_error(
            "invalid_request",
            "Enter a discovery focus or create an active Theme before scanning.",
        ));
    }
    Ok(names.join(" OR "))
}

fn load_theme_signals(connection: &Connection) -> ServiceResult<Vec<SignalRecord>> {
    load_signals(
        connection,
        "SELECT theme_id,name,description FROM themes WHERE status!='retired'",
        "theme",
    )
}

fn load_standing_signals(connection: &Connection) -> ServiceResult<Vec<SignalRecord>> {
    let mut out = load_signals(
        connection,
        "SELECT story_id,title,summary FROM stories WHERE lifecycle_status!='archived'",
        "story",
    )?;
    out.extend(load_signals(
        connection,
        "SELECT proof_id,statement,'' FROM proof_points WHERE status!='retired'",
        "proof_point",
    )?);
    Ok(out)
}

fn load_audience_signals(connection: &Connection) -> ServiceResult<Vec<SignalRecord>> {
    load_signals(
        connection,
        "SELECT target_id,title,summary FROM target_contexts WHERE status='active'",
        "target_context",
    )
}

fn load_recent_topic_signals(connection: &Connection) -> ServiceResult<Vec<SignalRecord>> {
    let sql = format!(
        "SELECT topic_id,title,summary FROM topic_candidates WHERE status!='retired'
         ORDER BY updated_at DESC LIMIT {}",
        RECENT_TOPIC_LIMIT
    );
    load_signals(connection, &sql, "topic")
}

fn load_signals(
    connection: &Connection,
    sql: &str,
    kind: &'static str,
) -> ServiceResult<Vec<SignalRecord>> {
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map([], |row| {
        let record_id = row.get::<_, String>(0)?;
        let label = row.get::<_, String>(1)?;
        let detail = row.get::<_, String>(2)?;
        let tokens = tokens(&format!("{label} {detail}"));
        Ok(SignalRecord {
            kind,
            record_id,
            label,
            detail,
            tokens,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn matching_signals(
    signals: &[SignalRecord],
    opportunity_tokens: &BTreeSet<String>,
) -> Vec<DiscoveryMatchView> {
    signals
        .iter()
        .filter(|signal| signal_matches(&signal.tokens, opportunity_tokens))
        .take(5)
        .map(|signal| DiscoveryMatchView {
            kind: signal.kind.to_string(),
            record_id: signal.record_id.clone(),
            label: signal.label.clone(),
            detail: signal.detail.clone(),
        })
        .collect()
}

fn signal_matches(left: &BTreeSet<String>, right: &BTreeSet<String>) -> bool {
    let common = left.intersection(right).count();
    common >= 2
        || (common == 1
            && left.len() <= 3
            && left
                .intersection(right)
                .next()
                .is_some_and(|value| value.len() >= 5))
}

fn load_prior_feedback(connection: &Connection) -> ServiceResult<Vec<PriorFeedback>> {
    let mut statement = connection.prepare(
        "SELECT f.feedback_id,f.verdict,f.normalized_signals_json
         FROM discovery_feedback f
         ORDER BY f.created_at DESC LIMIT ?1",
    )?;
    let rows = statement.query_map(
        [i64::try_from(MAX_FEEDBACK_EXAMPLES).unwrap_or(40)],
        |row| {
            let normalized = row.get::<_, String>(2)?;
            let value = serde_json::from_str::<Value>(&normalized).unwrap_or_else(|_| json!({}));
            let mut parsed = value
                .get("opportunityTokens")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<BTreeSet<_>>();
            parsed.extend(
                value
                    .get("noteTokens")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned),
            );
            Ok(PriorFeedback {
                feedback_id: row.get(0)?,
                verdict: row.get(1)?,
                tokens: parsed,
            })
        },
    )?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn token_similarity(left: &BTreeSet<String>, right: &BTreeSet<String>) -> bool {
    let common = left.intersection(right).count();
    if common < 2 {
        return false;
    }
    let union = left.union(right).count();
    union > 0 && common * 5 >= union
}

fn cluster_sources(results: Vec<DiscoverySourceView>) -> Vec<Vec<DiscoverySourceView>> {
    let mut clusters: Vec<(BTreeSet<String>, Vec<DiscoverySourceView>)> = Vec::new();
    for source in results {
        let source_tokens = tokens(&source.title);
        if let Some((cluster_tokens, cluster)) = clusters
            .iter_mut()
            .find(|(tokens, _)| title_similarity(tokens, &source_tokens))
        {
            cluster.extend(std::iter::once(source));
            cluster_tokens.extend(source_tokens);
        } else {
            clusters.push((source_tokens, vec![source]));
        }
    }
    clusters.into_iter().map(|(_, items)| items).collect()
}

fn title_similarity(left: &BTreeSet<String>, right: &BTreeSet<String>) -> bool {
    let common = left.intersection(right).count();
    let union = left.union(right).count();
    common >= 2 && union > 0 && common * 100 >= union * 55
}

fn tokens(value: &str) -> BTreeSet<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(str::to_ascii_lowercase)
        .filter(|token| token.len() >= 3 && !STOPWORDS.contains(&token.as_str()))
        .take(80)
        .collect()
}

const STOPWORDS: &[&str] = &[
    "about", "after", "also", "and", "are", "been", "before", "being", "but", "can", "could",
    "from", "has", "have", "into", "its", "more", "new", "not", "now", "our", "over", "said",
    "that", "the", "their", "them", "they", "this", "through", "today", "using", "was", "were",
    "what", "when", "where", "which", "while", "will", "with", "would", "you", "your",
];

fn normalize_reasons(values: Vec<String>) -> Vec<String> {
    const ALLOWED: &[&str] = &[
        "strong_personal_angle",
        "weak_standing",
        "too_generic",
        "audience_mismatch",
        "overdone",
        "bad_timing",
        "source_quality",
        "not_interested",
        "other",
    ];
    let mut normalized = values
        .into_iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| ALLOWED.contains(&value.as_str()))
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn required_text(value: &str, label: &str) -> ServiceResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(WorkLoreError::InvalidVault(format!(
            "{label} cannot be empty."
        )))
    } else {
        Ok(trimmed.to_string())
    }
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    let connection = Connection::open(canonical_store::database_path(vault_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

fn audit(
    connection: &Connection,
    event_type: &str,
    record_type: &str,
    record_id: &str,
    details: Value,
) -> ServiceResult<()> {
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,?2,?3,?4,'user',?5,?6)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            event_type,
            record_type,
            record_id,
            details.to_string(),
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

fn discovery_error(code: &'static str, message: impl Into<String>) -> WorkLoreError {
    WorkLoreError::ProviderOperation {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clustering_collapses_substantially_similar_headlines() {
        let results = vec![
            DiscoverySourceView {
                title: "OpenAI launches new agent workflow tools".into(),
                url: "https://example.com/a".into(),
                description: String::new(),
                age: None,
                domain: "example.com".into(),
            },
            DiscoverySourceView {
                title: "New OpenAI agent workflow tools launch".into(),
                url: "https://example.org/b".into(),
                description: String::new(),
                age: None,
                domain: "example.org".into(),
            },
            DiscoverySourceView {
                title: "Remote work policy changes at a bank".into(),
                url: "https://example.net/c".into(),
                description: String::new(),
                age: None,
                domain: "example.net".into(),
            },
        ];
        let clustered = cluster_sources(results);
        assert_eq!(clustered.len(), 2);
        assert_eq!(clustered[0].len(), 2);
    }

    #[test]
    fn feedback_similarity_requires_more_than_one_generic_token() {
        assert!(token_similarity(
            &tokens("agent workflow product management"),
            &tokens("product management agent workflow")
        ));
        assert!(!token_similarity(
            &tokens("agent workflow product management"),
            &tokens("product leadership healthcare")
        ));
    }

    #[test]
    fn normalized_feedback_reasons_are_bounded_and_deduplicated() {
        assert_eq!(
            normalize_reasons(vec![
                "too_generic".into(),
                "TOO_GENERIC".into(),
                "unregistered_reason".into()
            ]),
            vec!["too_generic".to_string()]
        );
    }
}
