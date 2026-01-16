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
  // User permission management types
  AlterUserPasswordRequest,
  DropUserRequest,
  RevokePrivilegesRequest,
  UserGrantInfo,
  UserGrantsResponse,
  // Table management types
  ColumnDefinition,
  IndexDefinition,
  CreateTableRequest,
  AlterTableRequest,
  RenameColumnRequest,
  RenameTableRequest,
  CopyTableRequest,
  // Index management types
  IndexInfo,
  CreateIndexRequest,
  // Foreign key management types
  ForeignKeyInfo,
  CreateForeignKeyRequest,
  // Data export/import types
  ExportFormat,
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
  // Trigger management types
  TriggerInfo,
  TriggerDefinition,
  // Server monitoring types
  ServerVariable,
  ProcessInfo,
  ExplainResult,
  TableMaintenanceResult,
  ColumnDef, // Legacy, deprecated
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
  /** Drop a database */
  dropDatabase(name: string): Promise<unknown>

  // Table operations
  /** List tables in a database */
  listTables(database: string): Promise<unknown>
  /** Create a new table */
  createTable(database: string, data: import('./index').CreateTableRequest): Promise<void>
  /** Drop a table */
  dropTable(database: string, table: string): Promise<void>
  /** Rename a table */
  renameTable(database: string, table: string, newName: string): Promise<void>
  /** Truncate a table (delete all rows, reset auto-increment) */
  truncateTable(database: string, table: string): Promise<void>
  /** Copy a table (structure only or with data) */
  copyTable(database: string, table: string, targetName: string, withData?: boolean): Promise<void>

  // Index operations
  /** List all indexes on a table */
  listIndexes(database: string, table: string): Promise<import('./index').IndexInfo[]>
  /** Create an index on a table */
  createIndex(database: string, table: string, data: import('./index').CreateIndexRequest): Promise<void>
  /** Drop an index from a table */
  dropIndex(database: string, table: string, indexName: string): Promise<void>

  // Foreign key operations
  /** List all foreign keys on a table */
  listForeignKeys(database: string, table: string): Promise<import('./index').ForeignKeyInfo[]>
  /** Create a foreign key on a table */
  createForeignKey(database: string, table: string, data: import('./index').CreateForeignKeyRequest): Promise<void>
  /** Drop a foreign key from a table */
  dropForeignKey(database: string, table: string, fkName: string): Promise<void>

  // Schema operations
  /** Get table schema */
  getTableSchema(database: string, table: string): Promise<unknown>
  /** Get database schema */
  getDatabaseSchema(database: string): Promise<unknown>
  /** Alter table schema */
  alterTable(database: string, table: string, data: import('./index').AlterTableRequest): Promise<void>
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
  exportTable(database: string, table: string, request: import('./index').ExportTableRequest): Promise<import('./index').ExportTableResponse>
  /** Import data into table */
  importData(database: string, table: string, request: import('./index').ImportDataRequest): Promise<import('./index').ImportResult>

  // User management
  /** List MySQL users */
  listUsers(): Promise<import('./index').UserInfo[]>
  /** Create a MySQL user */
  createUser(data: import('./index').CreateUserRequest): Promise<unknown>
  /** Grant privileges to a user */
  grantPrivileges(database: string, data: import('./index').GrantPrivilegesRequest): Promise<void>
  /** Alter user password */
  alterUserPassword(data: import('./index').AlterUserPasswordRequest): Promise<void>
  /** Drop a MySQL user */
  dropUser(data: import('./index').DropUserRequest): Promise<void>
  /** Show grants for a user */
  showGrants(username: string, host: string): Promise<import('./index').UserGrantsResponse>
  /** Revoke privileges from a user */
  revokePrivileges(data: import('./index').RevokePrivilegesRequest): Promise<void>

  // View operations
  /** List all views in a database */
  listViews(database: string): Promise<import('./index').ViewInfo[]>
  /** Get view definition */
  getViewDefinition(database: string, view: string): Promise<import('./index').ViewDefinition>
  /** Create a view */
  createView(database: string, data: import('./index').CreateViewRequest): Promise<void>
  /** Drop a view */
  dropView(database: string, view: string): Promise<void>

  // Stored procedure operations
  /** List all stored procedures and functions in a database */
  listProcedures(database: string): Promise<import('./index').ProcedureInfo[]>
  /** Get procedure/function definition */
  getProcedureDefinition(database: string, name: string, routineType?: string): Promise<import('./index').ProcedureDefinition>
  /** Drop a stored procedure */
  dropProcedure(database: string, name: string): Promise<void>
  /** Drop a function */
  dropFunction(database: string, name: string): Promise<void>

  // Trigger operations
  /** List all triggers in a database */
  listTriggers(database: string): Promise<import('./index').TriggerInfo[]>
  /** Get trigger definition */
  getTriggerDefinition(database: string, name: string): Promise<import('./index').TriggerDefinition>
  /** Drop a trigger */
  dropTrigger(database: string, name: string): Promise<void>

  // Server monitoring operations
  /** Get server variables */
  getServerVariables(filter?: string): Promise<import('./index').ServerVariable[]>
  /** Get process list */
  getProcessList(): Promise<import('./index').ProcessInfo[]>
  /** Kill a process */
  killProcess(processId: number): Promise<void>

  // Query analysis operations
  /** Explain a query */
  explainQuery(database: string, query: string): Promise<import('./index').ExplainResult>

  // Table maintenance operations
  /** Optimize a table */
  optimizeTable(database: string, table: string): Promise<import('./index').TableMaintenanceResult>
  /** Analyze a table */
  analyzeTable(database: string, table: string): Promise<import('./index').TableMaintenanceResult>
  /** Check a table */
  checkTable(database: string, table: string): Promise<import('./index').TableMaintenanceResult>
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
  /** ExternalName service target (only for ExternalName type) */
  external_name?: string
}

