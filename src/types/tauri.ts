// Database types
export interface DbConfig {
  path: string;
}

export interface DbResult {
  success: boolean;
  message: string;
}

export interface Account {
  id: number;
  name: string;
  type: string;
  created_at: number;
}

export interface Operation {
  id: number;
  account_id: number;
  amount: number;
  description: string;
  ts: number;
}

export interface State {
  id: number;
  account_id: number;
  balance: number;
  ts: number;
}

// Crypto types
export interface MasterKey {
  key: number[];
}

export interface DerivedKey {
  key: string;
  salt: string;
}

export interface CryptoConfig {
  argon2_mem_cost: number;
  argon2_time_cost: number;
  argon2_parallelism: number;
  master_key_size: number;
  algorithm: string;
}

// API types
export interface ApiRequest {
  url: string;
  method: string;
  headers?: [string, string][];
  body?: string;
}

export interface ApiResponse {
  status: number;
  body: string;
  headers: [string, string][];
}

