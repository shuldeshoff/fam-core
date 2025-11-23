import { useState, useEffect } from "react";
import { api } from "./lib/tauri-commands";
import type { Account, Operation, State, AssetAllocation, VersionLogRecord, SignedVersion } from "./types/tauri";
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
  
  // Режим верификации подписей
  const [showVerify, setShowVerify] = useState(false);
  const [verifyVersionId, setVerifyVersionId] = useState("");
  const [verifyResult, setVerifyResult] = useState<string>("");
  const [signedVersions, setSignedVersions] = useState<SignedVersion[]>([]);
  
  // Балансы и Net Worth
  const [accountBalances, setAccountBalances] = useState<Record<number, number>>({});
  const [netWorth, setNetWorth] = useState<number>(0);
  
  // Режимы отображения
  const [showAnalytics, setShowAnalytics] = useState(false);
  const [showBalanceHistory, setShowBalanceHistory] = useState(false);
  const [showAssetAllocation, setShowAssetAllocation] = useState(false);
  
  // Данные аналитики
  const [balanceHistory, setBalanceHistory] = useState<State[]>([]);
  const [assetAllocation, setAssetAllocation] = useState<AssetAllocation[]>([]);
  const [selectedHistoryAccountId, setSelectedHistoryAccountId] = useState<number | null>(null);
  
  // Сообщения об ошибках/успехе
  const [message, setMessage] = useState("");

  // Загрузка списка аккаунтов при старте
  useEffect(() => {
    loadAccounts();
    loadNetWorth();
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
      
      // Загружаем балансы для всех аккаунтов
      const balances: Record<number, number> = {};
      for (const acc of accountsList) {
        try {
          const balance = await api.getAccountBalance(acc.id);
          balances[acc.id] = balance;
        } catch {
          balances[acc.id] = 0;
        }
      }
      setAccountBalances(balances);
      
      setMessage("");
    } catch (error) {
      setMessage(`Ошибка загрузки аккаунтов: ${error}`);
    }
  };

  const loadNetWorth = async () => {
    try {
      const worth = await api.getNetWorth();
      setNetWorth(worth);
    } catch (error) {
      console.error('Ошибка загрузки Net Worth:', error);
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

  const loadBalanceHistory = async (accountId: number) => {
    try {
      const history = await api.getBalanceHistory(accountId);
      setBalanceHistory(history);
      setSelectedHistoryAccountId(accountId);
      setMessage("");
    } catch (error) {
      setMessage(`Ошибка загрузки истории балансов: ${error}`);
    }
  };

  const loadAssetAllocation = async () => {
    try {
      const allocation = await api.getAssetAllocation();
      setAssetAllocation(allocation);
      setMessage("");
    } catch (error) {
      setMessage(`Ошибка загрузки структуры активов: ${error}`);
    }
  };

  const handleShowAnalytics = async () => {
    setShowLog(false);
    setShowAnalytics(true);
    await loadNetWorth();
    await loadAssetAllocation();
  };

  const handleCloseAnalytics = () => {
    setShowAnalytics(false);
    setShowBalanceHistory(false);
    setShowAssetAllocation(false);
    setBalanceHistory([]);
    setSelectedHistoryAccountId(null);
  };

  const handleShowBalanceHistory = async (accountId: number) => {
    setShowBalanceHistory(true);
    await loadBalanceHistory(accountId);
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
      await loadNetWorth();
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
      
      // Обновляем баланс аккаунта и Net Worth
      try {
        const newBalance = await api.getAccountBalance(selectedAccountId);
        setAccountBalances(prev => ({ ...prev, [selectedAccountId]: newBalance }));
        await loadNetWorth();
      } catch (error) {
        console.error('Ошибка обновления баланса:', error);
      }
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

  const handleShowVerify = async () => {
    setShowLog(false);
    setShowAnalytics(false);
    setShowVerify(true);
    setVerifyResult("");
    setVerifyVersionId("");
    await loadAllSignedVersions();
  };

  const handleCloseVerify = () => {
    setShowVerify(false);
    setVerifyResult("");
    setVerifyVersionId("");
    setSignedVersions([]);
  };

  const handleVerifyEntry = async (e: React.FormEvent) => {
    e.preventDefault();
    
    const versionId = parseInt(verifyVersionId);
    if (isNaN(versionId)) {
      setVerifyResult("Ошибка: введите корректный version_id (число)");
      return;
    }

    try {
      const isValid = await api.verifyEntry(versionId);
      setVerifyResult(`Version ID ${versionId}: ${isValid ? "✓ Подпись валидна" : "✗ Подпись невалидна"}`);
    } catch (error) {
      setVerifyResult(`Ошибка верификации: ${error}`);
    }
  };

  const loadAllSignedVersions = async () => {
    try {
      const signed = await api.listSignedVersions();
      setSignedVersions(signed);
      setMessage("");
    } catch (error) {
      setMessage(`Ошибка загрузки подписанных версий: ${error}`);
    }
  };

  const selectedAccount = accounts.find(acc => acc.id === selectedAccountId);
  const selectedLogRecord = versionLog.find(log => log.id === selectedLogId);

  return (
    <div style={{ padding: '20px', maxWidth: '800px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
        <h1>FAM-Core</h1>
        <div style={{ display: 'flex', alignItems: 'center', gap: '20px' }}>
          <div style={{ 
            padding: '10px 15px', 
            backgroundColor: '#f0f0f0', 
            border: '2px solid #4CAF50', 
            borderRadius: '8px',
            color: '#333',
            fontWeight: 'bold'
          }}>
            Net Worth: {netWorth.toFixed(2)} ₽
          </div>
          <button onClick={showAnalytics ? handleCloseAnalytics : handleShowAnalytics}>
            {showAnalytics ? 'Закрыть аналитику' : 'Аналитика'}
          </button>
          <button onClick={showLog ? handleCloseLog : handleShowLog}>
            {showLog ? 'Закрыть журнал' : 'Журнал'}
          </button>
          <button onClick={showVerify ? handleCloseVerify : handleShowVerify}>
            {showVerify ? 'Закрыть' : 'Верификация'}
          </button>
        </div>
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

      {/* Режим аналитики */}
      {showAnalytics ? (
        <section style={{ padding: '15px', border: '1px solid #ddd' }}>
          <h2>Аналитика</h2>
          
          {/* Net Worth */}
          <div style={{ 
            marginBottom: '30px', 
            padding: '15px', 
            backgroundColor: '#f9f9f9', 
            border: '2px solid #4CAF50',
            borderRadius: '8px'
          }}>
            <h3>Net Worth</h3>
            <div style={{ fontSize: '24px', fontWeight: 'bold', color: '#4CAF50' }}>
              {netWorth.toFixed(2)} ₽
            </div>
          </div>

          {/* Структура активов */}
          <div style={{ marginBottom: '30px', padding: '15px', border: '1px solid #ddd' }}>
            <h3>Структура активов</h3>
            {assetAllocation.length === 0 ? (
              <p>Нет данных о структуре активов</p>
            ) : (
              <div>
                {assetAllocation.map((allocation) => (
                  <div 
                    key={allocation.type}
                    style={{
                      padding: '10px',
                      marginBottom: '5px',
                      border: '1px solid #ccc',
                      backgroundColor: '#fff',
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      color: '#333'
                    }}
                  >
                    <div>
                      <strong>{allocation.type}</strong>
                      <br />
                      <small>Аккаунтов: {allocation.account_count}</small>
                    </div>
                    <div style={{ 
                      fontSize: '18px', 
                      fontWeight: 'bold',
                      color: allocation.total_balance >= 0 ? '#4CAF50' : '#f44336'
                    }}>
                      {allocation.total_balance.toFixed(2)} ₽
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Временной ряд балансов */}
          <div style={{ marginBottom: '30px', padding: '15px', border: '1px solid #ddd' }}>
            <h3>Временной ряд балансов</h3>
            
            {/* Выбор аккаунта */}
            <div style={{ marginBottom: '15px' }}>
              <label style={{ display: 'block', marginBottom: '5px', color: '#333' }}>
                Выберите аккаунт:
              </label>
              <select 
                value={selectedHistoryAccountId || ''}
                onChange={(e) => {
                  const accountId = parseInt(e.target.value);
                  if (!isNaN(accountId)) {
                    handleShowBalanceHistory(accountId);
                  }
                }}
                style={{ width: '100%', padding: '8px' }}
              >
                <option value="">-- Выберите аккаунт --</option>
                {accounts.map(acc => (
                  <option key={acc.id} value={acc.id}>
                    {acc.name} ({acc.type})
                  </option>
                ))}
              </select>
            </div>

            {/* Список истории балансов */}
            {showBalanceHistory ? (
              balanceHistory.length === 0 ? (
                <p>Нет истории балансов для этого аккаунта</p>
              ) : (
                <div>
                  <p style={{ marginBottom: '10px', color: '#333' }}>
                    Найдено записей: {balanceHistory.length}
                  </p>
                  <ul style={{ listStyle: 'none', padding: 0 }}>
                    {balanceHistory.map((state, index) => (
                      <li 
                        key={state.id}
                        style={{
                          padding: '8px',
                          marginBottom: '3px',
                          border: '1px solid #ddd',
                          backgroundColor: index % 2 === 0 ? '#fff' : '#f9f9f9',
                          color: '#333',
                          fontSize: '14px',
                          display: 'flex',
                          justifyContent: 'space-between'
                        }}
                      >
                        <span>
                          <strong>ts:</strong> {state.ts} ({new Date(state.ts * 1000).toLocaleString()})
                        </span>
                        <span style={{ 
                          fontWeight: 'bold',
                          color: state.balance >= 0 ? '#4CAF50' : '#f44336'
                        }}>
                          balance: {state.balance.toFixed(2)}
                        </span>
                      </li>
                    ))}
                  </ul>
                </div>
              )
            ) : (
              <p style={{ color: '#666' }}>Выберите аккаунт для просмотра истории</p>
            )}
          </div>
        </section>
      ) : showLog ? (
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
      ) : showVerify ? (
        <section style={{ padding: '15px', border: '1px solid #ddd' }}>
          <h2>Верификация подписей</h2>
          
          {/* Форма проверки одной записи */}
          <div style={{ marginBottom: '30px', padding: '15px', border: '1px solid #ccc' }}>
            <h3>Проверить одну запись</h3>
            <form onSubmit={handleVerifyEntry}>
              <div style={{ marginBottom: '10px' }}>
                <input
                  type="text"
                  placeholder="Введите version_id"
                  value={verifyVersionId}
                  onChange={(e) => setVerifyVersionId(e.target.value)}
                  style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }}
                />
              </div>
              <button type="submit">Проверить подпись</button>
            </form>
            
            {/* Результат проверки */}
            {verifyResult && (
              <div style={{
                marginTop: '15px',
                padding: '10px',
                border: '1px solid #ccc',
                backgroundColor: verifyResult.includes('✓') ? '#d4edda' : '#f8d7da',
                color: '#333',
                fontFamily: 'monospace'
              }}>
                {verifyResult}
              </div>
            )}
          </div>
          
          {/* Список всех подписанных версий */}
          <div style={{ padding: '15px', border: '1px solid #ccc' }}>
            <h3>Все подписанные записи ({signedVersions.length})</h3>
            {signedVersions.length === 0 ? (
              <p>Нет подписанных записей</p>
            ) : (
              <ul style={{ listStyle: 'none', padding: 0 }}>
                {signedVersions.map((sv) => (
                  <li
                    key={sv.version_id}
                    style={{
                      padding: '10px',
                      marginBottom: '5px',
                      border: '1px solid #ccc',
                      backgroundColor: sv.is_valid ? '#d4edda' : '#f8d7da',
                      color: '#333',
                      fontSize: '14px'
                    }}
                  >
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                      <div>
                        <strong>Version ID: {sv.version_id}</strong>
                        {' | '}
                        <span>{sv.entity}</span>
                        {' '}
                        <small>(entity_id: {sv.entity_id})</small>
                        {' | '}
                        <span>{sv.action}</span>
                        <br />
                        <small>{new Date(sv.ts * 1000).toLocaleString()}</small>
                      </div>
                      <div style={{
                        fontWeight: 'bold',
                        fontSize: '16px',
                        color: sv.is_valid ? '#155724' : '#721c24'
                      }}>
                        {sv.is_valid ? '✓ Валидна' : '✗ Невалидна'}
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
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
                  color: '#333',
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center'
                }}
              >
                <div>
                  <strong>{account.name}</strong> ({account.type})
                  <br />
                  <small>ID: {account.id}, Создан: {new Date(account.created_at * 1000).toLocaleString()}</small>
                </div>
                <div style={{ 
                  fontSize: '18px', 
                  fontWeight: 'bold', 
                  color: (accountBalances[account.id] || 0) >= 0 ? '#4CAF50' : '#f44336' 
                }}>
                  {(accountBalances[account.id] || 0).toFixed(2)} ₽
                </div>
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
