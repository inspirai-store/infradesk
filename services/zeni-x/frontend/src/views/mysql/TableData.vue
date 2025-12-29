<script setup lang="ts">
import { onMounted, computed, ref, watch, h } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { 
  NCard, 
  NSpace, 
  NButton, 
  NIcon, 
  NDataTable,
  NBreadcrumb,
  NBreadcrumbItem,
  NPagination,
  NTabs,
  NTabPane,
  NModal,
  NForm,
  NFormItem,
  NInput,
  useMessage,
  useDialog,
} from 'naive-ui'
import { 
  AddOutline, 
  TrashOutline, 
  RefreshOutline,
  DownloadOutline,
} from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import { mysqlApi } from '@/api'
import type { DataTableColumns } from 'naive-ui'

const route = useRoute()
const router = useRouter()
const store = useMySQLStore()
const message = useMessage()
const dialog = useDialog()

const database = computed(() => route.params.database as string)
const table = computed(() => route.params.table as string)

const rows = ref<Record<string, unknown>[]>([])
const columns = ref<string[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(50)
const loading = ref(false)

const showAddModal = ref(false)
const newRowData = ref<Record<string, string>>({})

const tableColumns = computed<DataTableColumns<Record<string, unknown>>>(() => {
  const cols: DataTableColumns<Record<string, unknown>> = columns.value.map(col => ({
    title: col,
    key: col,
    ellipsis: { tooltip: true },
    render(row) {
      const val = row[col]
      if (val === null) return 'NULL'
      if (typeof val === 'object') return JSON.stringify(val)
      return String(val)
    },
  }))
  
  cols.push({
    title: '操作',
    key: 'actions',
    width: 60,
    fixed: 'right',
    render(row) {
      return [
        h(
          NButton,
          {
            size: 'tiny',
            type: 'error',
            quaternary: true,
            onClick: () => handleDeleteRow(row),
          },
          { icon: () => h(NIcon, { size: 14 }, { default: () => h(TrashOutline) }) }
        ),
      ]
    },
  })
  
  return cols
})

async function fetchData() {
  loading.value = true
  try {
    const response = await mysqlApi.getRows(database.value, table.value, page.value, pageSize.value)
    const result = response.data
    rows.value = result.rows || []
    columns.value = result.columns || []
    total.value = result.total
  } catch (e) {
    message.error((e as Error).message)
  } finally {
    loading.value = false
  }
}

async function handleDeleteRow(row: Record<string, unknown>) {
  const pkCol = store.tableSchema?.columns.find(c => c.key === 'PRI')
  if (!pkCol) {
    message.error('无法删除：未找到主键')
    return
  }
  
  dialog.warning({
    title: '删除行',
    content: `确定要删除这一行吗？`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        const where: Record<string, unknown> = {}
        where[pkCol.name] = row[pkCol.name]
        await mysqlApi.deleteRow(database.value, table.value, where)
        message.success('删除成功')
        fetchData()
      } catch (e) {
        message.error((e as Error).message)
      }
    },
  })
}

async function handleAddRow() {
  const data: Record<string, unknown> = {}
  for (const [key, value] of Object.entries(newRowData.value)) {
    if (value !== '') {
      data[key] = value
    }
  }
  
  try {
    await mysqlApi.insertRow(database.value, table.value, data)
    message.success('插入成功')
    showAddModal.value = false
    newRowData.value = {}
    fetchData()
  } catch (e) {
    message.error((e as Error).message)
  }
}

