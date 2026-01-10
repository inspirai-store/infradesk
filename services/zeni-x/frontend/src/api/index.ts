/**
 * Unified API Module
 *
 * This module exports API interfaces that work seamlessly in both
 * Web (HTTP) and Tauri (IPC) environments using the adapter pattern.
 */

import { getApiAdapter } from './adapter'
import { setActiveConnectionId as httpSetActiveConnectionId, getActiveConnectionId as httpGetActiveConnectionId } from './adapters/http'

// ==================== Active Connection Management ====================

/**
 * Set active connection ID for a type
 */
export function setActiveConnectionId(type: string, id: number | null): void {
  httpSetActiveConnectionId(type, id)
}

/**
 * Get active connection ID for a type
 */
export function getActiveConnectionId(type: string): number | null {
  return httpGetActiveConnectionId(type)
}

// ==================== API Exports ====================

// Get adapter instance
const adapter = getApiAdapter()

// Export individual API modules for backward compatibility
export const connectionApi = adapter.connection
export const clusterApi = adapter.cluster
export const mysqlApi = adapter.mysql
export const redisApi = adapter.redis
export const historyApi = adapter.history
export const savedQueryApi = adapter.savedQuery
export const k8sApi = adapter.k8s
export const portForwardApi = adapter.portForward

// Legacy system API (maps to connection API)
export const systemApi = {
  getConnections: () => adapter.connection.getAll(),
  createConnection: (data: Connection) => adapter.connection.create(data),
}

// Export default axios instance for any custom usage (HTTP mode only)
export { default } from './adapters/http'

// ==================== Types ====================
export interface CreateDatabaseRequest {
  name: string
  if_not_exists?: boolean
  charset?: string
  collate?: string
}

export interface AlterDatabaseRequest {
  charset?: string
  collate?: string
}

export interface GrantPrivilegesRequest {
  username: string
  user_host?: string
  password?: string
  privileges: string[]
  grant_option?: boolean
}

export interface CreateUserRequest {
  username: string
  user_host?: string
  password: string
}

export interface UserInfo {
  host: string
  user: string
}

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
  forward_status?: 'active' | 'error' | 'idle' | 'pending'
  k8s_namespace?: string
  k8s_service_name?: string
  k8s_service_port?: number
  cluster_id?: number | null
  source?: 'local' | 'k8s'
  created_at?: string
  updated_at?: string
}

export interface Cluster {
  id: number
  name: string
  description?: string
  environment?: string
  context?: string
  api_server?: string
  kubeconfig?: string // Usually empty when receiving from API
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface TestK8sConnectionRequest {
  type: string
  username?: string
  password?: string
  database_name?: string
  kubeconfig?: string
  context?: string
  k8s_namespace: string
  k8s_service_name: string
  k8s_service_port: number
}

export interface TestConnectionResult {
  success: boolean
  error?: string
  message?: string
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
  service_name: string
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

// ==================== Query History Types ====================
export interface QueryHistory {
  id: number
  connection_id: number
  database: string
  query_type: string
  query_text: string
  executed_at: string
  duration_ms: number
  row_count: number
  status: string
  error_message?: string
}

export interface QueryHistoryListResponse {
  history: QueryHistory[]
  total: number
}

export interface AddQueryHistoryRequest {
  connection_id: number
  database: string
  query_type: string
  query_text: string
  duration_ms: number
  row_count: number
  status: string
  error_message?: string
}

// ==================== Saved Query Types ====================
export interface SavedQuery {
  id?: number
  connection_id: number
  database: string
  name: string
  query_text: string
  description?: string
  category?: string
  created_at?: string
  updated_at?: string
}

export interface CreateSavedQueryRequest {
  connection_id: number
  database: string
  name: string
  query_text: string
  description?: string
  category?: string
}

export interface UpdateSavedQueryRequest {
  name?: string
  query_text?: string
  description?: string
  category?: string
}
