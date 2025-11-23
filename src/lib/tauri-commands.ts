import { invoke } from '@tauri-apps/api/core';
import type { DbResult, MasterKey, DerivedKey, CryptoConfig, ApiRequest, ApiResponse } from '../types/tauri';

// Database commands
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

// API commands
export const api = {
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

