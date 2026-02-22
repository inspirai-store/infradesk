<script setup lang="ts">
import { onMounted, computed, h, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NCard,
  NSpace,
  NButton,
  NIcon,
  NDataTable,
  NBreadcrumb,
  NBreadcrumbItem,
  useMessage,
  useDialog,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NSwitch,
  NDropdown,
} from 'naive-ui'
import { AddOutline, TrashOutline, RefreshOutline, CreateOutline, CopyOutline, WarningOutline, EllipsisHorizontal } from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import { mysqlApi } from '@/api'
import type { ColumnDefinition } from '@/api/types'
import type { DataTableColumns } from 'naive-ui'

// Internal column editor state (simpler for UI)
interface ColumnEditorState {
  name: string
  type: string
  nullable: boolean
  primary_key: boolean
  auto_increment: boolean
  default?: string
  comment?: string
}

const route = useRoute()
const router = useRouter()
const store = useMySQLStore()
const message = useMessage()
const dialog = useDialog()

const database = computed(() => route.params.database as string)

const showCreateModal = ref(false)
const newTableName = ref('')
const newTableEngine = ref('InnoDB')
const newTableColumns = ref<ColumnEditorState[]>([
  { name: 'id', type: 'INT', nullable: false, primary_key: true, auto_increment: true, comment: '' },
])

const engineOptions = [
  { label: 'InnoDB', value: 'InnoDB' },
  { label: 'MyISAM', value: 'MyISAM' },
]

// Rename modal state
const showRenameModal = ref(false)
const renameTableName = ref('')
const newTableNameForRename = ref('')

// Copy modal state
const showCopyModal = ref(false)
const copyTableName = ref('')
const copyTargetName = ref('')
const copyWithData = ref(false)

const dataTypes = [
  { label: 'INT', value: 'INT' },
  { label: 'BIGINT', value: 'BIGINT' },
  { label: 'VARCHAR(255)', value: 'VARCHAR(255)' },
  { label: 'TEXT', value: 'TEXT' },
  { label: 'DATETIME', value: 'DATETIME' },
  { label: 'TIMESTAMP', value: 'TIMESTAMP' },
  { label: 'BOOLEAN', value: 'BOOLEAN' },
  { label: 'DECIMAL(10,2)', value: 'DECIMAL(10,2)' },
  { label: 'JSON', value: 'JSON' },
]

// Table row action menu options
const getTableActions = (_tableName: string) => [
  { label: '重命名', key: 'rename', icon: () => h(NIcon, { size: 14 }, { default: () => h(CreateOutline) }) },
  { label: '复制表', key: 'copy', icon: () => h(NIcon, { size: 14 }, { default: () => h(CopyOutline) }) },
  { type: 'divider' as const, key: 'd1' },
  { label: '清空表', key: 'truncate', icon: () => h(NIcon, { size: 14 }, { default: () => h(WarningOutline) }) },
  { label: '删除表', key: 'drop', icon: () => h(NIcon, { size: 14, color: '#e88080' }, { default: () => h(TrashOutline) }) },
]

