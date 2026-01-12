/**
 * API Adapter Interface Definitions
 *
 * This module defines the unified API interfaces that both HTTP and IPC adapters must implement.
 * The adapter pattern allows the frontend to seamlessly switch between Web (HTTP) and Tauri (IPC) modes.
 */

// Re-export existing types from index.ts
export type {
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
  ColumnDef,
  AlterTableRequest,
  UpdateRowRequest,
  SetKeyRequest,
  ExportData,
  KeyInfo,
  DiscoveredService,
  ImportConnectionsResponse,
  ImportConnectionResult,
  ForwardInfo,
  ForwardListResponse,
  QueryHistory,
  QueryHistoryListResponse,
  AddQueryHistoryRequest,
  SavedQuery,
  CreateSavedQueryRequest,
  UpdateSavedQueryRequest,
} from './index'

// ==================== API Response Types ====================

/**
 * Standard API response wrapper (for axios responses)
 */
export interface ApiResponse<T> {
  data: T
  status: number
  statusText: string
}

// ==================== Connection API Interface ====================

/**
 * Connection management API interface
 */
export interface IConnectionApi {
  /** Get all connections */
  getAll(): Promise<import('./index').Connection[]>

  /** Get single connection by ID */
  getById(id: number): Promise<import('./index').Connection>

  /** Create a new connection */
  create(data: import('./index').Connection): Promise<import('./index').Connection>

  /** Update an existing connection */
  update(id: number, data: import('./index').Connection): Promise<import('./index').Connection>

  /** Delete a connection */
  delete(id: number): Promise<void>

  /** Test connection credentials */
  test(data: import('./index').Connection): Promise<import('./index').TestConnectionResult>

  /** Test K8s connection with port forwarding */
  testK8s(data: import('./index').TestK8sConnectionRequest): Promise<import('./index').TestConnectionResult>

  /** Get connections by type */
  getByType(type: string): Promise<import('./index').Connection[]>
}

// ==================== Cluster API Interface ====================

/**
 * K8s cluster management API interface
 */
export interface IClusterApi {
  /** Get all clusters */
  getAll(): Promise<import('./index').Cluster[]>

  /** Create a new cluster */
  create(data: Partial<import('./index').Cluster>): Promise<import('./index').Cluster>

  /** Update a cluster */
  update(id: number, data: Partial<import('./index').Cluster>): Promise<import('./index').Cluster>

  /** Delete a cluster */
  delete(id: number): Promise<void>
}

// ==================== MySQL API Interface ====================

/**
 * MySQL database operations API interface
 */
export interface IMysqlApi {
  /** Get MySQL server info */
  getInfo(): Promise<unknown>

  // Database operations
  /** List all databases */
  listDatabases(): Promise<unknown>
  /** Create a new database */
  createDatabase(data: import('./index').CreateDatabaseRequest): Promise<unknown>
  /** Alter database settings */
  alterDatabase(name: string, data: import('./index').AlterDatabaseRequest): Promise<unknown>
  /** Grant privileges on a database */
  grantPrivileges(name: string, data: import('./index').GrantPrivilegesRequest): Promise<unknown>
  /** Drop a database */
  dropDatabase(name: string): Promise<unknown>

  // Table operations
  /** List tables in a database */
  listTables(database: string): Promise<unknown>
  /** Create a new table */
  createTable(database: string, data: import('./index').CreateTableRequest): Promise<unknown>
  /** Drop a table */
  dropTable(database: string, table: string): Promise<unknown>

  // Schema operations
  /** Get table schema */
  getTableSchema(database: string, table: string): Promise<unknown>
  /** Get database schema */
  getDatabaseSchema(database: string): Promise<unknown>
  /** Alter table schema */
  alterTable(database: string, table: string, data: import('./index').AlterTableRequest): Promise<unknown>
  /** Get table primary key */
  getTablePrimaryKey(database: string, table: string): Promise<{ primary_key: string }>

  // Data operations
  /** Get table rows with pagination */
  getRows(database: string, table: string, page?: number, size?: number): Promise<unknown>
  /** Insert a new row */
  insertRow(database: string, table: string, data: Record<string, unknown>): Promise<unknown>
  /** Update a row */
  updateRow(database: string, table: string, data: import('./index').UpdateRowRequest): Promise<unknown>
  /** Update a record by primary key */
  updateRecord(
    database: string,
    table: string,
    primaryKey: string,
    primaryValue: unknown,
    updates: Record<string, unknown>
  ): Promise<unknown>
  /** Delete a row */
  deleteRow(database: string, table: string, where: Record<string, unknown>): Promise<unknown>

