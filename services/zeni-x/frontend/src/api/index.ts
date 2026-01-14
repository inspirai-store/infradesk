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
  host: string
  privileges: string[]
}

export interface CreateUserRequest {
  username: string
  host?: string
  password: string
}

export interface UserInfo {
  host: string
  user: string
}

// ==================== User Permission Management Types ====================

/**
 * Request to alter user password
 */
export interface AlterUserPasswordRequest {
  username: string
  host: string
  new_password: string
}

/**
 * Request to drop a MySQL user
 */
export interface DropUserRequest {
  username: string
  host: string
}

/**
 * Request to revoke privileges from a user
 */
export interface RevokePrivilegesRequest {
  username: string
  host: string
  privileges: string[]
  database: string  // "*" for all databases (*.*), or specific database name
}

/**
 * User grant information (a single GRANT statement)
 */
export interface UserGrantInfo {
  grant_statement: string
}

/**
 * Response containing user grants
 */
export interface UserGrantsResponse {
  username: string
  host: string
  grants: UserGrantInfo[]
}

/**
 * Column definition for table creation/modification
 */
export interface ColumnDefinition {
  name: string
  data_type: string // e.g., "INT", "VARCHAR(255)", "TEXT"
  nullable?: boolean
  default?: string
  auto_increment?: boolean
  comment?: string
}

/**
 * Index definition for table creation/modification
 */
export interface IndexDefinition {
  name: string
  columns: string[]
  unique?: boolean
  index_type?: string // "BTREE", "HASH"
}

/**
 * Request to create a new table
 */
export interface CreateTableRequest {
  name: string
  columns: ColumnDefinition[]
  primary_key?: string[]
  indexes?: IndexDefinition[]
  engine?: string // "InnoDB", "MyISAM"
  charset?: string
  collation?: string
  comment?: string
}

/**
 * Request to alter an existing table
 */
export interface AlterTableRequest {
  add_columns?: ColumnDefinition[]
  drop_columns?: string[]
  modify_columns?: ColumnDefinition[]
  rename_column?: RenameColumnRequest
  add_indexes?: IndexDefinition[]
  drop_indexes?: string[]
}

/**
 * Request to rename a column
 */
export interface RenameColumnRequest {
  old_name: string
  new_name: string
}

/**
 * Request to rename a table
 */
export interface RenameTableRequest {
  new_name: string
}

/**
 * Request to copy a table
 */
export interface CopyTableRequest {
  target_name: string
  with_data?: boolean
}

// ==================== Index Management Types ====================

/**
 * Index information returned from the database
 */
export interface IndexInfo {
  name: string
  columns: string[]
  unique: boolean
  index_type: string
  is_primary: boolean
  comment?: string
}

/**
 * Request to create an index
 */
export interface CreateIndexRequest {
  name: string
  columns: string[]
  unique?: boolean
  index_type?: string // "BTREE", "HASH"
  comment?: string
}

// ==================== Foreign Key Management Types ====================

/**
 * Foreign key information returned from the database
 */
export interface ForeignKeyInfo {
  name: string
  columns: string[]
  ref_table: string
  ref_columns: string[]
  on_delete: string // "RESTRICT", "CASCADE", "SET NULL", "NO ACTION"
  on_update: string
}

/**
 * Request to create a foreign key
 */
export interface CreateForeignKeyRequest {
  name?: string
  columns: string[]
  ref_table: string
  ref_columns: string[]
  on_delete?: string
  on_update?: string
}

// Legacy ColumnDef for backward compatibility (deprecated, use ColumnDefinition)
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
  cluster_id?: number  // Used to look up kubeconfig from saved cluster
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

// ==================== Data Export/Import Types ====================

/**
 * Export format for table data
 */
export type ExportFormat = 'csv' | 'json' | 'sql'

/**
 * Request to export table data
 */
export interface ExportTableRequest {
  format?: ExportFormat
  columns?: string[]
  where_clause?: string
  limit?: number
  include_headers?: boolean
}

/**
 * Response from export operation
 */
export interface ExportTableResponse {
  data: string
  format: string
  row_count: number
}

/**
 * Request to import data into a table
 */
export interface ImportDataRequest {
  data: string
  format: string
  column_mapping?: Record<string, string>
  skip_rows?: number
  on_duplicate?: 'ignore' | 'update' | 'error'
}

/**
 * Result of import operation
 */
export interface ImportResult {
  imported: number
  skipped: number
  failed: number
  errors: string[]
}

// ==================== View Management Types ====================

/**
 * View information returned from the database
 */
export interface ViewInfo {
  name: string
  definer?: string
  security_type?: string
  check_option?: string
  is_updatable: boolean
}

/**
 * View definition with SQL
 */
export interface ViewDefinition {
  name: string
  definition: string
}

/**
 * Request to create a view
 */
export interface CreateViewRequest {
  name: string
  definition: string
  or_replace?: boolean
  algorithm?: 'UNDEFINED' | 'MERGE' | 'TEMPTABLE'
  security?: 'DEFINER' | 'INVOKER'
  check_option?: 'CASCADED' | 'LOCAL'
}

// ==================== Stored Procedure Types ====================

/**
 * Procedure/Function information
 */
export interface ProcedureInfo {
  name: string
  routine_type: string  // "PROCEDURE" or "FUNCTION"
  definer?: string
  created?: string
  modified?: string
  security_type?: string
  comment?: string
}

/**
 * Procedure/Function definition with SQL
 */
export interface ProcedureDefinition {
  name: string
  routine_type: string
  definition: string
}

// ==================== Trigger Management Types ====================

/**
 * Trigger information
 */
export interface TriggerInfo {
  name: string
  event: string  // "INSERT", "UPDATE", "DELETE"
  timing: string  // "BEFORE", "AFTER"
  table_name: string
  definer?: string
  created?: string
}

/**
 * Trigger definition with SQL
 */
export interface TriggerDefinition {
  name: string
  definition: string
}

// ==================== Server Monitoring Types ====================

/**
 * Server variable
 */
export interface ServerVariable {
  name: string
  value: string
}

/**
 * Process information
 */
export interface ProcessInfo {
  id: number
  user: string
  host: string
  db?: string
  command: string
  time: number
  state?: string
  info?: string
}

/**
 * Query explain result
 */
export interface ExplainResult {
  query: string
  rows: Record<string, unknown>[]
}

/**
 * Table maintenance result
 */
export interface TableMaintenanceResult {
  table_name: string
  operation: string
  msg_type: string
  msg_text: string
}