/**
 * Proxy Pod info for ExternalName service connection
 */
export interface ProxyPodInfo {
  name: string
  namespace: string
  /** Target type: mysql, redis, etc. */
  target_type?: string
  /** Target host being proxied */
  target_host?: string
  /** Target port being proxied */
  target_port?: number
  /** Pod status: Running, Pending, etc. */
  status: string
  /** Associated Service name */
  service_name?: string
}

/**
 * Request to create a TCP proxy
 */
export interface CreateProxyRequest {
  proxy_name: string
  target_host: string
  target_port: number
  target_type: string
  /** Optional custom image for the proxy container (defaults to "alpine/socat") */
  image?: string
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

/**
 * K8s Pod detailed information
 */
export interface K8sPodDetail {
  name: string
  namespace: string
  status: string
  phase: string
  node?: string
  ip?: string
  host_ip?: string
  start_time?: string
  containers: K8sContainerInfo[]
  init_containers: K8sContainerInfo[]
  conditions: K8sPodCondition[]
  labels: Record<string, string>
}

/**
 * K8s Container information
 */
export interface K8sContainerInfo {
  name: string
  image: string
  image_pull_policy?: string
  ports: K8sContainerPort[]
  env: K8sEnvVar[]
  resources?: K8sResourceRequirements
  state: string
  ready: boolean
  restart_count: number
}

/**
 * K8s Container port
 */
export interface K8sContainerPort {
  name?: string
  container_port: number
  protocol: string
}

/**
 * K8s Environment variable
 */
export interface K8sEnvVar {
  name: string
  value?: string
  value_from?: string
}

/**
 * K8s Resource requirements
 */
export interface K8sResourceRequirements {
  cpu_request?: string
  memory_request?: string
  cpu_limit?: string
  memory_limit?: string
}

/**
 * K8s Pod condition
 */
export interface K8sPodCondition {
  condition_type: string
  status: string
  last_transition_time?: string
  reason?: string
  message?: string
}

// ==================== Extended K8s Workload Types ====================

/**
 * K8s Job
 */
export interface K8sJob {
  name: string
  namespace: string
  completions?: number
  succeeded: number
  failed: number
  active: number
  start_time?: string
  completion_time?: string
  created_at?: string
}

/**
 * K8s CronJob
 */
export interface K8sCronJob {
  name: string
  namespace: string
  schedule: string
  suspend: boolean
  active: number
  last_schedule_time?: string
  last_successful_time?: string
  created_at?: string
}

/**
 * K8s StatefulSet
 */
export interface K8sStatefulSet {
  name: string
  namespace: string
  replicas: number
  ready_replicas: number
  current_replicas: number
  updated_replicas: number
  created_at?: string
}

/**
 * K8s DaemonSet
 */
export interface K8sDaemonSet {
  name: string
  namespace: string
  desired_number_scheduled: number
  current_number_scheduled: number
  number_ready: number
  number_available: number
  created_at?: string
}

/**
 * K8s ReplicaSet
 */
export interface K8sReplicaSet {
  name: string
  namespace: string
  replicas: number
  ready_replicas: number
  available_replicas: number
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

