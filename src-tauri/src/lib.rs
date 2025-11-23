// Модули
pub mod db;
pub mod crypto;
pub mod api;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Database commands
            db::init_database,
            db::check_connection,
            db::execute_query,
            db::get_version,
            db::set_version,
            // Crypto commands
            crypto::generate_key,
            crypto::derive_password_key,
            crypto::verify_password_key,
            crypto::get_crypto_config,
            // API commands
            api::make_request,
            api::fetch_data,
            api::post_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
