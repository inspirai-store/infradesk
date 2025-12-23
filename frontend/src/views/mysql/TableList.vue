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
} from 'naive-ui'
import { AddOutline, TrashOutline, RefreshOutline } from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import { mysqlApi, type ColumnDef } from '@/api'
import type { DataTableColumns } from 'naive-ui'

const route = useRoute()
const router = useRouter()
const store = useMySQLStore()
const message = useMessage()
const dialog = useDialog()

const database = computed(() => route.params.database as string)

const showCreateModal = ref(false)
const newTableName = ref('')
const newTableColumns = ref<ColumnDef[]>([
  { name: 'id', type: 'INT', nullable: false, primary_key: true, auto_increment: true, comment: '' },
])

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
    width: 60,
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
  
  try {
    await mysqlApi.createTable(database.value, {
      name: newTableName.value.trim(),
      columns: newTableColumns.value,
    })
    message.success(`表 "${newTableName.value}" 创建成功`)
    showCreateModal.value = false
    newTableName.value = ''
    newTableColumns.value = [
      { name: 'id', type: 'INT', nullable: false, primary_key: true, auto_increment: true, comment: '' },
    ]
    store.fetchTables(database.value)
  } catch (e) {
    message.error((e as Error).message)
  }
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

function addColumn() {
  newTableColumns.value.push({
    name: '',
    type: 'VARCHAR(255)',
    nullable: true,
    primary_key: false,
    auto_increment: false,
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
        <NFormItem label="表名">
          <NInput v-model:value="newTableName" placeholder="输入表名" />
        </NFormItem>
        
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
          <NButton size="small" @click="showCreateModal = false">取消</NButton>
          <NButton size="small" type="primary" @click="handleCreate">创建</NButton>
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
