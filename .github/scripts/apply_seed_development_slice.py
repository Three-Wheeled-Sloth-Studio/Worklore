from pathlib import Path
from textwrap import dedent


def replace_once(path: str, before: str, after: str) -> None:
    file_path = Path(path)
    text = file_path.read_text(encoding="utf-8")
    if after in text:
        return
    count = text.count(before)
    if count != 1:
        raise SystemExit(f"Expected one match in {path}, found {count}: {before[:120]!r}")
    file_path.write_text(text.replace(before, after, 1), encoding="utf-8")


def replace_between(path: str, start: str, end: str, replacement: str) -> None:
    file_path = Path(path)
    text = file_path.read_text(encoding="utf-8")
    start_index = text.find(start)
    if start_index < 0:
        raise SystemExit(f"Start marker missing in {path}: {start!r}")
    end_index = text.find(end, start_index)
    if end_index < 0:
        raise SystemExit(f"End marker missing in {path}: {end!r}")
    file_path.write_text(text[:start_index] + replacement + text[end_index:], encoding="utf-8")


def write(path: str, content: str) -> None:
    file_path = Path(path)
    file_path.parent.mkdir(parents=True, exist_ok=True)
    file_path.write_text(dedent(content).lstrip(), encoding="utf-8")


write(
    "src-tauri/src/services/guided_development_rules.rs",
    r'''
    use crate::{
        domain::interviews::{
            AnswerClassification, CompletenessStatus, InterviewResponseAction,
            SubmitInterviewResponseRequest, TurnType,
        },
        error::{ServiceResult, WorkLoreError},
    };

    const DEFAULT_FIELDS: &[&str] = &[
        "problem_or_opportunity",
        "actions_and_decisions",
        "metrics_and_evidence",
        "stakeholders",
        "tools_and_systems",
        "constraints",
        "lessons_learned",
    ];

    pub fn response_values(
        request: &SubmitInterviewResponseRequest,
    ) -> ServiceResult<(
        TurnType,
        Option<AnswerClassification>,
        CompletenessStatus,
        f32,
        String,
    )> {
        match request.action {
            InterviewResponseAction::Answer => {
                let text = request.text.trim();
                if text.is_empty() {
                    return Err(WorkLoreError::InvalidInterviewAction(
                        "Answer text cannot be empty.".to_string(),
                    ));
                }
                let classification = request
                    .classification
                    .unwrap_or(AnswerClassification::ConfirmedFact);
                let confidence = match classification {
                    AnswerClassification::ConfirmedFact => 1.0,
                    AnswerClassification::UserEstimate => 0.75,
                    AnswerClassification::Uncertain => 0.45,
                    AnswerClassification::NotApplicable => 1.0,
                };
                let status = if classification == AnswerClassification::NotApplicable {
                    CompletenessStatus::NotApplicable
                } else if text.len() < 20 {
                    CompletenessStatus::Partial
                } else {
                    CompletenessStatus::Sufficient
                };
                Ok((
                    TurnType::Answer,
                    Some(classification),
                    status,
                    confidence,
                    text.to_string(),
                ))
            }
            InterviewResponseAction::Skip => Ok((
                TurnType::Skip,
                None,
                CompletenessStatus::Partial,
                0.0,
                "Skipped for now.".to_string(),
            )),
            InterviewResponseAction::DoNotRemember => Ok((
                TurnType::DoNotRemember,
                Some(AnswerClassification::Uncertain),
                CompletenessStatus::Partial,
                0.0,
                "User does not remember.".to_string(),
            )),
        }
    }

    pub fn question_for_field(field: &str, claim: &str) -> String {
        let claim_reference = compact_claim(claim);
        match field {
            "problem_or_opportunity" => format!(
                "Before {claim_reference}, what was broken, inefficient, risky, or newly possible?"
            ),
            "actions_and_decisions" => format!(
                "For {claim_reference}, what did you personally change or decide? Include the important sequence, not just the team result."
            ),
            "metrics_and_evidence" => format!(
                "How do you know {claim_reference} worked? Share exact metrics, reasonable estimates, or concrete evidence."
            ),
            "stakeholders" => format!(
                "Who used, approved, resisted, or benefited from {claim_reference}?"
            ),
            "tools_and_systems" => format!(
                "Which tools, systems, data sources, or technical constraints shaped {claim_reference}?"
            ),
            "constraints" => format!(
                "What constraints made {claim_reference} difficult or limited your options?"
            ),
            "lessons_learned" => format!(
                "What did {claim_reference} teach you, or what would you do differently now?"
            ),
            _ => format!("What important detail is missing from {claim_reference}?"),
        }
    }

    pub fn normalized_fields(candidate_fields: &[String]) -> Vec<String> {
        let mut fields = candidate_fields
            .iter()
            .filter(|field| !field.trim().is_empty())
            .cloned()
            .collect::<Vec<_>>();
        for default_field in DEFAULT_FIELDS {
            if !fields.iter().any(|field| field == default_field) {
                fields.push((*default_field).to_string());
            }
        }
        fields.sort();
        fields.dedup();
        fields
    }

    fn compact_claim(claim: &str) -> String {
        let trimmed = claim.trim().trim_end_matches('.');
        if trimmed.chars().count() <= 90 {
            format!("\"{trimmed}\"")
        } else {
            let shortened = trimmed.chars().take(87).collect::<String>();
            format!("\"{shortened}...\"")
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn question_map_keeps_work_grounded_in_the_claim() {
            let question = question_for_field(
                "metrics_and_evidence",
                "Reduced complaint investigation time from months to hours.",
            );
            assert!(question.contains("Reduced complaint investigation time"));
            assert!(question.contains("exact metrics"));
        }

        #[test]
        fn normalized_fields_keep_required_interview_coverage() {
            let fields = normalized_fields(&["metrics_and_evidence".to_string()]);
            assert!(fields.contains(&"constraints".to_string()));
            assert_eq!(
                fields
                    .iter()
                    .filter(|field| field.as_str() == "metrics_and_evidence")
                    .count(),
                1
            );
        }
    }
    ''',
)

