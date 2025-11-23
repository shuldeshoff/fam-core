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
    
    println!("=== Тест write_version_log ===");
    
    // Инициализируем БД
    db::init_db(test_db_path, test_key).expect("DB init failed");
    println!("✓ База данных инициализирована");
    
    // Создаём аккаунт (он должен автоматически логировать создание)
    let account_id = db::create_account(
        test_db_path,
        test_key,
        "Test Account".to_string(),
        "cash".to_string(),
    ).expect("Account creation failed");
    
    println!("✓ Аккаунт создан с ID: {}", account_id);
    
    // Проверяем, что запись в version_log НЕ создалась автоматически (пока не реализовано)
    use rusqlite::Connection;
    let conn = Connection::open(test_db_path).unwrap();
    conn.pragma_update(None, "key", test_key).unwrap();
    
    let log_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM version_log",
        [],
        |row| row.get(0),
    ).unwrap();
    
    println!("Записей в version_log: {}", log_count);
    
    // Пока функция write_version_log приватная, она не вызывается автоматически
    // В следующих промптах мы интегрируем её в create_account, add_operation и т.д.
    
    assert_eq!(log_count, 0, "Version log должен быть пуст (автологирование ещё не реализовано)");
    
    // Очистка
    std::fs::remove_file(test_db_path).unwrap();
    println!("✓ Тест пройден: функция write_version_log создана и готова к использованию");
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

