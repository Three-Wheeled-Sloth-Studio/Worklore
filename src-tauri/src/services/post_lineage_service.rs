use std::{path::Path, time::Duration};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    domain::posts::{
        AppendPostRevisionRequest, ApprovePostRevisionRequest, CreatePostRequest, PostLineageView,
        PostRecordView, PostRevisionAuthorship, PostRevisionOrigin, PostRevisionView, PostStatus,
        PostSupportRole, PostSupportingMaterialView,
    },
    error::{ServiceResult, WorkLoreError},
    services::canonical_store,
};

pub fn create_post(vault_path: &Path, request: CreatePostRequest) -> ServiceResult<PostLineageView> {
    canonical_store::initialize(vault_path)?;
    validate_text(&request.text)?;
    validate_revision_metadata(
        request.origin,
        request.authorship_state,
        None,
        request.provider_run_id.as_deref(),
        request.provider_id.as_deref(),
        request.model_id.as_deref(),
    )?;

    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    let now = Utc::now().to_rfc3339();
    let post_id = format!("post_{}", Uuid::now_v7());
    let revision_id = format!("post_revision_{}", Uuid::now_v7());
    let title = normalized_title(&request.title);

    tx.execute(
        "INSERT INTO posts(post_id,title,status,current_revision_id,final_approved_revision_id,
         approved_at,provenance_json,created_at,updated_at,revision)
         VALUES (?1,?2,'working',NULL,NULL,NULL,?3,?4,?4,1)",
        params![
            &post_id,
            &title,
            json!({
                "creationActor": "user",
                "initialRevisionOrigin": request.origin.as_str(),
                "initialAuthorshipState": request.authorship_state.as_str()
            })
            .to_string(),
            &now
        ],
    )?;

    insert_revision(
        &tx,
        &post_id,
        &revision_id,
        1,
        None,
        &request.text,
        request.origin,
        request.authorship_state,
        request.provider_run_id.as_deref(),
        request.provider_id.as_deref(),
        request.model_id.as_deref(),
        &now,
    )?;
    tx.execute(
        "UPDATE posts SET current_revision_id=?1 WHERE post_id=?2",
        params![&revision_id, &post_id],
    )?;
    audit_tx(
        &tx,
        "post_created",
        &post_id,
        json!({
            "revisionId": revision_id,
            "sequence": 1,
            "origin": request.origin.as_str(),
            "authorshipState": request.authorship_state.as_str(),
            "providerRunId": request.provider_run_id,
            "providerId": request.provider_id,
            "modelId": request.model_id
        }),
        &now,
    )?;
    tx.commit()?;
    get_post_lineage(vault_path, &post_id)
}

pub fn append_revision(
    vault_path: &Path,
    request: AppendPostRevisionRequest,
) -> ServiceResult<PostLineageView> {
    canonical_store::initialize(vault_path)?;
    validate_text(&request.text)?;
    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    let post = load_post_tx(&tx, &request.post_id)?;
    if post.status != PostStatus::Working {
        return Err(WorkLoreError::InvalidVault(
            "Final-approved posts cannot be edited without an explicit future reopen workflow."
                .to_string(),
        ));
    }
    let parent = load_revision_tx(&tx, &post.current_revision_id)?;
    if parent.text == request.text {
        return Err(WorkLoreError::InvalidVault(
            "A new post revision must change the text snapshot.".to_string(),
        ));
    }
    validate_revision_metadata(
        request.origin,
        request.authorship_state,
        Some(parent.authorship_state),
        request.provider_run_id.as_deref(),
        request.provider_id.as_deref(),
        request.model_id.as_deref(),
    )?;

    let sequence = parent.sequence.checked_add(1).ok_or_else(|| {
        WorkLoreError::InvalidVault("Post revision sequence overflowed.".to_string())
    })?;
    let revision_id = format!("post_revision_{}", Uuid::now_v7());
    let now = Utc::now().to_rfc3339();
    insert_revision(
        &tx,
        &request.post_id,
        &revision_id,
        sequence,
        Some(&parent.revision_id),
        &request.text,
        request.origin,
        request.authorship_state,
        request.provider_run_id.as_deref(),
        request.provider_id.as_deref(),
        request.model_id.as_deref(),
        &now,
    )?;
    tx.execute(
        "UPDATE posts SET current_revision_id=?1,updated_at=?2,revision=revision+1 WHERE post_id=?3",
        params![&revision_id, &now, &request.post_id],
    )?;
    audit_tx(
        &tx,
        "post_revision_created",
        &request.post_id,
        json!({
            "revisionId": revision_id,
            "parentRevisionId": parent.revision_id,
            "sequence": sequence,
            "origin": request.origin.as_str(),
            "authorshipState": request.authorship_state.as_str(),
            "providerRunId": request.provider_run_id,
            "providerId": request.provider_id,
            "modelId": request.model_id
        }),
        &now,
    )?;
    tx.commit()?;
    get_post_lineage(vault_path, &request.post_id)
}

