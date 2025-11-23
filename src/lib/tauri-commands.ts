import { invoke } from '@tauri-apps/api/core';
import type { DbResult, EncryptionResult, DecryptionResult, ApiRequest, ApiResponse } from '../types/tauri';

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

