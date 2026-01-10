import { defineStore } from 'pinia'
import { ref } from 'vue'
import { redisApi } from '@/api'

export interface RedisInfo {
  version: string
  host: string
  port: number
  connected: boolean
  used_memory: string
  total_keys: number
  connected_clients: number
}

export interface KeyInfo {
  key: string
  type: string
  ttl: number
  value?: unknown
}

export interface KeysResult {
  keys: KeyInfo[]
  cursor: number
  total: number
}

export const useRedisStore = defineStore('redis', () => {
  const serverInfo = ref<RedisInfo | null>(null)
  const keys = ref<KeyInfo[]>([])
  const currentKey = ref<KeyInfo | null>(null)
  const cursor = ref(0)
  const totalKeys = ref(0)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const searchPattern = ref('*')

  async function fetchServerInfo() {
    try {
      const data = await redisApi.getInfo() as RedisInfo
      serverInfo.value = data
    } catch (e) {
      error.value = (e as Error).message
    }
  }

  async function fetchKeys(pattern = '*', newCursor = 0, count = 100) {
    loading.value = true
    searchPattern.value = pattern
    try {
      const result = await redisApi.listKeys(pattern, newCursor, count) as KeysResult
      if (newCursor === 0) {
        keys.value = result.keys || []
      } else {
        keys.value = [...keys.value, ...(result.keys || [])]
      }
      cursor.value = result.cursor
      totalKeys.value = result.total
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function fetchKey(key: string) {
    loading.value = true
    try {
      const data = await redisApi.getKey(key) as KeyInfo
      currentKey.value = data
    } catch (e) {
      error.value = (e as Error).message
      currentKey.value = null
    } finally {
      loading.value = false
    }
  }

  async function setKey(key: string, type: string, value: unknown, ttl?: number) {
    try {
      await redisApi.setKey({ key, type: type as 'string', value, ttl })
      await fetchKeys(searchPattern.value)
    } catch (e) {
      throw e
    }
  }

  async function deleteKey(key: string) {
    try {
      await redisApi.deleteKey(key)
      await fetchKeys(searchPattern.value)
    } catch (e) {
      throw e
    }
  }

  async function setTTL(key: string, ttl: number) {
    try {
      await redisApi.setTTL(key, ttl)
      if (currentKey.value?.key === key) {
        await fetchKey(key)
      }
    } catch (e) {
      throw e
    }
  }

  function setCurrentKey(keyInfo: KeyInfo | null) {
    currentKey.value = keyInfo
  }

  return {
    serverInfo,
    keys,
    currentKey,
    cursor,
    totalKeys,
    loading,
    error,
    searchPattern,
    fetchServerInfo,
    fetchKeys,
    fetchKey,
    setKey,
    deleteKey,
    setTTL,
    setCurrentKey,
  }
})

