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
  K8sDeployment,
  K8sPod,
  K8sConfigMapInfo,
  K8sSecretInfo,
  K8sServiceInfo,
  K8sIngressInfo,
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

  async testK8s(data: TestK8sConnectionRequest): Promise<TestConnectionResult> {
    try {
      return await invoke<TestConnectionResult>('test_k8s_connection', { data })
    } catch (error) {
      handleInvokeError(error)
    }
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
  async getHistory(params?: {
    type?: string
    database?: string
    status?: string
    keyword?: string
    limit?: number
    offset?: number
  }): Promise<QueryHistoryListResponse> {
    try {
      return await invoke<QueryHistoryListResponse>('get_history', {
        connType: params?.type,
        database: params?.database,
        status: params?.status,
        keyword: params?.keyword,
        limit: params?.limit,
        offset: params?.offset,
      })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async addHistory(data: AddQueryHistoryRequest): Promise<QueryHistory> {
    try {
      return await invoke<QueryHistory>('add_history', { data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async deleteHistory(id: number): Promise<void> {
    try {
      await invoke('delete_history', { id })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async cleanupHistory(days: number): Promise<{ deleted: number }> {
    try {
      const deleted = await invoke<number>('cleanup_history', { days })
      return { deleted }
    } catch (error) {
      handleInvokeError(error)
    }
  }
}

class IpcSavedQueryApi implements ISavedQueryApi {
  async getSavedQueries(category?: string): Promise<SavedQuery[]> {
    try {
      return await invoke<SavedQuery[]>('get_saved_queries', { category })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async createSavedQuery(data: CreateSavedQueryRequest): Promise<SavedQuery> {
    try {
      return await invoke<SavedQuery>('create_saved_query', { data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async updateSavedQuery(id: number, data: UpdateSavedQueryRequest): Promise<SavedQuery> {
    try {
      return await invoke<SavedQuery>('update_saved_query', { id, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async deleteSavedQuery(id: number): Promise<void> {
    try {
      await invoke('delete_saved_query', { id })
    } catch (error) {
      handleInvokeError(error)
    }
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

  async readLocalKubeconfig(): Promise<string> {
    try {
      return await invoke<string>('k8s_read_local_kubeconfig')
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

  // K8s resource listing methods
  async listNamespaces(clusterId: number): Promise<string[]> {
    try {
      return await invoke<string[]>('k8s_list_namespaces', { clusterId })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listDeployments(clusterId: number, namespace: string): Promise<K8sDeployment[]> {
    try {
      return await invoke<K8sDeployment[]>('k8s_list_deployments', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listPods(clusterId: number, namespace: string): Promise<K8sPod[]> {
    try {
      return await invoke<K8sPod[]>('k8s_list_pods', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listConfigMaps(clusterId: number, namespace: string): Promise<K8sConfigMapInfo[]> {
    try {
      return await invoke<K8sConfigMapInfo[]>('k8s_list_configmaps', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getConfigMapData(clusterId: number, namespace: string, name: string): Promise<Record<string, string>> {
    try {
      return await invoke<Record<string, string>>('k8s_get_configmap_data', { clusterId, namespace, name })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listSecrets(clusterId: number, namespace: string): Promise<K8sSecretInfo[]> {
    try {
      return await invoke<K8sSecretInfo[]>('k8s_list_secrets', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listServices(clusterId: number, namespace: string): Promise<K8sServiceInfo[]> {
    try {
      return await invoke<K8sServiceInfo[]>('k8s_list_services', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listIngresses(clusterId: number, namespace: string): Promise<K8sIngressInfo[]> {
    try {
      return await invoke<K8sIngressInfo[]>('k8s_list_ingresses', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }
}

// Rust PortForward struct mapping
interface RustPortForward {
  id: string | null
  connection_id: number
  namespace: string
  service_name: string
  remote_port: number
  local_port: number
  status: string
  error: string | null
  last_used: string | null
  created_at: string | null
}

// Transform Rust PortForward to frontend ForwardInfo
function transformPortForward(pf: RustPortForward): ForwardInfo {
  return {
    id: pf.id || '',
    connection_id: pf.connection_id,
    local_host: '127.0.0.1',
    local_port: pf.local_port,
    remote_host: `${pf.service_name}.${pf.namespace}.svc.cluster.local`,
    remote_port: pf.remote_port,
    status: (pf.status === 'active' ? 'active' : pf.status === 'error' ? 'error' : 'idle') as 'active' | 'error' | 'idle',
    created_at: pf.created_at || new Date().toISOString(),
    last_used_at: pf.last_used || new Date().toISOString(),
    error_message: pf.error || undefined,
  }
}

class IpcPortForwardApi implements IPortForwardApi {
  async create(
    connectionId: number,
    _namespace: string,
    _serviceName: string,
    _remotePort: number,
    localPort?: number
  ): Promise<ForwardInfo> {
    try {
      // In Tauri mode, we start port forward by connection ID
      // The namespace/service info comes from the connection record
      // localPort is optional - if not provided or 0, auto-assign an available port
      const result = await invoke<RustPortForward>('start_port_forward', {
        connectionId,
        localPort: localPort && localPort > 0 ? localPort : null
      })
      return transformPortForward(result)
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async list(): Promise<ForwardListResponse> {
    try {
      const forwards = await invoke<RustPortForward[]>('list_port_forwards')
      return {
        forwards: forwards.map(transformPortForward),
        total: forwards.length,
      }
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async get(id: string): Promise<ForwardInfo> {
    try {
      const result = await invoke<RustPortForward>('get_port_forward', { id })
      return transformPortForward(result)
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getByConnection(connectionId: number): Promise<ForwardInfo> {
    try {
      const result = await invoke<RustPortForward>('get_port_forward_by_connection', { connectionId })
      return transformPortForward(result)
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async stop(id: string): Promise<void> {
    try {
      await invoke('stop_port_forward', { id })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async reconnect(id: string, localPort?: number): Promise<ForwardInfo> {
    try {
      // localPort is optional - if not provided, reuse the existing port
      const result = await invoke<RustPortForward>('reconnect_port_forward', {
        id,
        localPort: localPort && localPort > 0 ? localPort : null
      })
      return transformPortForward(result)
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async touch(id: string): Promise<void> {
    try {
      await invoke('touch_port_forward', { id })
    } catch (error) {
      handleInvokeError(error)
    }
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