pub fn approve_revision(
    vault_path: &Path,
    request: ApprovePostRevisionRequest,
) -> ServiceResult<PostLineageView> {
    canonical_store::initialize(vault_path)?;
    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    let post = load_post_tx(&tx, &request.post_id)?;
    if post.status == PostStatus::FinalApproved {
        return Err(WorkLoreError::InvalidVault(
            "This post already has an explicitly approved final revision.".to_string(),
        ));
    }
    let revision = load_revision_tx(&tx, &request.revision_id)?;
    if revision.post_id != request.post_id {
        return Err(WorkLoreError::InvalidVault(
            "The selected revision does not belong to this post.".to_string(),
        ));
    }
    let now = Utc::now().to_rfc3339();
    tx.execute(
        "UPDATE posts SET status='final_approved',current_revision_id=?1,
         final_approved_revision_id=?1,approved_at=?2,updated_at=?2,revision=revision+1
         WHERE post_id=?3",
        params![&revision.revision_id, &now, &request.post_id],
    )?;
    audit_tx(
        &tx,
        "post_final_approved",
        &request.post_id,
        json!({
            "approvedRevisionId": revision.revision_id,
            "sequence": revision.sequence,
            "origin": revision.origin.as_str(),
            "authorshipState": revision.authorship_state.as_str()
        }),
        &now,
    )?;
    tx.commit()?;
    get_post_lineage(vault_path, &request.post_id)
}

pub fn link_supporting_material(
    vault_path: &Path,
    post_id: &str,
    role: PostSupportRole,
    target_id: &str,
) -> ServiceResult<PostLineageView> {
    canonical_store::initialize(vault_path)?;
    let post_id = post_id.trim();
    let target_id = target_id.trim();
    if post_id.is_empty() || target_id.is_empty() {
        return Err(WorkLoreError::InvalidVault(
            "Post and supporting-material IDs are required.".to_string(),
        ));
    }
    let mut connection = open_connection(vault_path)?;
    let tx = connection.transaction()?;
    load_post_tx(&tx, post_id)?;
    ensure_support_target_exists(&tx, role, target_id)?;
    let now = Utc::now().to_rfc3339();
    let relationship_id = format!("relationship_{}", Uuid::now_v7());
    let inserted = tx.execute(
        "INSERT OR IGNORE INTO record_relationships(
         relationship_id,from_type,from_id,relationship_type,to_type,to_id,provenance_json,created_at)
         VALUES (?1,'post',?2,?3,?4,?5,?6,?7)",
        params![
            &relationship_id,
            post_id,
            role.relationship_type(),
            role.target_type(),
            target_id,
            json!({"creationActor":"user","semanticRole":role.relationship_type()}).to_string(),
            &now
        ],
    )?;
    if inserted > 0 {
        audit_tx(
            &tx,
            "post_support_linked",
            post_id,
            json!({
                "relationshipId": relationship_id,
                "role": role.relationship_type(),
                "targetType": role.target_type(),
                "targetId": target_id
            }),
            &now,
        )?;
    }
    tx.commit()?;
    get_post_lineage(vault_path, post_id)
}

pub fn get_post_lineage(vault_path: &Path, post_id: &str) -> ServiceResult<PostLineageView> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let post = load_post(&connection, post_id)?;
    let revisions = load_revisions(&connection, post_id)?;
    let supporting_material = load_supporting_material(&connection, post_id)?;
    Ok(PostLineageView {
        post,
        revisions,
        supporting_material,
    })
}

