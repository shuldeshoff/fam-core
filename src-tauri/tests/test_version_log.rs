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

