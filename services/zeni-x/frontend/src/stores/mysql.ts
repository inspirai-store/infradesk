import { defineStore } from 'pinia'
import { ref } from 'vue'
import { mysqlApi, type CreateDatabaseRequest, type AlterDatabaseRequest, type GrantPrivilegesRequest, type CreateUserRequest, type UserInfo } from '@/api'

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
  // 记录哪些数据库已经加载过表信息
  const fetchedDatabases = ref<Set<string>>(new Set())
  // 按数据库缓存表信息
  const tablesCache = ref<Map<string, Table[]>>(new Map())

  // ========== 查询限制功能 ==========
  const QUERY_LIMIT_STORAGE_KEY = 'mysql-query-limit'
  const queryLimit = ref<number>(100) // 默认值 100

  // 加载查询限制配置
  function loadQueryLimit(): void {
    const saved = localStorage.getItem(QUERY_LIMIT_STORAGE_KEY)
    if (saved) {
      const parsed = parseInt(saved, 10)
      if (!isNaN(parsed) && parsed >= 10 && parsed <= 1000) {
        queryLimit.value = parsed
      }
    }
  }

  // 保存查询限制配置
  function saveQueryLimit(limit: number): boolean {
    if (limit < 10 || limit > 1000) {
      return false
    }
    queryLimit.value = limit
    localStorage.setItem(QUERY_LIMIT_STORAGE_KEY, limit.toString())
    return true
  }

  // 检查 SQL 是否已经有 LIMIT 子句
  function hasLimitClause(sql: string): boolean {
    // 移除字符串字面量和注释，避免误判
    const cleaned = sql
      .replace(/--.*$/gm, '') // 移除单行注释
      .replace(/\/\*[\s\S]*?\*\//g, '') // 移除多行注释
      .replace(/'.*?'/g, "''") // 简化字符串
      .replace(/".*?"/g, '""') // 简化双引号字符串

    // 检查是否有 LIMIT（不区分大小写）
    return /\bLIMIT\s+\d+/i.test(cleaned)
  }

  // 应用查询限制到 SQL
  function applyLimit(sql: string, limit?: number): string {
    const limitValue = limit ?? queryLimit.value

    // 如果已经有 LIMIT，不修改
    if (hasLimitClause(sql)) {
      return sql
    }

    const trimmed = sql.trim()

    // 只对 SELECT 语句添加 LIMIT
    if (!/^\s*(SELECT|WITH)/i.test(trimmed)) {
      return sql
    }

    // 移除末尾的分号
    let baseSql = trimmed
    if (baseSql.endsWith(';')) {
      baseSql = baseSql.slice(0, -1).trim()
    }

    return `${baseSql} LIMIT ${limitValue}`
  }

  // 提取基础查询（移除 LIMIT 和 OFFSET）
  function extractBaseQuery(sql: string): string {
    let cleaned = sql.trim()

    // 移除末尾的分号
    if (cleaned.endsWith(';')) {
      cleaned = cleaned.slice(0, -1).trim()
    }

    // 移除 LIMIT 和 OFFSET 子句
    // 正则匹配: LIMIT 数字 [OFFSET 数字] 或 OFFSET 数字 LIMIT 数字
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

  // 初始化时加载配置
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
      // 不需要刷新列表，因为只修改属性
    } catch (e) {
      throw e
    }
  }

  async function grantPrivileges(database: string, req: GrantPrivilegesRequest) {
    try {
      await mysqlApi.grantPrivileges(database, req)
      // 不需要刷新列表
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
    // 如果已经加载过且不是强制刷新，使用缓存
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
      // 缓存结果
      fetchedDatabases.value.add(database)
      tablesCache.value.set(database, tableList)
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  // 检查数据库是否已加载过表信息
  function hasFetchedTables(database: string): boolean {
    return fetchedDatabases.value.has(database)
  }

  // 获取指定数据库的缓存表列表
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

