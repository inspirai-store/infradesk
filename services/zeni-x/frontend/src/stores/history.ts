import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { historyApi, savedQueryApi, type QueryHistory, type SavedQuery, type AddQueryHistoryRequest } from '@/api'

export const useHistoryStore = defineStore('history', () => {
  // 状态
  const history = ref<QueryHistory[]>([])
  const historyTotal = ref(0)
  const savedQueries = ref<SavedQuery[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 过滤条件
  const filters = ref({
    type: '',
    database: '',
    status: '',
    keyword: '',
    limit: 100,
    offset: 0
  })

  // 选中的收藏分类
  const selectedCategory = ref<string | undefined>(undefined)

  // 计算属性：分类列表
  const categories = computed(() => {
    const categorySet = new Set<string>()
    savedQueries.value.forEach(q => {
      if (q.category) {
        categorySet.add(q.category)
      }
    })
    return Array.from(categorySet).sort()
  })

  /**
   * 获取查询历史记录
   */
  async function fetchHistory(refresh = false) {
    if (refresh) {
      filters.value.offset = 0
    }
    loading.value = true
    error.value = null
    try {
      const data = await historyApi.getHistory(filters.value)
      history.value = data.history || []
      historyTotal.value = data.total || 0
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  /**
   * 添加查询历史记录
   */
  async function addHistory(data: AddQueryHistoryRequest) {
    try {
      await historyApi.addHistory(data)
      // 添加到本地列表
      history.value.unshift({
        id: 0, // 临时 ID
        connection_id: data.connection_id,
        database: data.database,
        query_type: data.query_type,
        query_text: data.query_text,
        executed_at: new Date().toISOString(),
        duration_ms: data.duration_ms,
        row_count: data.row_count,
        status: data.status,
        error_message: data.error_message
      })
      historyTotal.value++
    } catch (e) {
      console.error('Failed to add history:', e)
    }
  }

  /**
   * 删除查询历史记录
   */
  async function deleteHistoryItem(id: number) {
    try {
      await historyApi.deleteHistory(id)
      history.value = history.value.filter(h => h.id !== id)
      historyTotal.value--
    } catch (e) {
      throw e
    }
  }

  /**
   * 清理旧的历史记录
   */
  async function cleanupOldHistory(days: number) {
    try {
      const data = await historyApi.cleanupHistory(days)
      await fetchHistory(true)
      return data.deleted
    } catch (e) {
      throw e
    }
  }

  /**
   * 获取收藏的查询
   */
  async function fetchSavedQueries() {
    loading.value = true
    error.value = null
    try {
      const data = await savedQueryApi.getSavedQueries(selectedCategory.value)
      savedQueries.value = data || []
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  /**
   * 创建收藏的查询
   */
  async function createSavedQuery(data: {
    connection_id: number
    database: string
    name: string
    query_text: string
    description?: string
    category?: string
  }) {
    try {
      const result = await savedQueryApi.createSavedQuery(data)
      savedQueries.value.push(result)
      return result
    } catch (e) {
      throw e
    }
  }

  /**
   * 更新收藏的查询
   */
  async function updateSavedQuery(id: number, data: {
    name?: string
    query_text?: string
    description?: string
    category?: string
  }) {
    try {
      const result = await savedQueryApi.updateSavedQuery(id, data)
      const index = savedQueries.value.findIndex(q => q.id === id)
      if (index !== -1) {
        savedQueries.value[index] = result
      }
    } catch (e) {
      throw e
    }
  }

  /**
   * 删除收藏的查询
   */
  async function deleteSavedQueryItem(id: number) {
    try {
      await savedQueryApi.deleteSavedQuery(id)
      savedQueries.value = savedQueries.value.filter(q => q.id !== id)
    } catch (e) {
      throw e
    }
  }

  /**
   * 设置过滤条件
   */
  function setFilters(newFilters: Partial<typeof filters.value>) {
    Object.assign(filters.value, newFilters)
  }

  /**
   * 重置过滤条件
   */
  function resetFilters() {
    filters.value = {
      type: '',
      database: '',
      status: '',
      keyword: '',
      limit: 100,
      offset: 0
    }
  }

  /**
   * 设置选中的分类
   */
  function setCategory(category: string | undefined) {
    selectedCategory.value = category
  }

  return {
    // 状态
    history,
    historyTotal,
    savedQueries,
    loading,
    error,
    filters,
    selectedCategory,
    categories,

    // 方法
    fetchHistory,
    addHistory,
    deleteHistoryItem,
    cleanupOldHistory,
    fetchSavedQueries,
    createSavedQuery,
    updateSavedQuery,
    deleteSavedQueryItem,
    setFilters,
    resetFilters,
    setCategory,
  }
})
