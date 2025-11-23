use fam_core_lib::crypto;

#[test]
fn test_sign_payload_and_verify() {
    println!("=== Тест sign_payload и verify_payload ===\n");
    
    // Генерируем пару ключей
    let keypair = crypto::generate_ed25519_keypair()
        .expect("Failed to generate keypair");
    
    println!("✓ Пара ключей сгенерирована");
    println!("  Private key: {} bytes", keypair.private_key.len());
    println!("  Public key: {} bytes", keypair.public_key.len());
    
    // Тестовые данные
    let payload = b"This is a test payload for signing";
    println!("\n✓ Payload: {} bytes", payload.len());
    println!("  Content: {:?}", String::from_utf8_lossy(payload));
    
    // Подписываем payload используя sign_payload
    let signature = crypto::sign_payload(payload, &keypair.private_key)
        .expect("Failed to sign payload");
    
    println!("\n✓ Payload подписан");
    println!("  Signature: {} bytes", signature.len());
    assert_eq!(signature.len(), 64, "Signature should be 64 bytes");
    
    // Верифицируем подпись используя verify_payload
    let is_valid = crypto::verify_payload(payload, &signature, &keypair.public_key)
        .expect("Failed to verify payload");
    
    assert!(is_valid, "Signature should be valid");
    println!("✓ Подпись верифицирована успешно");
    
    // Проверяем что неправильный payload не проходит верификацию
    let wrong_payload = b"Wrong payload content";
    let is_invalid = crypto::verify_payload(wrong_payload, &signature, &keypair.public_key)
        .expect("Failed to verify wrong payload");
    
    assert!(!is_invalid, "Signature should be invalid for wrong payload");
    println!("✓ Неверный payload отклонён");
    
    // Проверяем что неправильная подпись не проходит верификацию
    let mut wrong_signature = signature.clone();
    wrong_signature[0] = wrong_signature[0].wrapping_add(1); // Меняем один байт
    
    let is_invalid2 = crypto::verify_payload(payload, &wrong_signature, &keypair.public_key)
        .expect("Failed to verify with wrong signature");
    
    assert!(!is_invalid2, "Wrong signature should be invalid");
    println!("✓ Неверная подпись отклонена");
    
    // Проверяем что неправильный публичный ключ не проходит верификацию
    let keypair2 = crypto::generate_ed25519_keypair()
        .expect("Failed to generate second keypair");
    
    let is_invalid3 = crypto::verify_payload(payload, &signature, &keypair2.public_key)
        .expect("Failed to verify with wrong public key");
    
    assert!(!is_invalid3, "Signature should be invalid with wrong public key");
    println!("✓ Неверный публичный ключ отклонён");
    
    // Тестируем с большим payload
    let large_payload = vec![0u8; 10000]; // 10KB данных
    let large_signature = crypto::sign_payload(&large_payload, &keypair.private_key)
        .expect("Failed to sign large payload");
    
    let large_valid = crypto::verify_payload(&large_payload, &large_signature, &keypair.public_key)
        .expect("Failed to verify large payload");
    
    assert!(large_valid, "Large payload signature should be valid");
    println!("\n✓ Большой payload (10KB) подписан и верифицирован");
    
    // Тестируем с пустым payload
    let empty_payload = b"";
    let empty_signature = crypto::sign_payload(empty_payload, &keypair.private_key)
        .expect("Failed to sign empty payload");
    
    let empty_valid = crypto::verify_payload(empty_payload, &empty_signature, &keypair.public_key)
        .expect("Failed to verify empty payload");
    
    assert!(empty_valid, "Empty payload signature should be valid");
    println!("✓ Пустой payload подписан и верифицирован");
    
    println!("\n✅ Все тесты sign_payload/verify_payload пройдены!");
}

#[test]
fn test_payload_signatures_are_deterministic() {
    println!("=== Тест детерминированности подписей ===\n");
    
    let keypair = crypto::generate_ed25519_keypair()
        .expect("Failed to generate keypair");
    
    let payload = b"Deterministic test payload";
    
    // Подписываем один и тот же payload дважды
    let signature1 = crypto::sign_payload(payload, &keypair.private_key)
        .expect("Failed to sign first time");
    
    let signature2 = crypto::sign_payload(payload, &keypair.private_key)
        .expect("Failed to sign second time");
    
    // Ed25519 подписи детерминированные - должны быть одинаковыми
    assert_eq!(signature1, signature2, "Ed25519 signatures should be deterministic");
    println!("✓ Подписи детерминированные (одинаковые при повторном подписании)");
    
    // Оба должны быть валидными
    let valid1 = crypto::verify_payload(payload, &signature1, &keypair.public_key)
        .expect("Failed to verify first signature");
    let valid2 = crypto::verify_payload(payload, &signature2, &keypair.public_key)
        .expect("Failed to verify second signature");
    
    assert!(valid1 && valid2, "Both signatures should be valid");
    println!("✓ Обе подписи валидны");
    
    println!("\n✅ Детерминированность подтверждена!");
}

#[test]
fn test_payload_error_handling() {
    println!("=== Тест обработки ошибок ===\n");
    
    let payload = b"Test payload";
    let keypair = crypto::generate_ed25519_keypair()
        .expect("Failed to generate keypair");
    
    // Тест с неправильным размером приватного ключа
    let wrong_private_key = vec![0u8; 16]; // Должно быть 32
    let result = crypto::sign_payload(payload, &wrong_private_key);
    
    assert!(result.is_err(), "Should fail with wrong private key size");
    println!("✓ Ошибка при неправильном размере приватного ключа (16 вместо 32)");
    
    // Тест с неправильным размером публичного ключа
    let signature = crypto::sign_payload(payload, &keypair.private_key)
        .expect("Failed to sign");
    
    let wrong_public_key = vec![0u8; 16]; // Должно быть 32
    let result2 = crypto::verify_payload(payload, &signature, &wrong_public_key);
    
    assert!(result2.is_err(), "Should fail with wrong public key size");
    println!("✓ Ошибка при неправильном размере публичного ключа (16 вместо 32)");
    
    // Тест с неправильным размером подписи
    let wrong_signature = vec![0u8; 32]; // Должно быть 64
    let result3 = crypto::verify_payload(payload, &wrong_signature, &keypair.public_key);
    
    assert!(result3.is_err(), "Should fail with wrong signature size");
    println!("✓ Ошибка при неправильном размере подписи (32 вместо 64)");
    
    println!("\n✅ Все проверки обработки ошибок пройдены!");
}

