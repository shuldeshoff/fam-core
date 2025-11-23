use fam_core_lib::db;
use std::path::PathBuf;

#[test]
fn test_get_version_log_asc_order() {
    let test_db_path = "/tmp/test_get_versions.db";
    let test_key = "test_key_get";
    
    // Удаляем старую БД
    if PathBuf::from(test_db_path).exists() {
        std::fs::remove_file(test_db_path).unwrap();
    }
    
    println!("\n=== Тест get_version_log с сортировкой ASC ===");
    
    // Инициализируем БД
    db::init_db(test_db_path, test_key).expect("DB init failed");
    
    // Создаём аккаунт
    let account_id = db::create_account(test_db_path, test_key, "Test Account".to_string(), "cash".to_string()).unwrap();
    println!("✓ Аккаунт создан с ID: {}", account_id);
    
    // Добавляем небольшую задержку
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // Добавляем операцию
    let operation_id = db::add_operation(test_db_path, test_key, account_id, 50.0, "Op 1".to_string()).unwrap();
    println!("✓ Операция создана с ID: {}", operation_id);
    
    // Получаем записи с сортировкой DESC (новые первыми) - list_version_log
    let desc_records = db::list_version_log(test_db_path, test_key, None, None).unwrap();
    println!("\n1. list_version_log (DESC - новые первыми):");
    for (i, record) in desc_records.iter().enumerate() {
        println!("  [{}] entity={}, entity_id={}, ts={}", i, record.entity, record.entity_id, record.ts);
    }
    
    // Получаем записи с сортировкой ASC (старые первыми) - get_version_log
    let asc_records = db::get_version_log(test_db_path, test_key, None, None).unwrap();
    println!("\n2. get_version_log (ASC - старые первыми):");
    for (i, record) in asc_records.iter().enumerate() {
        println!("  [{}] entity={}, entity_id={}, ts={}", i, record.entity, record.entity_id, record.ts);
    }
    
    // Проверяем, что количество записей одинаковое
    assert_eq!(desc_records.len(), asc_records.len());
    
    // Проверяем, что порядок обратный
    assert_eq!(desc_records[0].id, asc_records[asc_records.len() - 1].id);
    assert_eq!(desc_records[desc_records.len() - 1].id, asc_records[0].id);
    
    // Проверяем, что timestamps в ASC порядке увеличиваются
    for i in 0..asc_records.len() - 1 {
        assert!(asc_records[i].ts <= asc_records[i + 1].ts, 
                "Timestamps должны быть в возрастающем порядке");
    }
    
    println!("\n✓ Сортировка ASC работает корректно");
    
    // Тест с фильтром по entity
    let account_asc = db::get_version_log(test_db_path, test_key, Some("account"), None).unwrap();
    println!("\n3. get_version_log с entity='account' (ASC):");
    for (i, record) in account_asc.iter().enumerate() {
        println!("  [{}] entity={}, entity_id={}, ts={}", i, record.entity, record.entity_id, record.ts);
    }
    assert_eq!(account_asc.len(), 1);
    assert_eq!(account_asc[0].entity, "account");
    
    // Тест с фильтром по entity_id
    let entity1_asc = db::get_version_log(test_db_path, test_key, None, Some(account_id)).unwrap();
    println!("\n4. get_version_log с entity_id={} (ASC): {} записей", account_id, entity1_asc.len());
    // Должно быть 3 записи: account + operation + state
    assert_eq!(entity1_asc.len(), 3);
    
    // Проверяем хронологический порядок: сначала account, потом operation, потом state
    assert_eq!(entity1_asc[0].entity, "account");
    assert_eq!(entity1_asc[1].entity, "operation");
    assert_eq!(entity1_asc[2].entity, "state");
    
    println!("\n✓ Все тесты get_version_log пройдены");
    println!("✓ ASC сортировка позволяет воспроизвести историю изменений в хронологическом порядке");
    
    // Очистка
    std::fs::remove_file(test_db_path).unwrap();
}

