use fam_core_lib::db;
use std::fs;

#[test]
fn test_balance_history() {
    let db_path = "/tmp/test_balance_history.db";
    let key = "test_key_123";
    
    // Удаляем старую БД если есть
    let _ = fs::remove_file(db_path);
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    
    // Создаём 2 аккаунта
    let acc1 = db::create_account(db_path, key, "Account 1".to_string(), "cash".to_string())
        .expect("Failed to create account 1");
    let acc2 = db::create_account(db_path, key, "Account 2".to_string(), "bank".to_string())
        .expect("Failed to create account 2");
    
    println!("✓ Created 2 accounts: {}, {}", acc1, acc2);
    
    // Проверяем что история пустая для нового аккаунта
    let history1_initial = db::get_balance_history(db_path, key, acc1)
        .expect("Failed to get initial history");
    println!("✓ Initial history for account 1: {} records", history1_initial.len());
    assert_eq!(history1_initial.len(), 0, "New account should have no balance history");
    
    // Добавляем операции к первому аккаунту
    db::add_operation(db_path, key, acc1, 1000.0, "Initial deposit".to_string())
        .expect("Failed to add operation 1");
    println!("✓ Added operation 1: +1000.0");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc1, -200.0, "Purchase".to_string())
        .expect("Failed to add operation 2");
    println!("✓ Added operation 2: -200.0");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc1, 300.0, "Income".to_string())
        .expect("Failed to add operation 3");
    println!("✓ Added operation 3: +300.0");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc1, -100.0, "Expense".to_string())
        .expect("Failed to add operation 4");
    println!("✓ Added operation 4: -100.0");
    
    // Добавляем одну операцию ко второму аккаунту
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc2, 500.0, "Initial".to_string())
        .expect("Failed to add operation 5");
    println!("✓ Added operation 5 to account 2: +500.0");
    
    // Получаем историю первого аккаунта
    let history1 = db::get_balance_history(db_path, key, acc1)
        .expect("Failed to get history for account 1");
    
    println!("\n✓ Balance history for account 1:");
    for (i, state) in history1.iter().enumerate() {
        println!("  {}: balance={:.2}, ts={}, account_id={}", 
            i + 1, state.balance, state.ts, state.account_id);
    }
    
    // Проверяем количество записей
    assert_eq!(history1.len(), 4, "Account 1 should have 4 balance records");
    
    // Проверяем что записи отсортированы по времени (ASC)
    for i in 1..history1.len() {
        assert!(
            history1[i].ts > history1[i - 1].ts,
            "Balance history should be sorted by timestamp ASC"
        );
    }
    println!("✓ Records are sorted by timestamp ASC");
    
    // Проверяем значения балансов
    assert_eq!(history1[0].balance, 1000.0, "First balance should be 1000.0");
    assert_eq!(history1[1].balance, 800.0, "Second balance should be 800.0 (1000 - 200)");
    assert_eq!(history1[2].balance, 1100.0, "Third balance should be 1100.0 (800 + 300)");
    assert_eq!(history1[3].balance, 1000.0, "Fourth balance should be 1000.0 (1100 - 100)");
    println!("✓ All balance values are correct");
    
    // Получаем историю второго аккаунта
    let history2 = db::get_balance_history(db_path, key, acc2)
        .expect("Failed to get history for account 2");
    
    println!("\n✓ Balance history for account 2:");
    for (i, state) in history2.iter().enumerate() {
        println!("  {}: balance={:.2}, ts={}, account_id={}", 
            i + 1, state.balance, state.ts, state.account_id);
    }
    
    assert_eq!(history2.len(), 1, "Account 2 should have 1 balance record");
    assert_eq!(history2[0].balance, 500.0, "Account 2 balance should be 500.0");
    println!("✓ Account 2 history is correct");
    
    // Проверяем что account_id соответствует
    for state in &history1 {
        assert_eq!(state.account_id, acc1, "All states should belong to account 1");
    }
    for state in &history2 {
        assert_eq!(state.account_id, acc2, "All states should belong to account 2");
    }
    println!("✓ All records have correct account_id");
    
    // Проверяем получение истории для несуществующего аккаунта
    let history_empty = db::get_balance_history(db_path, key, 999)
        .expect("Failed to get history for non-existent account");
    assert_eq!(history_empty.len(), 0, "Non-existent account should have empty history");
    println!("✓ Non-existent account returns empty history");
    
    println!("\n✅ All balance history tests passed!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

