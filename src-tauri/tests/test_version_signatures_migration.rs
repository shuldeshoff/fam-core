use fam_core_lib::db;
use std::fs;
use rusqlite::Connection;

#[test]
fn test_version_signatures_migration() {
    let db_path = "/tmp/test_version_signatures.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест миграции M7: version_signatures ===\n");
    
    // Инициализация БД (должна выполнить миграцию M7)
    db::init_db(db_path, key).expect("Failed to init db");
    println!("✓ База данных инициализирована");
    
    // Проверяем версию БД
    let version = db::get_db_version(db_path, key).expect("Failed to get version");
    println!("✓ Версия БД: {}", version);
    assert!(version.parse::<i32>().unwrap() >= 7, "Database version should be at least 7");
    
    // Открываем соединение для проверки структуры
    let conn = Connection::open(db_path).expect("Failed to open connection");
    conn.pragma_update(None, "key", key).expect("Failed to set key");
    
    // Проверяем существование таблицы version_signatures
    let table_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='version_signatures'",
        [],
        |row| row.get(0),
    ).expect("Failed to check table");
    
    assert_eq!(table_exists, 1, "Table version_signatures should exist");
    println!("✓ Таблица version_signatures создана");
    
    // Проверяем структуру таблицы
    let mut stmt = conn.prepare(
        "PRAGMA table_info(version_signatures)"
    ).expect("Failed to prepare pragma");
    
    let columns: Vec<(String, String)> = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    })
    .expect("Failed to query")
    .collect::<Result<Vec<_>, _>>()
    .expect("Failed to collect");
    
    println!("✓ Структура таблицы:");
    for (name, col_type) in &columns {
        println!("  - {} ({})", name, col_type);
    }
    
    // Проверяем обязательные столбцы
    assert!(columns.iter().any(|(n, _)| n == "id"), "Column 'id' should exist");
    assert!(columns.iter().any(|(n, _)| n == "version_id"), "Column 'version_id' should exist");
    assert!(columns.iter().any(|(n, _)| n == "signature"), "Column 'signature' should exist");
    assert!(columns.iter().any(|(n, _)| n == "public_key"), "Column 'public_key' should exist");
    assert!(columns.iter().any(|(n, _)| n == "ts"), "Column 'ts' should exist");
    
    // Проверяем типы столбцов
    for (name, col_type) in &columns {
        match name.as_str() {
            "id" | "version_id" | "ts" => {
                assert!(col_type.contains("INTEGER"), "{} should be INTEGER", name);
            }
            "signature" | "public_key" => {
                assert!(col_type.contains("BLOB"), "{} should be BLOB", name);
            }
            _ => {}
        }
    }
    println!("✓ Все столбцы имеют корректные типы");
    
    // Проверяем индексы
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='version_signatures'"
    ).expect("Failed to prepare");
    
    let indexes: Vec<String> = stmt.query_map([], |row| {
        row.get(0)
    })
    .expect("Failed to query")
    .collect::<Result<Vec<_>, _>>()
    .expect("Failed to collect");
    
    println!("✓ Индексы:");
    for index in &indexes {
        println!("  - {}", index);
    }
    
    assert!(
        indexes.iter().any(|i| i.contains("version_id")),
        "Index on version_id should exist"
    );
    assert!(
        indexes.iter().any(|i| i.contains("ts")),
        "Index on ts should exist"
    );
    println!("✓ Все необходимые индексы созданы");
    
    // Проверяем что можем вставить данные
    let test_signature = vec![0u8; 64]; // 64-байтная подпись
    let test_public_key = vec![1u8; 32]; // 32-байтный ключ
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // Сначала создаём запись в version_log (для внешнего ключа)
    conn.execute(
        "INSERT INTO version_log (entity, entity_id, action, payload, ts) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["test", 1, "create", "{}", ts],
    ).expect("Failed to insert version_log");
    
    let version_log_id = conn.last_insert_rowid();
    
    // Вставляем подпись
    conn.execute(
        "INSERT INTO version_signatures (version_id, signature, public_key, ts) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![version_log_id, &test_signature, &test_public_key, ts],
    ).expect("Failed to insert signature");
    
    println!("✓ Тестовая запись вставлена");
    
    // Читаем обратно
    let (sig, pk): (Vec<u8>, Vec<u8>) = conn.query_row(
        "SELECT signature, public_key FROM version_signatures WHERE version_id = ?1",
        [version_log_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).expect("Failed to read signature");
    
    assert_eq!(sig.len(), 64, "Signature should be 64 bytes");
    assert_eq!(pk.len(), 32, "Public key should be 32 bytes");
    println!("✓ Данные корректно читаются (signature: {} bytes, public_key: {} bytes)", sig.len(), pk.len());
    
    // Проверяем внешний ключ (CASCADE DELETE)
    conn.execute(
        "DELETE FROM version_log WHERE id = ?1",
        [version_log_id],
    ).expect("Failed to delete version_log");
    
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM version_signatures WHERE version_id = ?1",
        [version_log_id],
        |row| row.get(0),
    ).expect("Failed to count");
    
    assert_eq!(count, 0, "Signature should be deleted when version_log is deleted (CASCADE)");
    println!("✓ CASCADE DELETE работает корректно");
    
    println!("\n✅ Миграция M7 успешно выполнена и протестирована!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

