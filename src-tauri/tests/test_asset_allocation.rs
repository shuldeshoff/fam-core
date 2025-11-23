use fam_core_lib::db;
use std::fs;

#[test]
fn test_asset_allocation() {
    let db_path = "/tmp/test_asset_allocation.db";
    let key = "test_key_123";
    
    // Удаляем старую БД если есть
    let _ = fs::remove_file(db_path);
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    
    println!("✓ Database initialized");
    
    // Проверяем пустую аллокацию (нет аккаунтов)
    let allocation_empty = db::get_asset_allocation(db_path, key)
        .expect("Failed to get empty allocation");
    assert_eq!(allocation_empty.len(), 0, "Empty database should have no asset allocation");
    println!("✓ Empty database returns empty allocation");
    
    // Создаём аккаунты разных типов
    let acc1 = db::create_account(db_path, key, "Cash Wallet".to_string(), "cash".to_string())
        .expect("Failed to create account 1");
    let acc2 = db::create_account(db_path, key, "Savings Account".to_string(), "deposit".to_string())
        .expect("Failed to create account 2");
    let acc3 = db::create_account(db_path, key, "Checking Account".to_string(), "bank".to_string())
        .expect("Failed to create account 3");
    let acc4 = db::create_account(db_path, key, "Credit Card".to_string(), "card".to_string())
        .expect("Failed to create account 4");
    let acc5 = db::create_account(db_path, key, "Second Cash".to_string(), "cash".to_string())
        .expect("Failed to create account 5");
    
    println!("✓ Created 5 accounts: {}, {}, {}, {}, {}", acc1, acc2, acc3, acc4, acc5);
    
    // Проверяем аллокацию без операций (все балансы 0)
    let allocation_no_ops = db::get_asset_allocation(db_path, key)
        .expect("Failed to get allocation without operations");
    
    println!("\n✓ Asset allocation without operations:");
    for alloc in &allocation_no_ops {
        println!("  Type: {}, Balance: {:.2}, Accounts: {}", 
            alloc.asset_type, alloc.total_balance, alloc.account_count);
    }
    
    // У нас должно быть 4 типа (cash, deposit, bank, card), но без балансов
    assert_eq!(allocation_no_ops.len(), 0, "Accounts without operations should not appear in allocation");
    println!("✓ Accounts without balances are not included");
    
    // Добавляем операции к аккаунтам
    db::add_operation(db_path, key, acc1, 1000.0, "Initial cash".to_string())
        .expect("Failed to add operation 1");
    println!("✓ Added +1000.0 to cash account 1");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc2, 5000.0, "Deposit".to_string())
        .expect("Failed to add operation 2");
    println!("✓ Added +5000.0 to deposit account");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc3, 2000.0, "Bank transfer".to_string())
        .expect("Failed to add operation 3");
    println!("✓ Added +2000.0 to bank account");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc4, -500.0, "Credit card debt".to_string())
        .expect("Failed to add operation 4");
    println!("✓ Added -500.0 to card account");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, acc5, 300.0, "Second cash deposit".to_string())
        .expect("Failed to add operation 5");
    println!("✓ Added +300.0 to cash account 2");
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    // Добавляем ещё операции к первому cash аккаунту
    db::add_operation(db_path, key, acc1, -200.0, "Cash expense".to_string())
        .expect("Failed to add operation 6");
    println!("✓ Added -200.0 to cash account 1");
    
    // Получаем финальную аллокацию
    let allocation = db::get_asset_allocation(db_path, key)
        .expect("Failed to get asset allocation");
    
    println!("\n✓ Final asset allocation:");
    for alloc in &allocation {
        println!("  Type: {}, Balance: {:.2}, Accounts: {}", 
            alloc.asset_type, alloc.total_balance, alloc.account_count);
    }
    
    // Проверяем количество типов
    assert_eq!(allocation.len(), 4, "Should have 4 asset types");
    println!("✓ Found 4 asset types");
    
    // Проверяем что результаты отсортированы по балансу (DESC)
    for i in 1..allocation.len() {
        assert!(
            allocation[i].total_balance <= allocation[i - 1].total_balance,
            "Asset allocation should be sorted by total_balance DESC"
        );
    }
    println!("✓ Results sorted by balance DESC");
    
    // Проверяем конкретные значения
    let cash_alloc = allocation.iter().find(|a| a.asset_type == "cash").expect("Cash type not found");
    assert_eq!(cash_alloc.total_balance, 1100.0, "Cash total should be 1100.0 (800 + 300)");
    assert_eq!(cash_alloc.account_count, 2, "Cash should have 2 accounts");
    println!("✓ Cash: {:.2} (2 accounts) ✓", cash_alloc.total_balance);
    
    let deposit_alloc = allocation.iter().find(|a| a.asset_type == "deposit").expect("Deposit type not found");
    assert_eq!(deposit_alloc.total_balance, 5000.0, "Deposit total should be 5000.0");
    assert_eq!(deposit_alloc.account_count, 1, "Deposit should have 1 account");
    println!("✓ Deposit: {:.2} (1 account) ✓", deposit_alloc.total_balance);
    
    let bank_alloc = allocation.iter().find(|a| a.asset_type == "bank").expect("Bank type not found");
    assert_eq!(bank_alloc.total_balance, 2000.0, "Bank total should be 2000.0");
    assert_eq!(bank_alloc.account_count, 1, "Bank should have 1 account");
    println!("✓ Bank: {:.2} (1 account) ✓", bank_alloc.total_balance);
    
    let card_alloc = allocation.iter().find(|a| a.asset_type == "card").expect("Card type not found");
    assert_eq!(card_alloc.total_balance, -500.0, "Card total should be -500.0");
    assert_eq!(card_alloc.account_count, 1, "Card should have 1 account");
    println!("✓ Card: {:.2} (1 account) ✓", card_alloc.total_balance);
    
    // Проверяем общую сумму
    let total: f64 = allocation.iter().map(|a| a.total_balance).sum();
    assert_eq!(total, 7600.0, "Total of all assets should be 7600.0");
    println!("✓ Total assets: {:.2} ✓", total);
    
    // Проверяем общее количество аккаунтов
    let total_accounts: i64 = allocation.iter().map(|a| a.account_count).sum();
    assert_eq!(total_accounts, 5, "Total account count should be 5");
    println!("✓ Total accounts: {} ✓", total_accounts);
    
    println!("\n✅ All asset allocation tests passed!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

