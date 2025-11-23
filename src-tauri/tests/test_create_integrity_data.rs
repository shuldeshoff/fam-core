use fam_core_lib::db;

#[test]
fn test_integrity_setup() {
    let test_db_path = "/tmp/test_integrity.db";
    let test_key = "test_key";
    
    // Удаляем старую БД
    if std::path::PathBuf::from(test_db_path).exists() {
        std::fs::remove_file(test_db_path).unwrap();
    }
    
    println!("\n=== Создание тестовых данных для проверки целостности ===");
    
    // Инициализируем БД
    db::init_db(test_db_path, test_key).unwrap();
    
    // Создаём 3 аккаунта
    let acc1 = db::create_account(test_db_path, test_key, "Cash".to_string(), "cash".to_string()).unwrap();
    let acc2 = db::create_account(test_db_path, test_key, "Card".to_string(), "card".to_string()).unwrap();
    let _acc3 = db::create_account(test_db_path, test_key, "Bank".to_string(), "bank".to_string()).unwrap();
    
    // Добавляем операции
    std::thread::sleep(std::time::Duration::from_millis(100));
    db::add_operation(test_db_path, test_key, acc1, 1000.0, "Initial".to_string()).unwrap();
    
    std::thread::sleep(std::time::Duration::from_millis(100));
    db::add_operation(test_db_path, test_key, acc1, -200.0, "Expense".to_string()).unwrap();
    
    std::thread::sleep(std::time::Duration::from_millis(100));
    db::add_operation(test_db_path, test_key, acc2, 500.0, "Income".to_string()).unwrap();
    
    println!("✓ Создано 3 аккаунта и 3 операции");
    println!("✓ БД сохранена в {}", test_db_path);
    println!("\nТеперь запустите: rust-script check_version_log_integrity.rs с путём к этой БД");
    
    // Не удаляем БД для проверки
}

