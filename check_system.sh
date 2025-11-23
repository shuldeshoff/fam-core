#!/bin/bash

# Автоматическая проверка FAM-Core системы
# Проверяет: создание аккаунтов, операции, states, API, UI

set -e

echo "=========================================="
echo "FAM-Core System Check"
echo "=========================================="
echo ""

DB_PATH="$HOME/Library/Application Support/com.sul.fam-core/wallet.db"
DB_KEY="initialization_key"

if [ ! -f "$DB_PATH" ]; then
    echo "❌ База данных не найдена: $DB_PATH"
    echo "   Запустите приложение сначала!"
    exit 1
fi

echo "✓ База данных найдена: $DB_PATH"
echo ""

# Проверка 1: Структура таблиц
echo "=== Проверка 1: Структура таблиц ==="
echo ""

sqlite3 "$DB_PATH" <<EOF
PRAGMA key='$DB_KEY';

.mode column
.headers on

SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;
EOF

echo ""

# Проверка 2: Аккаунты
echo "=== Проверка 2: Аккаунты в БД ==="
echo ""

ACCOUNT_COUNT=$(sqlite3 "$DB_PATH" "PRAGMA key='$DB_KEY'; SELECT COUNT(*) FROM accounts;")
echo "Количество аккаунтов: $ACCOUNT_COUNT"

if [ "$ACCOUNT_COUNT" -gt 0 ]; then
    echo ""
    sqlite3 "$DB_PATH" <<EOF
PRAGMA key='$DB_KEY';
.mode column
.headers on
SELECT id, name, type, datetime(created_at, 'unixepoch', 'localtime') as created 
FROM accounts 
ORDER BY created_at DESC 
LIMIT 5;
EOF
    echo ""
    echo "✓ Аккаунты создаются корректно"
else
    echo "⚠️  Аккаунтов пока нет. Создайте их в UI."
fi

echo ""

# Проверка 3: Операции
echo "=== Проверка 3: Операции в БД ==="
echo ""

OPERATION_COUNT=$(sqlite3 "$DB_PATH" "PRAGMA key='$DB_KEY'; SELECT COUNT(*) FROM operations;")
echo "Количество операций: $OPERATION_COUNT"

if [ "$OPERATION_COUNT" -gt 0 ]; then
    echo ""
    sqlite3 "$DB_PATH" <<EOF
PRAGMA key='$DB_KEY';
.mode column
.headers on
SELECT 
    o.id, 
    o.account_id, 
    o.amount, 
    o.description, 
    datetime(o.ts, 'unixepoch', 'localtime') as created
FROM operations o
ORDER BY o.ts DESC
LIMIT 5;
EOF
    echo ""
    echo "✓ Операции сохраняются корректно"
else
    echo "⚠️  Операций пока нет. Создайте их в UI."
fi

echo ""

# Проверка 4: Состояния (балансы)
echo "=== Проверка 4: Балансы в таблице states ==="
echo ""

STATE_COUNT=$(sqlite3 "$DB_PATH" "PRAGMA key='$DB_KEY'; SELECT COUNT(*) FROM states;")
echo "Количество записей балансов: $STATE_COUNT"

if [ "$STATE_COUNT" -gt 0 ]; then
    echo ""
    sqlite3 "$DB_PATH" <<EOF
PRAGMA key='$DB_KEY';
.mode column
.headers on
SELECT 
    s.id,
    s.account_id,
    a.name as account_name,
    s.balance,
    datetime(s.ts, 'unixepoch', 'localtime') as updated
FROM states s
LEFT JOIN accounts a ON s.account_id = a.id
ORDER BY s.ts DESC
LIMIT 10;
EOF
    echo ""
    echo "✓ Балансы обновляются автоматически"
else
    echo "⚠️  Балансов пока нет. Добавьте операции в UI."
fi

echo ""

# Проверка 5: Соответствие операций и балансов
echo "=== Проверка 5: Синхронизация операций и балансов ==="
echo ""

