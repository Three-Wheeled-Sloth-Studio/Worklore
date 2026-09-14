use std::{path::Path, time::Duration};

use rusqlite::Connection;

use crate::{
    domain::posts::{PostRecordView, PostStatus},
    error::{ServiceResult, WorkLoreError},
    services::canonical_store,
};

pub fn list_posts(vault_path: &Path) -> ServiceResult<Vec<PostRecordView>> {
    canonical_store::initialize(vault_path)?;
    let connection = open_connection(vault_path)?;
    let mut statement = connection.prepare(
        "SELECT post_id,title,status,current_revision_id,final_approved_revision_id,
         approved_at,created_at,updated_at,revision
         FROM posts ORDER BY updated_at DESC,post_id DESC",
    )?;
    let rows = statement.query_map([], |row| {
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
    })?;

    rows.map(|row| {
        let raw = row?;
        let post_id = raw.0.clone();
        post_from_raw(raw, &post_id)
    })
    .collect()
}

type PostRaw = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    String,
    u32,
);

fn post_from_raw(raw: PostRaw, post_id: &str) -> ServiceResult<PostRecordView> {
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
    use uuid::Uuid;

    fn vault() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("worklore-post-catalog-{}", Uuid::now_v7()));
        vault_service::create_vault(&path, "Post Catalog Test").expect("create vault");
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
    fn list_posts_reopens_current_status_and_revision_identity() {
        let path = vault();
        let working = post_lineage_service::create_post(
            &path,
            user_post("Working post", "A working draft."),
        )
        .unwrap();
        let final_post = post_lineage_service::create_post(
            &path,
            user_post("Approved post", "An approved draft."),
        )
        .unwrap();
        let approved = post_lineage_service::approve_revision(
            &path,
            ApprovePostRevisionRequest {
                post_id: final_post.post.post_id.clone(),
                revision_id: final_post.post.current_revision_id.clone(),
            },
        )
        .unwrap();

        canonical_store::initialize(&path).unwrap();
        let posts = list_posts(&path).unwrap();
        assert_eq!(posts.len(), 2);
        let reopened_working = posts
            .iter()
            .find(|post| post.post_id == working.post.post_id)
            .expect("working post listed");
        assert_eq!(reopened_working.status, PostStatus::Working);
        assert_eq!(
            reopened_working.current_revision_id,
            working.post.current_revision_id
        );
        let reopened_approved = posts
            .iter()
            .find(|post| post.post_id == approved.post.post_id)
            .expect("approved post listed");
        assert_eq!(reopened_approved.status, PostStatus::FinalApproved);
        assert_eq!(
            reopened_approved.final_approved_revision_id,
            approved.post.final_approved_revision_id
        );
        fs::remove_dir_all(path).unwrap();
    }
}
