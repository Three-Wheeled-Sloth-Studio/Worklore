mod commands;
mod domain;
mod error;
mod io_utils;
mod services;

use commands::{
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
            resolve_entity_review
        ])
        .run(tauri::generate_context!())
        .expect("error while running WorkLore");
}
