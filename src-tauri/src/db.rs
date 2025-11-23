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

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub acc_type: String,
    pub created_at: i64,
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
    
    // Включаем внешние ключи
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    
    // Выполняем миграции
    run_migrations(&conn)?;
    
    Ok(())
}

/// Запуск всех миграций
fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    // Создаём таблицу meta для отслеживания версии БД
    create_meta_table(conn)?;
    
    // Получаем текущую версию
    let version = get_current_version(conn)?;
    
    // Выполняем миграции по порядку
    if version < 1 {
        migration_v1(conn)?;
        update_version(conn, 1)?;
    }
    
    if version < 2 {
        migration_v2_accounts(conn)?;
        update_version(conn, 2)?;
    }
    
    if version < 3 {
        migration_v3_operations(conn)?;
        update_version(conn, 3)?;
    }
    
    if version < 4 {
        migration_v4_states(conn)?;
        update_version(conn, 4)?;
    }
    
    Ok(())
}

/// Получение текущей версии БД
fn get_current_version(conn: &Connection) -> Result<i32, DbError> {
    let version: Result<String, _> = conn.query_row(
        "SELECT version FROM meta LIMIT 1",
        [],
        |row| row.get(0),
    );
    
    match version {
        Ok(v) => v.parse::<i32>().map_err(|_| DbError::MigrationError("Invalid version format".to_string())),
        Err(_) => Ok(0),
    }
}

/// Обновление версии БД
fn update_version(conn: &Connection, version: i32) -> SqlResult<()> {
    conn.execute(
        "UPDATE meta SET version = ?1",
        [version.to_string()],
    )?;
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
            ["0"],
        )?;
    }
    
    Ok(())
}

/// Миграция v1: начальная настройка (уже выполнена через create_meta_table)
fn migration_v1(_conn: &Connection) -> SqlResult<()> {
    // Таблица meta уже создана, просто обновляем версию
    Ok(())
}

/// Миграция v2: создание таблицы accounts
fn migration_v2_accounts(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )",
        [],
    )?;
    
    // Создаём индексы для ускорения поиска
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(type)",
        [],
    )?;
    
    Ok(())
}

/// Миграция v3: создание таблицы operations
fn migration_v3_operations(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS operations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id INTEGER NOT NULL,
            amount REAL NOT NULL,
            description TEXT NOT NULL,
            ts INTEGER NOT NULL,
            FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    // Создаём индексы
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_operations_account_id ON operations(account_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_operations_ts ON operations(ts)",
        [],
    )?;
    
    Ok(())
}

/// Миграция v4: создание таблицы states
fn migration_v4_states(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS states (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id INTEGER NOT NULL,
            balance REAL NOT NULL,
            ts INTEGER NOT NULL,
            FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    // Создаём индексы
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_states_account_id ON states(account_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_states_ts ON states(ts)",
        [],
    )?;
    
    // Уникальный индекс для предотвращения дублирования снимков
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_states_account_ts ON states(account_id, ts)",
        [],
    )?;
    
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

// Функции работы со счетами

/// Создание нового счёта
pub fn create_account(path: &str, key: &str, name: String, acc_type: String) -> Result<i64, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    // Получаем текущий timestamp в секундах
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| DbError::InitError(format!("Failed to get timestamp: {}", e)))?
        .as_secs() as i64;
    
    conn.execute(
        "INSERT INTO accounts (name, type, created_at) VALUES (?1, ?2, ?3)",
        [&name, &acc_type, &created_at.to_string()],
    )?;
    
    let account_id = conn.last_insert_rowid();
    
    Ok(account_id)
}

/// Получение списка всех счетов
pub fn list_accounts(path: &str, key: &str) -> Result<Vec<Account>, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, name, type, created_at FROM accounts ORDER BY created_at DESC"
    )?;
    
    let accounts = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            acc_type: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(accounts)
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

/// Создание нового счёта
#[tauri::command]
pub async fn create_account_command(
    path: String,
    key: String,
    name: String,
    acc_type: String,
) -> Result<i64, String> {
    create_account(&path, &key, name, acc_type)
        .map_err(|e| format!("Failed to create account: {}", e))
}

/// Получение списка счетов
#[tauri::command]
pub async fn list_accounts_command(path: String, key: String) -> Result<Vec<Account>, String> {
    list_accounts(&path, &key)
        .map_err(|e| format!("Failed to list accounts: {}", e))
}
