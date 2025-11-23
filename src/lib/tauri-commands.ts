import { invoke } from '@tauri-apps/api/core';
import type { DbResult, Account, Operation, State, MasterKey, DerivedKey, CryptoConfig, ApiRequest, ApiResponse, VersionLogRecord } from '../types/tauri';

// Utility commands
export const app = {
  async getDbPath(): Promise<string> {
    return await invoke('get_db_path');
  },
};

// Database commands (low-level with path/key)
export const db = {
  async initDatabase(path: string, key: string): Promise<DbResult> {
    return await invoke('init_database', { path, key });
  },

  async checkConnection(path: string, key: string): Promise<DbResult> {
    return await invoke('check_connection', { path, key });
  },

  async executeQuery(path: string, key: string, query: string): Promise<string> {
    return await invoke('execute_query', { path, key, query });
  },

  async getVersion(path: string, key: string): Promise<string> {
    return await invoke('get_version', { path, key });
  },

  async setVersion(path: string, key: string, version: string): Promise<DbResult> {
    return await invoke('set_version', { path, key, version });
  },

  async getStatus(): Promise<string> {
    return await invoke('get_status');
  },

  async writeTestRecord(path: string, key: string, value: string): Promise<void> {
    return await invoke('write_test_record', { path, key, value });
  },

  async createAccount(path: string, key: string, name: string, accType: string): Promise<number> {
    return await invoke('create_account_command', { path, key, name, accType });
  },

  async listAccounts(path: string, key: string): Promise<Account[]> {
    return await invoke('list_accounts_command', { path, key });
  },

  async addOperation(path: string, key: string, accountId: number, amount: number, description: string): Promise<number> {
    return await invoke('add_operation_command', { path, key, accountId, amount, description });
  },

  async getOperations(path: string, key: string, accountId: number): Promise<Operation[]> {
    return await invoke('get_operations_command', { path, key, accountId });
  },
};

// API commands (high-level without path/key)
export const api = {
  // Account management
  async createAccount(name: string, accType: string): Promise<number> {
    return await invoke('create_account', { name, accType });
  },

  async listAccounts(): Promise<Account[]> {
    return await invoke('list_accounts');
  },

  // Operations management
  async addOperation(accountId: number, amount: number, description: string): Promise<number> {
    return await invoke('add_operation', { accountId, amount, description });
  },

  async getOperations(accountId: number): Promise<Operation[]> {
    return await invoke('get_operations', { accountId });
  },

  // Version log
  async listVersions(entity?: string, entityId?: number): Promise<VersionLogRecord[]> {
    return await invoke('list_versions', { 
      entity: entity || null, 
      entityId: entityId !== undefined ? entityId : null 
    });
  },

  // Aggregations
  async getAccountBalance(accountId: number): Promise<number> {
    return await invoke('get_account_balance', { accountId });
  },

  async getNetWorth(): Promise<number> {
    return await invoke('get_net_worth');
  },

  async getBalanceHistory(accountId: number): Promise<State[]> {
    return await invoke('get_balance_history', { accountId });
  },

  // HTTP requests
  async makeRequest(request: ApiRequest): Promise<ApiResponse> {
    return await invoke('make_request', { request });
  },

  async fetchData(url: string): Promise<string> {
    return await invoke('fetch_data', { url });
  },

  async postData(url: string, data: string): Promise<ApiResponse> {
    return await invoke('post_data', { url, data });
  },
};

// Crypto commands
export const crypto = {
  async generateKey(): Promise<MasterKey> {
    return await invoke('generate_key');
  },

  async derivePasswordKey(password: string): Promise<DerivedKey> {
    return await invoke('derive_password_key', { password });
  },

  async verifyPasswordKey(password: string, hash: string): Promise<boolean> {
    return await invoke('verify_password_key', { password, hash });
  },

  async getCryptoConfig(): Promise<CryptoConfig> {
    return await invoke('get_crypto_config');
  },
};

