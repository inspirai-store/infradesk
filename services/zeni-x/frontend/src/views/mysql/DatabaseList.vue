<script setup lang="ts">
import {
  NCard,
  NSpace,
  NButton,
  NIcon,
  NDataTable,
  NInput,
  NText,
  useMessage,
  useDialog,
  NAlert,
} from 'naive-ui'
import {
  AddOutline,
  TrashOutline,
  RefreshOutline,
  PlayOutline,
} from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import { useHistoryStore } from '@/stores/history'
import { mysqlApi, getActiveConnectionId } from '@/api'
import { useRouter } from 'vue-router'
import { ref, h, computed } from 'vue'
import type { DataTableColumns } from 'naive-ui'
import TableDataEditor from './components/TableDataEditor.vue'

const store = useMySQLStore()
const historyStore = useHistoryStore()
const router = useRouter()
const message = useMessage()
const dialog = useDialog()

const newDbName = ref('')
const quickQuery = ref('')
const queryResults = ref<Record<string, unknown>[]>([])
const queryColumns = ref<string[]>([])
const queryLoading = ref(false)
const queryError = ref('')
const showQueryResults = ref(false)
const actualExecutedSQL = ref('')

// 查询类型
function getQueryType(sql: string): string {
  const trimmed = sql.trim().toUpperCase()
  if (trimmed.startsWith('SELECT')) return 'SELECT'
  if (trimmed.startsWith('INSERT')) return 'INSERT'
  if (trimmed.startsWith('UPDATE')) return 'UPDATE'
  if (trimmed.startsWith('DELETE')) return 'DELETE'
  if (trimmed.startsWith('CREATE') || trimmed.startsWith('ALTER') || trimmed.startsWith('DROP')) return 'DDL'
  return 'OTHER'
}

// 执行快速查询
async function executeQuickQuery() {
  if (!quickQuery.value.trim()) {
    message.warning('请输入 SQL 语句')
    return
  }

  queryLoading.value = true
  queryError.value = ''

  const connectionId = getActiveConnectionId('mysql')
  const startTime = Date.now()

  try {
    // 应用查询限制
    const limitedSQL = store.applyLimit(quickQuery.value)
    actualExecutedSQL.value = limitedSQL

    const response = await mysqlApi.executeQuery('', limitedSQL)
    const result = response.data

    if (result.columns) {
      queryResults.value = result.rows || []
      queryColumns.value = result.columns
      showQueryResults.value = true
    }

    const duration = result.duration_ms || (Date.now() - startTime)
    message.success(`执行完成，耗时 ${duration}ms`)

    // 保存历史记录
    if (connectionId) {
      await historyStore.addHistory({
        connection_id: connectionId,
        database: '',
        query_type: getQueryType(quickQuery.value),
        query_text: quickQuery.value,
        duration_ms: duration,
        row_count: queryResults.value.length,
        status: 'success'
      })
    }
  } catch (e) {
    queryError.value = (e as Error).message
    message.error(queryError.value)

    // 保存失败历史
    if (connectionId) {
      const errorDuration = Date.now() - startTime
      await historyStore.addHistory({
        connection_id: connectionId,
        database: '',
        query_type: getQueryType(quickQuery.value),
        query_text: quickQuery.value,
        duration_ms: errorDuration,
        row_count: 0,
        status: 'error',
        error_message: queryError.value
      })
    }
  } finally {
    queryLoading.value = false
  }
}

// 刷新查询
async function refreshQuery() {
  if (actualExecutedSQL.value) {
    await executeQuickQuery()
  }
}

// 清空查询结果
function clearQuery() {
  quickQuery.value = ''
  queryResults.value = []
  queryColumns.value = []
  showQueryResults.value = false
  queryError.value = ''
  actualExecutedSQL.value = ''
}

// 是否有查询结果
const hasQueryResults = computed(() => queryResults.value.length > 0)

