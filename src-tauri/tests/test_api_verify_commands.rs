use fam_core_lib::db;
use std::fs;
use rusqlite::Connection;

#[test]
fn test_verify_entry_api_simulation() {
    let db_path = "/tmp/test_api_verify_entry.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест API команды verify_entry ===\n");
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    
    // Создаём аккаунт
    let account_id = db::create_account(db_path, key, "Test Account".to_string(), "cash".to_string())
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
    
    // Симулируем вызов API: verify_entry(version_id)
    let is_valid = db::verify_version_signature(db_path, key, version_id)
        .expect("API call failed");
    
    assert!(is_valid, "Entry should be valid");
    println!("✓ API verify_entry({}) → {}", version_id, is_valid);
    
    // Подделываем данные
    conn.execute(
        "UPDATE version_log SET payload = '{\"tampered\":true}' WHERE id = ?",
        [version_id],
    ).expect("Failed to tamper");
    
    println!("✓ Данные подделаны");
    
    // Снова проверяем через API
    let is_valid_after = db::verify_version_signature(db_path, key, version_id)
        .expect("API call failed");
    
    assert!(!is_valid_after, "Tampered entry should be invalid");
    println!("✓ API verify_entry({}) → {} (после подделки)", version_id, is_valid_after);
    
    println!("\n✅ API команда verify_entry работает корректно!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_list_signed_versions_api_simulation() {
    let db_path = "/tmp/test_api_list_signed.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест API команды list_signed_versions ===\n");
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    
    // Создаём несколько аккаунтов
    let acc1 = db::create_account(db_path, key, "Account 1".to_string(), "cash".to_string())
        .expect("Failed to create account 1");
    let acc2 = db::create_account(db_path, key, "Account 2".to_string(), "deposit".to_string())
        .expect("Failed to create account 2");
    
    println!("✓ Создано 2 аккаунта: {}, {}", acc1, acc2);
    
    // Добавляем операцию (создаст 2 дополнительные записи)
    let op_id = db::add_operation(db_path, key, acc1, 1000.0, "Test op".to_string())
        .expect("Failed to add operation");
    
    println!("✓ Добавлена операция: ID = {}", op_id);
    
    // Симулируем вызов API: list_signed_versions()
    // Получаем все записи
    let versions = db::list_version_log(db_path, key, None, None)
        .expect("Failed to list versions");
    
    println!("\n✓ Найдено {} записей в version_log", versions.len());
    
    // Для каждой записи проверяем подпись (как это делает API)
    let mut results = Vec::new();
    for version in &versions {
        let is_valid = db::verify_version_signature(db_path, key, version.id)
            .unwrap_or(false);
        
        results.push((version.id, version.entity.clone(), is_valid));
    }
    
    println!("\n✓ Результаты верификации всех записей:");
    for (vid, entity, valid) in &results {
        println!("  Version ID={}, Entity={:10}, Valid={}", vid, entity, valid);
    }
    
    // Все должны быть валидны
    assert!(results.iter().all(|(_, _, valid)| *valid), "All entries should be valid");
    println!("\n✓ Все подписи валидны!");
    
    // Подделываем одну запись (например, вторую)
    let conn = Connection::open(db_path).expect("Failed to open connection");
    conn.pragma_update(None, "key", key).expect("Failed to set key");
    
    let tampered_id = versions[1].id;
    conn.execute(
        "UPDATE version_log SET payload = '{\"tampered\":true}' WHERE id = ?",
        [tampered_id],
    ).expect("Failed to tamper");
    
    println!("\n✓ Запись {} подделана", tampered_id);
    
    // Снова вызываем API
    let mut results_after = Vec::new();
    for version in &versions {
        let is_valid = db::verify_version_signature(db_path, key, version.id)
            .unwrap_or(false);
        
        results_after.push((version.id, version.entity.clone(), is_valid));
    }
    
    println!("\n✓ Результаты после подделки:");
    for (vid, entity, valid) in &results_after {
        let marker = if *vid == tampered_id { " ← TAMPERED" } else { "" };
        println!("  Version ID={}, Entity={:10}, Valid={}{}", vid, entity, valid, marker);
    }
    
    // Проверяем что только подделанная запись невалидна
    let tampered_count = results_after.iter().filter(|(_, _, valid)| !*valid).count();
    assert_eq!(tampered_count, 1, "Should have exactly 1 invalid entry");
    
    let valid_count = results_after.iter().filter(|(_, _, valid)| *valid).count();
    assert_eq!(valid_count, versions.len() - 1, "Other entries should remain valid");
    
    println!("\n✅ API команда list_signed_versions работает корректно!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_list_signed_versions_empty_db() {
    let db_path = "/tmp/test_api_list_signed_empty.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест list_signed_versions на пустой БД ===\n");
    
    // Инициализация БД (без создания записей)
    db::init_db(db_path, key).expect("Failed to init db");
    
    // Симулируем вызов API на пустой БД
    let versions = db::list_version_log(db_path, key, None, None)
        .expect("Failed to list versions");
    
    assert_eq!(versions.len(), 0, "Should have no versions in empty DB");
    println!("✓ Пустая БД → 0 записей");
    
    // API должен вернуть пустой список
    let results: Vec<(i64, bool)> = versions.iter()
        .map(|v| {
            let valid = db::verify_version_signature(db_path, key, v.id).unwrap_or(false);
            (v.id, valid)
        })
        .collect();
    
    assert_eq!(results.len(), 0, "API should return empty list");
    println!("✓ API list_signed_versions() → []");
    
    println!("\n✅ Обработка пустой БД работает!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_api_performance() {
    let db_path = "/tmp/test_api_performance.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест производительности API команд ===\n");
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    
    // Создаём 10 аккаунтов и 20 операций
    println!("✓ Создание тестовых данных...");
    let start = std::time::Instant::now();
    
    for i in 1..=10 {
        db::create_account(
            db_path, 
            key, 
            format!("Account {}", i), 
            "cash".to_string()
        ).expect("Failed to create account");
    }
    
    for i in 1..=20 {
        db::add_operation(
            db_path, 
            key, 
            ((i % 10) + 1) as i64, 
            100.0 * i as f64, 
            format!("Operation {}", i)
        ).expect("Failed to add operation");
    }
    
    let creation_time = start.elapsed();
    println!("  Время создания: {:?}", creation_time);
    
    // Получаем количество записей
    let versions = db::list_version_log(db_path, key, None, None)
        .expect("Failed to list versions");
    println!("✓ Всего записей: {}", versions.len());
    
    // Тест производительности list_signed_versions
    let start = std::time::Instant::now();
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for version in &versions {
        let is_valid = db::verify_version_signature(db_path, key, version.id)
            .unwrap_or(false);
        
        if is_valid {
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }
    
    let verification_time = start.elapsed();
    
    println!("\n✓ Результаты верификации:");
    println!("  Валидных: {}", valid_count);
    println!("  Невалидных: {}", invalid_count);
    println!("  Время: {:?}", verification_time);
    println!("  Среднее время на запись: {:?}", verification_time / versions.len() as u32);
    
    assert_eq!(valid_count, versions.len(), "All should be valid");
    assert_eq!(invalid_count, 0, "None should be invalid");
    
    println!("\n✅ Тест производительности пройден!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

