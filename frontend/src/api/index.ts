import axios from 'axios'
import type { AxiosInstance, InternalAxiosRequestConfig } from 'axios'

const api: AxiosInstance = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Active connection IDs per type (managed by connection store)
let activeConnectionIds: Record<string, number | null> = {}

// Function to set active connection ID (called by connection store)
export function setActiveConnectionId(type: string, id: number | null) {
  activeConnectionIds[type] = id
}

// Function to get active connection ID
export function getActiveConnectionId(type: string): number | null {
  return activeConnectionIds[type] ?? null
}

// Request interceptor to inject X-Connection-ID header
api.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    // Determine connection type based on URL
    const url = config.url || ''
    let connectionType: string | null = null
    
    if (url.startsWith('/mysql')) {
      connectionType = 'mysql'
    } else if (url.startsWith('/redis')) {
      connectionType = 'redis'
    } else if (url.startsWith('/mongodb')) {
      connectionType = 'mongodb'
    } else if (url.startsWith('/minio')) {
      connectionType = 'minio'
    }
    
    if (connectionType) {
      const connId = activeConnectionIds[connectionType]
      if (connId) {
        config.headers.set('X-Connection-ID', connId.toString())
      }
    }
    
    return config
  },
  (error) => Promise.reject(error)
)

// Response interceptor
api.interceptors.response.use(
  (response) => response,
  (error) => {
    const message = error.response?.data?.error || error.message || 'Unknown error'
    return Promise.reject(new Error(message))
  }
)

export default api

// ==================== Connection Management API ====================
export const connectionApi = {
  // Get all connections
  getAll: () => api.get<Connection[]>('/connections'),
  
  // Get single connection
  getById: (id: number) => api.get<Connection>(`/connections/${id}`),
  
  // Create connection
  create: (data: Connection) => api.post<Connection>('/connections', data),
  
  // Update connection
  update: (id: number, data: Connection) => api.put<Connection>(`/connections/${id}`, data),
  
  // Delete connection
  delete: (id: number) => api.delete(`/connections/${id}`),
  
  // Test connection
  test: (data: Connection) => api.post<TestConnectionResult>('/connections/test', data),
  
  // Get connections by type
  getByType: (type: string) => api.get<Connection[]>(`/connections/types/${type}`),
}

// ==================== MySQL API ====================
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

// ==================== Redis API ====================
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

// ==================== System API ====================
export const systemApi = {
  getConnections: () => api.get('/connections'),
  createConnection: (data: Connection) => api.post('/connections', data),
  getHistory: (type = '') => api.get('/history', { params: { type } }),
  getSavedQueries: () => api.get('/saved-queries'),
  saveSavedQuery: (data: SavedQuery) => api.post('/saved-queries', data),
  deleteSavedQuery: (id: number) => api.delete(`/saved-queries/${id}`),
}

// ==================== K8s Service Discovery API ====================
export const k8sApi = {
  // Discover middleware services in Kubernetes cluster
  // Note: This may take a while in large clusters, using 60s timeout
  // Can provide kubeconfig content and context for discovery
  discover: (kubeconfig?: string, context?: string, signal?: AbortSignal) => 
    api.post<DiscoveredService[]>('/k8s/discover', { kubeconfig, context }, { 
      timeout: 60000,
      signal 
    }),
  
  // List clusters from kubeconfig
  listClusters: (kubeconfig: string) =>
    api.post<{ clusters: string[] }>('/k8s/clusters', { kubeconfig }),
  
  // Batch import discovered services as connections
  importConnections: (services: DiscoveredService[], forceOverride?: boolean) => 
    api.post<ImportConnectionsResponse>('/k8s/import', { 
      services,
      force_override: forceOverride || false 
    }),
}

// ==================== Port Forward API ====================
export const portForwardApi = {
  // Create port forward
  create: (connectionId: number, namespace: string, serviceName: string, remotePort: number) =>
    api.post<ForwardInfo>('/port-forward', {
      connection_id: connectionId,
      namespace,
      service_name: serviceName,
      remote_port: remotePort,
    }),
  
  // List all forwards
  list: () => api.get<ForwardListResponse>('/port-forward'),
  
  // Get single forward status
  get: (id: string) => api.get<ForwardInfo>(`/port-forward/${id}`),
  
  // Get forward by connection ID
  getByConnection: (connectionId: number) => 
    api.get<ForwardInfo>(`/port-forward/by-connection`, { params: { connection_id: connectionId } }),
  
  // Stop forward
  stop: (id: string) => api.delete(`/port-forward/${id}`),
  
  // Reconnect forward
  reconnect: (id: string) => api.post<ForwardInfo>(`/port-forward/${id}/reconnect`),
  
  // Update last used time
  touch: (id: string) => api.put(`/port-forward/${id}/touch`),
}

// ==================== Types ====================
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
  type: 'mysql' | 'redis' | 'mongodb' | 'minio'
  host: string
  port: number
  username?: string
  password?: string
  database_name?: string
  is_default?: boolean
  forward_id?: string
  forward_local_port?: number
  forward_status?: 'active' | 'error' | 'idle'
  created_at?: string
  updated_at?: string
}

export interface TestConnectionResult {
  success: boolean
  error?: string
  message?: string
}

export interface SavedQuery {
  id?: number
  connection_id?: number
  name: string
  query_text: string
}

// ==================== K8s Service Discovery Types ====================
export interface DiscoveredService {
  name: string
  type: 'mysql' | 'redis' | 'mongodb' | 'minio' | 'postgresql'
  namespace: string
  host: string
  port: number
  username?: string
  password?: string
  database?: string
  has_credentials: boolean
}

export interface ImportConnectionsResponse {
  success: number
  failed: number
  updated: number
  skipped: number
  results: ImportConnectionResult[]
}

export interface ImportConnectionResult {
  name: string
  success: boolean
  updated?: boolean
  skipped?: boolean
  error?: string
  id?: number
}

// ==================== Port Forward Types ====================
export interface ForwardInfo {
  id: string
  connection_id: number
  local_host: string
  local_port: number
  remote_host: string
  remote_port: number
  status: 'active' | 'error' | 'idle'
  created_at: string
  last_used_at: string
  error_message?: string
}

export interface ForwardListResponse {
  forwards: ForwardInfo[]
  total: number
}

