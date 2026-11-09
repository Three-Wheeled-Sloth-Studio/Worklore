use std::{collections::HashSet, fs, path::Path, time::Duration};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    domain::interviews::{
        CompletenessItem, CompletenessStatus, InterviewStatus, InterviewTurn,
        SubmitInterviewResponseRequest, TurnActor, TurnType,
    },
    error::{ServiceResult, WorkLoreError},
    services::{canonical_store, guided_development_rules},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedDevelopmentSession {
    schema_version: u32,
    interview_id: String,
    target_seed_id: String,
    target_story_id: Option<String>,
    status: InterviewStatus,
    question_round: u32,
    questions_in_current_round: u8,
    completeness: Vec<CompletenessItem>,
    turns: Vec<InterviewTurn>,
    active_question_turn_id: Option<String>,
    created_at: String,
    updated_at: String,
    revision: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedDevelopmentSummary {
    pub interview_id: String,
    pub seed_id: String,
    pub seed_title: String,
    pub seed_summary: String,
    pub story_id: Option<String>,
    pub status: InterviewStatus,
    pub current_question: Option<String>,
    pub current_target_field: Option<String>,
    pub completed_field_count: usize,
    pub total_field_count: usize,
    pub last_updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedDevelopmentStoryResult {
    pub development: SeedDevelopmentSummary,
    pub story_id: String,
    pub created: bool,
}

pub fn start_story_seed_development(
    vault_path: &Path,
    seed_id: &str,
) -> ServiceResult<SeedDevelopmentSummary> {
    canonical_store::initialize(vault_path)?;
    let seed = canonical_store::load_story_seed(vault_path, seed_id)?;
    let connection = open_connection(vault_path)?;

    if let Some(mut existing) = find_seed_session(&connection, seed_id)? {
        if existing.status == InterviewStatus::Paused
            || existing.status == InterviewStatus::NotStarted
        {
            existing.status = InterviewStatus::Active;
            if existing.active_question_turn_id.is_none() && next_unasked_field(&existing).is_some()
            {
                add_next_question(&mut existing, &seed.summary);
            }
            existing.updated_at = Utc::now().to_rfc3339();
            existing.revision += 1;
            persist_session(&connection, &existing)?;
        }
        return summarize(&existing, &seed.title, &seed.summary);
    }

    let now = Utc::now().to_rfc3339();
    let fields = guided_development_rules::normalized_fields(&[]);
    let mut session = SeedDevelopmentSession {
        schema_version: 1,
        interview_id: format!("interview_{}", Uuid::now_v7()),
        target_seed_id: seed.seed_id.clone(),
        target_story_id: None,
        status: InterviewStatus::Active,
        question_round: 1,
        questions_in_current_round: 0,
        completeness: fields
            .into_iter()
            .map(|field| CompletenessItem {
                field,
                status: CompletenessStatus::Missing,
                confidence: 0.0,
                notes: String::new(),
            })
            .collect(),
        turns: Vec::new(),
        active_question_turn_id: None,
        created_at: now.clone(),
        updated_at: now.clone(),
        revision: 1,
    };
    add_next_question(&mut session, &seed.summary);
    insert_session(&connection, &session)?;
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,'story_seed_development_started','interview',?2,'user',?3,?4)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            &session.interview_id,
            json!({"seedId": seed_id}).to_string(),
            &now
        ],
    )?;
    summarize(&session, &seed.title, &seed.summary)
}