  // Query operations
  /** Execute a SQL query */
  executeQuery(database: string, query: string): Promise<unknown>

  // Export/Import operations
  /** Export table data */
  exportData(database: string, table: string, format?: string): Promise<unknown>
  /** Import data into table */
  importData(database: string, table: string, rows: Record<string, unknown>[]): Promise<unknown>

  // User management
  /** List MySQL users */
  listUsers(): Promise<import('./index').UserInfo[]>
  /** Create a MySQL user */
  createUser(data: import('./index').CreateUserRequest): Promise<unknown>
  /** List user grants */
  listUserGrants(username: string, host?: string): Promise<unknown>
}

// ==================== Redis API Interface ====================

/**
 * Redis operations API interface
 */
export interface IRedisApi {
  /** Get Redis server info */
  getInfo(): Promise<unknown>

  // Key operations
  /** List keys with optional pattern and pagination */
  listKeys(pattern?: string, cursor?: number, count?: number): Promise<unknown>
  /** Get key value */
  getKey(key: string): Promise<unknown>
  /** Set a key */
  setKey(data: import('./index').SetKeyRequest): Promise<unknown>
  /** Update a key */
  updateKey(key: string, data: import('./index').SetKeyRequest): Promise<unknown>
  /** Delete a key */
  deleteKey(key: string): Promise<unknown>

  // TTL operations
  /** Set TTL for a key */
  setTTL(key: string, ttl: number): Promise<unknown>

  // Export/Import operations
  /** Export keys */
  exportKeys(keys: string[]): Promise<unknown>
  /** Import keys */
  importKeys(data: import('./index').ExportData): Promise<unknown>
}

// ==================== History API Interface ====================

/**
 * Query history API interface
 */
export interface IHistoryApi {
  /** Get query history with optional filters */
  getHistory(params?: {
    type?: string
    database?: string
    status?: string
    keyword?: string
    limit?: number
    offset?: number
  }): Promise<import('./index').QueryHistoryListResponse>

  /** Add a query history entry */
  addHistory(data: import('./index').AddQueryHistoryRequest): Promise<import('./index').QueryHistory>

  /** Delete a history entry */
  deleteHistory(id: number): Promise<void>

  /** Cleanup old history entries */
  cleanupHistory(days: number): Promise<{ deleted: number }>
}

// ==================== Saved Queries API Interface ====================

/**
 * Saved queries API interface
 */
export interface ISavedQueryApi {
  /** Get saved queries with optional category filter */
  getSavedQueries(category?: string): Promise<import('./index').SavedQuery[]>

  /** Create a saved query */
  createSavedQuery(data: import('./index').CreateSavedQueryRequest): Promise<import('./index').SavedQuery>

  /** Update a saved query */
  updateSavedQuery(id: number, data: import('./index').UpdateSavedQueryRequest): Promise<import('./index').SavedQuery>

  /** Delete a saved query */
  deleteSavedQuery(id: number): Promise<void>
}

// ==================== K8s Resource Types ====================

/**
 * K8s Deployment info
 */
export interface K8sDeployment {
  name: string
  namespace: string
  replicas: number
  ready_replicas: number
  available_replicas: number
  labels: Record<string, string>
  created_at?: string
}

/**
 * K8s Pod info
 */
export interface K8sPod {
  name: string
  namespace: string
  status: string
  ready: string
  restarts: number
  node?: string
  ip?: string
  created_at?: string
}

/**
 * K8s ConfigMap info (metadata only)
 */
export interface K8sConfigMapInfo {
  name: string
  namespace: string
  data_keys: string[]
  created_at?: string
}

/**
 * K8s Secret info (metadata only)
 */
export interface K8sSecretInfo {
  name: string
  namespace: string
  secret_type: string
  data_keys: string[]
  created_at?: string
}

/**
 * K8s Service info
 */
export interface K8sServiceInfo {
  name: string
  namespace: string
  service_type: string
  cluster_ip?: string
  external_ip?: string
  ports: string[]
  created_at?: string
}

/**
 * K8s Ingress info
 */
