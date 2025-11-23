#!/usr/bin/env rust-script
//! Test SQLCipher и миграций после M2
//! 
//! Запуск: cd src-tauri && cargo test --test test_sqlcipher

use fam_core_lib::db;
use std::fs;
use std::path::Path;

#[test]
fn test_sqlcipher_and_migrations() {
    // Используем временный файл для теста
    let test_db_path = "/tmp/test_fam_core_wallet.db";
    let test_key = "test_key_12345";
    
    // Удаляем старую БД если есть
    if Path::new(test_db_path).exists() {
        fs::remove_file(test_db_path).unwrap();
    }
    
    println!("=== Тест 1: Инициализация БД с шифрованием ===");
    
    // Инициализируем БД
    let init_result = db::init_db(test_db_path, test_key);
    assert!(init_result.is_ok(), "Инициализация БД должна быть успешной");
    println!("✓ БД инициализирована");
    
    println!("\n=== Тест 2: Проверка шифрования ===");
    
    // Попытка открыть с неправильным ключом должна провалиться
    let wrong_result = db::get_db_version(test_db_path, "wrong_key");
    assert!(wrong_result.is_err(), "Неправильный ключ должен давать ошибку");
    println!("✓ Неправильный ключ отклонён");
    
    // С правильным ключом должно работать
    let version_result = db::get_db_version(test_db_path, test_key);
    assert!(version_result.is_ok(), "Правильный ключ должен работать");
    let version = version_result.unwrap();
    println!("✓ Правильный ключ принят, версия БД: {}", version);
    
    println!("\n=== Тест 3: Проверка миграций (M2 - accounts) ===");
    
    // Создаём счёт
    let account_result = db::create_account(
        test_db_path,
        test_key,
        "Тестовый счёт".to_string(),
        "cash".to_string(),
    );
    assert!(account_result.is_ok(), "Создание счёта должно быть успешным");
    let account_id = account_result.unwrap();
    println!("✓ Счёт создан с ID: {}", account_id);
    
    // Получаем список счетов
    let accounts_result = db::list_accounts(test_db_path, test_key);
    assert!(accounts_result.is_ok(), "Получение списка счетов должно работать");
    let accounts = accounts_result.unwrap();
    assert_eq!(accounts.len(), 1, "Должен быть 1 счёт");
    assert_eq!(accounts[0].name, "Тестовый счёт");
    assert_eq!(accounts[0].acc_type, "cash");
    println!("✓ Счёт найден в БД: {} ({})", accounts[0].name, accounts[0].acc_type);
    
    println!("\n=== Тест 4: Проверка операций (M3 - operations) ===");
    
    // Добавляем операцию
    let op_result = db::add_operation(
        test_db_path,
        test_key,
        account_id,
        100.50,
        "Тестовая операция".to_string(),
    );
    assert!(op_result.is_ok(), "Добавление операции должно быть успешным");
    let op_id = op_result.unwrap();
    println!("✓ Операция создана с ID: {}", op_id);
    
    // Получаем операции
    let ops_result = db::get_operations(test_db_path, test_key, account_id);
    assert!(ops_result.is_ok(), "Получение операций должно работать");
    let operations = ops_result.unwrap();
    assert_eq!(operations.len(), 1, "Должна быть 1 операция");
    assert_eq!(operations[0].amount, 100.50);
    assert_eq!(operations[0].description, "Тестовая операция");
    println!("✓ Операция найдена: {} (сумма: {})", operations[0].description, operations[0].amount);
    
    println!("\n=== Тест 5: Проверка баланса (M4 - states) ===");
    
    // Проверяем, что создалась запись баланса
    {
        use rusqlite::Connection;
        let conn = Connection::open(test_db_path).unwrap();
        conn.pragma_update(None, "key", test_key).unwrap();
        
        let balance: f64 = conn
            .query_row(
                "SELECT balance FROM states WHERE account_id = ?1 ORDER BY ts DESC LIMIT 1",
                [account_id],
                |row| row.get(0),
            )
            .unwrap();
        
        assert_eq!(balance, 100.50, "Баланс должен быть 100.50");
        println!("✓ Баланс корректен: {}", balance);
    } // Закрываем соединение
    
    // Добавляем задержку, чтобы timestamp отличался
    std::thread::sleep(std::time::Duration::from_millis(1100));
    
    // Добавляем ещё одну операцию
    let op2_result = db::add_operation(
        test_db_path,
        test_key,
        account_id,
        -20.30,
        "Расход".to_string(),
    );
    assert!(op2_result.is_ok(), "Вторая операция должна добавиться: {:?}", op2_result.err());
    println!("✓ Вторая операция добавлена");
    
    println!("\n=== Тест 6: Атомарность транзакций ===");
    
    // Проверяем новый баланс и количество записей
    {
        use rusqlite::Connection;
        let conn = Connection::open(test_db_path).unwrap();
        conn.pragma_update(None, "key", test_key).unwrap();
        
        // Проверяем новый баланс
        let new_balance: f64 = conn
            .query_row(
                "SELECT balance FROM states WHERE account_id = ?1 ORDER BY ts DESC LIMIT 1",
                [account_id],
                |row| row.get(0),
            )
            .unwrap();
        
        let expected_balance = 100.50 - 20.30;
        assert!((new_balance - expected_balance).abs() < 0.01, "Баланс должен быть {}", expected_balance);
        println!("✓ Баланс после второй операции: {} (ожидалось: {})", new_balance, expected_balance);
        
        // Проверяем количество записей в states
        let states_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM states WHERE account_id = ?1",
                [account_id],
                |row| row.get(0),
            )
            .unwrap();
        
        assert_eq!(states_count, 2, "Должно быть 2 записи баланса (по одной на операцию)");
        println!("✓ Каждая операция создала запись баланса: {} записей", states_count);
    }
    
    // Очистка
    fs::remove_file(test_db_path).unwrap();
    println!("\n=== ✅ Все тесты пройдены успешно! ===");
}

#[test]
fn test_migration_version_tracking() {
    let test_db_path = "/tmp/test_fam_core_version.db";
    let test_key = "version_test_key";
    
    // Удаляем старую БД
    if Path::new(test_db_path).exists() {
        fs::remove_file(test_db_path).unwrap();
    }
    
    println!("\n=== Тест версионирования миграций ===");
    
    // Инициализируем БД
    db::init_db(test_db_path, test_key).unwrap();
    
    // Проверяем версию после миграций
    use rusqlite::Connection;
    let conn = Connection::open(test_db_path).unwrap();
    conn.pragma_update(None, "key", test_key).unwrap();
    
    let version: String = conn
        .query_row("SELECT version FROM meta LIMIT 1", [], |row| row.get(0))
        .unwrap();
    
    let version_num: i32 = version.parse().unwrap_or(0);
    
    println!("Версия БД после миграций: {}", version);
    assert!(version_num >= 4, "Версия должна быть >= 4 (прошли миграции M1-M4)");
    println!("✓ Все миграции применены (версия {})", version);
    
    // Очистка
    fs::remove_file(test_db_path).unwrap();
}