const columns: DataTableColumns<{ name: string }> = [
  {
    title: '数据库名称',
    key: 'name',
    render(row) {
      return h(
        'a',
        {
          style: 'color: var(--zx-accent-cyan); cursor: pointer; font-size: 12px;',
          onClick: () => router.push(`/mysql/${row.name}`),
        },
        row.name
      )
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 80,
    render(row) {
      return h(
        NButton,
        {
          size: 'tiny',
          type: 'error',
          quaternary: true,
          onClick: () => handleDrop(row.name),
        },
        { icon: () => h(NIcon, { size: 14 }, { default: () => h(TrashOutline) }) }
      )
    },
  },
]

async function handleCreate() {
  if (!newDbName.value.trim()) {
    message.warning('请输入数据库名称')
    return
  }
  
  try {
    await store.createDatabase({ name: newDbName.value.trim() })
    message.success(`数据库 "${newDbName.value}" 创建成功`)
    newDbName.value = ''
  } catch (e) {
    message.error((e as Error).message)
  }
}

function handleDrop(name: string) {
  dialog.warning({
    title: '删除数据库',
    content: `确定要删除数据库 "${name}" 吗？此操作不可撤销。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await store.dropDatabase(name)
        message.success(`数据库 "${name}" 已删除`)
      } catch (e) {
        message.error((e as Error).message)
      }
    },
  })
}

function handleRefresh() {
  store.fetchDatabases()
  message.success('已刷新')
}
</script>

<template>
  <div class="database-list">
    <!-- 快速查询卡片 -->
    <NCard class="glass-card query-card">
      <template #header>
        <NSpace align="center">
          <NIcon size="16" color="#00FFFF"><PlayOutline /></NIcon>
          <span class="title-font neon-text" style="font-size: 14px">快速查询 & 编辑</span>
        </NSpace>
      </template>

      <NSpace vertical :size="12">
        <!-- 查询输入 -->
        <NSpace :size="8">
          <NInput
            v-model:value="quickQuery"
            placeholder="输入 SQL 语句 (例如: SELECT * FROM users LIMIT 10)"
            size="small"
            style="flex: 1; min-width: 400px;"
            @keyup.enter.ctrl="executeQuickQuery"
          />
          <NButton
            type="primary"
            size="small"
            :loading="queryLoading"
            @click="executeQuickQuery"
          >
            <template #icon>
              <NIcon size="14"><PlayOutline /></NIcon>
            </template>
            执行
          </NButton>
          <NButton
            v-if="hasQueryResults"
            size="small"
            @click="clearQuery"
          >
            清空
          </NButton>
        </NSpace>

        <!-- 查询限制提示 -->
        <NText depth="3" style="font-size: 11px">
          💡 提示: Ctrl+Enter 快速执行 | 点击单元格可直接编辑 | 当前查询限制: {{ store.queryLimit }} 行
        </NText>

        <!-- 错误显示 -->
        <NAlert v-if="queryError" type="error" :bordered="false">
          {{ queryError }}
        </NAlert>

        <!-- 查询结果 - 可编辑表格 -->
        <TableDataEditor
          v-if="hasQueryResults"
          database=""
          :sql="actualExecutedSQL"
          :columns="queryColumns"
          :data="queryResults"
          :loading="queryLoading"
          @refresh="refreshQuery"
        />

        <!-- 空状态提示 -->
        <div v-else-if="!queryError && !queryLoading" class="query-hint">
          <NText depth="3" style="font-size: 12px">
            输入 SQL 查询语句后点击执行，结果将以可编辑表格显示
          </NText>
        </div>
      </NSpace>
    </NCard>

    <!-- 数据库列表卡片 -->
    <NCard class="glass-card" style="margin-top: 12px">
      <template #header>
        <NSpace align="center" justify="space-between">
          <span class="title-font" style="font-size: 14px">数据库列表</span>
          <NButton size="tiny" @click="handleRefresh">
            <template #icon>
              <NIcon size="14"><RefreshOutline /></NIcon>
            </template>
            刷新
          </NButton>
        </NSpace>
      </template>

      <!-- Create Database -->
      <NSpace class="create-section" align="center" :size="8">
        <NInput
          v-model:value="newDbName"
          placeholder="新数据库名称"
          size="small"
          @keyup.enter="handleCreate"
        />
        <NButton type="primary" size="small" @click="handleCreate">
          <template #icon>
            <NIcon size="14"><AddOutline /></NIcon>
          </template>
          创建
        </NButton>
      </NSpace>

      <!-- Database Table -->
      <NDataTable
        :columns="columns"
        :data="store.databases"
        :loading="store.loading"
        :bordered="false"
        size="small"
        striped
      />
    </NCard>
  </div>
</template>

<style scoped>
.database-list {
  padding: 16px;
}

.query-card {
  margin-bottom: 12px;
}

.query-hint {
  padding: 32px 16px;
  text-align: center;
  border: 1px dashed var(--zx-border);
  border-radius: 8px;
  background: var(--zx-bg-secondary);
}

.create-section {
  margin-bottom: 12px;
}

.create-section :deep(.n-input) {
  width: 240px;
}
</style>
