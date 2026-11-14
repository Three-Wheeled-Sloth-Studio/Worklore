use std::{collections::BTreeMap, path::Path, time::Duration};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    domain::{
        feedback::{
            FeedbackInsightView, FeedbackSnapshotView, ManualPublicationView,
            MarkPostPublishedRequest, PerformanceMetrics, PerformanceRecordView,
            RecordPostPerformanceRequest,
        },
        posts::PostStatus,
    },
    error::{ServiceResult, WorkLoreError},
    services::canonical_store,
};

const PUBLICATION_RECORD_TYPE: &str = "post_publication";
const PERFORMANCE_RECORD_TYPE: &str = "performance_record";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublicationDetails {
    post_id: String,
    revision_id: String,
    platform: String,
    published_at: String,
    publication_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PerformanceDetails {
    publication_id: String,
    metrics: PerformanceMetrics,
    notes: String,
}

pub fn mark_post_published(
    vault_path: &Path,
    request: MarkPostPublishedRequest,
) -> ServiceResult<ManualPublicationView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let post_id = required(&request.post_id, "Post ID")?;
    let revision_id = required(&request.revision_id, "Revision ID")?;
    let platform = required(&request.platform, "Publication platform")?;
    let published_at = required(&request.published_at, "Publication time")?;
    let publication_url = optional_trimmed(request.publication_url.as_deref());

    let (post_title, status, final_revision_id) = load_publishable_post(&connection, &post_id)?;
    if status != PostStatus::FinalApproved {
        return Err(WorkLoreError::InvalidVault(
            "Only a final-approved Post can be marked published.".to_string(),
        ));
    }
    if final_revision_id.as_deref() != Some(revision_id.as_str()) {
        return Err(WorkLoreError::InvalidVault(
            "Publication must reference the exact final-approved Revision.".to_string(),
        ));
    }

    let existing = load_publications(&connection)?;
    if let Some(publication) = existing.into_iter().find(|publication| {
        publication.post_id == post_id
            && publication.revision_id == revision_id
            && publication.platform.eq_ignore_ascii_case(&platform)
    }) {
        return Ok(publication);
    }

    let publication_id = format!("publication_{}", Uuid::now_v7());
    let recorded_at = Utc::now().to_rfc3339();
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,'post_marked_published',?2,?3,'user',?4,?5)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            PUBLICATION_RECORD_TYPE,
            &publication_id,
            json!({
                "postId": post_id,
                "revisionId": revision_id,
                "platform": platform,
                "publishedAt": published_at,
                "publicationUrl": publication_url
            })
            .to_string(),
            &recorded_at
        ],
    )?;

    Ok(ManualPublicationView {
        publication_id,
        post_id,
        post_title,
        revision_id,
        platform,
        published_at,
        publication_url,
        recorded_at,
    })
}

pub fn record_post_performance(
    vault_path: &Path,
    request: RecordPostPerformanceRequest,
) -> ServiceResult<PerformanceRecordView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let publication_id = required(&request.publication_id, "Publication ID")?;
    if !publication_exists(&connection, &publication_id)? {
        return Err(WorkLoreError::InvalidVault(format!(
            "Publication {publication_id} was not found."
        )));
    }

    let performance_id = format!("performance_{}", Uuid::now_v7());
    let recorded_at = Utc::now().to_rfc3339();
    let notes = request.notes.trim().to_string();
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,'post_performance_recorded',?2,?3,'user',?4,?5)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            PERFORMANCE_RECORD_TYPE,
            &performance_id,
            json!({
                "publicationId": publication_id,
                "metrics": request.metrics,
                "notes": notes
            })
            .to_string(),
            &recorded_at
        ],
    )?;

    Ok(PerformanceRecordView {
        performance_id,
        publication_id,
        metrics: request.metrics,
        notes,
        recorded_at,
    })
}

pub fn get_feedback_snapshot(vault_path: &Path) -> ServiceResult<FeedbackSnapshotView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let publications = load_publications(&connection)?;
    let all_performance = load_performance(&connection)?;
    let mut latest_by_publication = BTreeMap::new();
    for record in all_performance {
        latest_by_publication.insert(record.publication_id.clone(), record);
    }
    let latest_performance = latest_by_publication.into_values().collect::<Vec<_>>();
    let insights = derive_insights(&publications, &latest_performance);
    Ok(FeedbackSnapshotView {
        publications,
        latest_performance,
        insights,
    })
}

