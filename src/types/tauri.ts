// Database types
export interface DbConfig {
  path: string;
}

export interface DbResult {
  success: boolean;
  message: string;
}

// Crypto types
export interface EncryptionResult {
  data: string;
  algorithm: string;
}

export interface DecryptionResult {
  data: string;
  success: boolean;
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