  /** Read local kubeconfig file (~/.kube/config) */
  readLocalKubeconfig(): Promise<string>

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

  /** Get pod detailed information */
  getPodDetail(clusterId: number, namespace: string, name: string): Promise<K8sPodDetail>

  /** Get pod logs */
  getPodLogs(clusterId: number, namespace: string, name: string, container?: string, tailLines?: number): Promise<string>

  /** List configmaps in a namespace */
  listConfigMaps(clusterId: number, namespace: string): Promise<K8sConfigMapInfo[]>

  /** Get configmap data */
  getConfigMapData(clusterId: number, namespace: string, name: string): Promise<Record<string, string>>

  /** Update configmap data */
  updateConfigMap(clusterId: number, namespace: string, name: string, data: Record<string, string>): Promise<void>

  /** List secrets in a namespace (metadata only) */
  listSecrets(clusterId: number, namespace: string): Promise<K8sSecretInfo[]>

  /** Get secret data (decoded from base64) */
  getSecretData(clusterId: number, namespace: string, name: string): Promise<Record<string, string>>

  /** Update secret data */
  updateSecret(clusterId: number, namespace: string, name: string, data: Record<string, string>): Promise<void>

  /** List services in a namespace */
  listServices(clusterId: number, namespace: string): Promise<K8sServiceInfo[]>

  /** List ingresses in a namespace */
  listIngresses(clusterId: number, namespace: string): Promise<K8sIngressInfo[]>

  // Extended workload types
  /** List Jobs in a namespace */
  listJobs(clusterId: number, namespace: string): Promise<K8sJob[]>

  /** List CronJobs in a namespace */
  listCronJobs(clusterId: number, namespace: string): Promise<K8sCronJob[]>

  /** List StatefulSets in a namespace */
  listStatefulSets(clusterId: number, namespace: string): Promise<K8sStatefulSet[]>

  /** List DaemonSets in a namespace */
  listDaemonSets(clusterId: number, namespace: string): Promise<K8sDaemonSet[]>

  /** List ReplicaSets in a namespace */
  listReplicaSets(clusterId: number, namespace: string): Promise<K8sReplicaSet[]>

  // Deployment operations
  /** Get Deployment as YAML */
  getDeploymentYaml(clusterId: number, namespace: string, name: string): Promise<string>

  /** Update Deployment from YAML */
  updateDeploymentYaml(clusterId: number, namespace: string, name: string, yaml: string): Promise<void>

  /** Scale Deployment */
  scaleDeployment(clusterId: number, namespace: string, name: string, replicas: number): Promise<void>

  /** Restart Deployment (trigger rolling update) */
  restartDeployment(clusterId: number, namespace: string, name: string): Promise<void>

  // Proxy operations
  /** List proxy pods with zeni-x=proxy label in a namespace */
  listProxies(clusterId: number, namespace: string): Promise<ProxyPodInfo[]>

  /** List all proxy pods across all namespaces */
  listAllProxies(clusterId: number): Promise<ProxyPodInfo[]>

  /** Create a TCP proxy for ExternalName service */
  createProxy(clusterId: number, namespace: string, request: CreateProxyRequest): Promise<void>

  /** Delete a TCP proxy */
  deleteProxy(clusterId: number, namespace: string, proxyName: string): Promise<void>
}

// ==================== Settings Types ====================

/**
 * User setting entry
 */
export interface UserSetting {
  id?: number
  key: string
  value: string
  created_at?: string
  updated_at?: string
}

/**
 * Batch get settings request
 */
export interface BatchGetSettingsRequest {
  keys: string[]
}

/**
 * Batch settings response
 */
export interface BatchSettingsResponse {
  settings: Record<string, unknown>
}

// ==================== LLM Config Types ====================

/**
 * LLM config response (API key is not exposed)
 */
export interface LLMConfigResponse {
  id: number
  name: string
  provider: string
  has_api_key: boolean
  base_url?: string
  model: string
  max_tokens: number
  temperature: number
  is_default: boolean
  created_at?: string
  updated_at?: string
}

/**
 * Create LLM config request
 */
export interface CreateLLMConfigRequest {
  name: string
  provider: string
  api_key?: string
  base_url?: string
  model: string
  max_tokens?: number
  temperature?: number
  is_default?: boolean
}

/**
 * Update LLM config request
 */
export interface UpdateLLMConfigRequest {
  name?: string
  provider?: string
  api_key?: string
  base_url?: string
  model?: string
  max_tokens?: number
  temperature?: number
  is_default?: boolean
}

// ==================== Settings API Interface ====================

/**
 * User settings API interface
 */
export interface ISettingsApi {
  /** Get all settings */
  getAll(): Promise<UserSetting[]>

