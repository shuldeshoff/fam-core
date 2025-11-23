# Сборка FAM-Core

## Кроссплатформенная сборка

FAM-Core настроен для сборки под Windows, macOS и Linux с встроенным SQLCipher.

### Требования

- **Rust** 1.70+
- **Node.js** 18+
- **Platform-specific tools**:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `build-essential`, `libwebkit2gtk-4.1-dev`, `libssl-dev`
  - **Windows**: Visual Studio Build Tools

### Сборка для текущей платформы

```bash
npm run tauri build
```

### Целевые форматы

#### macOS
- **DMG** - установщик для macOS
- **APP** - приложение для macOS

#### Windows
- **MSI** - установщик Windows
- **NSIS** - установщик NSIS
- **EXE** - портативный исполняемый файл

#### Linux
- **DEB** - пакет для Debian/Ubuntu
- **AppImage** - универсальный формат для Linux
- **RPM** - пакет для Red Hat/Fedora

### SQLCipher

SQLCipher встроен через feature `bundled-sqlcipher` в rusqlite:
- Не требует системной установки OpenSSL
- Компилируется статически в бинарник
- Работает одинаково на всех платформах

### Размещение файлов

Приложение создаёт базу данных в:
- **macOS**: `~/Library/Application Support/com.sul.fam-core/`
- **Windows**: `C:\Users\{user}\AppData\Roaming\com.sul.fam-core\`
- **Linux**: `~/.local/share/com.sul.fam-core/`

### Конфигурация

Настройки сборки находятся в `src-tauri/tauri.conf.json`:
- `targets: "all"` - сборка всех форматов для платформы
- Настройки специфичные для платформ в секциях `windows`, `linux`, `macOS`

### Разработка

```bash
# Запуск в режиме разработки
npm run tauri dev

# Проверка кода
cargo check --manifest-path=src-tauri/Cargo.toml

# Сборка frontend
npm run build
```

## Минимальные системные требования

- **macOS**: 10.13+
- **Windows**: Windows 7+
- **Linux**: современный дистрибутив с GTK 3.24+

