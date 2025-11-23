use fam_core_lib::db;
use std::fs;

#[test]
fn test_analytics_correctness() {
    let db_path = "/tmp/test_analytics_correctness.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Комплексная проверка корректности аналитики ===\n");
    
    // Инициализация БД
    db::init_db(db_path, key).expect("Failed to init db");
    println!("✓ База данных инициализирована");
    
    // ========================================
    // Подготовка: создание аккаунтов и операций
    // ========================================
    
    println!("\n--- Создание тестовых данных ---");
    
    // Создаём аккаунты разных типов
    let cash1 = db::create_account(db_path, key, "Cash Wallet 1".to_string(), "cash".to_string())
        .expect("Failed to create cash1");
    let cash2 = db::create_account(db_path, key, "Cash Wallet 2".to_string(), "cash".to_string())
        .expect("Failed to create cash2");
    let deposit = db::create_account(db_path, key, "Savings".to_string(), "deposit".to_string())
        .expect("Failed to create deposit");
    let bank = db::create_account(db_path, key, "Checking".to_string(), "bank".to_string())
        .expect("Failed to create bank");
    
    println!("✓ Создано 4 аккаунта: cash1={}, cash2={}, deposit={}, bank={}", 
        cash1, cash2, deposit, bank);
    
    // Добавляем операции с задержками для разных timestamp
    db::add_operation(db_path, key, cash1, 1000.0, "Initial".to_string())
        .expect("Failed to add op1");
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, cash1, -200.0, "Expense".to_string())
        .expect("Failed to add op2");
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, cash1, 300.0, "Income".to_string())
        .expect("Failed to add op3");
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, cash2, 500.0, "Initial".to_string())
        .expect("Failed to add op4");
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, deposit, 10000.0, "Deposit".to_string())
        .expect("Failed to add op5");
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    db::add_operation(db_path, key, bank, 2000.0, "Transfer".to_string())
        .expect("Failed to add op6");
    
    println!("✓ Добавлено 6 операций");
    println!("  cash1: +1000, -200, +300 → ожидаемый баланс: 1100");
    println!("  cash2: +500 → ожидаемый баланс: 500");
    println!("  deposit: +10000 → ожидаемый баланс: 10000");
    println!("  bank: +2000 → ожидаемый баланс: 2000");
    
    // ========================================
    // ПРОВЕРКА 1: Net Worth из последних состояний
    // ========================================
    
    println!("\n--- ПРОВЕРКА 1: Net Worth рассчитывается из последних состояний ---");
    
    // Получаем последние балансы вручную
    let balance_cash1 = db::get_account_balance(db_path, key, cash1)
        .expect("Failed to get balance cash1");
    let balance_cash2 = db::get_account_balance(db_path, key, cash2)
        .expect("Failed to get balance cash2");
    let balance_deposit = db::get_account_balance(db_path, key, deposit)
        .expect("Failed to get balance deposit");
    let balance_bank = db::get_account_balance(db_path, key, bank)
        .expect("Failed to get balance bank");
    
    println!("Последние балансы:");
    println!("  cash1: {:.2}", balance_cash1);
    println!("  cash2: {:.2}", balance_cash2);
    println!("  deposit: {:.2}", balance_deposit);
    println!("  bank: {:.2}", balance_bank);
    
    let expected_net_worth = balance_cash1 + balance_cash2 + balance_deposit + balance_bank;
    println!("Ожидаемый Net Worth (сумма последних): {:.2}", expected_net_worth);
    
    let actual_net_worth = db::get_net_worth(db_path, key)
        .expect("Failed to get net worth");
    println!("Фактический Net Worth (из функции): {:.2}", actual_net_worth);
    
    assert_eq!(balance_cash1, 1100.0, "Cash1 balance should be 1100");
    assert_eq!(balance_cash2, 500.0, "Cash2 balance should be 500");
    assert_eq!(balance_deposit, 10000.0, "Deposit balance should be 10000");
    assert_eq!(balance_bank, 2000.0, "Bank balance should be 2000");
    assert_eq!(actual_net_worth, expected_net_worth, "Net Worth должен равняться сумме последних балансов");
    assert_eq!(actual_net_worth, 13600.0, "Net Worth должен быть 13600");
    
    println!("✅ ПРОВЕРКА 1 ПРОЙДЕНА: Net Worth = {:.2} (корректно рассчитан)", actual_net_worth);
    
    // ========================================
    // ПРОВЕРКА 2: Временные ряды в хронологическом порядке
    // ========================================
    
    println!("\n--- ПРОВЕРКА 2: Временные ряды в хронологическом порядке ---");
    
    let history_cash1 = db::get_balance_history(db_path, key, cash1)
        .expect("Failed to get history");
    
    println!("История cash1 ({} записей):", history_cash1.len());
    for (i, state) in history_cash1.iter().enumerate() {
        println!("  [{}] ts={}, balance={:.2}", i, state.ts, state.balance);
    }
    
    // Проверяем количество записей
    assert_eq!(history_cash1.len(), 3, "Cash1 должен иметь 3 записи баланса");
    
    // Проверяем хронологический порядок (ASC)
    for i in 1..history_cash1.len() {
        assert!(
            history_cash1[i].ts > history_cash1[i-1].ts,
            "Временные метки должны быть в порядке возрастания (ASC): {} > {}",
            history_cash1[i].ts, history_cash1[i-1].ts
        );
    }
    println!("✓ Временные метки идут в порядке возрастания (ASC)");
    
    // Проверяем корректность значений балансов
    assert_eq!(history_cash1[0].balance, 1000.0, "Первый баланс: 1000");
    assert_eq!(history_cash1[1].balance, 800.0, "Второй баланс: 800 (1000-200)");
    assert_eq!(history_cash1[2].balance, 1100.0, "Третий баланс: 1100 (800+300)");
    println!("✓ Значения балансов корректны: 1000 → 800 → 1100");
    
    println!("✅ ПРОВЕРКА 2 ПРОЙДЕНА: Временные ряды в хронологическом порядке");
    
    // ========================================
    // ПРОВЕРКА 3: Структура активов агрегирует суммы по типам
    // ========================================
    
    println!("\n--- ПРОВЕРКА 3: Структура активов корректно агрегирует ---");
    
    let allocation = db::get_asset_allocation(db_path, key)
        .expect("Failed to get asset allocation");
    
    println!("Структура активов ({} типов):", allocation.len());
    for alloc in &allocation {
        println!("  Type: {}, Balance: {:.2}, Accounts: {}", 
            alloc.asset_type, alloc.total_balance, alloc.account_count);
    }
    
    // Должно быть 3 типа: cash, deposit, bank
    assert_eq!(allocation.len(), 3, "Должно быть 3 типа активов");
    
    // Проверяем сортировку по балансу (DESC)
    for i in 1..allocation.len() {
        assert!(
            allocation[i].total_balance <= allocation[i-1].total_balance,
            "Типы должны быть отсортированы по балансу DESC"
        );
    }
    println!("✓ Типы отсортированы по балансу (DESC)");
    
    // Проверяем конкретные значения
    let cash_alloc = allocation.iter().find(|a| a.asset_type == "cash")
        .expect("Cash type not found");
    assert_eq!(cash_alloc.total_balance, 1600.0, "Cash total: 1100+500=1600");
    assert_eq!(cash_alloc.account_count, 2, "Cash accounts: 2");
    println!("✓ Cash: {:.2} (2 аккаунта) ✓", cash_alloc.total_balance);
    
    let deposit_alloc = allocation.iter().find(|a| a.asset_type == "deposit")
        .expect("Deposit type not found");
    assert_eq!(deposit_alloc.total_balance, 10000.0, "Deposit total: 10000");
    assert_eq!(deposit_alloc.account_count, 1, "Deposit accounts: 1");
    println!("✓ Deposit: {:.2} (1 аккаунт) ✓", deposit_alloc.total_balance);
    
    let bank_alloc = allocation.iter().find(|a| a.asset_type == "bank")
        .expect("Bank type not found");
    assert_eq!(bank_alloc.total_balance, 2000.0, "Bank total: 2000");
    assert_eq!(bank_alloc.account_count, 1, "Bank accounts: 1");
    println!("✓ Bank: {:.2} (1 аккаунт) ✓", bank_alloc.total_balance);
    
    // Проверяем что сумма всех типов = Net Worth
    let total_from_allocation: f64 = allocation.iter()
        .map(|a| a.total_balance)
        .sum();
    assert_eq!(total_from_allocation, actual_net_worth, 
        "Сумма из структуры активов должна равняться Net Worth");
    println!("✓ Сумма всех типов = Net Worth ({:.2})", total_from_allocation);
    
    println!("✅ ПРОВЕРКА 3 ПРОЙДЕНА: Структура активов корректна");
    
    // ========================================
    // ПРОВЕРКА 4: API возвращает валидные JSON-структуры
    // ========================================
    
    println!("\n--- ПРОВЕРКА 4: API возвращает валидные JSON-структуры ---");
    
    // Проверяем что структуры сериализуются в JSON
    let balance_json = serde_json::to_string(&balance_cash1)
        .expect("Balance should serialize to JSON");
    println!("✓ Balance сериализуется: {}", balance_json);
    
    let net_worth_json = serde_json::to_string(&actual_net_worth)
        .expect("Net Worth should serialize to JSON");
    println!("✓ Net Worth сериализуется: {}", net_worth_json);
    
    let history_json = serde_json::to_string(&history_cash1)
        .expect("History should serialize to JSON");
    println!("✓ Balance History сериализуется ({} байт)", history_json.len());
    
    let allocation_json = serde_json::to_string(&allocation)
        .expect("Asset Allocation should serialize to JSON");
    println!("✓ Asset Allocation сериализуется ({} байт)", allocation_json.len());
    
    // Проверяем что JSON валидный (можно распарсить обратно)
    let parsed_allocation: Vec<db::AssetAllocation> = serde_json::from_str(&allocation_json)
        .expect("Should parse back from JSON");
    assert_eq!(parsed_allocation.len(), allocation.len(), "Parsed data should match original");
    println!("✓ JSON корректно парсится обратно");
    
    println!("✅ ПРОВЕРКА 4 ПРОЙДЕНА: Все структуры валидно сериализуются в JSON");
    
    // ========================================
    // ПРОВЕРКА 5: Интеграция всех компонентов
    // ========================================
    
    println!("\n--- ПРОВЕРКА 5: Интеграционная проверка ---");
    
    // Проверяем что все компоненты работают вместе
    println!("Финальная проверка целостности данных:");
    println!("  Аккаунтов создано: 4");
    println!("  Операций выполнено: 6");
    println!("  Типов активов: {}", allocation.len());
    println!("  Net Worth: {:.2} ₽", actual_net_worth);
    
    // Проверяем что количество записей в истории соответствует операциям
    let total_history_records: usize = vec![cash1, cash2, deposit, bank]
        .iter()
        .map(|&acc_id| {
            db::get_balance_history(db_path, key, acc_id)
                .map(|h| h.len())
                .unwrap_or(0)
        })
        .sum();
    assert_eq!(total_history_records, 6, "Должно быть 6 записей истории (по числу операций)");
    println!("✓ Количество записей истории = количеству операций (6)");
    
    // Проверяем что все аккаунты учтены
    let total_accounts_in_allocation: i64 = allocation.iter()
        .map(|a| a.account_count)
        .sum();
    assert_eq!(total_accounts_in_allocation, 4, "Все 4 аккаунта должны быть учтены");
    println!("✓ Все аккаунты учтены в структуре активов (4)");
    
    println!("✅ ПРОВЕРКА 5 ПРОЙДЕНА: Все компоненты работают корректно");
    
    // ========================================
    // ИТОГИ
    // ========================================
    
    println!("\n=== ИТОГОВЫЕ РЕЗУЛЬТАТЫ ===");
    println!("✅ 1. Net Worth рассчитывается из последних состояний всех аккаунтов");
    println!("✅ 2. Временные ряды извлекаются в хронологическом порядке (ASC)");
    println!("✅ 3. Структура активов корректно агрегирует суммы по типам");
    println!("✅ 4. API отдаёт валидные JSON-структуры");
    println!("✅ 5. Все компоненты интегрированы корректно");
    println!("\n🎉 ВСЕ ПРОВЕРКИ ПРОЙДЕНЫ УСПЕШНО!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