fn insert_revision(
    tx: &Transaction<'_>,
    post_id: &str,
    revision_id: &str,
    sequence: u32,
    parent_revision_id: Option<&str>,
    text: &str,
    origin: PostRevisionOrigin,
    authorship_state: PostRevisionAuthorship,
    provider_run_id: Option<&str>,
    provider_id: Option<&str>,
    model_id: Option<&str>,
    created_at: &str,
) -> ServiceResult<()> {
    tx.execute(
        "INSERT INTO post_revisions(
         revision_id,post_id,sequence,parent_revision_id,text_snapshot,origin,authorship_state,
         provider_run_id,provider_id,model_id,provenance_json,created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            revision_id,
            post_id,
            sequence,
            parent_revision_id,
            text,
            origin.as_str(),
            authorship_state.as_str(),
            provider_run_id,
            provider_id,
            model_id,
            json!({
                "recordedBy": "user",
                "origin": origin.as_str(),
                "authorshipState": authorship_state.as_str()
            })
            .to_string(),
            created_at
        ],
    )?;
    Ok(())
}

fn validate_text(text: &str) -> ServiceResult<()> {
    if text.trim().is_empty() {
        Err(WorkLoreError::InvalidVault(
            "Post revision text cannot be empty.".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn validate_revision_metadata(
    origin: PostRevisionOrigin,
    authorship: PostRevisionAuthorship,
    parent_authorship: Option<PostRevisionAuthorship>,
    provider_run_id: Option<&str>,
    provider_id: Option<&str>,
    model_id: Option<&str>,
) -> ServiceResult<()> {
    let provider_run_id = nonblank(provider_run_id);
    let provider_id = nonblank(provider_id);
    let model_id = nonblank(model_id);
    match origin {
        PostRevisionOrigin::Model => {
            if authorship != PostRevisionAuthorship::ModelGenerated {
                return Err(WorkLoreError::InvalidVault(
                    "Model-origin revisions must retain model_generated authorship.".to_string(),
                ));
            }
            if provider_id.is_some() != model_id.is_some() {
                return Err(WorkLoreError::InvalidVault(
                    "Model provenance must provide provider and model IDs together.".to_string(),
                ));
            }
            if provider_run_id.is_some() && provider_id.is_none() {
                return Err(WorkLoreError::InvalidVault(
                    "A provider run ID requires provider and model IDs.".to_string(),
                ));
            }
        }
        PostRevisionOrigin::User => {
            if provider_run_id.is_some() || provider_id.is_some() || model_id.is_some() {
                return Err(WorkLoreError::InvalidVault(
                    "User-origin revisions cannot claim provider provenance.".to_string(),
                ));
            }
            let expected = match parent_authorship {
                Some(PostRevisionAuthorship::ModelGenerated | PostRevisionAuthorship::UserEditedModel) => {
                    PostRevisionAuthorship::UserEditedModel
                }
                _ => PostRevisionAuthorship::UserAuthored,
            };
            if authorship != expected {
                return Err(WorkLoreError::InvalidVault(format!(
                    "User-origin revision authorship must be {} for this lineage.",
                    expected.as_str()
                )));
            }
        }
    }
    Ok(())
}

fn nonblank(value: Option<&str>) -> Option<&str> {
    value.filter(|item| !item.trim().is_empty())
}

fn normalized_title(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "Untitled post".to_string()
    } else {
        value.to_string()
    }
}

fn load_post(connection: &Connection, post_id: &str) -> ServiceResult<PostRecordView> {
    connection
        .query_row(
            "SELECT post_id,title,status,current_revision_id,final_approved_revision_id,
             approved_at,created_at,updated_at,revision FROM posts WHERE post_id=?1",
            [post_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, u32>(8)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| post_not_found(post_id))
        .and_then(|raw| post_from_raw(raw, post_id))
}

fn load_post_tx(tx: &Transaction<'_>, post_id: &str) -> ServiceResult<PostRecordView> {
    tx.query_row(
        "SELECT post_id,title,status,current_revision_id,final_approved_revision_id,
         approved_at,created_at,updated_at,revision FROM posts WHERE post_id=?1",
        [post_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, u32>(8)?,
            ))
        },
    )
    .optional()?
    .ok_or_else(|| post_not_found(post_id))
    .and_then(|raw| post_from_raw(raw, post_id))
}

fn post_from_raw(
    raw: (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
        u32,
    ),
    post_id: &str,
) -> ServiceResult<PostRecordView> {
    let current_revision_id = raw.3.ok_or_else(|| {
        WorkLoreError::InvalidVault(format!("Post {post_id} has no current revision."))
    })?;
    Ok(PostRecordView {
        post_id: raw.0,
        title: raw.1,
        status: PostStatus::parse(&raw.2).map_err(WorkLoreError::InvalidVault)?,
        current_revision_id,
        final_approved_revision_id: raw.4,
        approved_at: raw.5,
        created_at: raw.6,
        updated_at: raw.7,
        revision: raw.8,
    })
}

fn load_revision_tx(tx: &Transaction<'_>, revision_id: &str) -> ServiceResult<PostRevisionView> {
    tx.query_row(
        "SELECT revision_id,post_id,sequence,parent_revision_id,text_snapshot,origin,authorship_state,
         provider_run_id,provider_id,model_id,created_at FROM post_revisions WHERE revision_id=?1",
        [revision_id],
        revision_row,
    )
    .optional()?
    .ok_or_else(|| revision_not_found(revision_id))
    .and_then(revision_from_raw)
}

fn load_revisions(connection: &Connection, post_id: &str) -> ServiceResult<Vec<PostRevisionView>> {
    let mut statement = connection.prepare(
        "SELECT revision_id,post_id,sequence,parent_revision_id,text_snapshot,origin,authorship_state,
         provider_run_id,provider_id,model_id,created_at FROM post_revisions
         WHERE post_id=?1 ORDER BY sequence ASC",
    )?;
    let rows = statement.query_map([post_id], revision_row)?;
    rows.map(|row| row.map_err(WorkLoreError::from).and_then(revision_from_raw))
        .collect()
}

type RevisionRaw = (
    String,
    String,
    u32,
    Option<String>,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
);

fn revision_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RevisionRaw> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
        row.get(9)?,
        row.get(10)?,
    ))
}

