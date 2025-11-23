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

#[derive(Debug, Serialize, Deserialize)]
pub struct Operation {
    pub id: i64,
    pub account_id: i64,
    pub amount: f64,
    pub description: String,
    pub ts: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub id: i64,
    pub account_id: i64,
    pub balance: f64,
    pub ts: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetAllocation {
    #[serde(rename = "type")]
    pub asset_type: String,
    pub total_balance: f64,
    pub account_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionLogRecord {
    pub id: i64,
    pub entity: String,
    pub entity_id: i64,
    pub action: String,
    pub payload: String,
    pub ts: i64,
}

// Вспомогательные функции для сериализации

/// Сериализация сущности в JSON-строку
/// 
/// Используется для создания payload в version_log
/// 
/// # Примеры
/// 
/// ```
/// let account = Account { id: 1, name: "Test".to_string(), acc_type: "cash".to_string(), created_at: 123456 };
/// let json = serialize_entity(&account).unwrap();
/// // json = '{"id":1,"name":"Test","type":"cash","created_at":123456}'
/// ```
pub fn serialize_entity<T: Serialize>(entity: &T) -> Result<String, DbError> {
    serde_json::to_string(entity)
        .map_err(|e| DbError::InitError(format!("Serialization error: {}", e)))
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
    
    // Генерируем Ed25519 ключи при первом запуске
    ensure_ed25519_keys(path, key)?;
    
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
    
    if version < 5 {
        migration_v5_version_log(conn)?;
        update_version(conn, 5)?;
    }
    
    if version < 6 {
        migration_v6_keystore(conn)?;
        update_version(conn, 6)?;
    }
    
    if version < 7 {
        migration_v7_version_signatures(conn)?;
        update_version(conn, 7)?;
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

/// Миграция v5: создание таблицы version_log для аудита изменений
fn migration_v5_version_log(conn: &Connection) -> SqlResult<()> {
    // Создаём таблицу version_log
    conn.execute(
        "CREATE TABLE IF NOT EXISTS version_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entity TEXT NOT NULL,
            entity_id INTEGER NOT NULL,
            action TEXT NOT NULL,
            payload TEXT NOT NULL,
            ts INTEGER NOT NULL
        )",
        [],
    )?;
    
    // Создаём индексы для быстрого поиска
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_version_log_entity ON version_log(entity, entity_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_version_log_ts ON version_log(ts)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_version_log_action ON version_log(action)",
        [],
    )?;
    
    Ok(())
}

/// Миграция M6: Таблица keystore для хранения криптографических ключей
fn migration_v6_keystore(conn: &Connection) -> SqlResult<()> {
    // Создаём таблицу keystore для хранения ключей
    conn.execute(
        "CREATE TABLE IF NOT EXISTS keystore (
            key TEXT PRIMARY KEY,
            value BLOB NOT NULL
        )",
        [],
    )?;
    
    // Индекс уже есть благодаря PRIMARY KEY
    Ok(())
}

/// Миграция M7: Таблица version_signatures для подписей версий
fn migration_v7_version_signatures(conn: &Connection) -> SqlResult<()> {
    // Создаём таблицу version_signatures
    conn.execute(
        "CREATE TABLE IF NOT EXISTS version_signatures (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version_id INTEGER NOT NULL,
            signature BLOB NOT NULL,
            public_key BLOB NOT NULL,
            ts INTEGER NOT NULL,
            FOREIGN KEY (version_id) REFERENCES version_log(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    // Создаём индексы для быстрого поиска
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_version_signatures_version_id ON version_signatures(version_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_version_signatures_ts ON version_signatures(ts)",
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

// Функции для version_log (аудит изменений)

/// Запись изменения в version_log
/// 
/// Функция должна вызываться внутри транзакций для обеспечения атомарности
/// 
/// # Параметры
/// - `conn` - ссылка на транзакцию или соединение
/// - `entity` - тип сущности (account, operation, state)
/// - `entity_id` - ID сущности
/// - `action` - тип действия (create, update, delete)
/// - `payload_json` - JSON-снимок состояния сущности
fn write_version_log(
    conn: &Connection,
    path: &str,
    db_key: &str,
    entity: &str,
    entity_id: i64,
    action: &str,
    payload_json: &str,
) -> Result<(), DbError> {
    // Получаем текущий timestamp в секундах
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| DbError::InitError(format!("Failed to get timestamp: {}", e)))?
        .as_secs() as i64;
    
    // Записываем в version_log
    conn.execute(
        "INSERT INTO version_log (entity, entity_id, action, payload, ts) VALUES (?1, ?2, ?3, ?4, ?5)",
        [
            entity,
            &entity_id.to_string(),
            action,
            payload_json,
            &ts.to_string(),
        ],
    )?;
    
    // Получаем id вставленной записи
    let version_id = conn.last_insert_rowid();
    
    // Загружаем приватный и публичный ключи из keystore
    let private_key = load_key_from_keystore(path, db_key, "ed25519_private")?
        .ok_or_else(|| DbError::InitError("Ed25519 private key not found in keystore".to_string()))?;
    
    let public_key = load_key_from_keystore(path, db_key, "ed25519_public")?
        .ok_or_else(|| DbError::InitError("Ed25519 public key not found in keystore".to_string()))?;
    
    // Сериализуем payload в bytes
    let payload_bytes = payload_json.as_bytes();
    
    // Подписываем payload
    let signature = crate::crypto::sign_payload(payload_bytes, &private_key)
        .map_err(|e| DbError::InitError(format!("Failed to sign payload: {}", e)))?;
    
    // Записываем подпись в version_signatures
    conn.execute(
        "INSERT INTO version_signatures (version_id, signature, public_key, ts) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![version_id, &signature, &public_key, ts],
    )?;
    
    Ok(())
}

/// Верификация подписи записи version_log
///
/// # Arguments
/// - `path` - путь к базе данных
/// - `key` - ключ шифрования
/// - `version_id` - ID записи в version_log
///
/// # Returns
/// - `Ok(true)` - подпись валидна
/// - `Ok(false)` - подпись невалидна
/// - `Err` - ошибка при выполнении (запись не найдена, отсутствует подпись и т.д.)
pub fn verify_version_signature(path: &str, key: &str, version_id: i64) -> Result<bool, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    // Извлекаем payload из version_log
    let payload: String = conn.query_row(
        "SELECT payload FROM version_log WHERE id = ?1",
        [version_id],
        |row| row.get(0),
    ).map_err(|e| DbError::InitError(format!("Failed to get version_log payload: {}", e)))?;
    
    // Извлекаем signature и public_key из version_signatures
    let (signature, public_key): (Vec<u8>, Vec<u8>) = conn.query_row(
        "SELECT signature, public_key FROM version_signatures WHERE version_id = ?1",
        [version_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| DbError::InitError(format!("Failed to get signature: {}", e)))?;
    
    // Верифицируем подпись
    let payload_bytes = payload.as_bytes();
    let is_valid = crate::crypto::verify_payload(payload_bytes, &signature, &public_key)
        .map_err(|e| DbError::InitError(format!("Failed to verify signature: {}", e)))?;
    
    Ok(is_valid)
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
    
    // Создаём объект Account для логирования
    let account = Account {
        id: account_id,
        name: name.clone(),
        acc_type: acc_type.clone(),
        created_at,
    };
    
    // Сериализуем аккаунт в JSON
    let payload_json = serialize_entity(&account)?;
    
    // Логируем создание аккаунта
    write_version_log(&conn, path, key, "account", account_id, "create", &payload_json)?;
    
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

// Функции работы с операциями

/// Получение текущего баланса счёта
fn get_current_balance(conn: &Connection, account_id: i64) -> SqlResult<f64> {
    let balance: Result<f64, _> = conn.query_row(
        "SELECT balance FROM states WHERE account_id = ?1 ORDER BY ts DESC LIMIT 1",
        [account_id],
        |row| row.get(0),
    );
    
    // Если баланса нет, начинаем с 0
    Ok(balance.unwrap_or(0.0))
}

/// Добавление операции с автоматическим обновлением баланса
pub fn add_operation(
    path: &str,
    key: &str,
    account_id: i64,
    amount: f64,
    description: String,
) -> Result<i64, DbError> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    // Начинаем транзакцию
    let tx = conn.transaction()?;
    
    // Получаем текущий timestamp
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| DbError::InitError(format!("Failed to get timestamp: {}", e)))?
        .as_secs() as i64;
    
    // Вставляем операцию
    tx.execute(
        "INSERT INTO operations (account_id, amount, description, ts) VALUES (?1, ?2, ?3, ?4)",
        [&account_id.to_string(), &amount.to_string(), &description, &ts.to_string()],
    )?;
    
    let operation_id = tx.last_insert_rowid();
    
    // Создаём объект Operation для логирования
    let operation = Operation {
        id: operation_id,
        account_id,
        amount,
        description: description.clone(),
        ts,
    };
    
    // Сериализуем операцию в JSON
    let operation_json = serialize_entity(&operation)?;
    
    // Логируем создание операции
    write_version_log(&tx, path, key, "operation", operation_id, "create", &operation_json)?;
    
    // Получаем текущий баланс
    let current_balance = get_current_balance(&tx, account_id)?;
    
    // Рассчитываем новый баланс
    let new_balance = current_balance + amount;
    
    // Создаём новую запись баланса в states
    tx.execute(
        "INSERT INTO states (account_id, balance, ts) VALUES (?1, ?2, ?3)",
        [&account_id.to_string(), &new_balance.to_string(), &ts.to_string()],
    )?;
    
    let state_id = tx.last_insert_rowid();
    
    // Создаём объект State для логирования
    let state = State {
        id: state_id,
        account_id,
        balance: new_balance,
        ts,
    };
    
    // Сериализуем state в JSON
    let state_json = serialize_entity(&state)?;
    
    // Логируем создание state
    write_version_log(&tx, path, key, "state", state_id, "create", &state_json)?;
    
    // Коммитим транзакцию (включая операцию, баланс и оба лога)
    tx.commit()?;
    
    Ok(operation_id)
}

/// Получение списка операций по счёту
pub fn get_operations(path: &str, key: &str, account_id: i64) -> Result<Vec<Operation>, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, account_id, amount, description, ts FROM operations 
         WHERE account_id = ?1 ORDER BY ts DESC"
    )?;
    
    let operations = stmt.query_map([account_id], |row| {
        Ok(Operation {
            id: row.get(0)?,
            account_id: row.get(1)?,
            amount: row.get(2)?,
            description: row.get(3)?,
            ts: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(operations)
}

// Функции агрегирования

/// Получение текущего баланса аккаунта
/// 
/// Возвращает последнее значение баланса из таблицы states
/// Если записей нет, возвращает 0.0
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `key` - ключ шифрования
/// - `account_id` - ID аккаунта
pub fn get_account_balance(path: &str, key: &str, account_id: i64) -> Result<f64, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    let balance: Result<f64, _> = conn.query_row(
        "SELECT balance FROM states WHERE account_id = ?1 ORDER BY ts DESC LIMIT 1",
        [account_id],
        |row| row.get(0),
    );
    
    // Если баланса нет, возвращаем 0.0
    Ok(balance.unwrap_or(0.0))
}

/// Вычисление общего Net Worth
/// 
/// Возвращает сумму всех текущих балансов по всем аккаунтам
/// Для каждого аккаунта берётся последняя запись из states
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `key` - ключ шифрования
pub fn get_net_worth(path: &str, key: &str) -> Result<f64, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    // Получаем последний баланс для каждого аккаунта и суммируем
    let net_worth: f64 = conn.query_row(
        "SELECT COALESCE(SUM(balance), 0.0) FROM (
            SELECT DISTINCT account_id, 
                   (SELECT balance FROM states s2 
                    WHERE s2.account_id = s1.account_id 
                    ORDER BY ts DESC LIMIT 1) as balance
            FROM states s1
        )",
        [],
        |row| row.get(0),
    )?;
    
    Ok(net_worth)
}

/// Получение временного ряда балансов для аккаунта
/// 
/// Возвращает все записи из таблицы states для указанного аккаунта
/// с сортировкой по временной метке (ts) в порядке возрастания
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `key` - ключ шифрования
/// - `account_id` - ID аккаунта
pub fn get_balance_history(
    path: &str,
    key: &str,
    account_id: i64,
) -> Result<Vec<State>, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, account_id, balance, ts FROM states 
         WHERE account_id = ?1 
         ORDER BY ts ASC"
    )?;
    
    let states = stmt.query_map([account_id], |row| {
        Ok(State {
            id: row.get(0)?,
            account_id: row.get(1)?,
            balance: row.get(2)?,
            ts: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(states)
}

/// Получение структуры активов (группировка по типам с агрегированием балансов)
/// 
/// Возвращает распределение активов по типам аккаунтов
/// Для каждого типа вычисляется:
/// - Общая сумма балансов всех аккаунтов этого типа
/// - Количество аккаунтов
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `key` - ключ шифрования
pub fn get_asset_allocation(
    path: &str,
    key: &str,
) -> Result<Vec<AssetAllocation>, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    // SQL запрос с группировкой по типам и суммированием балансов
    // Включаем только аккаунты, у которых есть хотя бы одна запись в states
    let mut stmt = conn.prepare(
        "SELECT 
            a.type,
            SUM(latest_balances.balance) as total_balance,
            COUNT(a.id) as account_count
         FROM accounts a
         INNER JOIN (
             SELECT DISTINCT account_id,
                    (SELECT balance FROM states s2 
                     WHERE s2.account_id = s1.account_id 
                     ORDER BY ts DESC LIMIT 1) as balance
             FROM states s1
         ) latest_balances ON latest_balances.account_id = a.id
         GROUP BY a.type
         ORDER BY total_balance DESC"
    )?;
    
    let allocations = stmt.query_map([], |row| {
        Ok(AssetAllocation {
            asset_type: row.get(0)?,
            total_balance: row.get(1)?,
            account_count: row.get(2)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(allocations)
}

// Функции для работы с keystore

/// Сохранение ключа в keystore
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `db_key` - ключ шифрования БД
/// - `key_name` - имя ключа (например, "ed25519_private", "ed25519_public")
/// - `key_value` - значение ключа в виде байтов
pub fn save_key_to_keystore(
    path: &str,
    db_key: &str,
    key_name: &str,
    key_value: &[u8],
) -> Result<(), DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", db_key)?;
    
    conn.execute(
        "INSERT OR REPLACE INTO keystore (key, value) VALUES (?1, ?2)",
        rusqlite::params![key_name, key_value],
    )?;
    
    Ok(())
}

/// Загрузка ключа из keystore
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `db_key` - ключ шифрования БД
/// - `key_name` - имя ключа
pub fn load_key_from_keystore(
    path: &str,
    db_key: &str,
    key_name: &str,
) -> Result<Option<Vec<u8>>, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", db_key)?;
    
    let result: Result<Vec<u8>, rusqlite::Error> = conn.query_row(
        "SELECT value FROM keystore WHERE key = ?1",
        [key_name],
        |row| row.get(0),
    );
    
    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DbError::SqliteError(e)),
    }
}

/// Проверка существования ключа в keystore
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `db_key` - ключ шифрования БД
/// - `key_name` - имя ключа
pub fn key_exists_in_keystore(
    path: &str,
    db_key: &str,
    key_name: &str,
) -> Result<bool, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", db_key)?;
    
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM keystore WHERE key = ?1",
        [key_name],
        |row| row.get(0),
    )?;
    
    Ok(count > 0)
}

/// Удаление ключа из keystore
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `db_key` - ключ шифрования БД
/// - `key_name` - имя ключа
pub fn delete_key_from_keystore(
    path: &str,
    db_key: &str,
    key_name: &str,
) -> Result<(), DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", db_key)?;
    
    conn.execute(
        "DELETE FROM keystore WHERE key = ?1",
        [key_name],
    )?;
    
    Ok(())
}

/// Генерация и сохранение Ed25519 ключей при первом запуске
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `db_key` - ключ шифрования БД
pub fn ensure_ed25519_keys(path: &str, db_key: &str) -> Result<(), DbError> {
    // Проверяем существование приватного ключа
    let private_exists = key_exists_in_keystore(path, db_key, "ed25519_private")?;
    let public_exists = key_exists_in_keystore(path, db_key, "ed25519_public")?;
    
    if !private_exists || !public_exists {
        // Генерируем новую пару ключей
        let keypair = crate::crypto::generate_ed25519_keypair()
            .map_err(|e| DbError::InitError(format!("Failed to generate Ed25519 keys: {}", e)))?;
        
        // Сохраняем в keystore
        save_key_to_keystore(path, db_key, "ed25519_private", &keypair.private_key)?;
        save_key_to_keystore(path, db_key, "ed25519_public", &keypair.public_key)?;
        
        println!("✓ Generated and saved new Ed25519 keypair to keystore");
    }
    
    Ok(())
}

/// Получение записей из version_log с опциональными фильтрами
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `key` - ключ шифрования
/// - `entity` - фильтр по типу сущности (account, operation, state)
/// - `entity_id` - фильтр по ID сущности
pub fn list_version_log(
    path: &str,
    key: &str,
    entity: Option<String>,
    entity_id: Option<i64>,
) -> Result<Vec<VersionLogRecord>, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    // Строим запрос динамически в зависимости от фильтров
    let mut query = String::from("SELECT id, entity, entity_id, action, payload, ts FROM version_log");
    let mut conditions = Vec::new();
    
