# Отчёт о тестировании SQLCipher и миграций M2-M4

**Дата:** 2025-11-23  
**Проект:** FAM-Core  
**Версия БД:** 4

## ✅ Все тесты пройдены успешно!

```
test test_migration_version_tracking ... ok
test test_sqlcipher_and_migrations ... ok

test result: ok. 2 passed; 0 failed
```

---

## Проверка 1: Инициализация БД с шифрованием

✓ База данных создаётся и инициализируется  
✓ `PRAGMA key` устанавливается корректно  
✓ Таблица `meta` создаётся с версией "4"

---

## Проверка 2: SQLCipher работает корректно

✓ **Неправильный ключ отклоняется** — попытка чтения с `wrong_key` возвращает ошибку  
✓ **Правильный ключ принимается** — доступ к данным открывается  
✓ **Версия БД:** `4` (все миграции применены)

---

## Проверка 3: Миграция M2 — Таблица `accounts`

✓ Счёт создаётся: `create_account("Тестовый счёт", "cash")` → ID: 1  
✓ Счёт читается: `list_accounts()` возвращает массив с 1 счётом  
✓ Поля корректны: `name = "Тестовый счёт"`, `type = "cash"`  
✓ `created_at` заполняется автоматически (timestamp)

---

## Проверка 4: Миграция M3 — Таблица `operations`

✓ Операция создаётся: `add_operation(account_id, 100.50, "Тестовая операция")` → ID: 1  
✓ Операция читается: `get_operations(account_id)` возвращает массив с 1 операцией  
✓ Поля корректны: `amount = 100.50`, `description = "Тестовая операция"`  
✓ `ts` заполняется автоматически (timestamp)

---

## Проверка 5: Миграция M4 — Таблица `states` (балансы)

✓ Первая операция создаёт запись баланса: `balance = 100.50`  
✓ Вторая операция обновляет баланс атомарно:
  - Операция: `-20.30`
  - Ожидаемый баланс: `80.20`
  - Фактический баланс: `80.20` ✓

---

## Проверка 6: Атомарность транзакций

✓ Каждая операция создаёт ровно одну запись баланса  
✓ Всего записей в `states`: **2** (по одной на каждую операцию)  
✓ Транзакция откатывается при ошибке (уникальный индекс на `account_id, ts` работает)

---

## Проверка 7: Версионирование миграций

✓ После `init_db()` версия БД автоматически устанавливается в **"4"**  
✓ Все миграции (M1-M4) применяются последовательно  
✓ Версия сохраняется в таблице `meta`

---

## Зарегистрированные команды в `tauri::Builder`

### Database Commands (низкоуровневые)
- ✓ `db::init_database`
- ✓ `db::check_connection`
- ✓ `db::execute_query`
- ✓ `db::get_version`
- ✓ `db::set_version`
- ✓ `db::get_status`
- ✓ `db::write_test_record`
- ✓ `db::get_db_path`
- ✓ `db::create_account_command`
- ✓ `db::list_accounts_command`
- ✓ `db::add_operation_command`
- ✓ `db::get_operations_command`

### Crypto Commands
- ✓ `crypto::generate_key`
- ✓ `crypto::derive_password_key`
- ✓ `crypto::verify_password_key`
- ✓ `crypto::get_crypto_config`

### API Commands (высокоуровневые)
- ✓ `api::create_account` — без path/key
- ✓ `api::list_accounts` — без path/key
- ✓ `api::add_operation` — без path/key
- ✓ `api::get_operations` — без path/key
- ✓ `api::make_request`
- ✓ `api::fetch_data`
- ✓ `api::post_data`

---

## Структура таблиц после миграций

### `meta`
```sql
CREATE TABLE IF NOT EXISTS meta (
    version TEXT PRIMARY KEY
);
```
Текущая версия: **"4"**

### `accounts` (M2)
```sql
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
```

### `operations` (M3)
```sql
CREATE TABLE IF NOT EXISTS operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    amount REAL NOT NULL,
    description TEXT NOT NULL,
    ts INTEGER NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);
```

### `states` (M4)
```sql
CREATE TABLE IF NOT EXISTS states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    balance REAL NOT NULL,
    ts INTEGER NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);
```

Индексы:
- `idx_operations_account` на `operations(account_id)`
- `idx_operations_ts` на `operations(ts)`
- `idx_states_account` на `states(account_id)`
- `idx_states_account_ts` (UNIQUE) на `states(account_id, ts)`

---

## Выводы

✅ SQLCipher работает корректно с bundled-режимом  
✅ Все миграции (M1-M4) применяются успешно  
✅ Шифрование защищает данные от доступа без ключа  
✅ Атомарные транзакции гарантируют консистентность балансов  
✅ Высокоуровневое API упрощает работу фронтенда  
✅ Версионирование миграций работает правильно  

**Система готова к дальнейшей разработке!**