fn revision_from_raw(raw: RevisionRaw) -> ServiceResult<PostRevisionView> {
    let origin = match raw.5.as_str() {
        "user" => PostRevisionOrigin::User,
        "model" => PostRevisionOrigin::Model,
        value => {
            return Err(WorkLoreError::InvalidVault(format!(
                "Unknown post revision origin {value}."
            )))
        }
    };
    Ok(PostRevisionView {
        revision_id: raw.0,
        post_id: raw.1,
        sequence: raw.2,
        parent_revision_id: raw.3,
        text: raw.4,
        origin,
        authorship_state: PostRevisionAuthorship::parse(&raw.6)
            .map_err(WorkLoreError::InvalidVault)?,
        provider_run_id: raw.7,
        provider_id: raw.8,
        model_id: raw.9,
        created_at: raw.10,
    })
}

fn load_supporting_material(
    connection: &Connection,
    post_id: &str,
) -> ServiceResult<Vec<PostSupportingMaterialView>> {
    let mut statement = connection.prepare(
        "SELECT relationship_id,relationship_type,to_id,created_at
         FROM record_relationships WHERE from_type='post' AND from_id=?1
         ORDER BY created_at,relationship_id",
    )?;
    let rows = statement.query_map([post_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (relationship_id, relationship_type, target_id, created_at) = row?;
        if let Some(role) = PostSupportRole::parse_relationship(&relationship_type) {
            out.push(PostSupportingMaterialView {
                relationship_id,
                role,
                target_id,
                created_at,
            });
        }
    }
    Ok(out)
}

fn ensure_support_target_exists(
    tx: &Transaction<'_>,
    role: PostSupportRole,
    target_id: &str,
) -> ServiceResult<()> {
    let (table, column) = match role {
        PostSupportRole::EvidenceSource => ("sources", "source_id"),
        PostSupportRole::Evidence => ("evidence_records", "evidence_id"),
        PostSupportRole::Story => ("stories", "story_id"),
        PostSupportRole::ProofPoint => ("proof_points", "proof_id"),
        PostSupportRole::Topic => ("topic_candidates", "topic_id"),
        PostSupportRole::Theme => ("themes", "theme_id"),
        PostSupportRole::Inspiration => ("inspirations", "inspiration_id"),
        PostSupportRole::TargetContext => ("target_contexts", "target_id"),
        PostSupportRole::VoiceEvidence => ("voice_evidence", "voice_evidence_id"),
    };
    let sql = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE {column}=?1)");
    let exists: bool = tx.query_row(&sql, [target_id], |row| row.get(0))?;
    if exists {
        Ok(())
    } else {
        Err(WorkLoreError::InvalidVault(format!(
            "Supporting material {target_id} does not exist for role {}.",
            role.relationship_type()
        )))
    }
}

