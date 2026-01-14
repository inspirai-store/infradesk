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
  ISettingsApi,
  ILLMConfigApi,
  ApiError,
  K8sDeployment,
  K8sPod,
  K8sConfigMapInfo,
  K8sSecretInfo,
  K8sServiceInfo,
  K8sIngressInfo,
  K8sJob,
  K8sCronJob,
  K8sStatefulSet,
  K8sDaemonSet,
  K8sReplicaSet,
  UserSetting,
  BatchSettingsResponse,
  LLMConfigResponse,
  CreateLLMConfigRequest,
  UpdateLLMConfigRequest,
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
  AlterUserPasswordRequest,
  DropUserRequest,
  RevokePrivilegesRequest,
  UserGrantsResponse,
  CreateTableRequest,
  AlterTableRequest,
  IndexInfo,
  CreateIndexRequest,
  ForeignKeyInfo,
  CreateForeignKeyRequest,
  ExportTableRequest,
  ExportTableResponse,
  ImportDataRequest,
  ImportResult,
  // View management types
  ViewInfo,
  ViewDefinition,
  CreateViewRequest,
  // Stored procedure types
  ProcedureInfo,
  ProcedureDefinition,
  // Trigger types
  TriggerInfo,
  TriggerDefinition,
  // Server monitoring types
  ServerVariable,
  ProcessInfo,
  ExplainResult,
  TableMaintenanceResult,
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

  async createTable(database: string, data: CreateTableRequest): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_create_table', { connectionId, database, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async dropTable(database: string, table: string): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_drop_table', { connectionId, database, table })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async renameTable(database: string, table: string, newName: string): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_rename_table', { connectionId, database, table, data: { new_name: newName } })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async truncateTable(database: string, table: string): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_truncate_table', { connectionId, database, table })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async copyTable(database: string, table: string, targetName: string, withData = false): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_copy_table', { connectionId, database, table, data: { target_name: targetName, with_data: withData } })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  // Index management
  async listIndexes(database: string, table: string): Promise<IndexInfo[]> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_list_indexes', { connectionId, database, table })
    } catch (error) {
      handleInvokeError(error)
    }
    return []
  }

  async createIndex(database: string, table: string, data: CreateIndexRequest): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_create_index', { connectionId, database, table, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async dropIndex(database: string, table: string, indexName: string): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_drop_index', { connectionId, database, table, indexName })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  // Foreign key management
  async listForeignKeys(database: string, table: string): Promise<ForeignKeyInfo[]> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke('mysql_list_foreign_keys', { connectionId, database, table })
    } catch (error) {
      handleInvokeError(error)
    }
    return []
  }

  async createForeignKey(database: string, table: string, data: CreateForeignKeyRequest): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_create_foreign_key', { connectionId, database, table, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async dropForeignKey(database: string, table: string, fkName: string): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_drop_foreign_key', { connectionId, database, table, fkName })
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

  async alterTable(database: string, table: string, data: AlterTableRequest): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_alter_table', { connectionId, database, table, data })
    } catch (error) {
      handleInvokeError(error)
    }
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

  async exportTable(database: string, table: string, request: ExportTableRequest): Promise<ExportTableResponse> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<ExportTableResponse>('mysql_export_table', {
        connectionId,
        database,
        table,
        data: request,
      })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async importData(database: string, table: string, request: ImportDataRequest): Promise<ImportResult> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<ImportResult>('mysql_import_data', {
        connectionId,
        database,
        table,
        data: request,
      })
    } catch (error) {
      handleInvokeError(error)
    }
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

  async grantPrivileges(database: string, data: GrantPrivilegesRequest): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_grant_privileges', { connectionId, database, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async alterUserPassword(data: AlterUserPasswordRequest): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_alter_user_password', { connectionId, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async dropUser(data: DropUserRequest): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_drop_user', { connectionId, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async showGrants(username: string, host: string): Promise<UserGrantsResponse> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<UserGrantsResponse>('mysql_show_grants', {
        connectionId,
        username,
        host,
      })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async revokePrivileges(data: RevokePrivilegesRequest): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_revoke_privileges', { connectionId, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  // View operations
  async listViews(database: string): Promise<ViewInfo[]> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<ViewInfo[]>('mysql_list_views', { connectionId, database })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getViewDefinition(database: string, view: string): Promise<ViewDefinition> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<ViewDefinition>('mysql_get_view_definition', { connectionId, database, view })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async createView(database: string, data: CreateViewRequest): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_create_view', { connectionId, database, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async dropView(database: string, view: string): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_drop_view', { connectionId, database, view })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  // Stored procedure operations
  async listProcedures(database: string): Promise<ProcedureInfo[]> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<ProcedureInfo[]>('mysql_list_procedures', { connectionId, database })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getProcedureDefinition(database: string, name: string, routineType = 'PROCEDURE'): Promise<ProcedureDefinition> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<ProcedureDefinition>('mysql_get_procedure_definition', {
        connectionId,
        database,
        name,
        routineType,
      })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async dropProcedure(database: string, name: string): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_drop_procedure', { connectionId, database, name })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async dropFunction(database: string, name: string): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_drop_function', { connectionId, database, name })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  // Trigger operations
  async listTriggers(database: string): Promise<TriggerInfo[]> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<TriggerInfo[]>('mysql_list_triggers', { connectionId, database })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getTriggerDefinition(database: string, name: string): Promise<TriggerDefinition> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<TriggerDefinition>('mysql_get_trigger_definition', { connectionId, database, name })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async dropTrigger(database: string, name: string): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_drop_trigger', { connectionId, database, name })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  // Server monitoring operations
  async getServerVariables(filter?: string): Promise<ServerVariable[]> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<ServerVariable[]>('mysql_get_server_variables', { connectionId, filter })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getProcessList(): Promise<ProcessInfo[]> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<ProcessInfo[]>('mysql_get_process_list', { connectionId })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async killProcess(processId: number): Promise<void> {
    try {
      const connectionId = this.getConnectionId()
      await invoke('mysql_kill_process', { connectionId, processId })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  // Query analysis operations
  async explainQuery(database: string, query: string): Promise<ExplainResult> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<ExplainResult>('mysql_explain_query', { connectionId, database, query })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  // Table maintenance operations
  async optimizeTable(database: string, table: string): Promise<TableMaintenanceResult> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<TableMaintenanceResult>('mysql_optimize_table', { connectionId, database, table })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async analyzeTable(database: string, table: string): Promise<TableMaintenanceResult> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<TableMaintenanceResult>('mysql_analyze_table', { connectionId, database, table })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async checkTable(database: string, table: string): Promise<TableMaintenanceResult> {
    try {
      const connectionId = this.getConnectionId()
      return await invoke<TableMaintenanceResult>('mysql_check_table', { connectionId, database, table })
    } catch (error) {
      handleInvokeError(error)
    }
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

  async getPodDetail(clusterId: number, namespace: string, name: string): Promise<import('../../types').K8sPodDetail> {
    try {
      return await invoke<import('../../types').K8sPodDetail>('k8s_get_pod_detail', { clusterId, namespace, name })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getPodLogs(clusterId: number, namespace: string, name: string, container?: string, tailLines?: number): Promise<string> {
    try {
      return await invoke<string>('k8s_get_pod_logs', { clusterId, namespace, name, container, tailLines })
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

  async updateConfigMap(clusterId: number, namespace: string, name: string, data: Record<string, string>): Promise<void> {
    try {
      await invoke('k8s_update_configmap', { clusterId, namespace, name, data })
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

  async getSecretData(clusterId: number, namespace: string, name: string): Promise<Record<string, string>> {
    try {
      return await invoke<Record<string, string>>('k8s_get_secret_data', { clusterId, namespace, name })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async updateSecret(clusterId: number, namespace: string, name: string, data: Record<string, string>): Promise<void> {
    try {
      await invoke('k8s_update_secret', { clusterId, namespace, name, data })
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

  // Extended workload types
  async listJobs(clusterId: number, namespace: string): Promise<K8sJob[]> {
    try {
      return await invoke<K8sJob[]>('k8s_list_jobs', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listCronJobs(clusterId: number, namespace: string): Promise<K8sCronJob[]> {
    try {
      return await invoke<K8sCronJob[]>('k8s_list_cronjobs', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listStatefulSets(clusterId: number, namespace: string): Promise<K8sStatefulSet[]> {
    try {
      return await invoke<K8sStatefulSet[]>('k8s_list_statefulsets', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listDaemonSets(clusterId: number, namespace: string): Promise<K8sDaemonSet[]> {
    try {
      return await invoke<K8sDaemonSet[]>('k8s_list_daemonsets', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async listReplicaSets(clusterId: number, namespace: string): Promise<K8sReplicaSet[]> {
    try {
      return await invoke<K8sReplicaSet[]>('k8s_list_replicasets', { clusterId, namespace })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  // Deployment operations
  async getDeploymentYaml(clusterId: number, namespace: string, name: string): Promise<string> {
    try {
      return await invoke<string>('k8s_get_deployment_yaml', { clusterId, namespace, name })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async updateDeploymentYaml(clusterId: number, namespace: string, name: string, yaml: string): Promise<void> {
    try {
      await invoke('k8s_update_deployment_yaml', { clusterId, namespace, name, yaml })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async scaleDeployment(clusterId: number, namespace: string, name: string, replicas: number): Promise<void> {
    try {
      await invoke('k8s_scale_deployment', { clusterId, namespace, name, replicas })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async restartDeployment(clusterId: number, namespace: string, name: string): Promise<void> {
    try {
      await invoke('k8s_restart_deployment', { clusterId, namespace, name })
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

// ==================== IPC Settings API ====================

class IpcSettingsApi implements ISettingsApi {
  async getAll(): Promise<UserSetting[]> {
    try {
      return await invoke<UserSetting[]>('get_all_settings')
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async get(key: string): Promise<unknown | null> {
    try {
      return await invoke<unknown | null>('get_setting', { key })
    } catch {
      return null
    }
  }

  async getBatch(keys: string[]): Promise<BatchSettingsResponse> {
    try {
      return await invoke<BatchSettingsResponse>('get_settings_batch', { keys })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async set(key: string, value: unknown): Promise<UserSetting> {
    try {
      return await invoke<UserSetting>('set_setting', { key, value })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async delete(key: string): Promise<void> {
    try {
      await invoke('delete_setting', { key })
    } catch (error) {
      handleInvokeError(error)
    }
  }
}

// ==================== IPC LLM Config API ====================

class IpcLLMConfigApi implements ILLMConfigApi {
  async getAll(): Promise<LLMConfigResponse[]> {
    try {
      return await invoke<LLMConfigResponse[]>('get_all_llm_configs')
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async get(id: number): Promise<LLMConfigResponse> {
    try {
      return await invoke<LLMConfigResponse>('get_llm_config', { id })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getDefault(): Promise<LLMConfigResponse | null> {
    try {
      return await invoke<LLMConfigResponse | null>('get_default_llm_config')
    } catch {
      return null
    }
  }

  async create(data: CreateLLMConfigRequest): Promise<LLMConfigResponse> {
    try {
      return await invoke<LLMConfigResponse>('create_llm_config', { data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async update(id: number, data: UpdateLLMConfigRequest): Promise<LLMConfigResponse> {
    try {
      return await invoke<LLMConfigResponse>('update_llm_config', { id, data })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async delete(id: number): Promise<void> {
    try {
      await invoke('delete_llm_config', { id })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async setDefault(id: number): Promise<LLMConfigResponse> {
    try {
      return await invoke<LLMConfigResponse>('set_default_llm_config', { id })
    } catch (error) {
      handleInvokeError(error)
    }
  }

  async getApiKey(id: number): Promise<string | null> {
    try {
      return await invoke<string | null>('get_llm_api_key', { id })
    } catch {
      return null
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
    settings: new IpcSettingsApi(),
    llmConfig: new IpcLLMConfigApi(),
  }
}