write(
    "src-tauri/src/services/seed_development_service.rs",
    r'''
    use std::{collections::HashSet, fs, path::Path, time::Duration};

    use chrono::Utc;
    use rusqlite::{params, Connection, OptionalExtension};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use uuid::Uuid;

    use crate::{
        domain::interviews::{
            CompletenessItem, CompletenessStatus, InterviewSession, InterviewStatus,
            InterviewTurn, SubmitInterviewResponseRequest, TurnActor, TurnType,
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
                if existing.active_question_turn_id.is_none()
                    && next_unasked_field(&existing).is_some()
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
                "Only an active Story Seed development interview can accept a response."
                    .to_string(),
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

    fn insert_session(
        connection: &Connection,
        session: &SeedDevelopmentSession,
    ) -> ServiceResult<()> {
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

    fn persist_session(
        connection: &Connection,
        session: &SeedDevelopmentSession,
    ) -> ServiceResult<()> {
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
            current_target_field: current_question
                .and_then(|turn| turn.target_fields.first().cloned()),
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
                interviews::{AnswerClassification, InterviewResponseAction},
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
            assert_eq!(user_turns[0].answer_classification, Some(AnswerClassification::ConfirmedFact));
            assert_eq!(user_turns[1].turn_type, TurnType::Skip);
            assert_eq!(user_turns[2].turn_type, TurnType::DoNotRemember);
            assert_eq!(user_turns[2].answer_classification, Some(AnswerClassification::Uncertain));
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
            let repeated = create_story_from_seed_development(&path, &development.interview_id).unwrap();
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
            let seed = canonical_store::create_story_seed(&path, "Legacy adapter seed", "Legacy claim").unwrap();
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
            let count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM interview_sessions WHERE target_seed_id=?1",
                [&seed.seed_id], |row| row.get(0)).unwrap();
            assert_eq!(count, 2);
            drop(connection);
            fs::remove_dir_all(path).unwrap();
        }
    }
    ''',
)

