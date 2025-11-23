use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    
    #[error("Failed to initialize database: {0}")]
    InitError(String),
    
    #[error("Migration error: {0}")]
    MigrationError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbConfig {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbResult {
    pub success: bool,
    pub message: String,
}

/// Инициализация базы данных с шифрованием
pub fn init_db(path: &str, key: &str) -> Result<(), DbError> {
    // Проверяем и создаем директорию если нужно
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DbError::InitError(format!("Failed to create directory: {}", e)))?;
        }
    }

    // Открываем или создаём базу данных
    let conn = Connection::open(path)?;
    
    // Включаем PRAGMA key для шифрования через SQLCipher
    conn.pragma_update(None, "key", key)?;
    
    // Выполняем миграцию: создаём таблицу meta
    create_meta_table(&conn)?;
    
    Ok(())
}

/// Создание таблицы meta
fn create_meta_table(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS meta (
            version TEXT NOT NULL
        )",
        [],
    )?;
    
    // Проверяем, есть ли уже версия, если нет - вставляем начальную
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM meta",
        [],
        |row| row.get(0),
    )?;
    
    if count == 0 {
        conn.execute(
            "INSERT INTO meta (version) VALUES (?1)",
            ["1.0.0"],
        )?;
    }
    
    Ok(())
}

/// Получение версии БД
pub fn get_db_version(path: &str, key: &str) -> Result<String, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    let version: String = conn.query_row(
        "SELECT version FROM meta LIMIT 1",
        [],
        |row| row.get(0),
    )?;
    
    Ok(version)
}

/// Обновление версии БД
pub fn update_db_version(path: &str, key: &str, new_version: &str) -> Result<(), DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    conn.execute(
        "UPDATE meta SET version = ?1",
        [new_version],
    )?;
    
    Ok(())
}

// Tauri команды

/// Инициализация базы данных
#[tauri::command]
pub async fn init_database(path: String, key: String) -> Result<DbResult, String> {
    match init_db(&path, &key) {
        Ok(_) => Ok(DbResult {
            success: true,
            message: format!("Database initialized at: {}", path),
        }),
        Err(e) => Err(format!("Failed to initialize database: {}", e)),
    }
}

/// Проверка соединения с БД
#[tauri::command]
pub async fn check_connection(path: String, key: String) -> Result<DbResult, String> {
    match Connection::open(&path) {
        Ok(conn) => {
            if let Err(e) = conn.pragma_update(None, "key", &key) {
                return Err(format!("Failed to set encryption key: {}", e));
            }
            
            // Проверяем доступность таблицы meta
            match conn.query_row("SELECT version FROM meta LIMIT 1", [], |row| {
                row.get::<_, String>(0)
            }) {
                Ok(version) => Ok(DbResult {
                    success: true,
                    message: format!("Connected. DB version: {}", version),
                }),
                Err(e) => Err(format!("Database connection failed: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to open database: {}", e)),
    }
}

/// Выполнение запроса к БД
#[tauri::command]
pub async fn execute_query(path: String, key: String, query: String) -> Result<String, String> {
    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    conn.pragma_update(None, "key", &key)
        .map_err(|e| format!("Failed to set encryption key: {}", e))?;
    
    conn.execute(&query, [])
        .map_err(|e| format!("Query execution failed: {}", e))?;
    
    Ok("Query executed successfully".to_string())
}

/// Получение версии БД
#[tauri::command]
pub async fn get_version(path: String, key: String) -> Result<String, String> {
    get_db_version(&path, &key)
        .map_err(|e| format!("Failed to get DB version: {}", e))
}

/// Обновление версии БД
#[tauri::command]
pub async fn set_version(path: String, key: String, version: String) -> Result<DbResult, String> {
    update_db_version(&path, &key, &version)
        .map_err(|e| format!("Failed to update DB version: {}", e))?;
    
    Ok(DbResult {
        success: true,
        message: format!("DB version updated to: {}", version),
    })
}

/// Получение статуса базы данных
#[tauri::command]
pub async fn get_status() -> String {
    "Database module is ready".to_string()
}

/// Запись тестового значения в таблицу meta
#[tauri::command]
pub async fn write_test_record(path: String, key: String, value: String) -> Result<(), String> {
    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    conn.pragma_update(None, "key", &key)
        .map_err(|e| format!("Failed to set encryption key: {}", e))?;
    
    // Обновляем значение в таблице meta, перезаписывая предыдущее значение
    conn.execute(
        "UPDATE meta SET version = ?1",
        [&value],
    )
    .map_err(|e| format!("Failed to write test record: {}", e))?;
    
    Ok(())
}
