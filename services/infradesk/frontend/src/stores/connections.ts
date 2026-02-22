import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { connectionApi, setActiveConnectionId, type Connection } from '@/api'
import { getApiAdapter } from '@/api/adapter'

// Settings key for active connections
const ACTIVE_CONNECTIONS_KEY = 'active_connections'

export const useConnectionsStore = defineStore('connections', () => {
  // State
  const connections = ref<Connection[]>([])
  const activeConnections = ref<Record<string, number | null>>({
    mysql: null,
    redis: null,
    mongodb: null,
    minio: null,
  })
  const loading = ref(false)
  const error = ref<string | null>(null)
  const initialized = ref(false)
  // Track if active connections were loaded from backend (to prevent auto-select override)
  const activeConnectionsLoadedFromBackend = ref(false)

  // Getters
  const mysqlConnections = computed(() =>
    connections.value.filter(c => c.type === 'mysql')
  )

  const redisConnections = computed(() =>
    connections.value.filter(c => c.type === 'redis')
  )

  const mongodbConnections = computed(() =>
    connections.value.filter(c => c.type === 'mongodb')
  )

  const minioConnections = computed(() =>
    connections.value.filter(c => c.type === 'minio')
  )

  const getConnectionsByType = (type: string) =>
    connections.value.filter(c => c.type === type)

  const getActiveConnection = (type: string): Connection | null => {
    const id = activeConnections.value[type]
    if (!id) return null
    return connections.value.find(c => c.id === id) ?? null
  }

  const hasActiveConnection = (type: string): boolean => {
    return activeConnections.value[type] !== null
  }

  // Actions
  async function fetchConnections() {
    loading.value = true
    error.value = null
    try {
      const data = await connectionApi.getAll()
      // Ensure we always have an array
      connections.value = Array.isArray(data) ? data : []

      // Only auto-select if we didn't load settings from backend
      // This prevents overriding user's saved preferences
      if (!activeConnectionsLoadedFromBackend.value) {
        for (const type of ['mysql', 'redis', 'mongodb', 'minio']) {
          if (!activeConnections.value[type]) {
            const typeConnections = getConnectionsByType(type)
            const defaultConn = typeConnections.find(c => c.is_default)
            if (defaultConn && defaultConn.id) {
              setActiveConnection(type, defaultConn.id)
            } else if (typeConnections.length > 0 && typeConnections[0].id) {
              setActiveConnection(type, typeConnections[0].id)
            }
          }
        }
      } else {
        // Validate that saved active connections still exist
        for (const type of ['mysql', 'redis', 'mongodb', 'minio']) {
          const savedId = activeConnections.value[type]
          if (savedId !== null) {
            const exists = connections.value.some(c => c.id === savedId && c.type === type)
            if (!exists) {
              // Saved connection no longer exists, clear it and auto-select
              const typeConnections = getConnectionsByType(type)
              const defaultConn = typeConnections.find(c => c.is_default)
              if (defaultConn && defaultConn.id) {
                setActiveConnection(type, defaultConn.id)
              } else if (typeConnections.length > 0 && typeConnections[0].id) {
                setActiveConnection(type, typeConnections[0].id)
              } else {
                setActiveConnection(type, null)
              }
            }
          }
        }
      }
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function fetchConnectionsByType(type: string) {
    loading.value = true
    error.value = null
    try {
      const data = await connectionApi.getByType(type)
      // Ensure data is an array
      const typeConnections = Array.isArray(data) ? data : []
      // Update only connections of this type
      const otherConnections = connections.value.filter(c => c.type !== type)
      connections.value = [...otherConnections, ...typeConnections]
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function createConnection(data: Connection): Promise<Connection | null> {
    try {
      const newConn = await connectionApi.create(data)
      connections.value.push(newConn)

      // Auto-select if it's the first connection of this type
      if (!activeConnections.value[data.type] && newConn.id) {
        setActiveConnection(data.type, newConn.id)
      }

      return newConn
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  async function updateConnection(id: number, data: Connection) {
    try {
      const updatedConn = await connectionApi.update(id, data)
      const index = connections.value.findIndex(c => c.id === id)
      if (index !== -1) {
        connections.value[index] = updatedConn
      }
      return updatedConn
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  async function deleteConnection(id: number) {
    try {
      await connectionApi.delete(id)
      const conn = connections.value.find(c => c.id === id)
      connections.value = connections.value.filter(c => c.id !== id)

      // Clear active if deleted
      if (conn && activeConnections.value[conn.type] === id) {
        const remaining = getConnectionsByType(conn.type)
        if (remaining.length > 0 && remaining[0].id) {
          setActiveConnection(conn.type, remaining[0].id)
        } else {
          setActiveConnection(conn.type, null)
        }
      }
    } catch (e) {
      error.value = (e as Error).message
      throw e
    }
  }

  async function testConnection(data: Connection): Promise<{ success: boolean; error?: string }> {
    try {
      const result = await connectionApi.test(data)
      return result
    } catch (e) {
      return { success: false, error: (e as Error).message }
    }
  }

  function setActiveConnection(type: string, id: number | null) {
    activeConnections.value[type] = id
    // Sync with API module
    setActiveConnectionId(type, id)
  }

  function getActiveConnectionId(type: string): number | null {
    return activeConnections.value[type]
  }

  // Load active connections from backend settings
  async function loadActiveConnections(): Promise<void> {
    if (initialized.value) return

    try {
      const api = getApiAdapter()
      const saved = await api.settings.get(ACTIVE_CONNECTIONS_KEY)
      if (saved && typeof saved === 'object') {
        const parsed = saved as Record<string, number | null>
        let hasAnyValue = false
        for (const [type, id] of Object.entries(parsed)) {
          if (typeof id === 'number') {
            activeConnections.value[type] = id
            setActiveConnectionId(type, id)
            hasAnyValue = true
          }
        }
        // Mark that we loaded settings from backend
        if (hasAnyValue) {
          activeConnectionsLoadedFromBackend.value = true
        }
      }
    } catch (e) {
      console.error('Failed to load active connections:', e)
    } finally {
      initialized.value = true
    }
  }

  // Save active connections to backend settings
  async function saveActiveConnections(): Promise<void> {
    try {
      const api = getApiAdapter()
      await api.settings.set(ACTIVE_CONNECTIONS_KEY, activeConnections.value)
    } catch (e) {
      console.error('Failed to save active connections:', e)
    }
  }

  // Watch for changes and auto-save
  watch(activeConnections, () => {
    if (initialized.value) {
      saveActiveConnections()
    }
  }, { deep: true })

  // Legacy compatibility methods (deprecated, use loadActiveConnections instead)
  function initFromStorage() {
    // For backward compatibility, try localStorage first, then migrate to backend
    try {
      const stored = localStorage.getItem('zeni-x-active-connections')
      if (stored) {
        const parsed = JSON.parse(stored)
        for (const [type, id] of Object.entries(parsed)) {
          if (typeof id === 'number') {
            activeConnections.value[type] = id
            setActiveConnectionId(type, id)
          }
        }
        // Migrate to backend and clear localStorage
        saveActiveConnections().then(() => {
          localStorage.removeItem('zeni-x-active-connections')
        })
      }
    } catch {
      // Ignore parse errors
    }
  }

  // Legacy compatibility (deprecated)
  function saveToStorage() {
    saveActiveConnections()
  }

  // Initialize store: load settings from backend, then fetch connections
  // This ensures proper order: settings first, then connections
  async function initialize(): Promise<void> {
    if (initialized.value) return

    // 1. First load active connections from backend settings
    await loadActiveConnections()

    // 2. Check localStorage for migration (legacy support)
    initFromStorage()

    // 3. Then fetch connections list
    await fetchConnections()
  }

  return {
    // State
    connections,
    activeConnections,
    loading,
    error,
    initialized,

    // Getters
    mysqlConnections,
    redisConnections,
    mongodbConnections,
    minioConnections,
    getConnectionsByType,
    getActiveConnection,
    hasActiveConnection,
    getActiveConnectionId,

    // Actions
    fetchConnections,
    fetchConnectionsByType,
    createConnection,
    updateConnection,
    deleteConnection,
    testConnection,
    setActiveConnection,
    loadActiveConnections,
    saveActiveConnections,

    // Initialization
    initialize,

    // Legacy compatibility (deprecated)
    initFromStorage,
    saveToStorage,
  }
})