export interface K8sIngressInfo {
  name: string
  namespace: string
  hosts: string[]
  address?: string
  created_at?: string
}

// ==================== K8s API Interface ====================

/**
 * K8s service discovery API interface
 */
export interface IK8sApi {
  /** Discover middleware services in K8s cluster */
  discover(kubeconfig?: string, context?: string, signal?: AbortSignal): Promise<import('./index').DiscoveredService[]>

  /** List clusters from kubeconfig */
  listClusters(kubeconfig: string): Promise<{ clusters: string[] }>

  /** Import discovered services as connections */
  importConnections(
    services: import('./index').DiscoveredService[],
    forceOverride?: boolean,
    kubeconfig?: string,
    context?: string,
    clusterName?: string
  ): Promise<import('./index').ImportConnectionsResponse>

  // K8s resource listing (requires cluster_id)
  /** List all namespaces in a cluster */
  listNamespaces(clusterId: number): Promise<string[]>

  /** List deployments in a namespace */
  listDeployments(clusterId: number, namespace: string): Promise<K8sDeployment[]>

  /** List pods in a namespace */
  listPods(clusterId: number, namespace: string): Promise<K8sPod[]>

  /** List configmaps in a namespace */
  listConfigMaps(clusterId: number, namespace: string): Promise<K8sConfigMapInfo[]>

  /** Get configmap data */
  getConfigMapData(clusterId: number, namespace: string, name: string): Promise<Record<string, string>>

  /** List secrets in a namespace (metadata only) */
  listSecrets(clusterId: number, namespace: string): Promise<K8sSecretInfo[]>

  /** List services in a namespace */
  listServices(clusterId: number, namespace: string): Promise<K8sServiceInfo[]>

  /** List ingresses in a namespace */
  listIngresses(clusterId: number, namespace: string): Promise<K8sIngressInfo[]>
}

// ==================== Port Forward API Interface ====================

/**
 * Port forwarding API interface
 */
export interface IPortForwardApi {
  /** Create a port forward
   * @param connectionId - The connection ID to forward
   * @param namespace - K8s namespace (used by HTTP adapter)
   * @param serviceName - K8s service name (used by HTTP adapter)
   * @param remotePort - Remote port to forward (used by HTTP adapter)
   * @param localPort - Optional preferred local port. If not provided, auto-assign an available port.
   */
  create(connectionId: number, namespace: string, serviceName: string, remotePort: number, localPort?: number): Promise<import('./index').ForwardInfo>

  /** List all forwards */
  list(): Promise<import('./index').ForwardListResponse>

  /** Get forward by ID */
  get(id: string): Promise<import('./index').ForwardInfo>

  /** Get forward by connection ID */
  getByConnection(connectionId: number): Promise<import('./index').ForwardInfo>

  /** Stop a forward */
  stop(id: string): Promise<void>

  /** Reconnect a forward
   * @param id - The port forward ID to reconnect
   * @param localPort - Optional preferred local port. If not provided, reuse the existing port.
   */
  reconnect(id: string, localPort?: number): Promise<import('./index').ForwardInfo>

  /** Update last used time */
  touch(id: string): Promise<void>
}

// ==================== Unified API Adapter Interface ====================

/**
 * Unified API adapter interface
 *
 * All adapters (HTTP, IPC) must implement this interface to ensure
 * consistent API access across different platforms.
 */
export interface IApiAdapter {
  /** Connection management API */
  connection: IConnectionApi

  /** K8s cluster management API */
  cluster: IClusterApi

  /** MySQL operations API */
  mysql: IMysqlApi

  /** Redis operations API */
  redis: IRedisApi

  /** Query history API */
  history: IHistoryApi

  /** Saved queries API */
  savedQuery: ISavedQueryApi

  /** K8s service discovery API */
  k8s: IK8sApi

  /** Port forwarding API */
  portForward: IPortForwardApi
}

// ==================== API Error ====================

/**
 * Unified API error class
 */
export class ApiError extends Error {
  constructor(
    message: string,
    public code?: string,
    public details?: unknown
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

/**
 * Handle and convert errors to ApiError
 */
export function handleApiError(error: unknown): never {
  if (error instanceof ApiError) {
    throw error
  }
  if (error instanceof Error) {
    throw new ApiError(error.message)
  }
  throw new ApiError('Unknown error occurred')
}
