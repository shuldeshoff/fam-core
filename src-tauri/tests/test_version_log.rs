use fam_core_lib::db;
use std::path::PathBuf;

#[test]
fn test_write_version_log() {
    // Используем временный файл для теста
    let test_db_path = "/tmp/test_version_log.db";
    let test_key = "test_key_version_log";
    
    // Удаляем старую БД если есть
    if PathBuf::from(test_db_path).exists() {
        std::fs::remove_file(test_db_path).unwrap();
    }
    
    println!("\n=== Тест автоматического логирования create_account ===");
    
    // Инициализируем БД
    db::init_db(test_db_path, test_key).expect("DB init failed");
    println!("✓ База данных инициализирована");
    
    // Создаём аккаунт (теперь должен автоматически логировать создание)
    let account_id = db::create_account(
        test_db_path,
        test_key,
        "Test Account".to_string(),
        "cash".to_string(),
    ).expect("Account creation failed");
    
    println!("✓ Аккаунт создан с ID: {}", account_id);
    
    // Проверяем, что запись в version_log СОЗДАЛАСЬ автоматически
    use rusqlite::Connection;
    let conn = Connection::open(test_db_path).unwrap();
    conn.pragma_update(None, "key", test_key).unwrap();
    
    let log_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM version_log",
        [],
        |row| row.get(0),
    ).unwrap();
    
    println!("Записей в version_log: {}", log_count);
    assert_eq!(log_count, 1, "Должна быть 1 запись в version_log");
    
    // Проверяем содержимое записи
    let (entity, entity_id, action, payload): (String, i64, String, String) = conn.query_row(
        "SELECT entity, entity_id, action, payload FROM version_log LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).unwrap();
    
    println!("✓ Запись в version_log:");
    println!("  entity: {}", entity);
    println!("  entity_id: {}", entity_id);
    println!("  action: {}", action);
    println!("  payload: {}", payload);
    
    assert_eq!(entity, "account");
    assert_eq!(entity_id, account_id);
    assert_eq!(action, "create");
    assert!(payload.contains("\"name\":\"Test Account\""));
    assert!(payload.contains("\"type\":\"cash\""));
    
    println!("✓ Все проверки пройдены");
    
    // Очистка
    std::fs::remove_file(test_db_path).unwrap();
    println!("✓ Тест пройден: create_account автоматически логирует создание в version_log");
}

#[test]
fn test_serialize_entity() {
    println!("\n=== Тест serialize_entity ===");
    
    // Тестируем сериализацию Account
    let account = db::Account {
        id: 123,
        name: "Тестовый счёт".to_string(),
        acc_type: "cash".to_string(),
        created_at: 1700000000,
    };
    
    let json = db::serialize_entity(&account).expect("Account serialization failed");
    println!("Account JSON: {}", json);
    
    assert!(json.contains("\"id\":123"));
    assert!(json.contains("\"name\":\"Тестовый счёт\""));
    assert!(json.contains("\"type\":\"cash\"")); // Проверяем rename
    assert!(json.contains("\"created_at\":1700000000"));
    println!("✓ Account сериализация работает");
    
    // Тестируем сериализацию Operation
    let operation = db::Operation {
        id: 456,
        account_id: 123,
        amount: 100.50,
        description: "Тестовая операция".to_string(),
        ts: 1700000100,
    };
    
    let json = db::serialize_entity(&operation).expect("Operation serialization failed");
    println!("Operation JSON: {}", json);
    
    assert!(json.contains("\"id\":456"));
    assert!(json.contains("\"account_id\":123"));
    assert!(json.contains("\"amount\":100.5"));
    assert!(json.contains("\"description\":\"Тестовая операция\""));
    assert!(json.contains("\"ts\":1700000100"));
    println!("✓ Operation сериализация работает");
    
    // Тестируем сериализацию State
    let state = db::State {
        id: 789,
        account_id: 123,
        balance: 1234.56,
        ts: 1700000200,
    };
    
    let json = db::serialize_entity(&state).expect("State serialization failed");
    println!("State JSON: {}", json);
    
    assert!(json.contains("\"id\":789"));
    assert!(json.contains("\"account_id\":123"));
    assert!(json.contains("\"balance\":1234.56"));
    assert!(json.contains("\"ts\":1700000200"));
    println!("✓ State сериализация работает");
    
    println!("✓ Все тесты сериализации пройдены");
}

#[test]
fn test_add_operation_logging() {
    use rusqlite::Connection;
    
    let test_db_path = "/tmp/test_operation_log.db";
    let test_key = "test_key_op_log";
    
    // Удаляем старую БД
    if PathBuf::from(test_db_path).exists() {
        std::fs::remove_file(test_db_path).unwrap();
    }
    
    println!("\n=== Тест логирования add_operation ===");
    
    // Инициализируем БД
    db::init_db(test_db_path, test_key).expect("DB init failed");
    println!("✓ База данных инициализирована");
    
    // Создаём аккаунт
    let account_id = db::create_account(
        test_db_path,
        test_key,
        "Test Account".to_string(),
        "cash".to_string(),
    ).expect("Account creation failed");
    println!("✓ Аккаунт создан с ID: {}", account_id);
    
    // Добавляем операцию
    let operation_id = db::add_operation(
        test_db_path,
        test_key,
        account_id,
        100.50,
        "Тестовая операция".to_string(),
    ).expect("Operation creation failed");
    println!("✓ Операция создана с ID: {}", operation_id);
    
    // Проверяем записи в version_log
    let conn = Connection::open(test_db_path).unwrap();
    conn.pragma_update(None, "key", test_key).unwrap();
    
    let log_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM version_log",
        [],
        |row| row.get(0),
    ).unwrap();
    
    println!("Всего записей в version_log: {}", log_count);
    
    // Должно быть 3 записи: 1 account + 1 operation + 1 state
    assert_eq!(log_count, 3, "Должно быть 3 записи: account, operation, state");
    
    // Проверяем запись operation
    let (entity, entity_id, action, payload): (String, i64, String, String) = conn.query_row(
        "SELECT entity, entity_id, action, payload FROM version_log WHERE entity = 'operation'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).unwrap();
    
    println!("\n✓ Запись operation в version_log:");
    println!("  entity: {}", entity);
    println!("  entity_id: {}", entity_id);
    println!("  action: {}", action);
    println!("  payload: {}", payload);
    
    assert_eq!(entity, "operation");
    assert_eq!(entity_id, operation_id);
    assert_eq!(action, "create");
    assert!(payload.contains("\"amount\":100.5"));
    assert!(payload.contains("\"description\":\"Тестовая операция\""));
    
    // Проверяем запись state
    let (state_entity, state_entity_id, state_action, state_payload): (String, i64, String, String) = conn.query_row(
        "SELECT entity, entity_id, action, payload FROM version_log WHERE entity = 'state'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).unwrap();
    
    println!("\n✓ Запись state в version_log:");
    println!("  entity: {}", state_entity);
    println!("  entity_id: {}", state_entity_id);
    println!("  action: {}", state_action);
    println!("  payload: {}", state_payload);
    
    assert_eq!(state_entity, "state");
    assert_eq!(state_action, "create");
    assert!(state_payload.contains("\"balance\":100.5"));
    assert!(state_payload.contains(&format!("\"account_id\":{}", account_id)));
    
    println!("\n✓ Все проверки пройдены");
    println!("✓ add_operation логирует и operation, и state в одной транзакции");
    
    // Очистка
    std::fs::remove_file(test_db_path).unwrap();
}

