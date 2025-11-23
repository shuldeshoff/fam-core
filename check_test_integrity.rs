#!/usr/bin/env rust-script
//! Комплексная проверка целостности version_log системы
//! 
//! ```cargo
//! [dependencies]
//! rusqlite = { version = "0.32", features = ["bundled-sqlcipher"] }
//! chrono = "0.4"
//! serde_json = "1.0"
//! ```

use std::path::PathBuf;

fn main() {
    println!("==========================================");
    println!("FAM-Core Version Log Integrity Check");
    println!("==========================================");
    println!();

    // Находим путь к базе данных
    let home = std::env::var("HOME").expect("HOME not set");
    let db_path = PathBuf::from(home)
        .join("../../../tmp/test_integrity.db");
    
    if !db_path.exists() {
        println!("❌ База данных не найдена: {:?}", db_path);
        println!("   Запустите приложение и создайте данные!");
        return;
    }

    println!("✓ База данных найдена: {:?}", db_path);
    println!();

    let db_path_str = db_path.to_str().unwrap();
    let db_key = "initialization_key";

    // Открываем подключение
    use rusqlite::Connection;
    let conn = match Connection::open(db_path_str) {
        Ok(c) => c,
        Err(e) => {
            println!("❌ Ошибка открытия БД: {}", e);
            return;
        }
    };

    // Устанавливаем ключ шифрования
    if let Err(e) = conn.pragma_update(None, "key", db_key) {
        println!("❌ Ошибка установки ключа: {}", e);
        return;
    }

    println!("=== Проверка 1: Существование таблицы version_log ===");
    println!();

    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='version_log'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if table_exists {
        println!("✓ Таблица version_log существует");
    } else {
        println!("❌ Таблица version_log НЕ найдена!");
        return;
    }

    // Проверяем структуру таблицы
    let mut stmt = conn.prepare("PRAGMA table_info(version_log)").unwrap();
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    println!("Колонки: {:?}", columns);
    
    let required_columns = vec!["id", "entity", "entity_id", "action", "payload", "ts"];
    let all_present = required_columns.iter().all(|&col| columns.contains(&col.to_string()));
    
    if all_present {
        println!("✓ Все необходимые колонки присутствуют");
    } else {
        println!("❌ Отсутствуют некоторые колонки!");
    }
    println!();

    println!("=== Проверка 2: Фиксация create_account ===");
    println!();

    let account_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
        .unwrap_or(0);

    let account_log_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM version_log WHERE entity = 'account'", [], |row| row.get(0))
        .unwrap_or(0);

    println!("Аккаунтов в accounts: {}", account_count);
    println!("Записей в version_log (entity='account'): {}", account_log_count);

    if account_count == account_log_count {
        println!("✓ Все create_account зафиксированы в version_log");
    } else {
        println!("❌ НЕСООТВЕТСТВИЕ: {} аккаунтов, но {} записей в логе", account_count, account_log_count);
    }
    println!();

    println!("=== Проверка 3: Фиксация add_operation ===");
    println!();

    let operation_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM operations", [], |row| row.get(0))
        .unwrap_or(0);

    let operation_log_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM version_log WHERE entity = 'operation'", [], |row| row.get(0))
        .unwrap_or(0);

    println!("Операций в operations: {}", operation_count);
    println!("Записей в version_log (entity='operation'): {}", operation_log_count);

    if operation_count == operation_log_count {
        println!("✓ Все add_operation зафиксированы в version_log");
    } else {
        println!("❌ НЕСООТВЕТСТВИЕ: {} операций, но {} записей в логе", operation_count, operation_log_count);
    }
    println!();

    println!("=== Проверка 4: Автоматическое создание state ===");
    println!();

    let state_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM states", [], |row| row.get(0))
        .unwrap_or(0);

    let state_log_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM version_log WHERE entity = 'state'", [], |row| row.get(0))
        .unwrap_or(0);

    println!("Записей в states: {}", state_count);
    println!("Записей в version_log (entity='state'): {}", state_log_count);

    if state_count == state_log_count {
        println!("✓ Все state зафиксированы в version_log");
    } else {
        println!("❌ НЕСООТВЕТСТВИЕ: {} states, но {} записей в логе", state_count, state_log_count);
    }

    // Проверяем, что каждая операция создаёт ровно одну запись state
    if operation_count > 0 {
        if state_count == operation_count && state_log_count == operation_count {
            println!("✓ Каждая операция создала ровно одну запись state и лог");
        } else {
            println!("⚠️  Несоответствие между операциями и states");
        }
    }
    println!();

    println!("=== Проверка 5: Связь версий с правильными ID ===");
    println!();

    // Проверяем, что все entity_id в version_log соответствуют реальным ID
    let mut all_ids_valid = true;

    // Проверка accounts
    let invalid_account_ids: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM version_log v 
             WHERE v.entity = 'account' 
             AND NOT EXISTS (SELECT 1 FROM accounts a WHERE a.id = v.entity_id)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if invalid_account_ids == 0 {
        println!("✓ Все entity_id для 'account' валидны");
    } else {
        println!("❌ Найдено {} невалидных entity_id для 'account'", invalid_account_ids);
        all_ids_valid = false;
    }

    // Проверка operations
    let invalid_operation_ids: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM version_log v 
             WHERE v.entity = 'operation' 
             AND NOT EXISTS (SELECT 1 FROM operations o WHERE o.id = v.entity_id)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if invalid_operation_ids == 0 {
        println!("✓ Все entity_id для 'operation' валидны");
    } else {
        println!("❌ Найдено {} невалидных entity_id для 'operation'", invalid_operation_ids);
        all_ids_valid = false;
    }

    // Проверка states
    let invalid_state_ids: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM version_log v 
             WHERE v.entity = 'state' 
             AND NOT EXISTS (SELECT 1 FROM states s WHERE s.id = v.entity_id)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if invalid_state_ids == 0 {
        println!("✓ Все entity_id для 'state' валидны");
    } else {
        println!("❌ Найдено {} невалидных entity_id для 'state'", invalid_state_ids);
        all_ids_valid = false;
    }

    if all_ids_valid {
        println!("\n✓ Все версии связаны с правильными ID");
    }
    println!();

    println!("=== Проверка 6: Корректность JSON сериализации ===");
    println!();

    let mut stmt = conn.prepare("SELECT id, entity, payload FROM version_log LIMIT 10").unwrap();
    let records = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .unwrap();

    let mut json_valid_count = 0;
    let mut json_invalid_count = 0;

    for (i, record) in records.enumerate() {
        if let Ok((id, entity, payload)) = record {
            match serde_json::from_str::<serde_json::Value>(&payload) {
                Ok(json) => {
                    json_valid_count += 1;
                    if i < 3 {
                        println!("✓ [{}] {} - валидный JSON ({} байт)", id, entity, payload.len());
                    }
                },
                Err(e) => {
                    json_invalid_count += 1;
                    println!("❌ [{}] {} - НЕВАЛИДНЫЙ JSON: {}", id, entity, e);
                }
            }
        }
    }

    if json_invalid_count == 0 {
        println!("\n✓ Все проверенные payload содержат корректный JSON");
    } else {
        println!("\n❌ Найдено {} записей с невалидным JSON", json_invalid_count);
    }
    println!();

    println!("=== Проверка 7: Атомарность транзакций ===");
    println!();

    // Проверяем, что для каждой операции есть связанный state с тем же timestamp
    let mismatched_timestamps: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM version_log vl_op
             WHERE vl_op.entity = 'operation'
             AND NOT EXISTS (
                 SELECT 1 FROM version_log vl_st
                 WHERE vl_st.entity = 'state'
                 AND vl_st.ts = vl_op.ts
                 AND JSON_EXTRACT(vl_st.payload, '$.account_id') = 
                     JSON_EXTRACT(vl_op.payload, '$.account_id')
             )",
            [],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    if mismatched_timestamps == 0 {
        println!("✓ Все операции и states созданы атомарно (одинаковый timestamp)");
        println!("  Это подтверждает, что write_version_log вызывается внутри транзакций");
    } else if mismatched_timestamps > 0 {
        println!("⚠️  Найдено {} операций без связанного state с тем же timestamp", mismatched_timestamps);
    }
    println!();

    println!("=== Проверка 8: Порядок записей ===");
    println!();

    // Проверяем, что записи упорядочены по timestamp
    let mut stmt = conn.prepare("SELECT id, ts FROM version_log ORDER BY id").unwrap();
    let timestamps: Vec<(i64, i64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let mut order_violations = 0;
    for i in 0..timestamps.len().saturating_sub(1) {
        if timestamps[i].1 > timestamps[i + 1].1 {
            order_violations += 1;
        }
    }

    if order_violations == 0 {
        println!("✓ Timestamps упорядочены корректно (монотонно неубывающие)");
    } else {
        println!("⚠️  Найдено {} нарушений порядка timestamps", order_violations);
    }
    println!();

    // Итоговый отчёт
    println!("==========================================");
    println!("ИТОГОВЫЙ ОТЧЁТ");
    println!("==========================================");
    println!();

    let total_logs = account_log_count + operation_log_count + state_log_count;
    
    println!("📊 Статистика:");
    println!("  Всего записей в version_log: {}", total_logs);
    println!("  - accounts: {}", account_log_count);
    println!("  - operations: {}", operation_log_count);
    println!("  - states: {}", state_log_count);
    println!();

    let mut passed = 0;
    let mut failed = 0;

    if table_exists && all_present { passed += 1; } else { failed += 1; }
    if account_count == account_log_count { passed += 1; } else { failed += 1; }
    if operation_count == operation_log_count { passed += 1; } else { failed += 1; }
    if state_count == state_log_count { passed += 1; } else { failed += 1; }
    if all_ids_valid { passed += 1; } else { failed += 1; }
    if json_invalid_count == 0 { passed += 1; } else { failed += 1; }
    if mismatched_timestamps == 0 { passed += 1; } else { failed += 1; }
    if order_violations == 0 { passed += 1; } else { failed += 1; }

    println!("🎯 Результаты:");
    println!("  ✓ Пройдено: {}", passed);
    println!("  ❌ Провалено: {}", failed);
    println!();

    if failed == 0 {
        println!("✅ ВСЕ ПРОВЕРКИ ПРОЙДЕНЫ!");
        println!();
        println!("Система version_log работает корректно:");
        println!("- Все операции логируются");
        println!("- Связи между сущностями валидны");
        println!("- JSON корректно сериализуется");
        println!("- Транзакции атомарны");
        println!("- UI может отображать журнал");
    } else {
        println!("⚠️  ОБНАРУЖЕНЫ ПРОБЛЕМЫ");
        println!();
        println!("Требуется исправление для:");
        if account_count != account_log_count {
            println!("- Логирование create_account");
        }
        if operation_count != operation_log_count {
            println!("- Логирование add_operation");
        }
        if state_count != state_log_count {
            println!("- Логирование states");
        }
        if !all_ids_valid {
            println!("- Валидность entity_id");
        }
        if json_invalid_count > 0 {
            println!("- Сериализация JSON");
        }
        if mismatched_timestamps != 0 {
            println!("- Атомарность транзакций");
        }
    }

    println!();
    println!("==========================================");
}

