mod commands;
mod domain;
mod error;
mod io_utils;
mod services;

use commands::{
    candidates::{
        extract_resume_candidates, list_story_candidates, set_story_candidate_status,
    },
    privacy::{list_entity_reviews, resolve_entity_review},
    vault::{
        create_vault, import_source, list_sources, open_vault, update_cloud_identifier_mode,
    },
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            create_vault,
            open_vault,
            import_source,
            list_sources,
            update_cloud_identifier_mode,
            list_entity_reviews,
            resolve_entity_review,
            extract_resume_candidates,
            list_story_candidates,
            set_story_candidate_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running WorkLore");
}
