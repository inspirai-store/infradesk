/**
 * IPC Adapter Implementation
 *
 * This module provides Tauri IPC-based API implementations.
 * Used in Tauri desktop environment to communicate with Rust backend.
 */

import { invoke } from '@tauri-apps/api/core'
import type {
  IApiAdapter,
  IConnectionApi,
  IClusterApi,
  IMysqlApi,
  IRedisApi,
  IHistoryApi,
  ISavedQueryApi,
  IK8sApi,
  IPortForwardApi,
  ApiError,
} from '../../types'
import type {
  Connection,
  TestConnectionResult,
  TestK8sConnectionRequest,
  Cluster,
  CreateDatabaseRequest,
  AlterDatabaseRequest,
  GrantPrivilegesRequest,
  CreateUserRequest,
  UserInfo,
  CreateTableRequest,
  AlterTableRequest,
  UpdateRowRequest,
  SetKeyRequest,
  ExportData,
  QueryHistoryListResponse,
  AddQueryHistoryRequest,
  QueryHistory,
  SavedQuery,
  CreateSavedQueryRequest,
  UpdateSavedQueryRequest,
  DiscoveredService,
  ImportConnectionsResponse,
  ForwardInfo,
  ForwardListResponse,
} from '../../index'
import { getActiveConnectionId } from '../http'

// ==================== Error Handling ====================

/**
 * Convert Tauri invoke errors to ApiError
 */
function handleInvokeError(error: unknown): never {
  if (error instanceof Error) {
    throw new Error(error.message) as ApiError
  }
  if (typeof error === 'string') {
    throw new Error(error) as ApiError
  }
  throw new Error('Unknown IPC error') as ApiError
}

// ==================== IPC Connection API ====================

