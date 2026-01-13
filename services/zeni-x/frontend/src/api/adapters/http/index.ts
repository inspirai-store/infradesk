/**
 * HTTP Adapter Implementation
 *
 * This module provides HTTP-based API implementations using axios.
 * Used in Web browser environment or as fallback in Tauri when IPC is not available.
 */

import axios from 'axios'
import type { AxiosInstance, InternalAxiosRequestConfig } from 'axios'
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
  K8sDeployment,
  K8sPod,
  K8sConfigMapInfo,
  K8sSecretInfo,
  K8sServiceInfo,
  K8sIngressInfo,
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

// ==================== HTTP Client Setup ====================

// Active connection IDs per type (managed by connection store)
let activeConnectionIds: Record<string, number | null> = {}

/**
 * Set active connection ID for a type
 */
export function setActiveConnectionId(type: string, id: number | null): void {
  activeConnectionIds[type] = id
}

/**
 * Get active connection ID for a type
 */
export function getActiveConnectionId(type: string): number | null {
  return activeConnectionIds[type] ?? null
}

/**
 * Create axios instance with interceptors
 */
function createHttpClient(): AxiosInstance {
  const client = axios.create({
    baseURL: '/api',
    timeout: 30000,
    headers: {
      'Content-Type': 'application/json',
    },
  })

  // Request interceptor to inject X-Connection-ID header
  client.interceptors.request.use(
    (config: InternalAxiosRequestConfig) => {
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
  client.interceptors.response.use(
    (response) => response,
    (error) => {
      const message = error.response?.data?.error || error.message || 'Unknown error'
      return Promise.reject(new Error(message))
    }
  )

  return client
}

const api = createHttpClient()

// ==================== HTTP Connection API ====================

class HttpConnectionApi implements IConnectionApi {
  async getAll(): Promise<Connection[]> {
    const response = await api.get<Connection[]>('/connections')
    return response.data
  }

  async getById(id: number): Promise<Connection> {
    const response = await api.get<Connection>(`/connections/${id}`)
    return response.data
  }

  async create(data: Connection): Promise<Connection> {
    const response = await api.post<Connection>('/connections', data)
    return response.data
  }

  async update(id: number, data: Connection): Promise<Connection> {
    const response = await api.put<Connection>(`/connections/${id}`, data)
    return response.data
  }

  async delete(id: number): Promise<void> {
    await api.delete(`/connections/${id}`)
  }

  async test(data: Connection): Promise<TestConnectionResult> {
    const response = await api.post<TestConnectionResult>('/connections/test', data)
    return response.data
  }

  async testK8s(data: TestK8sConnectionRequest): Promise<TestConnectionResult> {
    const response = await api.post<TestConnectionResult>('/connections/test-k8s', data)
    return response.data
  }

  async getByType(type: string): Promise<Connection[]> {
    const response = await api.get<Connection[]>(`/connections/type/${type}`)
    return response.data
  }
}

// ==================== HTTP Cluster API ====================

class HttpClusterApi implements IClusterApi {
  async getAll(): Promise<Cluster[]> {
    const response = await api.get<Cluster[]>('/clusters')
    return response.data
  }

  async create(data: Partial<Cluster>): Promise<Cluster> {
    const response = await api.post<Cluster>('/clusters', data)
    return response.data
  }

  async update(id: number, data: Partial<Cluster>): Promise<Cluster> {
    const response = await api.put<Cluster>(`/clusters/${id}`, data)
    return response.data
  }

  async delete(id: number): Promise<void> {
    await api.delete(`/clusters/${id}`)
  }
}

// ==================== HTTP MySQL API ====================

class HttpMysqlApi implements IMysqlApi {
  async getInfo(): Promise<unknown> {
    const response = await api.get('/mysql/info')
    return response.data
  }

  async listDatabases(): Promise<unknown> {
    const response = await api.get('/mysql/databases')
    return response.data
  }

  async createDatabase(data: CreateDatabaseRequest): Promise<unknown> {
    const response = await api.post('/mysql/databases', data)
    return response.data
  }

  async alterDatabase(name: string, data: AlterDatabaseRequest): Promise<unknown> {
    const response = await api.put(`/mysql/databases/${name}`, data)
    return response.data
  }

  async grantPrivileges(name: string, data: GrantPrivilegesRequest): Promise<unknown> {
    const response = await api.post(`/mysql/databases/${name}/grant`, data)
    return response.data
  }

  async dropDatabase(name: string): Promise<unknown> {
    const response = await api.delete(`/mysql/databases/${name}`)
    return response.data
  }

  async listTables(database: string): Promise<unknown> {
    const response = await api.get(`/mysql/databases/${database}/tables`)
    return response.data
  }

  async createTable(database: string, data: CreateTableRequest): Promise<unknown> {
    const response = await api.post(`/mysql/databases/${database}/tables`, data)
    return response.data
  }

  async dropTable(database: string, table: string): Promise<unknown> {
    const response = await api.delete(`/mysql/databases/${database}/tables/${table}`)
    return response.data
  }

  async getTableSchema(database: string, table: string): Promise<unknown> {
    const response = await api.get(`/mysql/databases/${database}/tables/${table}/schema`)
    return response.data
  }

  async getDatabaseSchema(database: string): Promise<unknown> {
    const response = await api.get(`/mysql/databases/${database}/schema`)
    return response.data
  }

  async alterTable(database: string, table: string, data: AlterTableRequest): Promise<unknown> {
    const response = await api.put(`/mysql/databases/${database}/tables/${table}/schema`, data)
    return response.data
  }

  async getTablePrimaryKey(database: string, table: string): Promise<{ primary_key: string }> {
    const response = await api.get<{ primary_key: string }>(
      `/mysql/databases/${database}/tables/${table}/primary-key`
    )
    return response.data
  }

  async getRows(database: string, table: string, page = 1, size = 50): Promise<unknown> {
    const response = await api.get(`/mysql/databases/${database}/tables/${table}/rows`, {
      params: { page, size },
    })
    return response.data
  }

  async insertRow(database: string, table: string, data: Record<string, unknown>): Promise<unknown> {
    const response = await api.post(`/mysql/databases/${database}/tables/${table}/rows`, data)
    return response.data
  }

  async updateRow(database: string, table: string, data: UpdateRowRequest): Promise<unknown> {
    const response = await api.put(`/mysql/databases/${database}/tables/${table}/rows`, data)
    return response.data
  }

  async updateRecord(
    database: string,
    table: string,
    primaryKey: string,
    primaryValue: unknown,
    updates: Record<string, unknown>
  ): Promise<unknown> {
    const response = await api.put(`/mysql/databases/${database}/tables/${table}/record`, {
      primary_key: primaryKey,
      primary_value: primaryValue,
      updates,
    })
    return response.data
  }

  async deleteRow(database: string, table: string, where: Record<string, unknown>): Promise<unknown> {
    const response = await api.delete(`/mysql/databases/${database}/tables/${table}/rows`, { data: where })
    return response.data
  }

  async executeQuery(database: string, query: string): Promise<unknown> {
    const response = await api.post('/mysql/query', { database, query })
    return response.data
  }

  async exportData(database: string, table: string, format = 'json'): Promise<unknown> {
    const response = await api.post('/mysql/export', { database, table, format })
    return response.data
  }

  async importData(database: string, table: string, rows: Record<string, unknown>[]): Promise<unknown> {
    const response = await api.post('/mysql/import', { database, table, rows })
    return response.data
  }

  async listUsers(): Promise<UserInfo[]> {
    const response = await api.get<UserInfo[]>('/mysql/users')
    return response.data
  }

  async createUser(data: CreateUserRequest): Promise<unknown> {
    const response = await api.post('/mysql/users', data)
    return response.data
  }

  async listUserGrants(username: string, host?: string): Promise<unknown> {
    const response = await api.get('/mysql/users/grants', { params: { username, host } })
    return response.data
  }
}

// ==================== HTTP Redis API ====================

class HttpRedisApi implements IRedisApi {
  async getInfo(): Promise<unknown> {
    const response = await api.get('/redis/info')
    return response.data
  }

  async listKeys(pattern = '*', cursor = 0, count = 100): Promise<unknown> {
    const response = await api.get('/redis/keys', { params: { pattern, cursor, count } })
    return response.data
  }

  async getKey(key: string): Promise<unknown> {
    const response = await api.get(`/redis/keys/${encodeURIComponent(key)}`)
    return response.data
  }

  async setKey(data: SetKeyRequest): Promise<unknown> {
    const response = await api.post('/redis/keys', data)
    return response.data
  }

  async updateKey(key: string, data: SetKeyRequest): Promise<unknown> {
    const response = await api.put(`/redis/keys/${encodeURIComponent(key)}`, data)
    return response.data
  }

  async deleteKey(key: string): Promise<unknown> {
    const response = await api.delete(`/redis/keys/${encodeURIComponent(key)}`)
    return response.data
  }

  async setTTL(key: string, ttl: number): Promise<unknown> {
    const response = await api.put(`/redis/ttl/${encodeURIComponent(key)}`, { ttl })
    return response.data
  }

  async exportKeys(keys: string[]): Promise<unknown> {
    const response = await api.post('/redis/export', { keys })
    return response.data
  }

  async importKeys(data: ExportData): Promise<unknown> {
    const response = await api.post('/redis/import', data)
    return response.data
  }
}

// ==================== HTTP History API ====================

class HttpHistoryApi implements IHistoryApi {
  async getHistory(params?: {
    type?: string
    database?: string
    status?: string
    keyword?: string
    limit?: number
    offset?: number
  }): Promise<QueryHistoryListResponse> {
    const response = await api.get<QueryHistoryListResponse>('/history', { params })
    return response.data
  }

  async addHistory(data: AddQueryHistoryRequest): Promise<QueryHistory> {
    const response = await api.post<QueryHistory>('/history', data)
    return response.data
  }

  async deleteHistory(id: number): Promise<void> {
    await api.delete(`/history/${id}`)
  }

  async cleanupHistory(days: number): Promise<{ deleted: number }> {
    const response = await api.post<{ deleted: number }>('/history/cleanup', { days })
    return response.data
  }
}

// ==================== HTTP Saved Query API ====================

class HttpSavedQueryApi implements ISavedQueryApi {
  async getSavedQueries(category?: string): Promise<SavedQuery[]> {
    const response = await api.get<SavedQuery[]>('/saved-queries', { params: { category } })
    return response.data
  }

  async createSavedQuery(data: CreateSavedQueryRequest): Promise<SavedQuery> {
    const response = await api.post<SavedQuery>('/saved-queries', data)
    return response.data
  }

  async updateSavedQuery(id: number, data: UpdateSavedQueryRequest): Promise<SavedQuery> {
    const response = await api.put<SavedQuery>(`/saved-queries/${id}`, data)
    return response.data
  }

  async deleteSavedQuery(id: number): Promise<void> {
    await api.delete(`/saved-queries/${id}`)
  }
}

// ==================== HTTP K8s API ====================

class HttpK8sApi implements IK8sApi {
  async discover(kubeconfig?: string, context?: string, signal?: AbortSignal): Promise<DiscoveredService[]> {
    const response = await api.post<DiscoveredService[]>(
      '/k8s/discover',
      { kubeconfig, context },
      { timeout: 60000, signal }
    )
    return response.data
  }

  async listClusters(kubeconfig: string): Promise<{ clusters: string[] }> {
    const response = await api.post<{ clusters: string[] }>('/k8s/clusters', { kubeconfig })
    return response.data
  }

  async readLocalKubeconfig(): Promise<string> {
    const response = await api.get<string>('/k8s/local-kubeconfig')
    return response.data
  }

  async importConnections(
    services: DiscoveredService[],
    forceOverride?: boolean,
    kubeconfig?: string,
    context?: string,
    clusterName?: string
  ): Promise<ImportConnectionsResponse> {
    const response = await api.post<ImportConnectionsResponse>('/k8s/import', {
      services,
      force_override: forceOverride || false,
      kubeconfig,
      context,
      cluster_name: clusterName,
    })
    return response.data
  }

  // K8s resource listing methods
  async listNamespaces(clusterId: number): Promise<string[]> {
    const response = await api.get<string[]>(`/k8s/clusters/${clusterId}/namespaces`)
    return response.data
  }

  async listDeployments(clusterId: number, namespace: string): Promise<K8sDeployment[]> {
    const response = await api.get<K8sDeployment[]>(
      `/k8s/clusters/${clusterId}/namespaces/${namespace}/deployments`
    )
    return response.data
  }

  async listPods(clusterId: number, namespace: string): Promise<K8sPod[]> {
    const response = await api.get<K8sPod[]>(
      `/k8s/clusters/${clusterId}/namespaces/${namespace}/pods`
    )
    return response.data
  }

  async listConfigMaps(clusterId: number, namespace: string): Promise<K8sConfigMapInfo[]> {
    const response = await api.get<K8sConfigMapInfo[]>(
      `/k8s/clusters/${clusterId}/namespaces/${namespace}/configmaps`
    )
    return response.data
  }

  async getConfigMapData(clusterId: number, namespace: string, name: string): Promise<Record<string, string>> {
    const response = await api.get<Record<string, string>>(
      `/k8s/clusters/${clusterId}/namespaces/${namespace}/configmaps/${name}`
    )
    return response.data
  }

  async listSecrets(clusterId: number, namespace: string): Promise<K8sSecretInfo[]> {
    const response = await api.get<K8sSecretInfo[]>(
      `/k8s/clusters/${clusterId}/namespaces/${namespace}/secrets`
    )
    return response.data
  }

  async listServices(clusterId: number, namespace: string): Promise<K8sServiceInfo[]> {
    const response = await api.get<K8sServiceInfo[]>(
      `/k8s/clusters/${clusterId}/namespaces/${namespace}/services`
    )
    return response.data
  }

  async listIngresses(clusterId: number, namespace: string): Promise<K8sIngressInfo[]> {
    const response = await api.get<K8sIngressInfo[]>(
      `/k8s/clusters/${clusterId}/namespaces/${namespace}/ingresses`
    )
    return response.data
  }
}

// ==================== HTTP Port Forward API ====================

class HttpPortForwardApi implements IPortForwardApi {
  async create(
    connectionId: number,
    namespace: string,
    serviceName: string,
    remotePort: number,
    localPort?: number
  ): Promise<ForwardInfo> {
    const response = await api.post<ForwardInfo>('/port-forward', {
      connection_id: connectionId,
      namespace,
      service_name: serviceName,
      remote_port: remotePort,
      local_port: localPort && localPort > 0 ? localPort : undefined,
    })
    return response.data
  }

  async list(): Promise<ForwardListResponse> {
    const response = await api.get<ForwardListResponse>('/port-forward')
    return response.data
  }

  async get(id: string): Promise<ForwardInfo> {
    const response = await api.get<ForwardInfo>(`/port-forward/${id}`)
    return response.data
  }

  async getByConnection(connectionId: number): Promise<ForwardInfo> {
    const response = await api.get<ForwardInfo>('/port-forward/by-connection', {
      params: { connection_id: connectionId },
    })
    return response.data
  }

  async stop(id: string): Promise<void> {
    await api.delete(`/port-forward/${id}`)
  }

  async reconnect(id: string, localPort?: number): Promise<ForwardInfo> {
    const response = await api.post<ForwardInfo>(`/port-forward/${id}/reconnect`, {
      local_port: localPort && localPort > 0 ? localPort : undefined,
    })
    return response.data
  }

  async touch(id: string): Promise<void> {
    await api.put(`/port-forward/${id}/touch`)
  }
}

// ==================== HTTP Settings API ====================

class HttpSettingsApi implements ISettingsApi {
  async getAll(): Promise<UserSetting[]> {
    const response = await api.get<UserSetting[]>('/settings')
    return response.data
  }

  async get(key: string): Promise<unknown | null> {
    try {
      const response = await api.get<{ value: unknown }>(`/settings/${encodeURIComponent(key)}`)
      return response.data.value
    } catch {
      return null
    }
  }

  async getBatch(keys: string[]): Promise<BatchSettingsResponse> {
    const response = await api.post<BatchSettingsResponse>('/settings/batch', { keys })
    return response.data
  }

  async set(key: string, value: unknown): Promise<UserSetting> {
    const response = await api.put<UserSetting>(`/settings/${encodeURIComponent(key)}`, { value })
    return response.data
  }

  async delete(key: string): Promise<void> {
    await api.delete(`/settings/${encodeURIComponent(key)}`)
  }
}

// ==================== HTTP LLM Config API ====================

class HttpLLMConfigApi implements ILLMConfigApi {
  async getAll(): Promise<LLMConfigResponse[]> {
    const response = await api.get<LLMConfigResponse[]>('/llm-configs')
    return response.data
  }

  async get(id: number): Promise<LLMConfigResponse> {
    const response = await api.get<LLMConfigResponse>(`/llm-configs/${id}`)
    return response.data
  }

  async getDefault(): Promise<LLMConfigResponse | null> {
    try {
      const response = await api.get<LLMConfigResponse>('/llm-configs/default')
      return response.data
    } catch {
      return null
    }
  }

  async create(data: CreateLLMConfigRequest): Promise<LLMConfigResponse> {
    const response = await api.post<LLMConfigResponse>('/llm-configs', data)
    return response.data
  }

  async update(id: number, data: UpdateLLMConfigRequest): Promise<LLMConfigResponse> {
    const response = await api.put<LLMConfigResponse>(`/llm-configs/${id}`, data)
    return response.data
  }

  async delete(id: number): Promise<void> {
    await api.delete(`/llm-configs/${id}`)
  }

  async setDefault(id: number): Promise<LLMConfigResponse> {
    const response = await api.put<LLMConfigResponse>(`/llm-configs/${id}/default`)
    return response.data
  }

  async getApiKey(id: number): Promise<string | null> {
    try {
      const response = await api.get<{ api_key: string | null }>(`/llm-configs/${id}/api-key`)
      return response.data.api_key
    } catch {
      return null
    }
  }
}

// ==================== HTTP Adapter Factory ====================

/**
 * Create HTTP adapter instance
 */
export function createHttpAdapter(): IApiAdapter {
  return {
    connection: new HttpConnectionApi(),
    cluster: new HttpClusterApi(),
    mysql: new HttpMysqlApi(),
    redis: new HttpRedisApi(),
    history: new HttpHistoryApi(),
    savedQuery: new HttpSavedQueryApi(),
    k8s: new HttpK8sApi(),
    portForward: new HttpPortForwardApi(),
    settings: new HttpSettingsApi(),
    llmConfig: new HttpLLMConfigApi(),
  }
}

// Export axios instance for legacy usage
export default api
