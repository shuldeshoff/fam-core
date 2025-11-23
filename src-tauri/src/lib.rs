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
    let app_data_dir = app.path().app_data_dir()?;
    
    // Создаём директорию если её нет
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)?;
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
            eprintln!("Failed to initialize database: {}", e);
            return Err(Box::new(e));
        }
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            setup_app(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_db_path,
            // Database commands
            db::init_database,
            db::check_connection,
            db::execute_query,
            db::get_version,
            db::set_version,
            db::get_status,
            db::write_test_record,
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
