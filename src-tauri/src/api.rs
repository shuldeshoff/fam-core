use serde::{Deserialize, Serialize};
use crate::db;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRequest {
    pub url: String,
    pub method: String,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

/// Получение пути к БД и ключа
fn get_db_config(app: tauri::AppHandle) -> Result<(String, String), String> {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    let db_path = app_data_dir.join("wallet.db");
    let db_path_str = db_path.to_string_lossy().to_string();
    let key = "initialization_key".to_string();
    
    Ok((db_path_str, key))
}

// API команды для работы с БД

/// Создание счёта
#[tauri::command]
pub async fn create_account(
    app: tauri::AppHandle,
    name: String,
    acc_type: String,
) -> Result<i64, String> {
    let (db_path, key) = get_db_config(app)?;
    db::create_account(&db_path, &key, name, acc_type)
        .map_err(|e| format!("Failed to create account: {}", e))
}

/// Получение списка счетов
#[tauri::command]
pub async fn list_accounts(app: tauri::AppHandle) -> Result<Vec<db::Account>, String> {
    let (db_path, key) = get_db_config(app)?;
    db::list_accounts(&db_path, &key)
        .map_err(|e| format!("Failed to list accounts: {}", e))
}

/// Добавление операции
#[tauri::command]
pub async fn add_operation(
    app: tauri::AppHandle,
    account_id: i64,
    amount: f64,
    description: String,
) -> Result<i64, String> {
    let (db_path, key) = get_db_config(app)?;
    db::add_operation(&db_path, &key, account_id, amount, description)
        .map_err(|e| format!("Failed to add operation: {}", e))
}

/// Получение операций по счёту
#[tauri::command]
pub async fn get_operations(
    app: tauri::AppHandle,
    account_id: i64,
) -> Result<Vec<db::Operation>, String> {
    let (db_path, key) = get_db_config(app)?;
    db::get_operations(&db_path, &key, account_id)
        .map_err(|e| format!("Failed to get operations: {}", e))
}

// HTTP команды (заглушки)

/// Выполнение HTTP запроса
#[tauri::command]
pub async fn make_request(request: ApiRequest) -> Result<ApiResponse, String> {
    // TODO: Реализовать HTTP запросы
    Ok(ApiResponse {
        status: 200,
        body: format!("Response from {}", request.url),
        headers: vec![("content-type".to_string(), "application/json".to_string())],
    })
}

/// Получение данных GET запросом
#[tauri::command]
pub async fn fetch_data(url: String) -> Result<String, String> {
    // TODO: Реализовать GET запрос
    Ok(format!("Data from {}", url))
}

/// Отправка данных POST запросом
#[tauri::command]
pub async fn post_data(url: String, data: String) -> Result<ApiResponse, String> {
    // TODO: Реализовать POST запрос
    Ok(ApiResponse {
        status: 201,
        body: format!("Posted to {}: {}", url, data),
        headers: vec![],
    })
}

