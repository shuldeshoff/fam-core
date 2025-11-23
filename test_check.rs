use rusqlite::Connection;
use std::path::PathBuf;

fn main() {
    let home = std::env::var("HOME").unwrap();
    let db_path = PathBuf::from(home)
        .join("Library/Application Support/com.sul.fam-core/wallet.db");
    
    println!("=== Проверка FAM-Core ===\n");
    
    // 1. Проверка файла
    println!("1. Файл базы данных:");
    if db_path.exists() {
        println!("   ✓ wallet.db создан: {}", db_path.display());
    } else {
        println!("   ✗ wallet.db НЕ создан");
        return;
    }
    
    println!();
    
    // 2. Проверка без ключа
    println!("2. Попытка открыть БД без ключа:");
    match Connection::open(&db_path) {
        Ok(conn) => {
            match conn.query_row("SELECT * FROM meta", [], |_| Ok(())) {
                Ok(_) => println!("   ✗ База НЕ зашифрована!"),
                Err(e) => {
                    if e.to_string().contains("file is not a database") || 
                       e.to_string().contains("encrypted") {
                        println!("   ✓ База зашифрована (ошибка: {})", e);
                    } else {
                        println!("   ? Неожиданная ошибка: {}", e);
                    }
                }
            }
        }
        Err(e) => println!("   ✗ Ошибка открытия: {}", e),
    }
    
    println!();
    
    // 3. Проверка с ключом
    println!("3. Открытие БД с ключом 'initialization_key':");
    match Connection::open(&db_path) {
        Ok(conn) => {
            // Устанавливаем ключ
            if let Err(e) = conn.pragma_update(None, "key", "initialization_key") {
                println!("   ✗ Ошибка установки ключа: {}", e);
                return;
            }
            println!("   ✓ PRAGMA key выполнен");
            
            // Читаем таблицу meta
            match conn.query_row(
                "SELECT version FROM meta LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            ) {
                Ok(version) => {
                    println!("   ✓ Таблица meta существует");
                    println!("   ✓ Значение в meta: '{}'", version);
                }
                Err(e) => println!("   ✗ Ошибка чтения meta: {}", e),
            }
        }
        Err(e) => println!("   ✗ Ошибка открытия: {}", e),
    }
    
    println!("\n=== Проверка завершена ===");
}

