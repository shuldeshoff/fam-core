use fam_core_lib::db;
use std::fs;

#[test]
fn test_aggregation_functions() {
    let db_path = "/tmp/test_aggregations.db";
    let key = "test_key_123";
    
    // Удаляем старую БД если есть
    let _ = fs::remove_file(db_path);
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    
    // Создаём 3 аккаунта
    let acc1 = db::create_account(db_path, key, "Account 1".to_string(), "cash".to_string())
        .expect("Failed to create account 1");
    let acc2 = db::create_account(db_path, key, "Account 2".to_string(), "bank".to_string())
        .expect("Failed to create account 2");
    let acc3 = db::create_account(db_path, key, "Account 3".to_string(), "card".to_string())
        .expect("Failed to create account 3");
    
    println!("✓ Created 3 accounts: {}, {}, {}", acc1, acc2, acc3);
    
    // Проверяем начальные балансы (должны быть 0.0)
    let balance1 = db::get_account_balance(db_path, key, acc1)
        .expect("Failed to get balance 1");
    let balance2 = db::get_account_balance(db_path, key, acc2)
        .expect("Failed to get balance 2");
    let balance3 = db::get_account_balance(db_path, key, acc3)
        .expect("Failed to get balance 3");
    
    println!("✓ Initial balances: {}, {}, {}", balance1, balance2, balance3);
    assert_eq!(balance1, 0.0, "Account 1 should start with 0.0 balance");
    assert_eq!(balance2, 0.0, "Account 2 should start with 0.0 balance");
    assert_eq!(balance3, 0.0, "Account 3 should start with 0.0 balance");
    
    // Проверяем Net Worth (должен быть 0.0)
    let net_worth = db::get_net_worth(db_path, key)
        .expect("Failed to get net worth");
    println!("✓ Initial net worth: {}", net_worth);
    assert_eq!(net_worth, 0.0, "Initial net worth should be 0.0");
    
    // Добавляем операции
    db::add_operation(db_path, key, acc1, 1000.0, "Initial deposit".to_string())
        .expect("Failed to add operation 1");
    println!("✓ Added +1000.0 to account 1");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc2, 500.0, "Salary".to_string())
        .expect("Failed to add operation 2");
    println!("✓ Added +500.0 to account 2");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc1, -200.0, "Purchase".to_string())
        .expect("Failed to add operation 3");
    println!("✓ Added -200.0 to account 1");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc3, 300.0, "Transfer".to_string())
        .expect("Failed to add operation 4");
    println!("✓ Added +300.0 to account 3");
    
    // Проверяем обновлённые балансы
    let balance1_updated = db::get_account_balance(db_path, key, acc1)
        .expect("Failed to get updated balance 1");
    let balance2_updated = db::get_account_balance(db_path, key, acc2)
        .expect("Failed to get updated balance 2");
    let balance3_updated = db::get_account_balance(db_path, key, acc3)
        .expect("Failed to get updated balance 3");
    
    println!("✓ Updated balances: {}, {}, {}", balance1_updated, balance2_updated, balance3_updated);
    assert_eq!(balance1_updated, 800.0, "Account 1 balance should be 800.0 (1000 - 200)");
    assert_eq!(balance2_updated, 500.0, "Account 2 balance should be 500.0");
    assert_eq!(balance3_updated, 300.0, "Account 3 balance should be 300.0");
    
    // Проверяем обновлённый Net Worth
    let net_worth_updated = db::get_net_worth(db_path, key)
        .expect("Failed to get updated net worth");
    println!("✓ Updated net worth: {}", net_worth_updated);
    assert_eq!(net_worth_updated, 1600.0, "Net worth should be 1600.0 (800 + 500 + 300)");
    
    // Добавляем ещё одну операцию к первому аккаунту
    std::thread::sleep(std::time::Duration::from_secs(1));
    db::add_operation(db_path, key, acc1, 100.0, "Bonus".to_string())
        .expect("Failed to add operation 5");
    println!("✓ Added +100.0 to account 1");
    
    // Проверяем что баланс первого аккаунта обновился
    let balance1_final = db::get_account_balance(db_path, key, acc1)
        .expect("Failed to get final balance 1");
    println!("✓ Final balance 1: {}", balance1_final);
    assert_eq!(balance1_final, 900.0, "Account 1 final balance should be 900.0 (800 + 100)");
    
    // Проверяем финальный Net Worth
    let net_worth_final = db::get_net_worth(db_path, key)
        .expect("Failed to get final net worth");
    println!("✓ Final net worth: {}", net_worth_final);
    assert_eq!(net_worth_final, 1700.0, "Final net worth should be 1700.0 (900 + 500 + 300)");
    
    println!("\n✅ All aggregation tests passed!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

