// Модули
pub mod db;
pub mod crypto;
pub mod api;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Получение пути к базе данных
#[tauri::command]
fn get_db_path(app: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    let db_path = app_data_dir.join("wallet.db");
    Ok(db_path.to_string_lossy().to_string())
}

/// Инициализация приложения
fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Получаем путь к директории данных приложения
    let app_data_dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Warning: Failed to get app data dir: {}", e);
            // Используем fallback путь
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join(".fam-core")
        }
    };
    
    // Создаём директорию если её нет
    if !app_data_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
            eprintln!("Warning: Failed to create directory: {}", e);
            // Не прерываем запуск приложения
            return Ok(());
        }
    }
    
    // Путь к базе данных
    let db_path = app_data_dir.join("wallet.db");
    let db_path_str = db_path.to_string_lossy().to_string();
    
    // Временный ключ для инициализации (в продакшене должен быть из настроек)
    let temp_key = "initialization_key";
    
    // Инициализируем базу данных
    match db::init_db(&db_path_str, temp_key) {
        Ok(_) => {
            println!("Database initialized at: {}", db_path_str);
        }
        Err(e) => {
            eprintln!("Warning: Failed to initialize database: {}", e);
            // Не прерываем запуск - БД будет инициализирована при первом использовании
        }
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Не паникуем при ошибке setup - просто логируем
            if let Err(e) = setup_app(app) {
                eprintln!("Setup warning: {}", e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_db_path,
            // Database commands (low-level with path/key)
            db::init_database,
            db::check_connection,
            db::execute_query,
            db::get_version,
            db::set_version,
            db::get_status,
            db::write_test_record,
            db::create_account_command,
            db::list_accounts_command,
            db::add_operation_command,
            db::get_operations_command,
            // Crypto commands
            crypto::generate_key,
            crypto::derive_password_key,
            crypto::verify_password_key,
            crypto::get_crypto_config,
            // API commands (high-level without path/key)
            api::create_account,
            api::list_accounts,
            api::add_operation,
            api::get_operations,
            api::list_versions,
            api::get_account_balance,
            api::get_net_worth,
            api::make_request,
            api::fetch_data,
            api::post_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