pub fn submit_story_seed_development_response(
    vault_path: &Path,
    request: SubmitInterviewResponseRequest,
) -> ServiceResult<SeedDevelopmentSummary> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut session = load_session(&connection, &request.interview_id)?;
    if session.status != InterviewStatus::Active {
        return Err(WorkLoreError::InvalidInterviewAction(
            "Only an active Story Seed development interview can accept a response.".to_string(),
        ));
    }
    let seed = canonical_store::load_story_seed(vault_path, &session.target_seed_id)?;
    let active_question_id = session.active_question_turn_id.clone().ok_or_else(|| {
        WorkLoreError::InvalidInterviewAction("No active question exists.".to_string())
    })?;
    let target_field = session
        .turns
        .iter()
        .find(|turn| turn.turn_id == active_question_id)
        .and_then(|turn| turn.target_fields.first())
        .cloned()
        .ok_or_else(|| {
            WorkLoreError::InvalidInterviewAction(
                "The active question does not identify a target field.".to_string(),
            )
        })?;

    let now = Utc::now().to_rfc3339();
    let (turn_type, classification, completeness_status, confidence, answer_text) =
        guided_development_rules::response_values(&request)?;
    session.turns.push(InterviewTurn {
        turn_id: format!("turn_{}", Uuid::now_v7()),
        actor: TurnActor::User,
        turn_type,
        text: answer_text.clone(),
        target_fields: vec![target_field.clone()],
        answer_classification: classification,
        provider_run_id: None,
        created_at: now.clone(),
    });
    if let Some(item) = session
        .completeness
        .iter_mut()
        .find(|item| item.field == target_field)
    {
        item.status = completeness_status;
        item.confidence = confidence;
        item.notes = answer_text;
    }
    session.active_question_turn_id = None;
    session.updated_at = now;
    session.revision += 1;

    if next_unasked_field(&session).is_some() {
        if session.questions_in_current_round >= 3 {
            session.question_round += 1;
            session.questions_in_current_round = 0;
        }
        add_next_question(&mut session, &seed.summary);
    } else {
        session.status = InterviewStatus::ReadyForSynthesis;
    }
    persist_session(&connection, &session)?;
    summarize(&session, &seed.title, &seed.summary)
}