fn audit_tx(
    tx: &Transaction<'_>,
    event_type: &str,
    post_id: &str,
    details: Value,
    occurred_at: &str,
) -> ServiceResult<()> {
    tx.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)
         VALUES (?1,?2,'post',?3,'user',?4,?5)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            event_type,
            post_id,
            details.to_string(),
            occurred_at
        ],
    )?;
    Ok(())
}

fn open_connection(vault_path: &Path) -> ServiceResult<Connection> {
    let connection = Connection::open(canonical_store::database_path(vault_path))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}

fn post_not_found(id: &str) -> WorkLoreError {
    WorkLoreError::InvalidVault(format!("Post {id} was not found."))
}

fn revision_not_found(id: &str) -> WorkLoreError {
    WorkLoreError::InvalidVault(format!("Post Revision {id} was not found."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::models::SourceType,
        services::{capture_service, confidentiality_service, vault_service},
    };
    use std::fs;

    fn vault() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-post-lineage-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Post Lineage Test").expect("create vault");
        path
    }

    fn user_post(title: &str, text: &str) -> CreatePostRequest {
        CreatePostRequest {
            title: title.to_string(),
            text: text.to_string(),
            origin: PostRevisionOrigin::User,
            authorship_state: PostRevisionAuthorship::UserAuthored,
            provider_run_id: None,
            provider_id: None,
            model_id: None,
        }
    }

    #[test]
    fn user_draft_edit_and_exact_final_approval_survive_reopen() {
        let path = vault();
        let created = create_post(&path, user_post("Synthetic post", "First private draft.")).unwrap();
        let first = created.revisions[0].clone();
        assert_eq!(created.revisions.len(), 1);
        assert_eq!(first.sequence, 1);
        assert!(first.parent_revision_id.is_none());

        let edited = append_revision(
            &path,
            AppendPostRevisionRequest {
                post_id: created.post.post_id.clone(),
                text: "Second human-edited draft.".to_string(),
                origin: PostRevisionOrigin::User,
                authorship_state: PostRevisionAuthorship::UserAuthored,
                provider_run_id: None,
                provider_id: None,
                model_id: None,
            },
        )
        .unwrap();
        let second = edited.revisions[1].clone();
        assert_eq!(second.sequence, 2);
        assert_eq!(second.parent_revision_id.as_deref(), Some(first.revision_id.as_str()));
        assert_eq!(edited.revisions[0].text, "First private draft.");

        let approved = approve_revision(
            &path,
            ApprovePostRevisionRequest {
                post_id: created.post.post_id.clone(),
                revision_id: second.revision_id.clone(),
            },
        )
        .unwrap();
        assert_eq!(approved.post.status, PostStatus::FinalApproved);
        assert_eq!(approved.post.final_approved_revision_id.as_deref(), Some(second.revision_id.as_str()));
        assert_eq!(approved.revisions[0].text, "First private draft.");
        assert_eq!(approved.revisions[1].text, "Second human-edited draft.");

        canonical_store::initialize(&path).unwrap();
        let reopened = get_post_lineage(&path, &created.post.post_id).unwrap();
        assert_eq!(reopened, approved);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn model_ancestor_stays_model_and_human_edit_is_user_edited_model() {
        let path = vault();
        let created = create_post(
            &path,
            CreatePostRequest {
                title: "Synthetic model lineage".to_string(),
                text: "Synthetic model-origin draft.".to_string(),
                origin: PostRevisionOrigin::Model,
                authorship_state: PostRevisionAuthorship::ModelGenerated,
                provider_run_id: Some("run_synthetic".to_string()),
                provider_id: Some("synthetic_provider".to_string()),
                model_id: Some("synthetic_model".to_string()),
            },
        )
        .unwrap();
        let model_revision = created.revisions[0].clone();

        let mislabeled = append_revision(
            &path,
            AppendPostRevisionRequest {
                post_id: created.post.post_id.clone(),
                text: "A human edit that must not erase model ancestry.".to_string(),
                origin: PostRevisionOrigin::User,
                authorship_state: PostRevisionAuthorship::UserAuthored,
                provider_run_id: None,
                provider_id: None,
                model_id: None,
            },
        );
        assert!(mislabeled.is_err());

        let edited = append_revision(
            &path,
            AppendPostRevisionRequest {
                post_id: created.post.post_id.clone(),
                text: "A human edit that retains model ancestry.".to_string(),
                origin: PostRevisionOrigin::User,
                authorship_state: PostRevisionAuthorship::UserEditedModel,
                provider_run_id: None,
                provider_id: None,
                model_id: None,
            },
        )
        .unwrap();
        assert_eq!(edited.revisions[0], model_revision);
        assert_eq!(edited.revisions[1].authorship_state, PostRevisionAuthorship::UserEditedModel);
        assert_eq!(edited.revisions[1].parent_revision_id.as_deref(), Some(model_revision.revision_id.as_str()));

        let connection = open_connection(&path).unwrap();
        let voice_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM voice_evidence", [], |row| row.get(0))
            .unwrap();
        assert_eq!(voice_count, 0);
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn support_links_keep_semantic_roles_distinct() {
        let path = vault();
        let post = create_post(&path, user_post("Links", "A supported draft.")).unwrap();
        let source = capture_service::create_capture_source(
            &path,
            "Synthetic source evidence.",
            SourceType::Note,
        )
        .unwrap();
        let evidence_id = canonical_store::create_evidence(&path, "Synthetic fact.").unwrap();
        let inspiration_id = canonical_store::create_inspiration(&path, "Synthetic article").unwrap();
        let target_id = canonical_store::create_target_context(&path, "job_description", "Synthetic role").unwrap();

        link_supporting_material(&path, &post.post.post_id, PostSupportRole::EvidenceSource, &source.source_id).unwrap();
        link_supporting_material(&path, &post.post.post_id, PostSupportRole::Evidence, &evidence_id).unwrap();
        link_supporting_material(&path, &post.post.post_id, PostSupportRole::Inspiration, &inspiration_id).unwrap();
        let linked = link_supporting_material(&path, &post.post.post_id, PostSupportRole::TargetContext, &target_id).unwrap();
        let roles = linked.supporting_material.iter().map(|item| item.role).collect::<Vec<_>>();
        assert!(roles.contains(&PostSupportRole::EvidenceSource));
        assert!(roles.contains(&PostSupportRole::Evidence));
        assert!(roles.contains(&PostSupportRole::Inspiration));
        assert!(roles.contains(&PostSupportRole::TargetContext));
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn schema_upgrade_does_not_invent_post_history() {
        let path = vault();
        canonical_store::create_story(&path, "Existing story", "Pre-lineage content").unwrap();
        canonical_store::initialize(&path).unwrap();
        let connection = open_connection(&path).unwrap();
        let post_count: i64 = connection.query_row("SELECT COUNT(*) FROM posts", [], |row| row.get(0)).unwrap();
        let revision_count: i64 = connection.query_row("SELECT COUNT(*) FROM post_revisions", [], |row| row.get(0)).unwrap();
        assert_eq!((post_count, revision_count), (0, 0));
        drop(connection);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn confidentiality_transform_is_derived_and_does_not_overwrite_revision_truth() {
        let path = vault();
        let private_text = "Contact synthetic.person@example.test before publication.";
        let post = create_post(&path, user_post("Private", private_text)).unwrap();
        let transformed = confidentiality_service::transform_for_public_use(
            &path,
            confidentiality_service::ConfidentialityTransformRequest {
                text: private_text.to_string(),
            },
        )
        .unwrap();
        assert_eq!(
            transformed.state,
            confidentiality_service::ConfidentialityState::Blocked
        );
        let reopened = get_post_lineage(&path, &post.post.post_id).unwrap();
        assert_eq!(reopened.revisions[0].text, private_text);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn final_approval_freezes_foundation_until_explicit_reopen_exists() {
        let path = vault();
        let created = create_post(&path, user_post("Approved", "Final candidate.")).unwrap();
        let approved = approve_revision(
            &path,
            ApprovePostRevisionRequest {
                post_id: created.post.post_id.clone(),
                revision_id: created.revisions[0].revision_id.clone(),
            },
        )
        .unwrap();
        assert_eq!(approved.post.status, PostStatus::FinalApproved);
        let result = append_revision(
            &path,
            AppendPostRevisionRequest {
                post_id: created.post.post_id.clone(),
                text: "An edit after approval.".to_string(),
                origin: PostRevisionOrigin::User,
                authorship_state: PostRevisionAuthorship::UserAuthored,
                provider_run_id: None,
                provider_id: None,
                model_id: None,
            },
        );
        assert!(result.is_err());
        fs::remove_dir_all(path).unwrap();
    }
}
