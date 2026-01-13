import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { mysqlApi, type CreateDatabaseRequest, type AlterDatabaseRequest, type GrantPrivilegesRequest, type CreateUserRequest, type UserInfo } from '@/api'
import { getApiAdapter } from '@/api/adapter'

export interface Database {
  name: string
  table_count: number
  size: string
}

export interface Table {
  name: string
  engine: string
  row_count: number
  data_size: number
  index_size: number
  comment: string
}

export interface Column {
  name: string
  type: string
  nullable: boolean
  key: string
  default: string | null
  extra: string
  comment: string
}

export interface TableSchema {
  name: string
  columns: Column[]
  indexes: Index[]
}

export interface Index {
  name: string
  columns: string[]
  unique: boolean
  type: string
}

export interface ServerInfo {
  version: string
  host: string
  port: number
  connected: boolean
}

// Settings key for query limit
const QUERY_LIMIT_KEY = 'mysql_query_limit'

export const useMySQLStore = defineStore('mysql', () => {
  const serverInfo = ref<ServerInfo | null>(null)
  const databases = ref<Database[]>([])
  const currentDatabase = ref<string>('')
  const tables = ref<Table[]>([])
  const currentTable = ref<string>('')
  const tableSchema = ref<TableSchema | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const users = ref<UserInfo[]>([])
  const fetchedDatabases = ref<Set<string>>(new Set())
  const tablesCache = ref<Map<string, Table[]>>(new Map())

  // ========== Query limit feature ==========
  const queryLimit = ref<number>(100)
  const queryLimitInitialized = ref(false)

  // Load query limit from backend settings
  async function loadQueryLimit(): Promise<void> {
    if (queryLimitInitialized.value) return

    try {
      const api = getApiAdapter()
      const saved = await api.settings.get(QUERY_LIMIT_KEY)
      if (saved && typeof saved === 'number' && saved >= 10 && saved <= 1000) {
        queryLimit.value = saved
      }
    } catch (e) {
      console.error('Failed to load query limit:', e)
    } finally {
      queryLimitInitialized.value = true
    }

    // Also check localStorage for migration
    migrateFromLocalStorage()
  }

  // Migrate from localStorage (one-time migration)
  function migrateFromLocalStorage(): void {
    try {
      const stored = localStorage.getItem('mysql-query-limit')
      if (stored) {
        const parsed = parseInt(stored, 10)
        if (!isNaN(parsed) && parsed >= 10 && parsed <= 1000) {
          queryLimit.value = parsed
          // Migrate to backend and clear localStorage
          saveQueryLimit(parsed).then(() => {
            localStorage.removeItem('mysql-query-limit')
          })
        }
      }
    } catch {
      // Ignore errors
    }
  }

  // Save query limit to backend settings
  async function saveQueryLimit(limit: number): Promise<boolean> {
    if (limit < 10 || limit > 1000) {
      return false
    }
    queryLimit.value = limit
    try {
      const api = getApiAdapter()
      await api.settings.set(QUERY_LIMIT_KEY, limit)
      return true
    } catch (e) {
      console.error('Failed to save query limit:', e)
      return false
    }
  }

  // Watch for changes and auto-save
  watch(queryLimit, (newValue) => {
    if (queryLimitInitialized.value) {
      saveQueryLimit(newValue)
    }
  })

  // Check if SQL already has LIMIT clause
  function hasLimitClause(sql: string): boolean {
    const cleaned = sql
      .replace(/--.*$/gm, '')
      .replace(/\/\*[\s\S]*?\*\//g, '')
      .replace(/'.*?'/g, "''")
      .replace(/".*?"/g, '""')

    return /\bLIMIT\s+\d+/i.test(cleaned)
  }

  // Apply query limit to SQL
  function applyLimit(sql: string, limit?: number): string {
    const limitValue = limit ?? queryLimit.value

    if (hasLimitClause(sql)) {
      return sql
    }

    const trimmed = sql.trim()

    if (!/^\s*(SELECT|WITH)/i.test(trimmed)) {
      return sql
    }

    let baseSql = trimmed
    if (baseSql.endsWith(';')) {
      baseSql = baseSql.slice(0, -1).trim()
    }

    return `${baseSql} LIMIT ${limitValue}`
  }

  // Extract base query (remove LIMIT and OFFSET)
  function extractBaseQuery(sql: string): string {
    let cleaned = sql.trim()

    if (cleaned.endsWith(';')) {
      cleaned = cleaned.slice(0, -1).trim()
    }

    cleaned = cleaned.replace(
      /\s+(LIMIT\s+\d+)(?:\s+(OFFSET\s+\d+))?(?=\s*(?:;|$))/gi,
      ''
    )
    cleaned = cleaned.replace(
      /\s+OFFSET\s+\d+(?:\s+LIMIT\s+\d+)?(?=\s*(?:;|$))/gi,
      ''
    )

    return cleaned.trim()
  }

  // Initialize: load query limit from backend
  loadQueryLimit()

  async function fetchServerInfo() {
    try {
      const data = await mysqlApi.getInfo() as ServerInfo
      serverInfo.value = data
    } catch (e) {
      error.value = (e as Error).message
    }
  }

  async function fetchDatabases() {
    loading.value = true
    try {
      const data = await mysqlApi.listDatabases() as Database[]
      databases.value = data || []
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function createDatabase(req: CreateDatabaseRequest) {
    try {
      await mysqlApi.createDatabase(req)
      await fetchDatabases()
    } catch (e) {
      throw e
    }
  }

  async function alterDatabase(name: string, req: AlterDatabaseRequest) {
    try {
      await mysqlApi.alterDatabase(name, req)
    } catch (e) {
      throw e
    }
  }

  async function grantPrivileges(database: string, req: GrantPrivilegesRequest) {
    try {
      await mysqlApi.grantPrivileges(database, req)
    } catch (e) {
      throw e
    }
  }

  async function fetchUsers() {
    loading.value = true
    try {
      const data = await mysqlApi.listUsers()
      users.value = data || []
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function createUser(req: CreateUserRequest) {
    try {
      await mysqlApi.createUser(req)
      await fetchUsers()
    } catch (e) {
      throw e
    }
  }

  async function dropDatabase(name: string) {
    try {
      await mysqlApi.dropDatabase(name)
      await fetchDatabases()
    } catch (e) {
      throw e
    }
  }

  async function fetchTables(database: string, forceRefresh = false) {
    if (!forceRefresh && fetchedDatabases.value.has(database)) {
      currentDatabase.value = database
      tables.value = tablesCache.value.get(database) || []
      return
    }

    loading.value = true
    currentDatabase.value = database
    try {
      const data = await mysqlApi.listTables(database) as Table[]
      const tableList = data || []
      tables.value = tableList
      fetchedDatabases.value.add(database)
      tablesCache.value.set(database, tableList)
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  function hasFetchedTables(database: string): boolean {
    return fetchedDatabases.value.has(database)
  }

  function getTablesForDatabase(database: string): Table[] {
    return tablesCache.value.get(database) || []
  }

  async function fetchTableSchema(database: string, table: string) {
    loading.value = true
    currentTable.value = table
    try {
      const data = await mysqlApi.getTableSchema(database, table) as TableSchema
      tableSchema.value = data
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function dropTable(database: string, table: string) {
    try {
      await mysqlApi.dropTable(database, table)
      await fetchTables(database)
    } catch (e) {
      throw e
    }
  }

  function setCurrentDatabase(name: string) {
    currentDatabase.value = name
  }

  function setCurrentTable(name: string) {
    currentTable.value = name
  }

  return {
    serverInfo,
    databases,
    currentDatabase,
    tables,
    currentTable,
    tableSchema,
    loading,
    error,
    users,
    queryLimit,
    fetchServerInfo,
    fetchDatabases,
    createDatabase,
    alterDatabase,
    grantPrivileges,
    fetchUsers,
    createUser,
    dropDatabase,
    fetchTables,
    fetchTableSchema,
    dropTable,
    setCurrentDatabase,
    setCurrentTable,
    hasFetchedTables,
    getTablesForDatabase,
    loadQueryLimit,
    saveQueryLimit,
    applyLimit,
    extractBaseQuery,
  }
})
