# Схема базы данных FAM-Core

## Таблицы

### meta
Таблица для отслеживания версии схемы БД.

```sql
CREATE TABLE meta (
    version TEXT NOT NULL
)
```

### accounts
Счета пользователя.

```sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    created_at INTEGER NOT NULL
)
```

**Индексы:**
- `idx_accounts_type` на поле `type`

**Описание полей:**
- `id` - уникальный идентификатор счёта
- `name` - название счёта
- `type` - тип счёта (например: "cash", "card", "savings")
- `created_at` - timestamp создания (Unix time)

### operations
Операции по счетам.

```sql
CREATE TABLE operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    amount REAL NOT NULL,
    description TEXT NOT NULL,
    ts INTEGER NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
)
```

**Индексы:**
- `idx_operations_account_id` на поле `account_id`
- `idx_operations_ts` на поле `ts`

**Описание полей:**
- `id` - уникальный идентификатор операции
- `account_id` - ссылка на счёт
- `amount` - сумма операции (положительная или отрицательная)
- `description` - описание операции
- `ts` - timestamp операции (Unix time)

### states
Снимки балансов счетов.

```sql
CREATE TABLE states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    balance REAL NOT NULL,
    ts INTEGER NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
)
```

**Индексы:**
- `idx_states_account_id` на поле `account_id`
- `idx_states_ts` на поле `ts`
- `idx_states_account_ts` уникальный индекс на `(account_id, ts)`

**Описание полей:**
- `id` - уникальный идентификатор снимка
- `account_id` - ссылка на счёт
- `balance` - баланс на момент снимка
- `ts` - timestamp снимка (Unix time)

## Миграции

Система миграций автоматически применяется при инициализации БД:

- **v1**: Создание таблицы meta
- **v2**: Создание таблицы accounts
- **v3**: Создание таблицы operations
- **v4**: Создание таблицы states

Каждая миграция выполняется только один раз. Версия БД отслеживается в таблице `meta`.

## Особенности

- **Шифрование**: Вся БД зашифрована через SQLCipher
- **Внешние ключи**: Включены (`PRAGMA foreign_keys = ON`)
- **Каскадное удаление**: При удалении счёта удаляются все связанные операции и состояния
- **Индексы**: Оптимизированы для быстрого поиска по времени и счетам