write(
    "src-tauri/src/commands/development.rs",
    r'''
    use std::path::PathBuf;

    use crate::{
        domain::interviews::SubmitInterviewResponseRequest,
        error::{CommandError, CommandResult},
        services::seed_development_service::{
            self, SeedDevelopmentStoryResult, SeedDevelopmentSummary,
        },
    };

    #[tauri::command]
    pub fn start_story_seed_development(
        vault_path: String,
        seed_id: String,
    ) -> CommandResult<SeedDevelopmentSummary> {
        seed_development_service::start_story_seed_development(
            &PathBuf::from(vault_path),
            &seed_id,
        )
        .map_err(CommandError::from)
    }

    #[tauri::command]
    pub fn submit_story_seed_development_response(
        vault_path: String,
        request: SubmitInterviewResponseRequest,
    ) -> CommandResult<SeedDevelopmentSummary> {
        seed_development_service::submit_story_seed_development_response(
            &PathBuf::from(vault_path),
            request,
        )
        .map_err(CommandError::from)
    }

    #[tauri::command]
    pub fn create_story_from_seed_development(
        vault_path: String,
        interview_id: String,
    ) -> CommandResult<SeedDevelopmentStoryResult> {
        seed_development_service::create_story_from_seed_development(
            &PathBuf::from(vault_path),
            &interview_id,
        )
        .map_err(CommandError::from)
    }
    ''',
)

replace_once(
    "src-tauri/src/services/mod.rs",
    "pub mod entity_scan;\npub mod interview_service;\n",
    "pub mod entity_scan;\npub mod guided_development_rules;\npub mod interview_service;\n",
)
replace_once(
    "src-tauri/src/services/mod.rs",
    "pub mod role_service;\npub mod source_service;\n",
    "pub mod role_service;\npub mod seed_development_service;\npub mod source_service;\n",
)
replace_once(
    "src-tauri/src/commands/mod.rs",
    "pub mod capture;\npub mod interviews;\n",
    "pub mod capture;\npub mod development;\npub mod interviews;\n",
)

interview_service = "src-tauri/src/services/interview_service.rs"
replace_once(
    interview_service,
    "        interviews::{\n            AnswerClassification, CompletenessItem, CompletenessStatus, InterviewResponseAction,\n            InterviewSession, InterviewStatus, InterviewSummary, InterviewTurn,\n            SubmitInterviewResponseRequest, TurnActor, TurnType,\n        },\n    },\n    error::{ServiceResult, WorkLoreError},\n    io_utils::{read_json, write_json_atomic},\n};\n\nconst DEFAULT_FIELDS: &[&str] = &[\n    \"problem_or_opportunity\",\n    \"actions_and_decisions\",\n    \"metrics_and_evidence\",\n    \"stakeholders\",\n    \"tools_and_systems\",\n    \"constraints\",\n    \"lessons_learned\",\n];\n",
    "        interviews::{\n            CompletenessItem, CompletenessStatus, InterviewSession, InterviewStatus,\n            InterviewSummary, InterviewTurn, SubmitInterviewResponseRequest, TurnActor, TurnType,\n        },\n    },\n    error::{ServiceResult, WorkLoreError},\n    io_utils::{read_json, write_json_atomic},\n    services::guided_development_rules,\n};\n",
)
replace_once(
    interview_service,
    "    let fields = normalized_fields(&candidate.missing_fields);",
    "    let fields = guided_development_rules::normalized_fields(&candidate.missing_fields);",
)
replace_once(
    interview_service,
    "        response_values(&request)?;",
    "        guided_development_rules::response_values(&request)?;",
)
replace_once(
    interview_service,
    "    let question = question_for_field(&field, claim);",
    "    let question = guided_development_rules::question_for_field(&field, claim);",
)
replace_between(interview_service, "fn response_values(\n", "fn add_next_question(", "fn add_next_question(")
replace_between(interview_service, "fn question_for_field(", "fn summarize(\n", "fn summarize(\n")
replace_once(
    interview_service,
    "        let question = question_for_field(\n",
    "        let question = guided_development_rules::question_for_field(\n",
)
replace_once(
    interview_service,
    "        let fields = normalized_fields(&[\"metrics_and_evidence\".to_string()]);",
    "        let fields = guided_development_rules::normalized_fields(&[\"metrics_and_evidence\".to_string()]);",
)

lib_path = "src-tauri/src/lib.rs"
replace_once(
    lib_path,
    "    capture::{\n        classify_capture_source, create_capture_source, get_capture_source,\n        list_unclassified_captures,\n    },\n    interviews::{",
    "    capture::{\n        classify_capture_source, create_capture_source, get_capture_source,\n        list_unclassified_captures,\n    },\n    development::{\n        create_story_from_seed_development, start_story_seed_development,\n        submit_story_seed_development_response,\n    },\n    interviews::{",
)
replace_once(
    lib_path,
    "            classify_capture_source,\n            open_vault,",
    "            classify_capture_source,\n            start_story_seed_development,\n            submit_story_seed_development_response,\n            create_story_from_seed_development,\n            open_vault,",
)

