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

  async function fetchServerInfo() {
    try {
      const response = await mysqlApi.getInfo()
      serverInfo.value = response.data
    } catch (e) {
      error.value = (e as Error).message
    }
  }

  async function fetchDatabases() {
    loading.value = true
    try {
      const response = await mysqlApi.listDatabases()
      databases.value = response.data || []
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
      const response = await mysqlApi.listUsers()
      users.value = response.data || []
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
      const response = await mysqlApi.listTables(database)
      const tableList = response.data || []
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

  async function fetchTableSchema(database: string, table: string) {
    loading.value = true
    currentTable.value = table
    try {
      const response = await mysqlApi.getTableSchema(database, table)
      tableSchema.value = response.data
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
  }
})

