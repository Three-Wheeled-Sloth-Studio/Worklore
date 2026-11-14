use std::path::PathBuf;

use crate::{
    error::{CommandError, CommandResult},
    services::topic_service::{
        self, CreateThemeRequest, CreateTopicRequest, ThemeRecordView, TopicLinkTargetView,
        TopicRecordView, TopicRelationKind, TopicRelationshipMutationResult, UpdateThemeRequest,
        UpdateTopicRequest,
    },
};

#[tauri::command]
pub fn create_topic(
    vault_path: String,
    request: CreateTopicRequest,
) -> CommandResult<TopicRecordView> {
    topic_service::create_topic(&PathBuf::from(vault_path), request).map_err(CommandError::from)
}

#[tauri::command]
pub fn get_topic(vault_path: String, topic_id: String) -> CommandResult<TopicRecordView> {
    topic_service::load_topic(&PathBuf::from(vault_path), &topic_id).map_err(CommandError::from)
}

#[tauri::command]
pub fn list_topics(vault_path: String) -> CommandResult<Vec<TopicRecordView>> {
    topic_service::list_topics(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn update_topic(
    vault_path: String,
    request: UpdateTopicRequest,
) -> CommandResult<TopicRecordView> {
    topic_service::update_topic(&PathBuf::from(vault_path), request).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_theme(
    vault_path: String,
    request: CreateThemeRequest,
) -> CommandResult<ThemeRecordView> {
    topic_service::create_theme(&PathBuf::from(vault_path), request).map_err(CommandError::from)
}

#[tauri::command]
pub fn get_theme(vault_path: String, theme_id: String) -> CommandResult<ThemeRecordView> {
    topic_service::load_theme(&PathBuf::from(vault_path), &theme_id).map_err(CommandError::from)
}

#[tauri::command]
pub fn list_themes(vault_path: String) -> CommandResult<Vec<ThemeRecordView>> {
    topic_service::list_themes(&PathBuf::from(vault_path)).map_err(CommandError::from)
}

#[tauri::command]
pub fn update_theme(
    vault_path: String,
    request: UpdateThemeRequest,
) -> CommandResult<ThemeRecordView> {
    topic_service::update_theme(&PathBuf::from(vault_path), request).map_err(CommandError::from)
}

#[tauri::command]
pub fn add_topic_relationship(
    vault_path: String,
    topic_id: String,
    relation_kind: TopicRelationKind,
    target_id: String,
) -> CommandResult<TopicRelationshipMutationResult> {
    topic_service::add_topic_relationship(
        &PathBuf::from(vault_path),
        &topic_id,
        relation_kind,
        &target_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn remove_topic_relationship(
    vault_path: String,
    topic_id: String,
    relation_kind: TopicRelationKind,
    target_id: String,
) -> CommandResult<TopicRelationshipMutationResult> {
    topic_service::remove_topic_relationship(
        &PathBuf::from(vault_path),
        &topic_id,
        relation_kind,
        &target_id,
    )
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn list_topic_link_targets(
    vault_path: String,
    relation_kind: TopicRelationKind,
) -> CommandResult<Vec<TopicLinkTargetView>> {
    topic_service::list_topic_link_targets(&PathBuf::from(vault_path), relation_kind)
        .map_err(CommandError::from)
}