const columns: DataTableColumns<{ name: string; engine: string; row_count: number; data_size: number }> = [
  {
    title: '表名',
    key: 'name',
    render(row) {
      return h(
        'a',
        {
          style: 'color: var(--zx-accent-cyan); cursor: pointer; font-size: 12px;',
          onClick: () => router.push(`/mysql/${database.value}/${row.name}`),
        },
        row.name
      )
    },
  },
  {
    title: '引擎',
    key: 'engine',
    width: 80,
  },
  {
    title: '行数',
    key: 'row_count',
    width: 80,
    render(row) {
      return row.row_count.toLocaleString()
    },
  },
  {
    title: '大小',
    key: 'data_size',
    width: 90,
    render(row) {
      return formatBytes(row.data_size)
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 70,
    render(row) {
      return h(
        NDropdown,
        {
          trigger: 'click',
          options: getTableActions(row.name),
          onSelect: (key: string) => handleTableAction(key, row.name),
        },
        {
          default: () =>
            h(
              NButton,
              { size: 'tiny', quaternary: true },
              { icon: () => h(NIcon, { size: 14 }, { default: () => h(EllipsisHorizontal) }) }
            ),
        }
      )
    },
  },
]

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

async function handleCreate() {
  if (!newTableName.value.trim()) {
    message.warning('请输入表名')
    return
  }

  if (newTableColumns.value.length === 0) {
    message.warning('请至少添加一个字段')
    return
  }

  // Validate column names
  for (const col of newTableColumns.value) {
    if (!col.name.trim()) {
      message.warning('字段名不能为空')
      return
    }
  }

  try {
    // Convert editor state to API format
    const columns: ColumnDefinition[] = newTableColumns.value.map(col => ({
      name: col.name.trim(),
      data_type: col.type,
      nullable: col.nullable,
      auto_increment: col.auto_increment,
      default: col.default || undefined,
      comment: col.comment || undefined,
    }))

    // Extract primary key columns
    const primaryKey = newTableColumns.value
      .filter(col => col.primary_key)
      .map(col => col.name.trim())

    await mysqlApi.createTable(database.value, {
      name: newTableName.value.trim(),
      columns,
      primary_key: primaryKey.length > 0 ? primaryKey : undefined,
      engine: newTableEngine.value,
    })

    message.success(`表 "${newTableName.value}" 创建成功`)
    showCreateModal.value = false
    resetCreateForm()
    store.fetchTables(database.value)
  } catch (e) {
    message.error((e as Error).message)
  }
}

function resetCreateForm() {
  newTableName.value = ''
  newTableEngine.value = 'InnoDB'
  newTableColumns.value = [
    { name: 'id', type: 'INT', nullable: false, primary_key: true, auto_increment: true, comment: '' },
  ]
}

function handleDrop(name: string) {
  dialog.warning({
    title: '删除表',
    content: `确定要删除表 "${name}" 吗？此操作不可撤销。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await store.dropTable(database.value, name)
        message.success(`表 "${name}" 已删除`)
      } catch (e) {
        message.error((e as Error).message)
      }
    },
  })
}

function handleRefresh() {
  store.fetchTables(database.value)
  message.success('已刷新')
}

function handleTableAction(key: string, tableName: string) {
  switch (key) {
    case 'rename':
      renameTableName.value = tableName
      newTableNameForRename.value = tableName
      showRenameModal.value = true
      break
    case 'copy':
      copyTableName.value = tableName
      copyTargetName.value = `${tableName}_copy`
      copyWithData.value = false
      showCopyModal.value = true
      break
    case 'truncate':
      handleTruncate(tableName)
      break
    case 'drop':
      handleDrop(tableName)
      break
  }
}

async function handleRename() {
  if (!newTableNameForRename.value.trim()) {
    message.warning('请输入新表名')
    return
  }

  if (newTableNameForRename.value === renameTableName.value) {
    message.warning('新表名与原表名相同')
    return
  }

  try {
    await mysqlApi.renameTable(database.value, renameTableName.value, newTableNameForRename.value.trim())
    message.success(`表 "${renameTableName.value}" 已重命名为 "${newTableNameForRename.value}"`)
    showRenameModal.value = false
    store.fetchTables(database.value)
  } catch (e) {
    message.error((e as Error).message)
  }
}

function handleTruncate(tableName: string) {
  dialog.warning({
    title: '清空表',
    content: `确定要清空表 "${tableName}" 的所有数据吗？此操作不可撤销。`,
    positiveText: '清空',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await mysqlApi.truncateTable(database.value, tableName)
        message.success(`表 "${tableName}" 已清空`)
        store.fetchTables(database.value)
      } catch (e) {
        message.error((e as Error).message)
      }
    },
  })
}

async function handleCopy() {
  if (!copyTargetName.value.trim()) {
    message.warning('请输入目标表名')
    return
  }

  if (copyTargetName.value === copyTableName.value) {
    message.warning('目标表名与原表名相同')
    return
  }

  try {
    await mysqlApi.copyTable(database.value, copyTableName.value, copyTargetName.value.trim(), copyWithData.value)
    message.success(`表 "${copyTableName.value}" 已复制为 "${copyTargetName.value}"`)
    showCopyModal.value = false
    store.fetchTables(database.value)
  } catch (e) {
    message.error((e as Error).message)
  }
}

function addColumn() {
  newTableColumns.value.push({
    name: '',
    type: 'VARCHAR(255)',
    nullable: true,
    primary_key: false,
    auto_increment: false,
    default: '',
    comment: '',
  })
}

function removeColumn(index: number) {
  newTableColumns.value.splice(index, 1)
}

onMounted(() => {
  if (database.value) {
    store.fetchTables(database.value)
  }
})
</script>

<template>
  <div class="table-list">
    <!-- Breadcrumb -->
    <NBreadcrumb class="breadcrumb">
      <NBreadcrumbItem @click="router.push('/mysql')">MySQL</NBreadcrumbItem>
      <NBreadcrumbItem>{{ database }}</NBreadcrumbItem>
    </NBreadcrumb>
    
    <NCard class="glass-card">
      <template #header>
        <NSpace align="center" justify="space-between">
          <span class="title-font" style="font-size: 14px">{{ database }} 的表</span>
          <NSpace :size="6">
            <NButton size="tiny" @click="handleRefresh">
              <template #icon>
                <NIcon size="14"><RefreshOutline /></NIcon>
              </template>
              刷新
            </NButton>
            <NButton size="tiny" type="primary" @click="showCreateModal = true">
              <template #icon>
                <NIcon size="14"><AddOutline /></NIcon>
              </template>
              创建表
            </NButton>
          </NSpace>
        </NSpace>
      </template>
      
      <!-- Tables -->
      <NDataTable
        :columns="columns"
        :data="store.tables"
        :loading="store.loading"
        :bordered="false"
        size="small"
        striped
      />
    </NCard>
    
    <!-- Create Table Modal -->
    <NModal
      v-model:show="showCreateModal"
      title="创建表"
      preset="card"
      style="width: 700px"
    >
      <NForm size="small">
        <NSpace :size="12">
          <NFormItem label="表名" style="flex: 1">
            <NInput v-model:value="newTableName" placeholder="输入表名" />
          </NFormItem>
          <NFormItem label="引擎" style="width: 120px">
            <NSelect v-model:value="newTableEngine" :options="engineOptions" />
          </NFormItem>
        </NSpace>

        <NFormItem label="字段">
          <div class="columns-editor">
            <div
              v-for="(col, index) in newTableColumns"
              :key="index"
              class="column-row"
            >
              <NInput
                v-model:value="col.name"
                placeholder="字段名"
                style="width: 120px"
              />
              <NSelect
                v-model:value="col.type"
                :options="dataTypes"
                style="width: 130px"
              />
              <NSpace align="center" :size="2">
                <NSwitch v-model:value="col.nullable" size="small" />
                <span style="font-size: 11px">空</span>
              </NSpace>
              <NSpace align="center" :size="2">
                <NSwitch v-model:value="col.primary_key" size="small" />
                <span style="font-size: 11px">主键</span>
              </NSpace>
              <NSpace align="center" :size="2">
                <NSwitch v-model:value="col.auto_increment" size="small" />
                <span style="font-size: 11px">自增</span>
              </NSpace>
              <NButton
                size="tiny"
                type="error"
                quaternary
                :disabled="newTableColumns.length <= 1"
                @click="removeColumn(index)"
              >
                <template #icon>
                  <NIcon size="14"><TrashOutline /></NIcon>
                </template>
              </NButton>
            </div>
            <NButton size="tiny" dashed @click="addColumn">
              <template #icon>
                <NIcon size="14"><AddOutline /></NIcon>
              </template>
              添加字段
            </NButton>
          </div>
        </NFormItem>
      </NForm>

      <template #footer>
        <NSpace justify="end" :size="8">
          <NButton size="small" @click="showCreateModal = false; resetCreateForm()">取消</NButton>
          <NButton size="small" type="primary" @click="handleCreate">创建</NButton>
        </NSpace>
      </template>
    </NModal>

    <!-- Rename Table Modal -->
    <NModal
      v-model:show="showRenameModal"
      title="重命名表"
      preset="card"
      style="width: 400px"
    >
      <NForm size="small">
        <NFormItem label="原表名">
          <NInput :value="renameTableName" disabled />
        </NFormItem>
        <NFormItem label="新表名">
          <NInput v-model:value="newTableNameForRename" placeholder="输入新表名" />
        </NFormItem>
      </NForm>

      <template #footer>
        <NSpace justify="end" :size="8">
          <NButton size="small" @click="showRenameModal = false">取消</NButton>
          <NButton size="small" type="primary" @click="handleRename">确认</NButton>
        </NSpace>
      </template>
    </NModal>

    <!-- Copy Table Modal -->
    <NModal
      v-model:show="showCopyModal"
      title="复制表"
      preset="card"
      style="width: 400px"
    >
      <NForm size="small">
        <NFormItem label="源表">
          <NInput :value="copyTableName" disabled />
        </NFormItem>
        <NFormItem label="目标表名">
          <NInput v-model:value="copyTargetName" placeholder="输入目标表名" />
        </NFormItem>
        <NFormItem label="复制数据">
          <NSwitch v-model:value="copyWithData" />
          <span style="margin-left: 8px; font-size: 12px; color: var(--n-text-color-3)">
            {{ copyWithData ? '复制表结构和数据' : '仅复制表结构' }}
          </span>
        </NFormItem>
      </NForm>

      <template #footer>
        <NSpace justify="end" :size="8">
          <NButton size="small" @click="showCopyModal = false">取消</NButton>
          <NButton size="small" type="primary" @click="handleCopy">复制</NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>

<style scoped>
.table-list {
  padding: 16px;
}

.breadcrumb {
  margin-bottom: 12px;
}

.columns-editor {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 100%;
}

.column-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