types_path = "src/domain/types.ts"
replace_once(
    types_path,
    "export interface SubmitInterviewResponseRequest {\n",
    "export interface StorySeedDevelopmentSummary {\n  interviewId: string;\n  seedId: string;\n  seedTitle: string;\n  seedSummary: string;\n  storyId: string | null;\n  status: InterviewStatus;\n  currentQuestion: string | null;\n  currentTargetField: string | null;\n  completedFieldCount: number;\n  totalFieldCount: number;\n  lastUpdatedAt: string;\n}\n\nexport interface StorySeedDevelopmentStoryResult {\n  development: StorySeedDevelopmentSummary;\n  storyId: string;\n  created: boolean;\n}\n\nexport interface SubmitInterviewResponseRequest {\n",
)

api_path = "src/lib/workloreApi.ts"
replace_once(
    api_path,
    "  StoryStatus,\n  StorySummary,\n",
    "  StorySeedDevelopmentStoryResult,\n  StorySeedDevelopmentSummary,\n  StoryStatus,\n  StorySummary,\n",
)
replace_once(
    api_path,
    "export async function updateCloudIdentifierMode(\n",
    "export async function startStorySeedDevelopment(\n  vaultPath: string,\n  seedId: string,\n): Promise<StorySeedDevelopmentSummary> {\n  return invoke<StorySeedDevelopmentSummary>(\"start_story_seed_development\", {\n    vaultPath,\n    seedId,\n  });\n}\n\nexport async function submitStorySeedDevelopmentResponse(\n  vaultPath: string,\n  request: SubmitInterviewResponseRequest,\n): Promise<StorySeedDevelopmentSummary> {\n  return invoke<StorySeedDevelopmentSummary>(\"submit_story_seed_development_response\", {\n    vaultPath,\n    request,\n  });\n}\n\nexport async function createStoryFromSeedDevelopment(\n  vaultPath: string,\n  interviewId: string,\n): Promise<StorySeedDevelopmentStoryResult> {\n  return invoke<StorySeedDevelopmentStoryResult>(\"create_story_from_seed_development\", {\n    vaultPath,\n    interviewId,\n  });\n}\n\nexport async function updateCloudIdentifierMode(\n",
)

