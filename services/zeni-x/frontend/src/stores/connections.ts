import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { connectionApi, setActiveConnectionId, type Connection } from '@/api'

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
      connections.value = data || []
      
      // Auto-select default connections or first available for each type
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
      // Update only connections of this type
      const otherConnections = connections.value.filter(c => c.type !== type)
      connections.value = [...otherConnections, ...(data || [])]
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

  // Initialize active connections from localStorage
  function initFromStorage() {
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
      }
    } catch {
      // Ignore parse errors
    }
  }

  // Save active connections to localStorage
  function saveToStorage() {
    try {
      localStorage.setItem('zeni-x-active-connections', JSON.stringify(activeConnections.value))
    } catch {
      // Ignore storage errors
    }
  }

  return {
    // State
    connections,
    activeConnections,
    loading,
    error,
    
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
    initFromStorage,
    saveToStorage,
  }
})