pub fn create_story_from_seed_development(
    vault_path: &Path,
    interview_id: &str,
) -> ServiceResult<SeedDevelopmentStoryResult> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    let mut session = load_session(&connection, interview_id)?;
    let seed = canonical_store::load_story_seed(vault_path, &session.target_seed_id)?;

    if let Some(story_id) = session.target_story_id.clone() {
        let _ = canonical_store::load_story(vault_path, &story_id)?;
        return Ok(SeedDevelopmentStoryResult {
            development: summarize(&session, &seed.title, &seed.summary)?,
            story_id,
            created: false,
        });
    }
    if session.status != InterviewStatus::ReadyForSynthesis {
        return Err(WorkLoreError::InvalidInterviewAction(
            "Finish the guided Story Seed interview before creating the developing Story."
                .to_string(),
        ));
    }

    let now = Utc::now().to_rfc3339();
    let story_id = format!("story_{}", Uuid::now_v7());
    let answers = session
        .turns
        .iter()
        .filter(|turn| turn.actor == TurnActor::User)
        .map(|turn| {
            json!({
                "turnId": &turn.turn_id,
                "turnType": turn.turn_type,
                "targetFields": &turn.target_fields,
                "text": &turn.text,
                "answerClassification": turn.answer_classification,
                "createdAt": &turn.created_at,
                "interviewId": &session.interview_id,
                "seedId": &session.target_seed_id
            })
        })
        .collect::<Vec<_>>();
    let content = json!({
        "summary": &seed.summary,
        "seedId": &seed.seed_id,
        "developmentInterviewId": &session.interview_id,
        "developmentAnswers": answers
    });
    let provenance = json!({
        "creationActor": "user",
        "storySeedId": &seed.seed_id,
        "interviewId": &session.interview_id
    });

    let tx = connection.transaction()?;
    tx.execute(
        "INSERT INTO stories(story_id,title,summary,lifecycle_status,maturity,story_type,content_json,
         disclosure_json,privacy_json,tags_json,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,'active','developing','other',?4,'{}','{}','[]',?5,?6,?6,1)",
        params![
            &story_id,
            &seed.title,
            &seed.summary,
            content.to_string(),
            provenance.to_string(),
            &now
        ],
    )?;
    insert_relationship(
        &tx,
        "story_seed",
        &seed.seed_id,
        "seed_story_lineage",
        "story",
        &story_id,
        &now,
    )?;
    insert_relationship(
        &tx,
        "interview",
        &session.interview_id,
        "interview_story",
        "story",
        &story_id,
        &now,
    )?;
    tx.execute(
        "UPDATE story_seeds SET status='converted',updated_at=?2,revision=revision+1 WHERE seed_id=?1",
        params![&seed.seed_id, &now],
    )?;
    session.target_story_id = Some(story_id.clone());
    session.status = InterviewStatus::Completed;
    session.updated_at = now.clone();
    session.revision += 1;
    tx.execute(
        "UPDATE interview_sessions SET target_story_id=?2,status=?3,payload_json=?4,updated_at=?5,revision=?6
         WHERE interview_id=?1",
        params![
            &session.interview_id,
            &story_id,
            status_text(session.status),
            serde_json::to_string(&session)?,
            &now,
            session.revision
        ],
    )?;
    tx.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,'story_created_from_seed','story',?2,'user',?3,?4)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            &story_id,
            json!({"seedId": &seed.seed_id, "interviewId": &session.interview_id}).to_string(),
            &now
        ],
    )?;
    tx.commit()?;

    Ok(SeedDevelopmentStoryResult {
        development: summarize(&session, &seed.title, &seed.summary)?,
        story_id,
        created: true,
    })
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    fs::create_dir_all(vault_path.join("data"))?;
    let connection = Connection::open(canonical_store::database_path(vault_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

fn find_seed_session(
    connection: &Connection,
    seed_id: &str,
) -> ServiceResult<Option<SeedDevelopmentSession>> {
    let mut statement = connection.prepare(
        "SELECT payload_json FROM interview_sessions
         WHERE target_seed_id=?1 AND status <> 'abandoned'
         ORDER BY updated_at DESC",
    )?;
    let rows = statement.query_map([seed_id], |row| row.get::<_, String>(0))?;
    for row in rows {
        let payload = row?;
        if let Ok(session) = serde_json::from_str::<SeedDevelopmentSession>(&payload) {
            return Ok(Some(session));
        }
    }
    Ok(None)
}

fn load_session(
    connection: &Connection,
    interview_id: &str,
) -> ServiceResult<SeedDevelopmentSession> {
    let payload = connection
        .query_row(
            "SELECT payload_json FROM interview_sessions WHERE interview_id=?1 AND target_seed_id IS NOT NULL",
            [interview_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or(WorkLoreError::InterviewNotFound)?;
    serde_json::from_str(&payload).map_err(WorkLoreError::from)
}

fn insert_session(connection: &Connection, session: &SeedDevelopmentSession) -> ServiceResult<()> {
    connection.execute(
        "INSERT INTO interview_sessions(interview_id,target_seed_id,target_story_id,status,payload_json,created_at,updated_at,revision)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            &session.interview_id,
            &session.target_seed_id,
            &session.target_story_id,
            status_text(session.status),
            serde_json::to_string(session)?,
            &session.created_at,
            &session.updated_at,
            session.revision
        ],
    )?;
    Ok(())
}

fn persist_session(connection: &Connection, session: &SeedDevelopmentSession) -> ServiceResult<()> {
    connection.execute(
        "UPDATE interview_sessions SET target_story_id=?2,status=?3,payload_json=?4,updated_at=?5,revision=?6
         WHERE interview_id=?1",
        params![
            &session.interview_id,
            &session.target_story_id,
            status_text(session.status),
            serde_json::to_string(session)?,
            &session.updated_at,
            session.revision
        ],
    )?;
    Ok(())
}

fn add_next_question(session: &mut SeedDevelopmentSession, context: &str) {
    let Some(field) = next_unasked_field(session) else {
        return;
    };
    let now = Utc::now().to_rfc3339();
    let turn_id = format!("turn_{}", Uuid::now_v7());
    session.turns.push(InterviewTurn {
        turn_id: turn_id.clone(),
        actor: TurnActor::Agent,
        turn_type: TurnType::Question,
        text: guided_development_rules::question_for_field(&field, context),
        target_fields: vec![field],
        answer_classification: None,
        provider_run_id: None,
        created_at: now.clone(),
    });
    session.active_question_turn_id = Some(turn_id);
    session.questions_in_current_round += 1;
    session.updated_at = now;
}

fn next_unasked_field(session: &SeedDevelopmentSession) -> Option<String> {
    let asked = session
        .turns
        .iter()
        .filter(|turn| turn.turn_type == TurnType::Question)
        .flat_map(|turn| turn.target_fields.iter().cloned())
        .collect::<HashSet<_>>();
    session
        .completeness
        .iter()
        .find(|item| !asked.contains(&item.field))
        .map(|item| item.field.clone())
}

fn summarize(
    session: &SeedDevelopmentSession,
    seed_title: &str,
    seed_summary: &str,
) -> ServiceResult<SeedDevelopmentSummary> {
    let current_question = session
        .active_question_turn_id
        .as_ref()
        .and_then(|turn_id| session.turns.iter().find(|turn| &turn.turn_id == turn_id));
    let completed_field_count = session
        .completeness
        .iter()
        .filter(|item| {
            matches!(
                item.status,
                CompletenessStatus::Sufficient | CompletenessStatus::NotApplicable
            )
        })
        .count();
    Ok(SeedDevelopmentSummary {
        interview_id: session.interview_id.clone(),
        seed_id: session.target_seed_id.clone(),
        seed_title: seed_title.to_string(),
        seed_summary: seed_summary.to_string(),
        story_id: session.target_story_id.clone(),
        status: session.status,
        current_question: current_question.map(|turn| turn.text.clone()),
        current_target_field: current_question.and_then(|turn| turn.target_fields.first().cloned()),
        completed_field_count,
        total_field_count: session.completeness.len(),
        last_updated_at: session.updated_at.clone(),
    })
}

fn insert_relationship(
    tx: &rusqlite::Transaction<'_>,
    from_type: &str,
    from_id: &str,
    relationship_type: &str,
    to_type: &str,
    to_id: &str,
    now: &str,
) -> ServiceResult<()> {
    tx.execute(
        "INSERT OR IGNORE INTO record_relationships(relationship_id,from_type,from_id,relationship_type,to_type,to_id,
         provenance_json,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            format!("relationship_{}", Uuid::now_v7()),
            from_type,
            from_id,
            relationship_type,
            to_type,
            to_id,
            json!({"creationActor":"user"}).to_string(),
            now
        ],
    )?;
    Ok(())
}

fn status_text(status: InterviewStatus) -> &'static str {
    match status {
        InterviewStatus::NotStarted => "not_started",
        InterviewStatus::Active => "active",
        InterviewStatus::Paused => "paused",
        InterviewStatus::ReadyForSynthesis => "ready_for_synthesis",
        InterviewStatus::Completed => "completed",
        InterviewStatus::Abandoned => "abandoned",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            interviews::{AnswerClassification, InterviewResponseAction, InterviewSession},
            models::SourceType,
        },
        services::{capture_service, vault_service},
    };

    fn vault() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-seed-dev-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Seed Development Test").expect("create vault");
        path
    }

    fn captured_seed(path: &Path) -> String {
        let capture = capture_service::create_capture_source(
            path,
            "Led a difficult launch and changed the release process after finding a recurring delivery risk.",
            SourceType::Other,
        )
        .expect("capture");
        capture_service::classify_capture_source(
            path,
            &capture.source_id,
            capture_service::CaptureRole::StorySeed,
        )
        .expect("classify")
        .target_id
    }

    #[test]
    fn direct_seed_starts_reopens_and_reuses_one_canonical_interview() {
        let path = vault();
        let seed_id = captured_seed(&path);
        let first = start_story_seed_development(&path, &seed_id).unwrap();
        assert_eq!(first.seed_id, seed_id);
        assert!(first
            .current_question
            .as_deref()
            .unwrap_or_default()
            .contains("Led a difficult launch"));

        canonical_store::initialize_and_migrate(&path).unwrap();
        let reopened = start_story_seed_development(&path, &seed_id).unwrap();
        assert_eq!(first.interview_id, reopened.interview_id);

        let connection = open_connection(&path).unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM interview_sessions WHERE target_seed_id=?1",
                [&seed_id],
                |row| row.get(0),
            )
            .unwrap();
        let payload: String = connection
            .query_row(
                "SELECT payload_json FROM interview_sessions WHERE interview_id=?1",
                [&first.interview_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        assert!(!payload.contains("candidateIds"));
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn direct_seed_preserves_answer_skip_and_memory_semantics_without_provider_runs() {
        let path = vault();
        let seed_id = captured_seed(&path);
        let mut development = start_story_seed_development(&path, &seed_id).unwrap();
        development = submit_story_seed_development_response(
            &path,
            SubmitInterviewResponseRequest {
                interview_id: development.interview_id.clone(),
                action: InterviewResponseAction::Answer,
                text: "I personally changed the launch gate and release review sequence.".into(),
                classification: Some(AnswerClassification::ConfirmedFact),
            },
        )
        .unwrap();
        development = submit_story_seed_development_response(
            &path,
            SubmitInterviewResponseRequest {
                interview_id: development.interview_id.clone(),
                action: InterviewResponseAction::Skip,
                text: String::new(),
                classification: None,
            },
        )
        .unwrap();
        let interview_id = development.interview_id.clone();
        let _ = submit_story_seed_development_response(
            &path,
            SubmitInterviewResponseRequest {
                interview_id: interview_id.clone(),
                action: InterviewResponseAction::DoNotRemember,
                text: String::new(),
                classification: None,
            },
        )
        .unwrap();

        let connection = open_connection(&path).unwrap();
        let session = load_session(&connection, &interview_id).unwrap();
        let user_turns = session
            .turns
            .iter()
            .filter(|turn| turn.actor == TurnActor::User)
            .collect::<Vec<_>>();
        assert_eq!(user_turns.len(), 3);
        assert_eq!(
            user_turns[0].answer_classification,
            Some(AnswerClassification::ConfirmedFact)
        );
        assert_eq!(user_turns[1].turn_type, TurnType::Skip);
        assert_eq!(user_turns[2].turn_type, TurnType::DoNotRemember);
        assert_eq!(
            user_turns[2].answer_classification,
            Some(AnswerClassification::Uncertain)
        );
        assert!(user_turns.iter().all(|turn| turn.provider_run_id.is_none()));
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn completed_seed_development_creates_one_role_free_story_with_lineage() {
        let path = vault();
        let seed_id = captured_seed(&path);
        let mut development = start_story_seed_development(&path, &seed_id).unwrap();
        let mut answer_number = 0;
        while development.status == InterviewStatus::Active {
            answer_number += 1;
            development = submit_story_seed_development_response(
                &path,
                SubmitInterviewResponseRequest {
                    interview_id: development.interview_id.clone(),
                    action: InterviewResponseAction::Answer,
                    text: format!(
                        "Synthetic supporting detail number {answer_number} with enough context to be sufficient."
                    ),
                    classification: Some(if answer_number % 2 == 0 {
                        AnswerClassification::UserEstimate
                    } else {
                        AnswerClassification::ConfirmedFact
                    }),
                },
            )
            .unwrap();
        }
        assert_eq!(development.status, InterviewStatus::ReadyForSynthesis);

        let created = create_story_from_seed_development(&path, &development.interview_id).unwrap();
        assert!(created.created);
        assert_eq!(created.development.status, InterviewStatus::Completed);
        let repeated =
            create_story_from_seed_development(&path, &development.interview_id).unwrap();
        assert!(!repeated.created);
        assert_eq!(created.story_id, repeated.story_id);

        let story = canonical_store::load_story(&path, &created.story_id).unwrap();
        assert_eq!(story.maturity, "developing");
        let connection = open_connection(&path).unwrap();
        let lineage: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM record_relationships
                 WHERE from_type='story_seed' AND from_id=?1 AND relationship_type='seed_story_lineage'
                   AND to_type='story' AND to_id=?2",
                params![&seed_id, &created.story_id],
                |row| row.get(0),
            )
            .unwrap();
        let roles: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM record_relationships
                 WHERE from_type='story' AND from_id=?1 AND relationship_type='story_role'",
                [&created.story_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(lineage, 1);
        assert_eq!(roles, 0);
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn legacy_interview_payload_is_not_mistaken_for_direct_seed_development() {
        let path = vault();
        let seed = canonical_store::create_story_seed(&path, "Legacy adapter seed", "Legacy claim")
            .unwrap();
        let now = Utc::now().to_rfc3339();
        let legacy = InterviewSession {
            schema_version: 1,
            interview_id: "interview_legacy_synthetic".into(),
            candidate_ids: vec!["candidate_synthetic".into()],
            story_id: None,
            status: InterviewStatus::Active,
            question_round: 1,
            questions_in_current_round: 0,
            completeness: Vec::new(),
            turns: Vec::new(),
            active_question_turn_id: None,
            created_at: now.clone(),
            updated_at: now.clone(),
            revision: 1,
        };
        let connection = open_connection(&path).unwrap();
        connection.execute(
            "INSERT INTO interview_sessions(interview_id,target_seed_id,status,payload_json,created_at,updated_at,revision)
             VALUES (?1,?2,'active',?3,?4,?4,1)",
            params![&legacy.interview_id,&seed.seed_id,serde_json::to_string(&legacy).unwrap(),&now],
        ).unwrap();
        drop(connection);

        let direct = start_story_seed_development(&path, &seed.seed_id).unwrap();
        assert_ne!(direct.interview_id, legacy.interview_id);
        let connection = open_connection(&path).unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM interview_sessions WHERE target_seed_id=?1",
                [&seed.seed_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }
}