fn load_publishable_post(
    connection: &Connection,
    post_id: &str,
) -> ServiceResult<(String, PostStatus, Option<String>)> {
    let raw = connection
        .query_row(
            "SELECT title,status,final_approved_revision_id FROM posts WHERE post_id=?1",
            [post_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| WorkLoreError::InvalidVault(format!("Post {post_id} was not found.")))?;
    Ok((
        raw.0,
        PostStatus::parse(&raw.1).map_err(WorkLoreError::InvalidVault)?,
        raw.2,
    ))
}

fn publication_exists(connection: &Connection, publication_id: &str) -> ServiceResult<bool> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM audit_events WHERE record_type=?1 AND record_id=?2)",
        params![PUBLICATION_RECORD_TYPE, publication_id],
        |row| row.get(0),
    )?)
}

fn load_publications(connection: &Connection) -> ServiceResult<Vec<ManualPublicationView>> {
    let mut statement = connection.prepare(
        "SELECT record_id,details_json,occurred_at FROM audit_events
         WHERE record_type=?1 ORDER BY occurred_at ASC,audit_id ASC",
    )?;
    let rows = statement.query_map([PUBLICATION_RECORD_TYPE], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut publications = Vec::new();
    for row in rows {
        let (publication_id, details_json, recorded_at) = row?;
        let details: PublicationDetails = serde_json::from_str(&details_json)?;
        let post_title = connection
            .query_row(
                "SELECT title FROM posts WHERE post_id=?1",
                [&details.post_id],
                |post_row| post_row.get::<_, String>(0),
            )
            .optional()?
            .unwrap_or_else(|| "Unknown Post".to_string());
        publications.push(ManualPublicationView {
            publication_id,
            post_id: details.post_id,
            post_title,
            revision_id: details.revision_id,
            platform: details.platform,
            published_at: details.published_at,
            publication_url: details.publication_url,
            recorded_at,
        });
    }
    Ok(publications)
}

fn load_performance(connection: &Connection) -> ServiceResult<Vec<PerformanceRecordView>> {
    let mut statement = connection.prepare(
        "SELECT record_id,details_json,occurred_at FROM audit_events
         WHERE record_type=?1 ORDER BY occurred_at ASC,audit_id ASC",
    )?;
    let rows = statement.query_map([PERFORMANCE_RECORD_TYPE], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    rows.map(|row| {
        let (performance_id, details_json, recorded_at) = row?;
        let details: PerformanceDetails = serde_json::from_str(&details_json)?;
        Ok(PerformanceRecordView {
            performance_id,
            publication_id: details.publication_id,
            metrics: details.metrics,
            notes: details.notes,
            recorded_at,
        })
    })
    .collect()
}

fn derive_insights(
    publications: &[ManualPublicationView],
    performance: &[PerformanceRecordView],
) -> Vec<FeedbackInsightView> {
    if publications.is_empty() {
        return vec![FeedbackInsightView {
            kind: "baseline".to_string(),
            statement: "No manually published Posts are recorded yet.".to_string(),
            sample_size: 0,
        }];
    }
    if performance.is_empty() {
        return vec![FeedbackInsightView {
            kind: "baseline".to_string(),
            statement: format!(
                "{} published Post(s) are recorded, but none has a performance snapshot yet.",
                publications.len()
            ),
            sample_size: 0,
        }];
    }

    let mut insights = Vec::new();
    let measured = performance.len();
    if measured < 3 {
        insights.push(FeedbackInsightView {
            kind: "sample_warning".to_string(),
            statement: format!(
                "Only {measured} published Post(s) have measurements. Treat this as baseline data, not a reusable content rule."
            ),
            sample_size: measured,
        });
    } else {
        if let Some(best) = best_rate(performance, |metrics| metrics.saves) {
            insights.push(rate_insight(
                "save_rate",
                "save",
                best,
                publications,
                measured,
            ));
        }
        if let Some(best) = best_rate(performance, |metrics| metrics.profile_views) {
            insights.push(rate_insight(
                "profile_view_rate",
                "profile view",
                best,
                publications,
                measured,
            ));
        }
        insights.push(FeedbackInsightView {
            kind: "interpretation_boundary".to_string(),
            statement: "Observed rates are descriptive only. WorkLore is not claiming the Post caused the outcome; test the pattern again before turning it into a rule.".to_string(),
            sample_size: measured,
        });
    }
    insights
}

fn best_rate<F>(performance: &[PerformanceRecordView], metric: F) -> Option<&PerformanceRecordView>
where
    F: Fn(&PerformanceMetrics) -> u64,
{
    performance
        .iter()
        .filter(|record| record.metrics.impressions > 0)
        .max_by(|left, right| {
            let left_score =
                u128::from(metric(&left.metrics)) * u128::from(right.metrics.impressions);
            let right_score =
                u128::from(metric(&right.metrics)) * u128::from(left.metrics.impressions);
            left_score.cmp(&right_score)
        })
}

fn rate_insight(
    kind: &str,
    label: &str,
    record: &PerformanceRecordView,
    publications: &[ManualPublicationView],
    sample_size: usize,
) -> FeedbackInsightView {
    let publication = publications
        .iter()
        .find(|publication| publication.publication_id == record.publication_id);
    let title = publication
        .map(|publication| publication.post_title.as_str())
        .unwrap_or("Unknown Post");
    let count = if label == "save" {
        record.metrics.saves
    } else {
        record.metrics.profile_views
    };
    let rate = count as f64 * 1000.0 / record.metrics.impressions as f64;
    FeedbackInsightView {
        kind: kind.to_string(),
        statement: format!(
            "{title} currently has the highest observed {label} rate at {rate:.1} per 1,000 impressions."
        ),
        sample_size,
    }
}

fn required(value: &str, label: &str) -> ServiceResult<String> {
    let value = value.trim();
    if value.is_empty() {
        Err(WorkLoreError::InvalidVault(format!("{label} is required.")))
    } else {
        Ok(value.to_string())
    }
}

fn optional_trimmed(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    let connection = Connection::open(canonical_store::database_path(vault_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::posts::{
            ApprovePostRevisionRequest, CreatePostRequest, PostRevisionAuthorship,
            PostRevisionOrigin,
        },
        services::{post_lineage_service, vault_service},
    };
    use std::fs;

    fn vault() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-feedback-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Feedback Test").expect("create vault");
        path
    }

    fn user_post(title: &str) -> CreatePostRequest {
        CreatePostRequest {
            title: title.to_string(),
            text: "Synthetic approved text.".to_string(),
            origin: PostRevisionOrigin::User,
            authorship_state: PostRevisionAuthorship::UserAuthored,
            provider_run_id: None,
            provider_id: None,
            model_id: None,
        }
    }

    #[test]
    fn publication_requires_exact_final_approved_revision_and_is_idempotent() {
        let path = vault();
        let post = post_lineage_service::create_post(&path, user_post("Synthetic Post")).unwrap();
        let request = MarkPostPublishedRequest {
            post_id: post.post.post_id.clone(),
            revision_id: post.post.current_revision_id.clone(),
            platform: "linkedin".to_string(),
            published_at: "2026-09-14T12:00:00-04:00".to_string(),
            publication_url: Some("https://example.test/post/1".to_string()),
        };
        assert!(mark_post_published(&path, request.clone()).is_err());
        let approved = post_lineage_service::approve_revision(
            &path,
            ApprovePostRevisionRequest {
                post_id: post.post.post_id.clone(),
                revision_id: post.post.current_revision_id.clone(),
            },
        )
        .unwrap();
        assert_eq!(approved.post.status, PostStatus::FinalApproved);
        let first = mark_post_published(&path, request.clone()).unwrap();
        let duplicate = mark_post_published(&path, request).unwrap();
        assert_eq!(first.publication_id, duplicate.publication_id);
        let snapshot = get_feedback_snapshot(&path).unwrap();
        assert_eq!(snapshot.publications.len(), 1);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn performance_snapshots_are_append_only_and_latest_value_drives_baseline() {
        let path = vault();
        let post = post_lineage_service::create_post(&path, user_post("Measured Post")).unwrap();
        post_lineage_service::approve_revision(
            &path,
            ApprovePostRevisionRequest {
                post_id: post.post.post_id.clone(),
                revision_id: post.post.current_revision_id.clone(),
            },
        )
        .unwrap();
        let publication = mark_post_published(
            &path,
            MarkPostPublishedRequest {
                post_id: post.post.post_id,
                revision_id: post.post.current_revision_id,
                platform: "linkedin".to_string(),
                published_at: "2026-09-14T12:00:00-04:00".to_string(),
                publication_url: None,
            },
        )
        .unwrap();
        record_post_performance(
            &path,
            RecordPostPerformanceRequest {
                publication_id: publication.publication_id.clone(),
                metrics: PerformanceMetrics {
                    impressions: 100,
                    saves: 2,
                    ..PerformanceMetrics::default()
                },
                notes: "First snapshot".to_string(),
            },
        )
        .unwrap();
        let latest = record_post_performance(
            &path,
            RecordPostPerformanceRequest {
                publication_id: publication.publication_id,
                metrics: PerformanceMetrics {
                    impressions: 250,
                    saves: 7,
                    profile_views: 5,
                    ..PerformanceMetrics::default()
                },
                notes: "Later snapshot".to_string(),
            },
        )
        .unwrap();
        let snapshot = get_feedback_snapshot(&path).unwrap();
        assert_eq!(snapshot.latest_performance.len(), 1);
        assert_eq!(
            snapshot.latest_performance[0].performance_id,
            latest.performance_id
        );
        assert_eq!(snapshot.latest_performance[0].metrics.impressions, 250);
        assert_eq!(snapshot.insights[0].kind, "sample_warning");
        fs::remove_dir_all(path).unwrap();
    }
}
