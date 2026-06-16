use tauri::{Builder, Manager};
use tokio::sync::Mutex;

mod commands;
mod cryptography;
mod s2secret;
mod http_client;

use crate::s2secret::S2SecretData;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(S2SecretData::new()));
            Ok(())
        })
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            commands::login, 
            commands::register_user,
            commands::is_authenticated,
            commands::logout, 
            commands::add_secret,
            commands::update_secret,
            commands::delete_secret,
            commands::delete_emergency_contact,
            commands::logged_user_data,
            commands::user_name,
            commands::passwords,
            commands::filter_by_search_term,
            commands::load_secrets_descriptive_data,
            commands::load_secret_descriptive_data,
            commands::load_emergency_contacts,
            commands::emergency_contacts,
            commands::reveal_password,
            commands::reveal_emergency_contact_password,
            commands::send_2fa_secret_code,
            commands::copy_password,
            commands::copy_username,
            commands::select_database_file,
            commands::select_emergency_file,
            commands::enable_proactive_protection,
            commands::disable_proactive_protection,
            commands::add_emergency_contact,
            commands::add_access_to_emergency_contact_for_secret,
            commands::get_emergency_access_file_data,
            commands::get_emergency_accesses_for_all_secrets,
            commands::remove_access_to_emergency_contact_for_secret,
            commands::renew_shares,
            commands::renew_share,
            commands::recover_secret,
            commands::copy_to_clipboard,
            commands::send_2fa_emergency_access_secret_code,
            ])
        .run(tauri::generate_context!())
        .expect("error while running S2Secret application");
}