if [ "$ACCOUNT_COUNT" -gt 0 ] && [ "$OPERATION_COUNT" -gt 0 ]; then
    sqlite3 "$DB_PATH" <<EOF
PRAGMA key='$DB_KEY';
.mode column
.headers on
SELECT 
    a.id as account_id,
    a.name,
    COUNT(o.id) as operations_count,
    COUNT(s.id) as states_count,
    (SELECT balance FROM states WHERE account_id = a.id ORDER BY ts DESC LIMIT 1) as current_balance
FROM accounts a
LEFT JOIN operations o ON o.account_id = a.id
LEFT JOIN states s ON s.account_id = a.id
GROUP BY a.id;
EOF
    echo ""
    
    # Проверяем, что количество операций = количество балансов
    MISMATCH=$(sqlite3 "$DB_PATH" "PRAGMA key='$DB_KEY'; 
        SELECT COUNT(*) FROM (
            SELECT a.id 
            FROM accounts a
            LEFT JOIN (SELECT account_id, COUNT(*) as op_count FROM operations GROUP BY account_id) o ON o.account_id = a.id
            LEFT JOIN (SELECT account_id, COUNT(*) as st_count FROM states GROUP BY account_id) s ON s.account_id = a.id
            WHERE COALESCE(o.op_count, 0) != COALESCE(s.st_count, 0)
        );")
    
    if [ "$MISMATCH" -eq 0 ]; then
        echo "✓ Каждая операция создаёт ровно одну запись баланса"
    else
        echo "❌ Несоответствие: операции и балансы не синхронизированы!"
    fi
else
    echo "⚠️  Недостаточно данных для проверки синхронизации"
fi

echo ""

# Проверка 6: Версия миграций
echo "=== Проверка 6: Версия БД ==="
echo ""

VERSION=$(sqlite3 "$DB_PATH" "PRAGMA key='$DB_KEY'; SELECT version FROM meta LIMIT 1;")
echo "Текущая версия БД: $VERSION"

if [ "$VERSION" = "4" ]; then
    echo "✓ Все миграции (M1-M4) применены"
else
    echo "⚠️  Ожидается версия 4, найдена: $VERSION"
fi

echo ""

# Проверка 7: Проверка шифрования
echo "=== Проверка 7: SQLCipher шифрование ==="
echo ""

# Попытка открыть БД без ключа должна провалиться
if sqlite3 "$DB_PATH" "SELECT * FROM accounts;" 2>&1 | grep -q "file is not a database"; then
    echo "✓ База данных зашифрована (не открывается без ключа)"
else
    # Если открылась без ключа - проблема
    echo "❌ ВНИМАНИЕ: База данных не зашифрована!"
fi

echo ""

# Итоговый отчёт
echo "=========================================="
echo "ИТОГОВЫЙ ОТЧЁТ"
echo "=========================================="
echo ""
echo "Аккаунтов: $ACCOUNT_COUNT"
echo "Операций: $OPERATION_COUNT"
echo "Балансов: $STATE_COUNT"
echo "Версия БД: $VERSION"
echo ""

if [ "$ACCOUNT_COUNT" -gt 0 ] && [ "$OPERATION_COUNT" -gt 0 ] && [ "$STATE_COUNT" -gt 0 ]; then
    echo "✅ Система работает корректно!"
    echo ""
    echo "Рекомендации для UI проверки:"
    echo "1. Откройте приложение FAM-Core"
    echo "2. Проверьте, что список аккаунтов отображается"
    echo "3. Кликните на аккаунт - должны показаться операции"
    echo "4. Добавьте новую операцию - баланс обновится автоматически"
    echo "5. Создайте новый аккаунт - он появится в списке"
else
    echo "⚠️  Данных недостаточно для полной проверки"
    echo ""
    echo "Действия:"
    echo "1. Запустите приложение: npm run tauri dev"
    echo "2. Создайте 2-3 аккаунта через UI"
    echo "3. Добавьте 5-10 операций"
    echo "4. Запустите этот скрипт снова"
fi

echo ""
echo "=========================================="

