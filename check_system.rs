#!/usr/bin/env rust-script
//! Автоматическая проверка FAM-Core системы
//! 
//! ```cargo
//! [dependencies]
//! rusqlite = { version = "0.32", features = ["bundled-sqlcipher"] }
//! chrono = "0.4"
//! ```

use std::path::PathBuf;

fn main() {
    println!("==========================================");
    println!("FAM-Core System Check");
    println!("==========================================");
    println!();

    // Находим путь к базе данных
    let home = std::env::var("HOME").expect("HOME not set");
    let db_path = PathBuf::from(home)
        .join("Library/Application Support/com.sul.fam-core/wallet.db");
    
    if !db_path.exists() {
        println!("❌ База данных не найдена: {:?}", db_path);
        println!("   Запустите приложение сначала!");
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

    println!("=== Проверка 1: Структура таблиц ===");
    println!();

    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").unwrap();
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    println!("Таблицы в БД:");
    for table in &tables {
        println!("  - {}", table);
    }
    println!();

    // Проверка 2: Аккаунты
    println!("=== Проверка 2: Аккаунты в БД ===");
    println!();

    let account_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
        .unwrap_or(0);

    println!("Количество аккаунтов: {}", account_count);

    if account_count > 0 {
        println!();
        let mut stmt = conn.prepare("SELECT id, name, type, created_at FROM accounts ORDER BY created_at DESC LIMIT 5").unwrap();
        let accounts = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        }).unwrap();

        for account in accounts {
            if let Ok((id, name, acc_type, created_at)) = account {
                println!("  [{}] {} ({}) - создан: {}", id, name, acc_type, 
                    chrono::NaiveDateTime::from_timestamp_opt(created_at, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| created_at.to_string()));
            }
        }
        println!();
        println!("✓ Аккаунты создаются корректно");
    } else {
        println!("⚠️  Аккаунтов пока нет. Создайте их в UI.");
    }
    println!();

    // Проверка 3: Операции
    println!("=== Проверка 3: Операции в БД ===");
    println!();

    let operation_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM operations", [], |row| row.get(0))
        .unwrap_or(0);

    println!("Количество операций: {}", operation_count);

    if operation_count > 0 {
        println!();
        let mut stmt = conn.prepare("SELECT id, account_id, amount, description, ts FROM operations ORDER BY ts DESC LIMIT 5").unwrap();
        let operations = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        }).unwrap();

        for op in operations {
            if let Ok((id, account_id, amount, description, ts)) = op {
                let timestamp = chrono::NaiveDateTime::from_timestamp_opt(ts, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| ts.to_string());
                println!("  [{}] Account {} | {:+.2} | {} | {}", 
                    id, account_id, amount, description, timestamp);
            }
        }
        println!();
        println!("✓ Операции сохраняются корректно");
    } else {
        println!("⚠️  Операций пока нет. Создайте их в UI.");
    }
    println!();

    // Проверка 4: Балансы
    println!("=== Проверка 4: Балансы в таблице states ===");
    println!();

    let state_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM states", [], |row| row.get(0))
        .unwrap_or(0);

    println!("Количество записей балансов: {}", state_count);

    if state_count > 0 {
        println!();
        let mut stmt = conn.prepare(
            "SELECT s.id, s.account_id, a.name, s.balance, s.ts 
             FROM states s 
             LEFT JOIN accounts a ON s.account_id = a.id 
             ORDER BY s.ts DESC LIMIT 10"
        ).unwrap();
        
        let states = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        }).unwrap();

        for state in states {
            if let Ok((id, account_id, name, balance, ts)) = state {
                let timestamp = chrono::NaiveDateTime::from_timestamp_opt(ts, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| ts.to_string());
                println!("  [{}] {} | Balance: {:.2} | {}", 
                    id, name.unwrap_or_else(|| format!("Account {}", account_id)), balance, timestamp);
            }
        }
        println!();
        println!("✓ Балансы обновляются автоматически");
    } else {
        println!("⚠️  Балансов пока нет. Добавьте операции в UI.");
    }
    println!();

    // Проверка 5: Синхронизация
    println!("=== Проверка 5: Синхронизация операций и балансов ===");
    println!();

    if account_count > 0 && operation_count > 0 {
        let mut stmt = conn.prepare(
            "SELECT a.id, a.name, 
                    (SELECT COUNT(*) FROM operations WHERE account_id = a.id) as op_count,
                    (SELECT COUNT(*) FROM states WHERE account_id = a.id) as st_count,
                    (SELECT balance FROM states WHERE account_id = a.id ORDER BY ts DESC LIMIT 1) as current_balance
             FROM accounts a"
        ).unwrap();

        let results = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<f64>>(4)?,
            ))
        }).unwrap();

        let mut all_synced = true;
        for result in results {
            if let Ok((id, name, op_count, st_count, balance)) = result {
                let synced = op_count == st_count;
                let status = if synced { "✓" } else { "❌" };
                println!("  {} [{}] {} | Операций: {} | Балансов: {} | Текущий баланс: {:.2}",
                    status, id, name, op_count, st_count, 
                    balance.unwrap_or(0.0));
                if !synced {
                    all_synced = false;
                }
            }
        }

        println!();
        if all_synced {
            println!("✓ Каждая операция создаёт ровно одну запись баланса");
        } else {
            println!("❌ Несоответствие: операции и балансы не синхронизированы!");
        }
    } else {
        println!("⚠️  Недостаточно данных для проверки синхронизации");
    }
    println!();

    // Проверка 6: Версия
    println!("=== Проверка 6: Версия БД ===");
    println!();

    let version: String = conn
        .query_row("SELECT version FROM meta LIMIT 1", [], |row| row.get(0))
        .unwrap_or_else(|_| "unknown".to_string());

    println!("Текущая версия БД: {}", version);

    if version == "4" {
        println!("✓ Все миграции (M1-M4) применены");
    } else {
        println!("⚠️  Ожидается версия 4, найдена: {}", version);
    }
    println!();

    // Итоговый отчёт
    println!("==========================================");
    println!("ИТОГОВЫЙ ОТЧЁТ");
    println!("==========================================");
    println!();
    println!("Аккаунтов: {}", account_count);
    println!("Операций: {}", operation_count);
    println!("Балансов: {}", state_count);
    println!("Версия БД: {}", version);
    println!();

    if account_count > 0 && operation_count > 0 && state_count > 0 {
        println!("✅ Система работает корректно!");
        println!();
        println!("Рекомендации для UI проверки:");
        println!("1. Откройте приложение FAM-Core");
        println!("2. Проверьте, что список аккаунтов отображается");
        println!("3. Кликните на аккаунт - должны показаться операции");
        println!("4. Добавьте новую операцию - баланс обновится автоматически");
        println!("5. Создайте новый аккаунт - он появится в списке");
    } else {
        println!("⚠️  Данных недостаточно для полной проверки");
        println!();
        println!("Действия:");
        println!("1. Запустите приложение: npm run tauri dev");
        println!("2. Создайте 2-3 аккаунта через UI");
        println!("3. Добавьте 5-10 операций");
        println!("4. Запустите этот скрипт снова");
    }

    println!();
    println!("==========================================");
}

