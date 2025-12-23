<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { 
  NCard, 
  NSpace, 
  NButton, 
  NIcon, 
  NDataTable,
  NSelect,
  NText,
  useMessage,
} from 'naive-ui'
import { 
  PlayOutline, 
  TimeOutline,
} from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import { mysqlApi } from '@/api'
import type { DataTableColumns } from 'naive-ui'

const store = useMySQLStore()
const message = useMessage()

const selectedDatabase = ref('')
const queryText = ref('SELECT * FROM ')
const results = ref<Record<string, unknown>[]>([])
const resultColumns = ref<string[]>([])
const rowsAffected = ref(0)
const duration = ref(0)
const loading = ref(false)
const error = ref('')

const databaseOptions = computed(() => 
  store.databases.map(db => ({ label: db.name, value: db.name }))
)

const tableColumns = computed<DataTableColumns<Record<string, unknown>>>(() => {
  return resultColumns.value.map(col => ({
    title: col,
    key: col,
    ellipsis: { tooltip: true },
    render(row: Record<string, unknown>) {
      const val = row[col]
      if (val === null) return 'NULL'
      if (typeof val === 'object') return JSON.stringify(val)
      return String(val)
    },
  }))
})

async function executeQuery() {
  if (!queryText.value.trim()) {
    message.warning('请输入 SQL 语句')
    return
  }
  
  loading.value = true
  error.value = ''
  results.value = []
  resultColumns.value = []
  
  try {
    const response = await mysqlApi.executeQuery(selectedDatabase.value, queryText.value)
    const result = response.data
    
    if (result.columns) {
      results.value = result.rows || []
      resultColumns.value = result.columns
    }
    
    rowsAffected.value = result.rows_affected || results.value.length
    duration.value = result.duration_ms || 0
    
    message.success(`执行完成，耗时 ${duration.value}ms`)
  } catch (e) {
    error.value = (e as Error).message
    message.error(error.value)
  } finally {
    loading.value = false
  }
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault()
    executeQuery()
  }
}

onMounted(() => {
  store.fetchDatabases()
})
</script>

<template>
  <div class="query-editor">
    <NCard class="glass-card">
      <template #header>
        <NSpace align="center" justify="space-between">
          <span class="title-font neon-text" style="font-size: 14px">SQL 查询</span>
          <NSpace :size="8">
            <NSelect
              v-model:value="selectedDatabase"
              :options="databaseOptions"
              placeholder="选择数据库"
              size="small"
              style="width: 160px"
              clearable
            />
            <NButton 
              type="primary" 
              size="small"
              :loading="loading"
              @click="executeQuery"
            >
              <template #icon>
                <NIcon size="14"><PlayOutline /></NIcon>
              </template>
              执行 (Ctrl+Enter)
            </NButton>
          </NSpace>
        </NSpace>
      </template>
      
      <!-- Query Input -->
      <div class="editor-container">
        <textarea
          v-model="queryText"
          class="sql-editor code-editor"
          placeholder="在此输入 SQL 语句..."
          @keydown="handleKeydown"
        />
      </div>
      
      <!-- Results -->
      <div class="results-section">
        <NSpace align="center" class="results-header">
          <NText v-if="results.length > 0 || rowsAffected > 0" style="font-size: 12px">
            <NIcon size="12"><TimeOutline /></NIcon>
            {{ duration }}ms | 
            {{ results.length > 0 ? `${results.length} 行` : `影响 ${rowsAffected} 行` }}
          </NText>
        </NSpace>
        
        <!-- Error Display -->
        <NCard v-if="error" class="error-card">
          <NText type="error" style="font-size: 12px">{{ error }}</NText>
        </NCard>
        
        <!-- Results Table -->
        <NDataTable
          v-if="results.length > 0"
          :columns="tableColumns"
          :data="results"
          :bordered="false"
          :max-height="360"
          :scroll-x="resultColumns.length * 120"
          size="small"
          striped
          virtual-scroll
        />
        
        <!-- Empty State -->
        <div v-else-if="!error && !loading" class="empty-state">
          <NText depth="3" style="font-size: 12px">输入 SQL 语句后点击执行</NText>
        </div>
      </div>
    </NCard>
  </div>
</template>

<style scoped>
.query-editor {
  padding: 16px;
  height: 100%;
}

.editor-container {
  margin-bottom: 12px;
}

.sql-editor {
  width: 100%;
  min-height: 160px;
  padding: 12px;
  background: var(--zx-bg-tertiary);
  border: 1px solid var(--zx-border);
  border-radius: 6px;
  color: var(--zx-text-primary);
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  line-height: 1.4;
  resize: vertical;
}

.sql-editor:focus {
  outline: none;
  border-color: var(--zx-accent-cyan);
  box-shadow: 0 0 0 2px rgba(0, 255, 255, 0.2);
}

.results-section {
  border-top: 1px solid var(--zx-border);
  padding-top: 12px;
}

.results-header {
  margin-bottom: 8px;
}

.error-card {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  margin-bottom: 12px;
}

.empty-state {
  padding: 32px;
  text-align: center;
}
</style>
