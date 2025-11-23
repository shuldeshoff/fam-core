use fam_core_lib::{db, crypto};
use std::fs;

#[test]
fn test_ed25519_keypair_generation_and_storage() {
    let db_path = "/tmp/test_ed25519_keys.db";
    let key = "test_key_123";
    
    // Удаляем старую БД
    let _ = fs::remove_file(db_path);
    
    println!("=== Тест генерации и хранения Ed25519 ключей ===\n");
    
    // Инициализация БД (должна автоматически создать ключи)
    db::init_db(db_path, key).expect("Failed to init db");
    println!("✓ База данных инициализирована");
    
    // Проверяем что ключи созданы
    let private_exists = db::key_exists_in_keystore(db_path, key, "ed25519_private")
        .expect("Failed to check private key");
    let public_exists = db::key_exists_in_keystore(db_path, key, "ed25519_public")
        .expect("Failed to check public key");
    
    assert!(private_exists, "Private key should exist in keystore");
    assert!(public_exists, "Public key should exist in keystore");
    println!("✓ Ed25519 ключи автоматически созданы в keystore");
    
    // Загружаем ключи
    let private_key = db::load_key_from_keystore(db_path, key, "ed25519_private")
        .expect("Failed to load private key")
        .expect("Private key should exist");
    let public_key = db::load_key_from_keystore(db_path, key, "ed25519_public")
        .expect("Failed to load public key")
        .expect("Public key should exist");
    
    println!("✓ Ключи успешно загружены из keystore");
    println!("  Private key length: {} bytes", private_key.len());
    println!("  Public key length: {} bytes", public_key.len());
    
    // Проверяем размеры ключей
    assert_eq!(private_key.len(), 32, "Private key must be 32 bytes");
    assert_eq!(public_key.len(), 32, "Public key must be 32 bytes");
    println!("✓ Размеры ключей корректны (32 байта)");
    
    // Тестируем подпись и верификацию
    let test_data = b"Hello, FAM-Core!";
    
    let signature = crypto::sign_data(&private_key, test_data)
        .expect("Failed to sign data");
    println!("✓ Данные подписаны");
    println!("  Signature length: {} bytes", signature.len());
    
    assert_eq!(signature.len(), 64, "Signature must be 64 bytes");
    println!("✓ Размер подписи корректен (64 байта)");
    
    let is_valid = crypto::verify_signature(&public_key, test_data, &signature)
        .expect("Failed to verify signature");
    assert!(is_valid, "Signature should be valid");
    println!("✓ Подпись верифицирована успешно");
    
    // Проверяем что неправильные данные не верифицируются
    let wrong_data = b"Wrong data";
    let is_invalid = crypto::verify_signature(&public_key, wrong_data, &signature)
        .expect("Failed to verify signature");
    assert!(!is_invalid, "Signature should be invalid for wrong data");
    println!("✓ Неверные данные не проходят верификацию");
    
    // Проверяем что при повторной инициализации ключи не пересоздаются
    db::init_db(db_path, key).expect("Failed to re-init db");
    
    let private_key2 = db::load_key_from_keystore(db_path, key, "ed25519_private")
        .expect("Failed to load private key")
        .expect("Private key should exist");
    let public_key2 = db::load_key_from_keystore(db_path, key, "ed25519_public")
        .expect("Failed to load public key")
        .expect("Public key should exist");
    
    assert_eq!(private_key, private_key2, "Private key should not change");
    assert_eq!(public_key, public_key2, "Public key should not change");
    println!("✓ Ключи сохраняются между запусками (не пересоздаются)");
    
    // Тестируем удаление ключа
    db::delete_key_from_keystore(db_path, key, "ed25519_private")
        .expect("Failed to delete private key");
    
    let private_exists_after_delete = db::key_exists_in_keystore(db_path, key, "ed25519_private")
        .expect("Failed to check private key");
    assert!(!private_exists_after_delete, "Private key should be deleted");
    println!("✓ Удаление ключа работает корректно");
    
    // При следующей инициализации ключи должны пересоздаться
    db::init_db(db_path, key).expect("Failed to re-init db after delete");
    
    let private_key3 = db::load_key_from_keystore(db_path, key, "ed25519_private")
        .expect("Failed to load private key")
        .expect("Private key should exist");
    
    assert_ne!(private_key, private_key3, "New private key should be generated");
    println!("✓ Новая пара ключей создана после удаления");
    
    println!("\n✅ Все тесты Ed25519 пройдены успешно!");
    
    // Очистка
    let _ = fs::remove_file(db_path);
}

#[test]
fn test_ed25519_manual_generation() {
    println!("=== Тест ручной генерации Ed25519 ключей ===\n");
    
    // Генерируем пару ключей вручную
    let keypair = crypto::generate_ed25519_keypair()
        .expect("Failed to generate keypair");
    
    println!("✓ Пара ключей сгенерирована");
    println!("  Private key: {} bytes", keypair.private_key.len());
    println!("  Public key: {} bytes", keypair.public_key.len());
    
    assert_eq!(keypair.private_key.len(), 32);
    assert_eq!(keypair.public_key.len(), 32);
    
    // Подписываем тестовые данные
    let test_message = b"Test message for Ed25519";
    let signature = crypto::sign_data(&keypair.private_key, test_message)
        .expect("Failed to sign");
    
    println!("✓ Сообщение подписано ({} байт)", signature.len());
    
    // Верифицируем
    let valid = crypto::verify_signature(&keypair.public_key, test_message, &signature)
        .expect("Failed to verify");
    
    assert!(valid, "Signature must be valid");
    println!("✓ Подпись верифицирована");
    
    // Проверяем с другим сообщением
    let other_message = b"Different message";
    let invalid = crypto::verify_signature(&keypair.public_key, other_message, &signature)
        .expect("Failed to verify");
    
    assert!(!invalid, "Signature must be invalid for different message");
    println!("✓ Верификация отклоняет неверное сообщение");
    
    println!("\n✅ Тест ручной генерации пройден!");
}

