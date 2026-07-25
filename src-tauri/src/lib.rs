mod commands;
mod domain;
mod error;
mod io_utils;
mod services;

use commands::{
    candidates::{extract_resume_candidates, list_story_candidates, set_story_candidate_status},
    interviews::{
        list_guided_interviews, resume_guided_interview, start_guided_interview,
        submit_guided_interview_response,
    },
    performance::get_performance_snapshot,
    preferences::{clear_last_vault, get_last_vault_path, remember_last_vault},
    privacy::{list_entity_reviews, resolve_entity_review},
    providers::create_manual_workspace,
    roles::list_roles,
    stories::{import_story_response, list_stories, set_story_status},
    vault::{
        create_vault, create_vault_in_parent, import_source, list_sources, open_vault,
        update_cloud_identifier_mode,
    },
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            create_vault,
            create_vault_in_parent,
            open_vault,
            import_source,
            list_sources,
            update_cloud_identifier_mode,
            get_last_vault_path,
            remember_last_vault,
            clear_last_vault,
            list_entity_reviews,
            resolve_entity_review,
            extract_resume_candidates,
            list_story_candidates,
            set_story_candidate_status,
            start_guided_interview,
            list_guided_interviews,
            submit_guided_interview_response,
            resume_guided_interview,
            get_performance_snapshot,
            create_manual_workspace,
            list_roles,
            import_story_response,
            list_stories,
            set_story_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running WorkLore");
}
