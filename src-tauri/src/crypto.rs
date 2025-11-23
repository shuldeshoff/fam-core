use argon2::{
    Argon2, 
    password_hash::{PasswordHasher, SaltString, PasswordHash, PasswordVerifier},
    ParamsBuilder, Version, Algorithm,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Безопасные константы для Argon2id
const ARGON2_MEM_COST: u32 = 65536; // 64 MB
const ARGON2_TIME_COST: u32 = 3;    // 3 итерации
const ARGON2_PARALLELISM: u32 = 4;  // 4 параллельных потока
const MASTER_KEY_SIZE: usize = 32;  // 32 байта для мастер-ключа

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Failed to generate master key: {0}")]
    KeyGenerationError(String),
    
    #[error("Failed to derive key: {0}")]
    KeyDerivationError(String),
    
    #[error("Invalid key format: {0}")]
    InvalidKeyError(String),
    
    #[error("Key verification failed")]
    VerificationError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MasterKey {
    pub key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DerivedKey {
    pub key: String,
    pub salt: String,
}

/// Генерация мастер-ключа (32 байта)
pub fn generate_master_key() -> Result<Vec<u8>, CryptoError> {
    let mut key = vec![0u8; MASTER_KEY_SIZE];
    OsRng.fill_bytes(&mut key);
    Ok(key)
}

/// Деривация ключа из пароля через Argon2id
pub fn derive_key(password: &str) -> Result<DerivedKey, CryptoError> {
    // Генерируем случайную соль
    let salt = SaltString::generate(&mut OsRng);
    
    // Настраиваем параметры Argon2id
    let params = ParamsBuilder::new()
        .m_cost(ARGON2_MEM_COST)
        .t_cost(ARGON2_TIME_COST)
        .p_cost(ARGON2_PARALLELISM)
        .build()
        .map_err(|e| CryptoError::KeyDerivationError(format!("Failed to build params: {}", e)))?;
    
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    // Хешируем пароль
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CryptoError::KeyDerivationError(format!("Failed to hash password: {}", e)))?;
    
    Ok(DerivedKey {
        key: password_hash.to_string(),
        salt: salt.to_string(),
    })
}

/// Проверка корректности ключа
pub fn verify_key(password: &str, hash: &str) -> Result<bool, CryptoError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| CryptoError::InvalidKeyError(format!("Failed to parse hash: {}", e)))?;
    
    let params = ParamsBuilder::new()
        .m_cost(ARGON2_MEM_COST)
        .t_cost(ARGON2_TIME_COST)
        .p_cost(ARGON2_PARALLELISM)
        .build()
        .map_err(|_| CryptoError::VerificationError)?;
    
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// Tauri команды

/// Генерация мастер-ключа
#[tauri::command]
pub async fn generate_key() -> Result<MasterKey, String> {
    generate_master_key()
        .map(|key| MasterKey { key })
        .map_err(|e| format!("Failed to generate key: {}", e))
}

/// Деривация ключа из пароля
#[tauri::command]
pub async fn derive_password_key(password: String) -> Result<DerivedKey, String> {
    derive_key(&password)
        .map_err(|e| format!("Failed to derive key: {}", e))
}

/// Проверка ключа
#[tauri::command]
pub async fn verify_password_key(password: String, hash: String) -> Result<bool, String> {
    verify_key(&password, &hash)
        .map_err(|e| format!("Failed to verify key: {}", e))
}

/// Получение конфигурационных констант
#[tauri::command]
pub async fn get_crypto_config() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "argon2_mem_cost": ARGON2_MEM_COST,
        "argon2_time_cost": ARGON2_TIME_COST,
        "argon2_parallelism": ARGON2_PARALLELISM,
        "master_key_size": MASTER_KEY_SIZE,
        "algorithm": "Argon2id",
    }))
}