  /** Get a setting by key */
  get(key: string): Promise<unknown | null>

  /** Get multiple settings by keys */
  getBatch(keys: string[]): Promise<BatchSettingsResponse>

  /** Set a setting (upsert) */
  set(key: string, value: unknown): Promise<UserSetting>

  /** Delete a setting */
  delete(key: string): Promise<void>
}

// ==================== LLM Config API Interface ====================

/**
 * LLM configuration API interface
 */
export interface ILLMConfigApi {
  /** Get all LLM configs */
  getAll(): Promise<LLMConfigResponse[]>

  /** Get an LLM config by ID */
  get(id: number): Promise<LLMConfigResponse>

  /** Get the default LLM config */
  getDefault(): Promise<LLMConfigResponse | null>

  /** Create a new LLM config */
  create(data: CreateLLMConfigRequest): Promise<LLMConfigResponse>

  /** Update an LLM config */
  update(id: number, data: UpdateLLMConfigRequest): Promise<LLMConfigResponse>

  /** Delete an LLM config */
  delete(id: number): Promise<void>

  /** Set an LLM config as default */
  setDefault(id: number): Promise<LLMConfigResponse>

  /** Get the API key for a config (for making LLM calls) */
  getApiKey(id: number): Promise<string | null>
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

// ==================== K8s Favorites Types ====================

/**
 * K8s favorite entry - saved cluster + namespace combination with alias
 */
export interface K8sFavorite {
  id?: number
  /** Display name / alias (e.g., "游戏UAT") */
  name: string
  /** Associated cluster ID */
  cluster_id: number
  /** Kubernetes namespace */
  namespace: string
  /** Optional description */
  description?: string
  /** Category for grouping */
  category?: string
  /** Sort order for display */
  sort_order: number
  /** Creation timestamp */
  created_at?: string
  /** Last update timestamp */
  updated_at?: string
}

/**
 * K8s favorite with cluster info for display
 */
export interface K8sFavoriteWithCluster {
  id: number
  /** Display name / alias */
  name: string
  /** Associated cluster ID */
  cluster_id: number
  /** Cluster name */
  cluster_name: string
  /** Kubernetes namespace */
  namespace: string
  /** Optional description */
  description?: string
  /** Category for grouping */
  category?: string
  /** Sort order */
  sort_order: number
  /** Creation timestamp */
  created_at?: string
  /** Last update timestamp */
  updated_at?: string
}

/**
 * Request to create a K8s favorite
 */
export interface CreateK8sFavoriteRequest {
  /** Display name / alias */
  name: string
  /** Associated cluster ID */
  cluster_id: number
  /** Kubernetes namespace */
  namespace: string
  /** Optional description */
  description?: string
  /** Category for grouping */
  category?: string
  /** Sort order (optional, defaults to 0) */
  sort_order?: number
}

/**
 * Request to update a K8s favorite
 */
export interface UpdateK8sFavoriteRequest {
  /** Display name / alias */
  name?: string
  /** Optional description */
  description?: string
  /** Category for grouping */
  category?: string
  /** Sort order */
  sort_order?: number
}

// ==================== K8s Favorites API Interface ====================

/**
 * K8s favorites API interface
 */
export interface IK8sFavoriteApi {
  /** Get all favorites with optional category filter */
  getAll(category?: string): Promise<K8sFavoriteWithCluster[]>

  /** Get a single favorite by ID */
  get(id: number): Promise<K8sFavorite>

  /** Check if a favorite exists for the given cluster and namespace */
  exists(clusterId: number, namespace: string): Promise<K8sFavorite | null>

  /** Create a new favorite */
  create(request: CreateK8sFavoriteRequest): Promise<K8sFavorite>

  /** Update an existing favorite */
  update(id: number, request: UpdateK8sFavoriteRequest): Promise<K8sFavorite>

  /** Delete a favorite */
  delete(id: number): Promise<void>
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

  /** User settings API */
  settings: ISettingsApi

  /** LLM configuration API */
  llmConfig: ILLMConfigApi

  /** K8s favorites API */
  k8sFavorite: IK8sFavoriteApi
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
