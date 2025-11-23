import { invoke } from '@tauri-apps/api/core';
import type { DbConfig, DbResult, EncryptionResult, DecryptionResult, ApiRequest, ApiResponse } from '../types/tauri';

// Database commands
export const db = {
  async initDatabase(config: DbConfig): Promise<DbResult> {
    return await invoke('init_database', { config });
  },

  async checkConnection(): Promise<DbResult> {
    return await invoke('check_connection');
  },

  async executeQuery(query: string): Promise<string> {
    return await invoke('execute_query', { query });
  },
};

// Crypto commands
export const crypto = {
  async encryptData(data: string, key: string): Promise<EncryptionResult> {
    return await invoke('encrypt_data', { data, key });
  },

  async decryptData(data: string, key: string): Promise<DecryptionResult> {
    return await invoke('decrypt_data', { data, key });
  },

  async generateHash(data: string): Promise<string> {
    return await invoke('generate_hash', { data });
  },

  async verifyHash(data: string, hash: string): Promise<boolean> {
    return await invoke('verify_hash', { data, hash });
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