write(
    "src/components/SeedDevelopmentPanel.tsx",
    r'''
    import { useEffect, useState } from "react";
    import type {
      AnswerClassification,
      InterviewResponseAction,
      StorySeedDevelopmentSummary,
    } from "../domain/types";
    import { errorMessage } from "../domain/types";
    import {
      createStoryFromSeedDevelopment,
      startStorySeedDevelopment,
      submitStorySeedDevelopmentResponse,
    } from "../lib/workloreApi";
    import "../seed-development.css";

    interface SeedDevelopmentPanelProps {
      vaultPath: string;
      seedId: string;
      onClose: () => void;
    }

    export function SeedDevelopmentPanel({
      vaultPath,
      seedId,
      onClose,
    }: SeedDevelopmentPanelProps) {
      const [development, setDevelopment] = useState<StorySeedDevelopmentSummary | null>(null);
      const [answer, setAnswer] = useState("");
      const [classification, setClassification] =
        useState<AnswerClassification>("confirmed_fact");
      const [busy, setBusy] = useState(false);
      const [notice, setNotice] = useState<string | null>(null);
      const [error, setError] = useState<string | null>(null);

      useEffect(() => {
        let cancelled = false;
        setBusy(true);
        setError(null);
        void startStorySeedDevelopment(vaultPath, seedId)
          .then((result) => {
            if (!cancelled) {
              setDevelopment(result);
            }
          })
          .catch((caught) => {
            if (!cancelled) {
              setError(errorMessage(caught));
            }
          })
          .finally(() => {
            if (!cancelled) {
              setBusy(false);
            }
          });
        return () => {
          cancelled = true;
        };
      }, [vaultPath, seedId]);

      useEffect(() => {
        setAnswer("");
        setClassification("confirmed_fact");
      }, [development?.interviewId, development?.currentQuestion]);

      async function submit(action: InterviewResponseAction) {
        if (!development) {
          return;
        }
        setBusy(true);
        setNotice(null);
        setError(null);
        try {
          const updated = await submitStorySeedDevelopmentResponse(vaultPath, {
            interviewId: development.interviewId,
            action,
            text: action === "answer" ? answer : "",
            classification: action === "answer" ? classification : null,
          });
          setDevelopment(updated);
          setNotice(
            updated.status === "ready_for_synthesis"
              ? "Guided development pass complete. You can now create the developing Story."
              : "Saved locally. Here is the next useful question.",
          );
        } catch (caught) {
          setError(errorMessage(caught));
        } finally {
          setBusy(false);
        }
      }

      async function createStory() {
        if (!development) {
          return;
        }
        setBusy(true);
        setNotice(null);
        setError(null);
        try {
          const result = await createStoryFromSeedDevelopment(
            vaultPath,
            development.interviewId,
          );
          setDevelopment(result.development);
          setNotice(
            result.created
              ? `Developing Story created as ${result.storyId}.`
              : `This seed is already linked to ${result.storyId}.`,
          );
        } catch (caught) {
          setError(errorMessage(caught));
        } finally {
          setBusy(false);
        }
      }

      if (!development) {
        return (
          <div className="seed-development-card">
            <div className="seed-development-header">
              <div>
                <p className="eyebrow">Develop story</p>
                <h3>Opening Story Seed...</h3>
              </div>
              <button className="quiet-button compact" onClick={onClose}>Close</button>
            </div>
            {error ? <p className="capture-error">{error}</p> : null}
          </div>
        );
      }

      const ready = development.status === "ready_for_synthesis";
      const completed = development.status === "completed";

      return (
        <div className="seed-development-card" aria-live="polite">
          <div className="seed-development-header">
            <div>
              <p className="eyebrow">Develop story</p>
              <h3>{development.seedTitle}</h3>
              <p>{development.seedSummary}</p>
            </div>
            <button className="quiet-button compact" onClick={onClose}>Close</button>
          </div>

          <div className="seed-development-progress">
            <span>{development.completedFieldCount}/{development.totalFieldCount} evidence-rich fields</span>
            <span>{development.interviewId}</span>
          </div>

          {completed ? (
            <div className="seed-development-ready">
              <strong>Developing Story created</strong>
              <p>{development.storyId}</p>
              <p className="quiet-copy">
                The Story keeps explicit lineage to this Story Seed and its classified interview answers.
              </p>
            </div>
          ) : ready ? (
            <div className="seed-development-ready">
              <strong>Guided pass complete</strong>
              <p>
                Create a local developing Story from this seed and the attributed answers. No provider call is required.
              </p>
              <button
                className="primary-button compact"
                disabled={busy}
                onClick={() => void createStory()}
              >
                Create developing story
              </button>
            </div>
          ) : (
            <div className="seed-development-question">
              <span>{humanizeField(development.currentTargetField ?? "story detail")}</span>
              <h4>{development.currentQuestion}</h4>
              <textarea
                aria-label="Story development answer"
                value={answer}
                onChange={(event) => setAnswer(event.target.value)}
                placeholder="Answer naturally. Preserve uncertainty rather than polishing over it."
                rows={5}
              />
              <div className="seed-development-controls">
                <label>
                  <span>Evidence level</span>
                  <select
                    value={classification}
                    onChange={(event) =>
                      setClassification(event.target.value as AnswerClassification)
                    }
                  >
                    <option value="confirmed_fact">Confirmed fact</option>
                    <option value="user_estimate">Reasonable estimate</option>
                    <option value="uncertain">Uncertain memory</option>
                    <option value="not_applicable">Not applicable</option>
                  </select>
                </label>
                <div className="seed-development-actions">
                  <button
                    className="primary-button compact"
                    disabled={busy || answer.trim().length === 0}
                    onClick={() => void submit("answer")}
                  >
                    Save and continue
                  </button>
                  <button
                    className="secondary-button compact"
                    disabled={busy}
                    onClick={() => void submit("do_not_remember")}
                  >
                    I do not remember
                  </button>
                  <button
                    className="text-button"
                    disabled={busy}
                    onClick={() => void submit("skip")}
                  >
                    Skip for now
                  </button>
                </div>
              </div>
            </div>
          )}

          <p className="seed-development-boundary">
            This direct Story Seed workflow stays local. The legacy manual-AI export is not reused until its privacy preflight is generalized for canonical seed/story targets.
          </p>
          {busy ? <p>Saving...</p> : null}
          {notice ? <p className="capture-notice">{notice}</p> : null}
          {error ? <p className="capture-error">{error}</p> : null}
        </div>
      );
    }

    function humanizeField(value: string): string {
      return value.replaceAll("_", " ");
    }
    ''',
)

