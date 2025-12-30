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
  NTooltip,
  NBadge,
  useMessage,
} from 'naive-ui'
import {
  PlayOutline,
  TimeOutline,
  RefreshOutline,
  ListOutline,
  SparklesOutline,
} from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import { useHistoryStore } from '@/stores/history'
import { mysqlApi, getActiveConnectionId } from '@/api'
import type { DataTableColumns } from 'naive-ui'
import MonacoSQLEditor from './components/MonacoSQLEditor.vue'
import QueryHistoryPanel from './components/QueryHistoryPanel.vue'
import AIAssistantPanel from './components/AIAssistantPanel.vue'

const store = useMySQLStore()
const historyStore = useHistoryStore()
const message = useMessage()

const selectedDatabase = ref('')
const queryText = ref('SELECT * FROM ')
const results = ref<Record<string, unknown>[]>([])
const resultColumns = ref<string[]>([])
const rowsAffected = ref(0)
const duration = ref(0)
const loading = ref(false)
const error = ref('')

// 历史面板
const showHistoryPanel = ref(false)
// AI 面板
const showAIPanel = ref(false)

// 编辑器引用
const editorRef = ref<InstanceType<typeof MonacoSQLEditor>>()

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

// 获取查询类型
function getQueryType(sql: string): string {
  const trimmed = sql.trim().toUpperCase()
  if (trimmed.startsWith('SELECT')) return 'SELECT'
  if (trimmed.startsWith('INSERT')) return 'INSERT'
  if (trimmed.startsWith('UPDATE')) return 'UPDATE'
  if (trimmed.startsWith('DELETE')) return 'DELETE'
  if (trimmed.startsWith('CREATE') || trimmed.startsWith('ALTER') || trimmed.startsWith('DROP')) return 'DDL'
  return 'OTHER'
}

async function executeQuery() {
  if (!queryText.value.trim()) {
    message.warning('请输入 SQL 语句')
    return
  }

  const startTime = Date.now()
  loading.value = true
  error.value = ''
  results.value = []
  resultColumns.value = []

  const connectionId = getActiveConnectionId('mysql')

  try {
    const response = await mysqlApi.executeQuery(selectedDatabase.value, queryText.value)
    const result = response.data

    if (result.columns) {
      results.value = result.rows || []
      resultColumns.value = result.columns
    }

    rowsAffected.value = result.rows_affected || results.value.length
    duration.value = result.duration_ms || (Date.now() - startTime)

    message.success(`执行完成，耗时 ${duration.value}ms`)

    // 保存历史记录
    if (connectionId) {
      await historyStore.addHistory({
        connection_id: connectionId,
        database: selectedDatabase.value,
        query_type: getQueryType(queryText.value),
        query_text: queryText.value,
        duration_ms: duration.value,
        row_count: results.value.length,
        status: 'success'
      })
    }
  } catch (e) {
    error.value = (e as Error).message
    const errorDuration = Date.now() - startTime

    message.error(error.value)

    // 保存失败的历史记录
    if (connectionId) {
      await historyStore.addHistory({
        connection_id: connectionId,
        database: selectedDatabase.value,
        query_type: getQueryType(queryText.value),
        query_text: queryText.value,
        duration_ms: errorDuration,
        row_count: 0,
        status: 'error',
        error_message: error.value
      })
    }
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  store.fetchDatabases()
})

// 格式化 SQL
function formatSQL() {
  if (editorRef.value) {
    editorRef.value.format()
    message.success('SQL 已格式化')
  }
}

// 从历史加载查询
function loadQueryFromHistory(query: string) {
  queryText.value = query
}

// 应用 AI 生成的 SQL
function applyAISQL(sql: string) {
  queryText.value = sql
  showAIPanel.value = false
}
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
            <NTooltip placement="bottom">
              <template #trigger>
                <NButton size="small" @click="showHistoryPanel = true">
                  <template #icon>
                    <NIcon size="14"><ListOutline /></NIcon>
                  </template>
                  <NBadge :value="historyStore.historyTotal" :max="99" />
                </NButton>
              </template>
              查询历史 & 收藏
            </NTooltip>
            <NTooltip placement="bottom">
              <template #trigger>
                <NButton size="small" @click="showAIPanel = true">
                  <template #icon>
                    <NIcon size="14"><SparklesOutline /></NIcon>
                  </template>
                  AI 助手
                </NButton>
              </template>
              AI 辅助生成/优化/诊断
            </NTooltip>
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
            <NTooltip placement="bottom">
              <template #trigger>
                <NButton size="small" @click="formatSQL">
                  <template #icon>
                    <NIcon size="14"><RefreshOutline /></NIcon>
                  </template>
                  格式化
                </NButton>
              </template>
              格式化 SQL (Ctrl+Shift+F)
            </NTooltip>
          </NSpace>
        </NSpace>
      </template>

      <!-- Query Input -->
      <div class="editor-container">
        <MonacoSQLEditor
          ref="editorRef"
          v-model="queryText"
          :database="selectedDatabase"
          @execute="executeQuery"
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

    <!-- History Panel -->
    <QueryHistoryPanel
      v-model:show="showHistoryPanel"
      :on-load-query="loadQueryFromHistory"
    />

    <!-- AI Assistant Panel -->
    <AIAssistantPanel
      v-model:show="showAIPanel"
      :database="selectedDatabase"
      @update:show="showAIPanel = $event"
      @applySQL="applyAISQL"
    />
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
