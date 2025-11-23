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

#[test]
fn test_list_version_log() {
    let test_db_path = "/tmp/test_list_versions.db";
    let test_key = "test_key_list";
    
    // Удаляем старую БД
    if PathBuf::from(test_db_path).exists() {
        std::fs::remove_file(test_db_path).unwrap();
    }
    
    println!("\n=== Тест list_version_log с фильтрами ===");
    
    // Инициализируем БД
    db::init_db(test_db_path, test_key).expect("DB init failed");
    
    // Создаём 2 аккаунта
    let account1_id = db::create_account(test_db_path, test_key, "Account 1".to_string(), "cash".to_string()).unwrap();
    let _account2_id = db::create_account(test_db_path, test_key, "Account 2".to_string(), "card".to_string()).unwrap();
    
    // Добавляем операцию для account1
    db::add_operation(test_db_path, test_key, account1_id, 100.0, "Op 1".to_string()).unwrap();
    
    println!("✓ Созданы 2 аккаунта и 1 операция");
    
    // Тест 1: Получение всех записей (без фильтров)
    let all_records = db::list_version_log(test_db_path, test_key, None, None).unwrap();
    println!("\n1. Все записи: {}", all_records.len());
    // 2 accounts + 1 operation + 1 state = 4 записи
    assert_eq!(all_records.len(), 4, "Должно быть 4 записи");
    
    // Проверяем сортировку (по ts DESC, id DESC)
    for (i, record) in all_records.iter().enumerate() {
        println!("  [{}] entity={}, entity_id={}, action={}", i, record.entity, record.entity_id, record.action);
    }
    
    // Тест 2: Фильтр по entity = "account"
    let account_records = db::list_version_log(test_db_path, test_key, Some("account".to_string()), None).unwrap();
    println!("\n2. Записи с entity='account': {}", account_records.len());
    assert_eq!(account_records.len(), 2, "Должно быть 2 записи account");
    for record in &account_records {
        assert_eq!(record.entity, "account");
    }
    
    // Тест 3: Фильтр по entity = "operation"
    let operation_records = db::list_version_log(test_db_path, test_key, Some("operation".to_string()), None).unwrap();
    println!("\n3. Записи с entity='operation': {}", operation_records.len());
    assert_eq!(operation_records.len(), 1, "Должна быть 1 запись operation");
    assert_eq!(operation_records[0].entity, "operation");
    
    // Тест 4: Фильтр по entity = "state"
    let state_records = db::list_version_log(test_db_path, test_key, Some("state".to_string()), None).unwrap();
    println!("\n4. Записи с entity='state': {}", state_records.len());
    assert_eq!(state_records.len(), 1, "Должна быть 1 запись state");
    
    // Тест 5: Фильтр по entity_id (для account1)
    let account1_records = db::list_version_log(test_db_path, test_key, None, Some(account1_id)).unwrap();
    println!("\n5. Записи с entity_id={}: {}", account1_id, account1_records.len());
    // 1 account + 1 operation + 1 state = 3 записи для account1
    assert_eq!(account1_records.len(), 3, "Должно быть 3 записи для account1");
    
    // Тест 6: Фильтр по entity="account" И entity_id=account1_id
    let specific_records = db::list_version_log(
        test_db_path, 
        test_key, 
        Some("account".to_string()), 
        Some(account1_id)
    ).unwrap();
    println!("\n6. Записи с entity='account' AND entity_id={}: {}", account1_id, specific_records.len());
    assert_eq!(specific_records.len(), 1, "Должна быть 1 запись");
    assert_eq!(specific_records[0].entity, "account");
    assert_eq!(specific_records[0].entity_id, account1_id);
    assert!(specific_records[0].payload.contains("Account 1"));
    
    println!("\n✓ Все тесты фильтрации пройдены");
    
    // Очистка
    std::fs::remove_file(test_db_path).unwrap();
}

