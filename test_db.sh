#!/bin/bash

DB_PATH="$HOME/Library/Application Support/com.sul.fam-core/wallet.db"
KEY="initialization_key"

echo "=== Проверка FAM-Core ==="
echo ""

# 1. Проверка существования БД
echo "1. Файл базы данных:"
if [ -f "$DB_PATH" ]; then
    echo "   ✓ wallet.db создан"
    ls -lh "$DB_PATH"
else
    echo "   ✗ wallet.db НЕ создан"
    exit 1
fi

echo ""

# 2. Проверка шифрования - попытка открыть без ключа
echo "2. Проверка шифрования (попытка без ключа):"
if sqlite3 "$DB_PATH" "SELECT * FROM meta;" 2>&1 | grep -q "file is not a database\|encrypted"; then
    echo "   ✓ База зашифрована (не открывается без ключа)"
else
    echo "   ⚠ База может быть не зашифрована"
fi

echo ""

# 3. Проверка содержимого с ключом
echo "3. Проверка таблицы meta с ключом:"
sqlcipher "$DB_PATH" <<EOF 2>&1
PRAGMA key = '$KEY';
SELECT * FROM meta;
.quit
EOF

echo ""
echo "=== Проверка завершена ==="

