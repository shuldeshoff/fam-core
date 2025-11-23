use fam_core_lib::db;
use std::fs;
use rusqlite::Connection;

#[test]
fn test_verify_version_signature_valid() {
    let db_path = "/tmp/test_verify_signature.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест верификации валидной подписи ===\n");
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    println!("✓ База данных инициализирована");
    
    // Создаём аккаунт (автоматически создаётся подпись)
    let account_id = db::create_account(db_path, key, "Test Account".to_string(), "cash".to_string())
        .expect("Failed to create account");
    println!("✓ Аккаунт создан: ID = {}", account_id);
    
    // Получаем version_id для этого аккаунта
    let conn = Connection::open(db_path).expect("Failed to open connection");
    conn.pragma_update(None, "key", key).expect("Failed to set key");
    
    let version_id: i64 = conn.query_row(
        "SELECT id FROM version_log WHERE entity = 'account' AND entity_id = ?1",
        [account_id],
        |row| row.get(0),
    ).expect("Failed to get version_id");
    
    println!("✓ Version ID найден: {}", version_id);
    
    // Верифицируем подпись
    let is_valid = db::verify_version_signature(db_path, key, version_id)
        .expect("Failed to verify signature");
    
    assert!(is_valid, "Signature should be valid");
    println!("✓ Подпись верифицирована: валидна = {}", is_valid);
    
    println!("\n✅ Тест успешно пройден!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_verify_version_signature_invalid() {
    let db_path = "/tmp/test_verify_signature_invalid.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест верификации подделанной подписи ===\n");
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    
    // Создаём аккаунт
    let account_id = db::create_account(db_path, key, "Original Account".to_string(), "cash".to_string())
        .expect("Failed to create account");
    println!("✓ Аккаунт создан: ID = {}", account_id);
    
    // Получаем version_id
    let conn = Connection::open(db_path).expect("Failed to open connection");
    conn.pragma_update(None, "key", key).expect("Failed to set key");
    
    let version_id: i64 = conn.query_row(
        "SELECT id FROM version_log WHERE entity = 'account' AND entity_id = ?1",
        [account_id],
        |row| row.get(0),
    ).expect("Failed to get version_id");
    
    println!("✓ Version ID: {}", version_id);
    
    // Подделываем payload в version_log
    conn.execute(
        "UPDATE version_log SET payload = ? WHERE id = ?",
        [r#"{"id":1,"name":"TAMPERED","acc_type":"cash","created_at":1234567890}"#, &version_id.to_string()],
    ).expect("Failed to tamper payload");
    
    println!("✓ Payload подделан");
    
    // Пытаемся верифицировать подпись
    let is_valid = db::verify_version_signature(db_path, key, version_id)
        .expect("Failed to verify signature");
    
    assert!(!is_valid, "Tampered signature should be invalid");
    println!("✓ Подпись верифицирована: валидна = {} (ожидалось false)", is_valid);
    
    println!("\n✅ Защита от подмены работает!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_verify_version_signature_not_found() {
    let db_path = "/tmp/test_verify_signature_not_found.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест верификации несуществующей записи ===\n");
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    
    // Пытаемся верифицировать несуществующий version_id
    let result = db::verify_version_signature(db_path, key, 999);
    
    assert!(result.is_err(), "Should return error for non-existent version_id");
    println!("✓ Ошибка корректно возвращена для несуществующей записи");
    println!("  Ошибка: {}", result.unwrap_err());
    
    println!("\n✅ Обработка ошибок работает!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_verify_multiple_versions() {
    let db_path = "/tmp/test_verify_multiple.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест верификации множественных записей ===\n");
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    
    // Создаём несколько аккаунтов
    let acc1 = db::create_account(db_path, key, "Account 1".to_string(), "cash".to_string())
        .expect("Failed to create account 1");
    let acc2 = db::create_account(db_path, key, "Account 2".to_string(), "deposit".to_string())
        .expect("Failed to create account 2");
    let acc3 = db::create_account(db_path, key, "Account 3".to_string(), "investment".to_string())
        .expect("Failed to create account 3");
    
    println!("✓ Создано 3 аккаунта: {}, {}, {}", acc1, acc2, acc3);
    
    // Добавляем операцию (создаст ещё 2 записи: operation + state)
    let op_id = db::add_operation(db_path, key, acc1, 1000.0, "Test op".to_string())
        .expect("Failed to add operation");
    
    println!("✓ Добавлена операция: ID = {}", op_id);
    
    // Получаем все version_id
    let conn = Connection::open(db_path).expect("Failed to open connection");
    conn.pragma_update(None, "key", key).expect("Failed to set key");
    
    let mut stmt = conn.prepare("SELECT id FROM version_log ORDER BY id").expect("Failed to prepare");
    let version_ids: Vec<i64> = stmt.query_map([], |row| row.get(0))
        .expect("Failed to query")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect");
    
    println!("✓ Найдено {} записей в version_log", version_ids.len());
    
    // Верифицируем все записи
    println!("\n✓ Верификация всех записей:");
    for (i, &version_id) in version_ids.iter().enumerate() {
        let is_valid = db::verify_version_signature(db_path, key, version_id)
            .expect(&format!("Failed to verify version {}", version_id));
        assert!(is_valid, "Version {} should be valid", version_id);
        println!("  [{}] Version ID={}, Valid={}", i + 1, version_id, is_valid);
    }
    
    // Подделываем одну запись
    let tampered_id = version_ids[1];
    conn.execute(
        "UPDATE version_log SET payload = '{\"tampered\":true}' WHERE id = ?",
        [tampered_id],
    ).expect("Failed to tamper");
    
    println!("\n✓ Запись {} подделана", tampered_id);
    
    // Проверяем что только эта запись стала невалидной
    println!("\n✓ Повторная верификация:");
    for (i, &version_id) in version_ids.iter().enumerate() {
        let is_valid = db::verify_version_signature(db_path, key, version_id)
            .expect(&format!("Failed to verify version {}", version_id));
        
        if version_id == tampered_id {
            assert!(!is_valid, "Tampered version {} should be invalid", version_id);
            println!("  [{}] Version ID={}, Valid={} ← TAMPERED", i + 1, version_id, is_valid);
        } else {
            assert!(is_valid, "Version {} should still be valid", version_id);
            println!("  [{}] Version ID={}, Valid={}", i + 1, version_id, is_valid);
        }
    }
    
    println!("\n✅ Селективная верификация работает корректно!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