class IpcConnectionApi implements IConnectionApi {
  async getAll(): Promise<Connection[]> {
    try {
      return await invoke<Connection[]>('get_all_connections')
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getById(id: number): Promise<Connection> {
    try {
      return await invoke<Connection>('get_connection', { id })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async create(data: Connection): Promise<Connection> {
    try {
      return await invoke<Connection>('create_connection', { data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async update(id: number, data: Connection): Promise<Connection> {
    try {
      return await invoke<Connection>('update_connection', { id, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async delete(id: number): Promise<void> {
    try {
      await invoke('delete_connection', { id })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async test(data: Connection): Promise<TestConnectionResult> {
    try {
      return await invoke<TestConnectionResult>('test_connection', { data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async testK8s(_data: TestK8sConnectionRequest): Promise<TestConnectionResult> {
    // K8s testing not supported in Tauri mode
    throw new Error('K8s connection testing is not supported in desktop mode')
  }

  async getByType(type: string): Promise<Connection[]> {
    try {
      return await invoke<Connection[]>('get_connections_by_type', { connType: type })
    } catch (error) {
      handleInvokeError(error)
    }
  }
}

// ==================== Placeholder APIs (Not Implemented Yet) ====================

class IpcClusterApi implements IClusterApi {
  async getAll(): Promise<Cluster[]> {
    try {
      return await invoke<Cluster[]>('get_all_clusters')
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async create(data: Partial<Cluster>): Promise<Cluster> {
    try {
      return await invoke<Cluster>('create_cluster', { data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async update(id: number, data: Partial<Cluster>): Promise<Cluster> {
    try {
      return await invoke<Cluster>('update_cluster', { id, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async delete(id: number): Promise<void> {
    try {
      await invoke('delete_cluster', { id })
    } catch (error) {
      handleInvokeError(error)
    }
  }
}

class IpcMysqlApi implements IMysqlApi {
  private getConnectionId(): number {
    const connectionId = getActiveConnectionId('mysql')
    if (!connectionId) {
      throw new Error('No active MySQL connection')
    }
    return connectionId
  }

  async getInfo(): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_get_info', { connectionId })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listDatabases(): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_list_databases', { connectionId })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async createDatabase(data: CreateDatabaseRequest): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_create_database', { connectionId, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async alterDatabase(name: string, data: AlterDatabaseRequest): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_alter_database', { connectionId, name, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async grantPrivileges(name: string, data: GrantPrivilegesRequest): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_grant_privileges', { connectionId, database: name, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async dropDatabase(name: string): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_drop_database', { connectionId, name })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listTables(database: string): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_list_tables', { connectionId, database })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async createTable(_database: string, _data: CreateTableRequest): Promise<unknown> {
    // Table creation via schema not implemented in Rust backend yet
    throw new Error('Table creation not implemented in IPC adapter')
  }

  async dropTable(database: string, table: string): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_drop_table', { connectionId, database, table })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getTableSchema(database: string, table: string): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_get_table_schema', { connectionId, database, table })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getDatabaseSchema(_database: string): Promise<unknown> {
    // Database schema not implemented in Rust backend yet
    throw new Error('Database schema not implemented in IPC adapter')
  }

  async alterTable(_database: string, _table: string, _data: AlterTableRequest): Promise<unknown> {
    // Alter table not implemented in Rust backend yet
    throw new Error('Alter table not implemented in IPC adapter')
  }

  async getTablePrimaryKey(database: string, table: string): Promise<{ primary_key: string }> {
    try {
      const connectionId = this.getConnectionId()
      const primaryKey = await invoke<string>('mysql_get_table_primary_key', { connectionId, database, table })
      return { primary_key: primaryKey }
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getRows(database: string, table: string, page?: number, size?: number): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_get_rows', { connectionId, database, table, page, pageSize: size })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async insertRow(database: string, table: string, data: Record<string, unknown>): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_insert_row', { connectionId, database, table, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async updateRow(_database: string, _table: string, _data: UpdateRowRequest): Promise<unknown> {
    // Use updateRecord instead
    throw new Error('Use updateRecord method instead')
  }

  async updateRecord(
    database: string,
    table: string,
    primaryKey: string,
    primaryValue: unknown,
    updates: Record<string, unknown>
  ): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_update_record', { connectionId, database, table, primaryKey, primaryValue, updates })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async deleteRow(database: string, table: string, where: Record<string, unknown>): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_delete_row', { connectionId, database, table, whereClause: where })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async executeQuery(database: string, query: string): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_execute_query', { connectionId, database, query })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async exportData(_database: string, _table: string, _format?: string): Promise<unknown> {
    // Export not implemented in Rust backend yet
    throw new Error('Export data not implemented in IPC adapter')
  }

  async importData(_database: string, _table: string, _rows: Record<string, unknown>[]): Promise<unknown> {
    // Import not implemented in Rust backend yet
    throw new Error('Import data not implemented in IPC adapter')
  }

  async listUsers(): Promise<UserInfo[]> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<UserInfo[]>('mysql_list_users', { connectionId })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async createUser(data: CreateUserRequest): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_create_user', { connectionId, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listUserGrants(_username: string, _host?: string): Promise<unknown> {
    // List grants not implemented in Rust backend yet
    throw new Error('List user grants not implemented in IPC adapter')
  }
}

class IpcRedisApi implements IRedisApi {
  private getConnectionId(): number {
    const connectionId = getActiveConnectionId('redis')
    if (!connectionId) {
      throw new Error('No active Redis connection')
    }
    return connectionId
  }

  async getInfo(): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('redis_get_info', { connectionId })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listKeys(pattern?: string, cursor?: number, count?: number): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('redis_list_keys', { connectionId, pattern, cursor, count })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getKey(key: string): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('redis_get_key', { connectionId, key })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async setKey(data: SetKeyRequest): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('redis_set_key', { connectionId, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async updateKey(key: string, data: SetKeyRequest): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('redis_update_key', { connectionId, key, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async deleteKey(key: string): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('redis_delete_key', { connectionId, key })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async setTTL(key: string, ttl: number): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('redis_set_ttl', { connectionId, key, ttl })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async exportKeys(keys: string[]): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('redis_export_keys', { connectionId, keys })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async importKeys(data: ExportData): Promise<unknown> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('redis_import_keys', { connectionId, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }
}

class IpcHistoryApi implements IHistoryApi {
  private notImplemented(): never {
    throw new Error('History API not implemented in IPC adapter')
  }

  async getHistory(_params?: {
    type?: string
    database?: string
    status?: string
    keyword?: string
    limit?: number
    offset?: number
  }): Promise<QueryHistoryListResponse> {
    this.notImplemented()
  }

  async addHistory(_data: AddQueryHistoryRequest): Promise<QueryHistory> {
    this.notImplemented()
  }

  async deleteHistory(_id: number): Promise<void> {
    this.notImplemented()
  }

  async cleanupHistory(_days: number): Promise<{ deleted: number }> {
    this.notImplemented()
  }
}

class IpcSavedQueryApi implements ISavedQueryApi {
  private notImplemented(): never {
    throw new Error('SavedQuery API not implemented in IPC adapter')
  }

  async getSavedQueries(_category?: string): Promise<SavedQuery[]> {
    this.notImplemented()
  }

  async createSavedQuery(_data: CreateSavedQueryRequest): Promise<SavedQuery> {
    this.notImplemented()
  }

  async updateSavedQuery(_id: number, _data: UpdateSavedQueryRequest): Promise<SavedQuery> {
    this.notImplemented()
  }

  async deleteSavedQuery(_id: number): Promise<void> {
    this.notImplemented()
  }
}

class IpcK8sApi implements IK8sApi {
  async discover(kubeconfig?: string, context?: string, _signal?: AbortSignal): Promise<DiscoveredService[]> {
    try {
      return await invoke<DiscoveredService[]>('k8s_discover', { kubeconfig, context })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listClusters(kubeconfig: string): Promise<{ clusters: string[] }> {
    try {
      return await invoke<{ clusters: string[] }>('k8s_list_clusters', { kubeconfig })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async importConnections(
    services: DiscoveredService[],
    forceOverride?: boolean,
    kubeconfig?: string,
    context?: string,
    clusterName?: string
  ): Promise<ImportConnectionsResponse> {
    try {
      // Transform DiscoveredService to ImportServiceItem format
      const importServices = services.map(svc => ({
        name: svc.name,
        type: svc.type,
        namespace: svc.namespace,
        host: svc.host,
        port: svc.port,
        username: svc.username,
        password: svc.password,
        database: svc.database,
        service_name: svc.service_name,
      }))

      return await invoke<ImportConnectionsResponse>('k8s_import_connections', {
        request: {
          services: importServices,
          force_override: forceOverride || false,
          kubeconfig,
          context,
          cluster_name: clusterName,
        }
      })
    } catch (error) {
      handleInvokeError(error)
    }
  }
}

class IpcPortForwardApi implements IPortForwardApi {
  async create(
    _connectionId: number,
    _namespace: string,
    _serviceName: string,
    _remotePort: number
  ): Promise<ForwardInfo> {
    throw new Error('Port forwarding not supported in desktop mode')
  }

  async list(): Promise<ForwardListResponse> {
    throw new Error('Port forwarding not supported in desktop mode')
  }

  async get(_id: string): Promise<ForwardInfo> {
    throw new Error('Port forwarding not supported in desktop mode')
  }

  async getByConnection(_connectionId: number): Promise<ForwardInfo> {
    throw new Error('Port forwarding not supported in desktop mode')
  }

  async stop(_id: string): Promise<void> {
    throw new Error('Port forwarding not supported in desktop mode')
  }

  async reconnect(_id: string): Promise<ForwardInfo> {
    throw new Error('Port forwarding not supported in desktop mode')
  }

  async touch(_id: string): Promise<void> {
    throw new Error('Port forwarding not supported in desktop mode')
  }
}

// ==================== IPC Adapter Factory ====================

/**
 * Create IPC adapter instance
 */
export function createIpcAdapter(): IApiAdapter {
  return {
    connection: new IpcConnectionApi(),
    cluster: new IpcClusterApi(),
    mysql: new IpcMysqlApi(),
    redis: new IpcRedisApi(),
    history: new IpcHistoryApi(),
    savedQuery: new IpcSavedQueryApi(),
    k8s: new IpcK8sApi(),
    portForward: new IpcPortForwardApi(),
  }
}