write(
    "src/seed-development.css",
    r'''
    .seed-development-card {
      margin-top: 1rem;
      padding: 1rem;
      border: 1px solid var(--border, #d8d8d8);
      border-radius: 0.75rem;
      background: var(--surface, #fff);
    }

    .seed-development-header,
    .seed-development-progress,
    .seed-development-controls,
    .seed-development-actions {
      display: flex;
      gap: 0.75rem;
      align-items: center;
      justify-content: space-between;
    }

    .seed-development-header h3,
    .seed-development-question h4 {
      margin: 0.2rem 0 0.4rem;
    }

    .seed-development-progress {
      margin: 0.75rem 0;
      font-size: 0.85rem;
      opacity: 0.75;
      flex-wrap: wrap;
    }

    .seed-development-question textarea {
      width: 100%;
      box-sizing: border-box;
      margin: 0.75rem 0;
    }

    .seed-development-controls {
      align-items: flex-end;
      flex-wrap: wrap;
    }

    .seed-development-controls label {
      display: grid;
      gap: 0.3rem;
    }

    .seed-development-actions {
      justify-content: flex-start;
      flex-wrap: wrap;
    }

    .seed-development-ready {
      display: grid;
      gap: 0.5rem;
      padding: 0.9rem;
      border-radius: 0.6rem;
      background: rgba(127, 127, 127, 0.08);
    }

    .seed-development-boundary {
      margin: 0.9rem 0 0;
      font-size: 0.85rem;
      opacity: 0.72;
    }

    @media (max-width: 760px) {
      .seed-development-header,
      .seed-development-controls {
        align-items: stretch;
        flex-direction: column;
      }
    }
    ''',
)

capture_path = "src/components/CapturePanel.tsx"
replace_once(
    capture_path,
    'import { useEffect, useState } from "react";\n',
    'import { useEffect, useState } from "react";\nimport { SeedDevelopmentPanel } from "./SeedDevelopmentPanel";\n',
)
replace_once(
    capture_path,
    '  const [error, setError] = useState<string | null>(null);\n',
    '  const [error, setError] = useState<string | null>(null);\n  const [developmentSeedId, setDevelopmentSeedId] = useState<string | null>(null);\n',
)
replace_once(
    capture_path,
    '    setError(null);\n    void refreshRecent();\n',
    '    setError(null);\n    setDevelopmentSeedId(null);\n    void refreshRecent();\n',
)
replace_once(
    capture_path,
    '  return (\n    <section className="workspace-panel capture-panel" aria-labelledby="capture-heading">\n',
    '  const storySeedClassification = saved?.classifications.find(\n    (classification) => classification.role === "story_seed",\n  );\n\n  return (\n    <section className="workspace-panel capture-panel" aria-labelledby="capture-heading">\n',
)
replace_once(
    capture_path,
    '            <p className="capture-source-only">\n              Leave it alone to keep this as Source-only material. Writing samples do not become Voice Evidence here.\n            </p>\n',
    '            <p className="capture-source-only">\n              Leave it alone to keep this as Source-only material. Writing samples do not become Voice Evidence here.\n            </p>\n            {storySeedClassification ? (\n              <button\n                className="primary-button compact"\n                disabled={busy !== null}\n                onClick={() => setDevelopmentSeedId(storySeedClassification.targetId)}\n              >\n                Develop this story seed\n              </button>\n            ) : null}\n',
)
replace_once(
    capture_path,
    '      <div className="capture-feedback" aria-live="polite">\n',
    '      {developmentSeedId ? (\n        <SeedDevelopmentPanel\n          vaultPath={vaultPath}\n          seedId={developmentSeedId}\n          onClose={() => setDevelopmentSeedId(null)}\n        />\n      ) : null}\n\n      <div className="capture-feedback" aria-live="polite">\n',
)

print("Applied bounded Story Seed development slice")
