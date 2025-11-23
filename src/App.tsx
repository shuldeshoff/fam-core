import { useState, useEffect } from "react";
import { api } from "./lib/tauri-commands";
import type { Account, Operation, VersionLogRecord } from "./types/tauri";
import "./App.css";

function App() {
  // Форма создания аккаунта
  const [accountName, setAccountName] = useState("");
  const [accountType, setAccountType] = useState("");
  
  // Список аккаунтов
  const [accounts, setAccounts] = useState<Account[]>([]);
  
  // Выбранный аккаунт
  const [selectedAccountId, setSelectedAccountId] = useState<number | null>(null);
  
  // Форма добавления операции
  const [operationAmount, setOperationAmount] = useState("");
  const [operationDescription, setOperationDescription] = useState("");
  
  // Список операций
  const [operations, setOperations] = useState<Operation[]>([]);
  
  // Режим просмотра журнала
  const [showLog, setShowLog] = useState(false);
  const [versionLog, setVersionLog] = useState<VersionLogRecord[]>([]);
  const [selectedLogId, setSelectedLogId] = useState<number | null>(null);
  
  // Сообщения об ошибках/успехе
  const [message, setMessage] = useState("");

  // Загрузка списка аккаунтов при старте
  useEffect(() => {
    loadAccounts();
  }, []);

  // Загрузка операций при выборе аккаунта
  useEffect(() => {
    if (selectedAccountId !== null) {
      loadOperations(selectedAccountId);
    } else {
      setOperations([]);
    }
  }, [selectedAccountId]);

  const loadAccounts = async () => {
    try {
      const accountsList = await api.listAccounts();
      setAccounts(accountsList);
      setMessage("");
    } catch (error) {
      setMessage(`Ошибка загрузки аккаунтов: ${error}`);
    }
  };

  const loadOperations = async (accountId: number) => {
    try {
      const operationsList = await api.getOperations(accountId);
      setOperations(operationsList);
      setMessage("");
    } catch (error) {
      setMessage(`Ошибка загрузки операций: ${error}`);
    }
  };

  const handleCreateAccount = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!accountName.trim() || !accountType.trim()) {
      setMessage("Заполните имя и тип аккаунта");
      return;
    }

    try {
      await api.createAccount(accountName, accountType);
      setMessage(`Аккаунт "${accountName}" создан`);
      setAccountName("");
      setAccountType("");
      await loadAccounts();
    } catch (error) {
      setMessage(`Ошибка создания аккаунта: ${error}`);
    }
  };

  const handleSelectAccount = (accountId: number) => {
    setSelectedAccountId(accountId);
    setOperationAmount("");
    setOperationDescription("");
    setMessage("");
  };

  const handleAddOperation = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (selectedAccountId === null) {
      setMessage("Выберите аккаунт");
      return;
    }

    if (!operationAmount || !operationDescription.trim()) {
      setMessage("Заполните сумму и описание операции");
      return;
    }

    try {
      const amount = parseFloat(operationAmount);
      if (isNaN(amount)) {
        setMessage("Неверный формат суммы");
        return;
      }

      await api.addOperation(selectedAccountId, amount, operationDescription);
      setMessage(`Операция добавлена: ${amount > 0 ? '+' : ''}${amount}`);
      setOperationAmount("");
      setOperationDescription("");
      await loadOperations(selectedAccountId);
    } catch (error) {
      setMessage(`Ошибка добавления операции: ${error}`);
    }
  };

  const handleShowLog = async () => {
    try {
      const log = await api.listVersions();
      setVersionLog(log);
      setShowLog(true);
      setSelectedLogId(null);
      setMessage("");
    } catch (error) {
      setMessage(`Ошибка загрузки журнала: ${error}`);
    }
  };

  const handleCloseLog = () => {
    setShowLog(false);
    setSelectedLogId(null);
  };

  const handleSelectLogRecord = (logId: number) => {
    setSelectedLogId(selectedLogId === logId ? null : logId);
  };

  const selectedAccount = accounts.find(acc => acc.id === selectedAccountId);
  const selectedLogRecord = versionLog.find(log => log.id === selectedLogId);

  return (
    <div style={{ padding: '20px', maxWidth: '800px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h1>FAM-Core</h1>
        <button onClick={showLog ? handleCloseLog : handleShowLog}>
          {showLog ? 'Закрыть журнал' : 'Журнал'}
        </button>
      </div>

      {/* Сообщения */}
      {message && (
        <div style={{ 
          padding: '10px', 
          marginBottom: '20px', 
          backgroundColor: '#f0f0f0',
          border: '1px solid #ccc',
          color: '#333'
        }}>
          {message}
        </div>
      )}

      {/* Режим просмотра журнала */}
      {showLog ? (
        <section style={{ padding: '15px', border: '1px solid #ddd' }}>
          <h2>Журнал изменений ({versionLog.length})</h2>
          {versionLog.length === 0 ? (
            <p>Журнал пуст</p>
          ) : (
            <ul style={{ listStyle: 'none', padding: 0 }}>
              {versionLog.map((log) => (
                <li key={log.id}>
                  <div
                    onClick={() => handleSelectLogRecord(log.id)}
                    style={{
                      padding: '10px',
                      marginBottom: '5px',
                      border: '1px solid #ccc',
                      backgroundColor: selectedLogId === log.id ? '#e0e0e0' : '#fff',
                      cursor: 'pointer',
                      color: '#333'
                    }}
                  >
                    <div>
                      <strong>{new Date(log.ts * 1000).toLocaleString()}</strong>
                      {' | '}
                      <span style={{ 
                        color: log.action === 'create' ? 'green' : log.action === 'delete' ? 'red' : 'blue' 
                      }}>
                        {log.action}
                      </span>
                      {' | '}
                      <span>{log.entity}</span>
                      {' '}
                      <small>(ID: {log.entity_id})</small>
                    </div>
                  </div>
                  {selectedLogId === log.id && selectedLogRecord && (
                    <div style={{
                      padding: '10px',
                      marginBottom: '10px',
                      backgroundColor: '#f9f9f9',
                      border: '1px solid #ddd',
                      borderTop: 'none',
                      color: '#333',
                      fontSize: '12px',
                      fontFamily: 'monospace',
                      whiteSpace: 'pre-wrap',
                      wordBreak: 'break-all'
                    }}>
                      {JSON.stringify(JSON.parse(selectedLogRecord.payload), null, 2)}
                    </div>
                  )}
                </li>
              ))}
            </ul>
          )}
        </section>
      ) : (
        <>
          {/* Форма создания аккаунта */}
      <section style={{ marginBottom: '30px', padding: '15px', border: '1px solid #ddd' }}>
        <h2>Создать аккаунт</h2>
        <form onSubmit={handleCreateAccount}>
          <div style={{ marginBottom: '10px' }}>
            <input
              type="text"
              placeholder="Название аккаунта"
              value={accountName}
              onChange={(e) => setAccountName(e.target.value)}
              style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
            />
          </div>
          <div style={{ marginBottom: '10px' }}>
            <div style={{ marginBottom: '8px', fontSize: '14px', color: '#333', fontWeight: '500' }}>Тип:</div>
            <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap' }}>
              {[
                { value: 'cash', label: '💵 Наличные' },
                { value: 'card', label: '💳 Карта' },
                { value: 'bank', label: '🏦 Банк' }
              ].map((type) => (
                <div
                  key={type.value}
                  onClick={() => setAccountType(type.value)}
                  style={{
                    padding: '10px 20px',
                    border: '2px solid',
                    borderColor: accountType === type.value ? '#007bff' : '#ddd',
                    borderRadius: '24px',
                    backgroundColor: accountType === type.value ? '#007bff' : '#fff',
                    color: accountType === type.value ? '#fff' : '#333',
                    cursor: 'pointer',
                    transition: 'all 0.2s ease',
                    userSelect: 'none',
                    fontWeight: accountType === type.value ? '600' : '400',
                    fontSize: '14px',
                    boxShadow: accountType === type.value ? '0 2px 8px rgba(0,123,255,0.3)' : 'none'
                  }}
                >
                  {type.label}
                </div>
              ))}
            </div>
          </div>
          <button type="submit">Создать</button>
        </form>
      </section>

      {/* Список аккаунтов */}
      <section style={{ marginBottom: '30px', padding: '15px', border: '1px solid #ddd' }}>
        <h2>Аккаунты</h2>
        {accounts.length === 0 ? (
          <p>Нет аккаунтов. Создайте первый аккаунт выше.</p>
        ) : (
          <ul style={{ listStyle: 'none', padding: 0 }}>
            {accounts.map((account) => (
              <li
                key={account.id}
                onClick={() => handleSelectAccount(account.id)}
                style={{
                  padding: '10px',
                  marginBottom: '5px',
                  border: '1px solid #ccc',
                  backgroundColor: selectedAccountId === account.id ? '#e0e0e0' : '#fff',
                  cursor: 'pointer',
                  color: '#333'
                }}
              >
                <strong>{account.name}</strong> ({account.type})
                <br />
                <small>ID: {account.id}, Создан: {new Date(account.created_at * 1000).toLocaleString()}</small>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* Форма добавления операции (показывается только если выбран аккаунт) */}
      {selectedAccountId !== null && (
        <section style={{ marginBottom: '30px', padding: '15px', border: '1px solid #ddd' }}>
          <h2>Добавить операцию для: {selectedAccount?.name}</h2>
          <form onSubmit={handleAddOperation}>
            <div style={{ marginBottom: '10px' }}>
              <input
                type="text"
                placeholder="Сумма (+ для дохода, - для расхода)"
                value={operationAmount}
                onChange={(e) => setOperationAmount(e.target.value)}
                style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
              />
            </div>
            <div style={{ marginBottom: '10px' }}>
        <input
                type="text"
                placeholder="Описание операции"
                value={operationDescription}
                onChange={(e) => setOperationDescription(e.target.value)}
                style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
              />
            </div>
            <button type="submit">Добавить операцию</button>
      </form>
        </section>
      )}

      {/* Список операций (показывается только если выбран аккаунт) */}
      {selectedAccountId !== null && (
        <section style={{ padding: '15px', border: '1px solid #ddd' }}>
          <h2>Операции: {selectedAccount?.name}</h2>
          {operations.length === 0 ? (
            <p>Нет операций для этого аккаунта.</p>
          ) : (
            <ul style={{ listStyle: 'none', padding: 0 }}>
              {operations.map((operation) => (
                <li
                  key={operation.id}
                  style={{
                    padding: '10px',
                    marginBottom: '5px',
                    border: '1px solid #ccc',
                    backgroundColor: '#fff',
                    color: '#333'
                  }}
                >
                  <strong style={{ color: operation.amount >= 0 ? 'green' : 'red' }}>
                    {operation.amount >= 0 ? '+' : ''}{operation.amount}
                  </strong>
                  {' — '}
                  {operation.description}
                  <br />
                  <small>{new Date(operation.ts * 1000).toLocaleString()}</small>
                </li>
              ))}
            </ul>
          )}
        </section>
      )}
        </>
      )}
    </div>
  );
}

export default App;