    if entity.is_some() {
        conditions.push("entity = ?1");
    }
    
    if entity_id.is_some() {
        if entity.is_some() {
            conditions.push("entity_id = ?2");
        } else {
            conditions.push("entity_id = ?1");
        }
    }
    
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    
    query.push_str(" ORDER BY ts DESC, id DESC");
    
    let mut stmt = conn.prepare(&query)?;
    
    // Выполняем запрос с нужными параметрами
    let records = match (&entity, &entity_id) {
        (Some(e), Some(eid)) => {
            stmt.query_map([e.as_str(), &eid.to_string()], |row| {
                Ok(VersionLogRecord {
                    id: row.get(0)?,
                    entity: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    payload: row.get(4)?,
                    ts: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        },
        (Some(e), None) => {
            stmt.query_map([e.as_str()], |row| {
                Ok(VersionLogRecord {
                    id: row.get(0)?,
                    entity: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    payload: row.get(4)?,
                    ts: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        },
        (None, Some(eid)) => {
            stmt.query_map([&eid.to_string()], |row| {
                Ok(VersionLogRecord {
                    id: row.get(0)?,
                    entity: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    payload: row.get(4)?,
                    ts: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        },
        (None, None) => {
            stmt.query_map([], |row| {
                Ok(VersionLogRecord {
                    id: row.get(0)?,
                    entity: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    payload: row.get(4)?,
                    ts: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        },
    };
    
    Ok(records)
}

/// Получение записей из version_log с фильтрами (сортировка ASC)
/// 
/// Отличие от list_version_log: сортировка по ts ASC, id ASC (хронологический порядок)
/// Полезно для воспроизведения истории изменений от старых к новым
/// 
/// # Параметры
/// - `path` - путь к базе данных
/// - `key` - ключ шифрования
/// - `entity` - фильтр по типу сущности (account, operation, state)
/// - `entity_id` - фильтр по ID сущности
pub fn get_version_log(
    path: &str,
    key: &str,
    entity: Option<&str>,
    entity_id: Option<i64>,
) -> Result<Vec<VersionLogRecord>, DbError> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)?;
    
    // Строим запрос динамически в зависимости от фильтров
    let mut query = String::from("SELECT id, entity, entity_id, action, payload, ts FROM version_log");
    let mut conditions = Vec::new();
    
    if entity.is_some() {
        conditions.push("entity = ?1");
    }
    
    if entity_id.is_some() {
        if entity.is_some() {
            conditions.push("entity_id = ?2");
        } else {
            conditions.push("entity_id = ?1");
        }
    }
    
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    
    // Сортировка ASC (от старых к новым)
    query.push_str(" ORDER BY ts ASC, id ASC");
    
    let mut stmt = conn.prepare(&query)?;
    
    // Выполняем запрос с нужными параметрами
    let records = match (&entity, &entity_id) {
        (Some(e), Some(eid)) => {
            stmt.query_map([e, &eid.to_string().as_str()], |row| {
                Ok(VersionLogRecord {
                    id: row.get(0)?,
                    entity: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    payload: row.get(4)?,
                    ts: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        },
        (Some(e), None) => {
            stmt.query_map([e], |row| {
                Ok(VersionLogRecord {
                    id: row.get(0)?,
                    entity: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    payload: row.get(4)?,
                    ts: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        },
        (None, Some(eid)) => {
            stmt.query_map([&eid.to_string().as_str()], |row| {
                Ok(VersionLogRecord {
                    id: row.get(0)?,
                    entity: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    payload: row.get(4)?,
                    ts: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        },
        (None, None) => {
            stmt.query_map([], |row| {
                Ok(VersionLogRecord {
                    id: row.get(0)?,
                    entity: row.get(1)?,
                    entity_id: row.get(2)?,
                    action: row.get(3)?,
                    payload: row.get(4)?,
                    ts: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        },
    };
    
    Ok(records)
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

/// Добавление операции
#[tauri::command]
pub async fn add_operation_command(
    path: String,
    key: String,
    account_id: i64,
    amount: f64,
    description: String,
) -> Result<i64, String> {
    add_operation(&path, &key, account_id, amount, description)
        .map_err(|e| format!("Failed to add operation: {}", e))
}

/// Получение операций по счёту
#[tauri::command]
pub async fn get_operations_command(
    path: String,
    key: String,
    account_id: i64,
) -> Result<Vec<Operation>, String> {
    get_operations(&path, &key, account_id)
        .map_err(|e| format!("Failed to get operations: {}", e))
}

/// Получение текущего баланса аккаунта
#[tauri::command]
pub async fn get_account_balance_command(
    path: String,
    key: String,
    account_id: i64,
) -> Result<f64, String> {
    get_account_balance(&path, &key, account_id)
        .map_err(|e| format!("Failed to get account balance: {}", e))
}

/// Получение общего Net Worth
#[tauri::command]
pub async fn get_net_worth_command(path: String, key: String) -> Result<f64, String> {
    get_net_worth(&path, &key)
        .map_err(|e| format!("Failed to get net worth: {}", e))
}

/// Получение временного ряда балансов для аккаунта
#[tauri::command]
pub async fn get_balance_history_command(
    path: String,
    key: String,
    account_id: i64,
) -> Result<Vec<State>, String> {
    get_balance_history(&path, &key, account_id)
        .map_err(|e| format!("Failed to get balance history: {}", e))
}

/// Получение структуры активов (распределение по типам)
#[tauri::command]
pub async fn get_asset_allocation_command(
    path: String,
    key: String,
) -> Result<Vec<AssetAllocation>, String> {
    get_asset_allocation(&path, &key)
        .map_err(|e| format!("Failed to get asset allocation: {}", e))
}