async function handleExport() {
  try {
    const response = await mysqlApi.exportData(database.value, table.value, 'json')
    const blob = new Blob([JSON.stringify(response.data, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${table.value}.json`
    a.click()
    URL.revokeObjectURL(url)
    message.success('导出完成')
  } catch (e) {
    message.error((e as Error).message)
  }
}

function handlePageChange(newPage: number) {
  page.value = newPage
  fetchData()
}

function handleRefresh() {
  fetchData()
  message.success('已刷新')
}

function openAddModal() {
  newRowData.value = {}
  if (store.tableSchema) {
    for (const col of store.tableSchema.columns) {
      if (!col.extra.includes('auto_increment')) {
        newRowData.value[col.name] = ''
      }
    }
  }
  showAddModal.value = true
}

onMounted(() => {
  if (database.value && table.value) {
    store.fetchTableSchema(database.value, table.value)
    fetchData()
  }
})

watch([database, table], () => {
  if (database.value && table.value) {
    page.value = 1
    store.fetchTableSchema(database.value, table.value)
    fetchData()
  }
})
</script>

<template>
  <div class="table-data">
    <!-- Breadcrumb -->
    <NBreadcrumb class="breadcrumb">
      <NBreadcrumbItem @click="router.push('/mysql')">MySQL</NBreadcrumbItem>
      <NBreadcrumbItem @click="router.push(`/mysql/${database}`)">{{ database }}</NBreadcrumbItem>
      <NBreadcrumbItem>{{ table }}</NBreadcrumbItem>
    </NBreadcrumb>
    
    <NTabs type="line" animated size="small">
      <!-- Data Tab -->
      <NTabPane name="data" tab="数据">
        <NCard class="glass-card">
          <template #header>
            <NSpace align="center" justify="space-between">
              <span style="font-size: 12px">共 {{ total.toLocaleString() }} 行</span>
              <NSpace :size="4">
                <NButton size="tiny" @click="handleRefresh">
                  <template #icon>
                    <NIcon size="14"><RefreshOutline /></NIcon>
                  </template>
                </NButton>
                <NButton size="tiny" @click="handleExport">
                  <template #icon>
                    <NIcon size="14"><DownloadOutline /></NIcon>
                  </template>
                  导出
                </NButton>
                <NButton size="tiny" type="primary" @click="openAddModal">
                  <template #icon>
                    <NIcon size="14"><AddOutline /></NIcon>
                  </template>
                  新增
                </NButton>
              </NSpace>
            </NSpace>
          </template>
          
          <NDataTable
            :columns="tableColumns"
            :data="rows"
            :loading="loading"
            :bordered="false"
            :max-height="420"
            :scroll-x="columns.length * 120"
            size="small"
            striped
          />
          
          <div class="pagination">
            <NPagination
              v-model:page="page"
              :page-size="pageSize"
              :item-count="total"
              show-size-picker
              :page-sizes="[20, 50, 100]"
              size="small"
              @update:page="handlePageChange"
              @update:page-size="(size: number) => { pageSize = size; fetchData() }"
            />
          </div>
        </NCard>
      </NTabPane>
      
      <!-- Schema Tab -->
      <NTabPane name="schema" tab="结构">
        <NCard class="glass-card">
          <template #header>
            <span class="title-font" style="font-size: 13px">表结构</span>
          </template>
          
          <NDataTable
            v-if="store.tableSchema"
            :columns="[
              { title: '字段', key: 'name' },
              { title: '类型', key: 'type' },
              { 
                title: '可空', 
                key: 'nullable',
                width: 60,
                render: (row: any) => row.nullable ? '是' : '否'
              },
              { title: '键', key: 'key', width: 50 },
              { title: '默认值', key: 'default' },
              { title: '其他', key: 'extra' },
            ]"
            :data="store.tableSchema.columns"
            :bordered="false"
            size="small"
            striped
          />
        </NCard>
        
        <!-- Indexes -->
        <NCard v-if="store.tableSchema?.indexes?.length" class="glass-card" style="margin-top: 12px">
          <template #header>
            <span class="title-font" style="font-size: 13px">索引</span>
          </template>
          
          <NDataTable
            :columns="[
              { title: '名称', key: 'name' },
              { 
                title: '字段', 
                key: 'columns',
                render: (row: any) => row.columns.join(', ')
              },
              { 
                title: '唯一', 
                key: 'unique',
                width: 60,
                render: (row: any) => row.unique ? '是' : '否'
              },
              { title: '类型', key: 'type', width: 80 },
            ]"
            :data="store.tableSchema.indexes"
            :bordered="false"
            size="small"
            striped
          />
        </NCard>
      </NTabPane>
    </NTabs>
    
    <!-- Add Row Modal -->
    <NModal
      v-model:show="showAddModal"
      title="新增数据行"
      preset="card"
      style="width: 500px"
    >
      <NForm size="small">
        <NFormItem 
          v-for="(_value, key) in newRowData" 
          :key="key"
          :label="String(key)"
        >
          <NInput v-model:value="newRowData[key]" :placeholder="`输入 ${key}`" />
        </NFormItem>
      </NForm>
      
      <template #footer>
        <NSpace justify="end" :size="8">
          <NButton size="small" @click="showAddModal = false">取消</NButton>
          <NButton size="small" type="primary" @click="handleAddRow">插入</NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>

<style scoped>
.table-data {
  padding: 16px;
}

.breadcrumb {
  margin-bottom: 12px;
}

.pagination {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
</style>
