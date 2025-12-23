import axios from 'axios'
import type { AxiosInstance } from 'axios'

const api: AxiosInstance = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Response interceptor
api.interceptors.response.use(
  (response) => response,
  (error) => {
    const message = error.response?.data?.error || error.message || 'Unknown error'
    return Promise.reject(new Error(message))
  }
)

export default api

// MySQL API
export const mysqlApi = {
  getInfo: () => api.get('/mysql/info'),
  
  // Databases
  listDatabases: () => api.get('/mysql/databases'),
  createDatabase: (name: string) => api.post('/mysql/databases', { name }),
  dropDatabase: (name: string) => api.delete(`/mysql/databases/${name}`),
  
  // Tables
  listTables: (database: string) => api.get(`/mysql/databases/${database}/tables`),
  createTable: (database: string, data: CreateTableRequest) => 
    api.post(`/mysql/databases/${database}/tables`, data),
  dropTable: (database: string, table: string) => 
    api.delete(`/mysql/databases/${database}/tables/${table}`),
  
  // Schema
  getTableSchema: (database: string, table: string) => 
    api.get(`/mysql/databases/${database}/tables/${table}/schema`),
  alterTable: (database: string, table: string, data: AlterTableRequest) => 
    api.put(`/mysql/databases/${database}/tables/${table}/schema`, data),
  
  // Data
  getRows: (database: string, table: string, page = 1, size = 50) => 
    api.get(`/mysql/databases/${database}/tables/${table}/rows`, { params: { page, size } }),
  insertRow: (database: string, table: string, data: Record<string, unknown>) => 
    api.post(`/mysql/databases/${database}/tables/${table}/rows`, data),
  updateRow: (database: string, table: string, data: UpdateRowRequest) => 
    api.put(`/mysql/databases/${database}/tables/${table}/rows`, data),
  deleteRow: (database: string, table: string, where: Record<string, unknown>) => 
    api.delete(`/mysql/databases/${database}/tables/${table}/rows`, { data: where }),
  
  // Query
  executeQuery: (database: string, query: string) => 
    api.post('/mysql/query', { database, query }),
  
  // Export/Import
  exportData: (database: string, table: string, format = 'json') => 
    api.post('/mysql/export', { database, table, format }),
  importData: (database: string, table: string, rows: Record<string, unknown>[]) => 
    api.post('/mysql/import', { database, table, rows }),
}

// Redis API
export const redisApi = {
  getInfo: () => api.get('/redis/info'),
  
  // Keys
  listKeys: (pattern = '*', cursor = 0, count = 100) => 
    api.get('/redis/keys', { params: { pattern, cursor, count } }),
  getKey: (key: string) => api.get(`/redis/keys/${encodeURIComponent(key)}`),
  setKey: (data: SetKeyRequest) => api.post('/redis/keys', data),
  updateKey: (key: string, data: SetKeyRequest) => 
    api.put(`/redis/keys/${encodeURIComponent(key)}`, data),
  deleteKey: (key: string) => api.delete(`/redis/keys/${encodeURIComponent(key)}`),
  
  // TTL
  setTTL: (key: string, ttl: number) => 
    api.put(`/redis/ttl/${encodeURIComponent(key)}`, { ttl }),
  
  // Export/Import
  exportKeys: (keys: string[]) => api.post('/redis/export', { keys }),
  importKeys: (data: ExportData) => api.post('/redis/import', data),
}

// System API
export const systemApi = {
  getConnections: () => api.get('/connections'),
  createConnection: (data: Connection) => api.post('/connections', data),
  getHistory: (type = '') => api.get('/history', { params: { type } }),
  getSavedQueries: () => api.get('/saved-queries'),
  saveSavedQuery: (data: SavedQuery) => api.post('/saved-queries', data),
  deleteSavedQuery: (id: number) => api.delete(`/saved-queries/${id}`),
}

// Types
export interface CreateTableRequest {
  name: string
  columns: ColumnDef[]
  engine?: string
  comment?: string
}

export interface ColumnDef {
  name: string
  type: string
  length?: number
  nullable: boolean
  default?: string
  primary_key: boolean
  auto_increment: boolean
  comment?: string
}

export interface AlterTableRequest {
  add_columns?: ColumnDef[]
  drop_columns?: string[]
  modify_columns?: ColumnDef[]
  rename_column?: { old_name: string; new_name: string }
}

export interface UpdateRowRequest {
  where: Record<string, unknown>
  data: Record<string, unknown>
}

export interface SetKeyRequest {
  key: string
  type: 'string' | 'hash' | 'list' | 'set' | 'zset'
  value: unknown
  ttl?: number
}

export interface ExportData {
  keys: KeyInfo[]
}

export interface KeyInfo {
  key: string
  type: string
  ttl: number
  value?: unknown
}

export interface Connection {
  id?: number
  name: string
  type: string
  host: string
  port: number
  username?: string
  password?: string
  database_name?: string
  is_default?: boolean
}

export interface SavedQuery {
  id?: number
  connection_id?: number
  name: string
  query_text: string
}

