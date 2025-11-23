import { useState, useEffect } from "react";
import { app, db } from "./lib/tauri-commands";
import "./App.css";

function App() {
  const [inputValue, setInputValue] = useState("");
  const [result, setResult] = useState("");
  const [dbPath, setDbPath] = useState("");
  const [dbKey] = useState("initialization_key");

  useEffect(() => {
    // Получаем путь к базе данных при загрузке
    app.getDbPath()
      .then(path => setDbPath(path))
      .catch(err => setResult(`Error getting DB path: ${err}`));
  }, []);

  const handleSave = async () => {
    if (!inputValue.trim()) {
      setResult("Введите значение для сохранения");
      return;
    }

    try {
      await db.writeTestRecord(dbPath, dbKey, inputValue);
      setResult(`✓ Сохранено: "${inputValue}"`);
    } catch (error) {
      setResult(`✗ Ошибка сохранения: ${error}`);
    }
  };

  const handleStatus = async () => {
    try {
      const status = await db.getStatus();
      setResult(`Статус: ${status}`);
    } catch (error) {
      setResult(`✗ Ошибка получения статуса: ${error}`);
    }
  };

  const handleGetVersion = async () => {
    try {
      const version = await db.getVersion(dbPath, dbKey);
      setResult(`Текущее значение в БД: "${version}"`);
    } catch (error) {
      setResult(`✗ Ошибка чтения: ${error}`);
    }
  };

  return (
    <main className="container">
      <h1>FAM-Core Test Interface</h1>

      <div style={{ marginBottom: '20px', fontSize: '12px', color: '#666' }}>
        <strong>БД:</strong> {dbPath || 'загрузка...'}
      </div>

      <div style={{ marginBottom: '20px' }}>
        <input
          type="text"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          placeholder="Введите значение..."
          style={{ 
            width: '100%', 
            padding: '8px', 
            marginBottom: '10px',
            fontSize: '14px'
          }}
        />
        
        <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap' }}>
          <button onClick={handleSave}>Сохранить</button>
          <button onClick={handleStatus}>Статус</button>
          <button onClick={handleGetVersion}>Прочитать</button>
        </div>
      </div>

      {result && (
        <div style={{
          padding: '12px',
          backgroundColor: '#f5f5f5',
          borderRadius: '4px',
          fontSize: '14px',
          wordBreak: 'break-word'
        }}>
          {result}
        </div>
      )}
    </main>
  );
}

export default App;
