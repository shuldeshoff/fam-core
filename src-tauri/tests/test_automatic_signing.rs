use fam_core_lib::{db, crypto};
use std::fs;
use rusqlite::Connection;

#[test]
fn test_automatic_version_log_signing() {
    let db_path = "/tmp/test_version_log_signing.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест автоматической подписи version_log ===\n");
    
    // Инициализация БД (создаются Ed25519 ключи автоматически)
    db::init_db(db_path, key).expect("Failed to init db");
    println!("✓ База данных инициализирована (ключи созданы)");
    
    // Создаём аккаунт (должен автоматически создать подпись)
    let account_id = db::create_account(db_path, key, "Test Account".to_string(), "cash".to_string())
        .expect("Failed to create account");
    println!("✓ Аккаунт создан: ID = {}", account_id);
    
    // Открываем соединение для проверки
    let conn = Connection::open(db_path).expect("Failed to open connection");
    conn.pragma_update(None, "key", key).expect("Failed to set key");
    
    // Проверяем что в version_log есть запись
    let (log_id, log_payload, log_ts): (i64, String, i64) = conn.query_row(
        "SELECT id, payload, ts FROM version_log WHERE entity = 'account' AND entity_id = ?1",
        [account_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).expect("Failed to get version_log");
    
    println!("✓ Запись в version_log найдена: ID = {}, payload size = {} bytes", 
        log_id, log_payload.len());
    
    // Проверяем что в version_signatures есть подпись
    let (sig_id, signature, public_key, sig_ts): (i64, Vec<u8>, Vec<u8>, i64) = conn.query_row(
        "SELECT id, signature, public_key, ts FROM version_signatures WHERE version_id = ?1",
        [log_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).expect("Failed to get signature");
    
    println!("✓ Подпись найдена: ID = {}", sig_id);
    println!("  Signature: {} bytes", signature.len());
    println!("  Public key: {} bytes", public_key.len());
    println!("  Timestamp: {} ({})", sig_ts, 
        if sig_ts == log_ts { "same as log" } else { "different from log" });
    
    // Проверяем размеры
    assert_eq!(signature.len(), 64, "Signature should be 64 bytes");
    assert_eq!(public_key.len(), 32, "Public key should be 32 bytes");
    assert_eq!(sig_ts, log_ts, "Signature timestamp should match log timestamp");
    println!("✓ Размеры корректны");
    
    // Верифицируем подпись
    let payload_bytes = log_payload.as_bytes();
    let is_valid = crypto::verify_payload(payload_bytes, &signature, &public_key)
        .expect("Failed to verify signature");
    
    assert!(is_valid, "Signature should be valid");
    println!("✓ Подпись верифицирована успешно!");
    
    // Создаём операцию (должна создать 2 подписи: для operation и для state)
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    let operation_id = db::add_operation(db_path, key, account_id, 1000.0, "Test operation".to_string())
        .expect("Failed to add operation");
    println!("\n✓ Операция добавлена: ID = {}", operation_id);
    
    // Проверяем количество записей в version_log
    let log_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM version_log",
        [],
        |row| row.get(0),
    ).expect("Failed to count version_log");
    
    // Должно быть 3 записи: 1 account + 1 operation + 1 state
    assert_eq!(log_count, 3, "Should have 3 version_log entries");
    println!("✓ Всего записей в version_log: {}", log_count);
    
    // Проверяем количество подписей
    let sig_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM version_signatures",
        [],
        |row| row.get(0),
    ).expect("Failed to count signatures");
    
    // Должно быть 3 подписи (по одной на каждую запись)
    assert_eq!(sig_count, 3, "Should have 3 signatures");
    println!("✓ Всего подписей в version_signatures: {}", sig_count);
    
    // Верифицируем все подписи
    let mut stmt = conn.prepare(
        "SELECT vl.id, vl.payload, vs.signature, vs.public_key
         FROM version_log vl
         INNER JOIN version_signatures vs ON vs.version_id = vl.id"
    ).expect("Failed to prepare");
    
    let signatures: Vec<(i64, String, Vec<u8>, Vec<u8>)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })
    .expect("Failed to query")
    .collect::<Result<Vec<_>, _>>()
    .expect("Failed to collect");
    
    println!("\n✓ Верификация всех подписей:");
    for (i, (log_id, payload, sig, pk)) in signatures.iter().enumerate() {
        let valid = crypto::verify_payload(payload.as_bytes(), sig, pk)
            .expect("Failed to verify");
        assert!(valid, "Signature {} should be valid", i + 1);
        println!("  [{}] Log ID={}, Valid={}", i + 1, log_id, valid);
    }
    
    // Проверяем что подписи уникальны
    let unique_sigs: std::collections::HashSet<_> = signatures.iter()
        .map(|(_, _, sig, _)| sig.clone())
        .collect();
    assert_eq!(unique_sigs.len(), signatures.len(), "All signatures should be unique");
    println!("✓ Все подписи уникальны");
    
    // Проверяем CASCADE DELETE
    conn.execute("DELETE FROM version_log WHERE entity = 'account'", [])
        .expect("Failed to delete log");
    
    let remaining_sigs: i64 = conn.query_row(
        "SELECT COUNT(*) FROM version_signatures",
        [],
        |row| row.get(0),
    ).expect("Failed to count remaining signatures");
    
    // Должно остаться 2 подписи (operation и state)
    assert_eq!(remaining_sigs, 2, "Should have 2 signatures after deletion");
    println!("✓ CASCADE DELETE работает (осталось {} подписей)", remaining_sigs);
    
    println!("\n✅ Все тесты автоматической подписи пройдены!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_signature_verification_with_wrong_data() {
    let db_path = "/tmp/test_signature_tampering.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест защиты от подмены данных ===\n");
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    
    // Создаём аккаунт
    let account_id = db::create_account(db_path, key, "Original Account".to_string(), "cash".to_string())
        .expect("Failed to create account");
    println!("✓ Аккаунт создан: ID = {}", account_id);
    
    // Открываем соединение
    let conn = Connection::open(db_path).expect("Failed to open connection");
    conn.pragma_update(None, "key", key).expect("Failed to set key");
    
    // Получаем оригинальную подпись
    let (_log_id, original_payload, signature, public_key): (i64, String, Vec<u8>, Vec<u8>) = conn.query_row(
        "SELECT vl.id, vl.payload, vs.signature, vs.public_key
         FROM version_log vl
         INNER JOIN version_signatures vs ON vs.version_id = vl.id
         WHERE vl.entity = 'account'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).expect("Failed to get data");
    
    // Верифицируем оригинальную подпись
    let valid_original = crypto::verify_payload(original_payload.as_bytes(), &signature, &public_key)
        .expect("Failed to verify original");
    assert!(valid_original, "Original signature should be valid");
    println!("✓ Оригинальная подпись валидна");
    
    // Подделываем payload (меняем имя аккаунта в JSON)
    let tampered_payload = original_payload.replace("Original Account", "Tampered Account");
    
    // Пытаемся верифицировать с подделанным payload
    let valid_tampered = crypto::verify_payload(tampered_payload.as_bytes(), &signature, &public_key)
        .expect("Failed to verify tampered");
    
    assert!(!valid_tampered, "Tampered payload should not verify");
    println!("✓ Подделанный payload НЕ прошёл верификацию (защита работает!)");
    
    println!("\n✅ Защита от подмены данных работает!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

