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
  CheckmarkCircleOutline,
  CloseCircleOutline,
  SaveOutline,
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

// 编辑状态
const editingCell = ref<{ rowIndex: number; column: string } | null>(null)
const editValue = ref<unknown>('')
const isSaving = ref(false)

// 修改追踪
interface CellModification {
  rowIndex: number
  column: string
  oldValue: unknown
  newValue: unknown
}
const modifications = ref<Map<string, CellModification>>(new Map())
const modifiedCount = computed(() => modifications.value.size)
const hasModifications = computed(() => modifiedCount.value > 0)

const showAddModal = ref(false)
const newRowData = ref<Record<string, string>>({})

// 获取单元格修改状态
function getCellModification(rowIndex: number, column: string): CellModification | null {
  const key = `${rowIndex}-${column}`
  return modifications.value.get(key) || null
}

// 检查单元格是否被修改
function isCellModified(rowIndex: number, column: string): boolean {
  return getCellModification(rowIndex, column) !== null
}

// 开始编辑单元格
function startEdit(rowIndex: number, column: string) {
  if (editingCell.value) return // 已经在一个单元格编辑中
  editingCell.value = { rowIndex, column }
  editValue.value = rows.value[rowIndex][column]
}

// 保存单元格编辑
function saveEdit() {
  if (!editingCell.value) return
  const { rowIndex, column } = editingCell.value
  const oldValue = rows.value[rowIndex][column]
  const newValue = editValue.value
  
  if (newValue === oldValue) {
    editingCell.value = null
    return
  }
  
  const key = `${rowIndex}-${column}`
  modifications.value.set(key, { rowIndex, column, oldValue, newValue })
  editingCell.value = null
}

// 取消编辑
function cancelEdit() {
  editingCell.value = null
}

// 恢复单个单元格
function revertCell(rowIndex: number, column: string) {
  const key = `${rowIndex}-${column}`
  modifications.value.delete(key)
}

// 放弃所有修改
function discardAllChanges() {
  modifications.value.clear()
  message.info('已放弃所有修改')
}

// 保存所有修改
async function handleSaveAllChanges() {
  if (modifications.value.size === 0) {
    message.warning('没有需要保存的修改')
    return
  }

  const pkCol = store.tableSchema?.columns.find(c => c.key === 'PRI')
  if (!pkCol) {
    message.error('无法保存：未找到主键')
    return
  }

  dialog.info({
    title: '保存修改',
    content: `确定要保存 ${modifications.value.size} 处修改吗？`,
    positiveText: '确认',
    negativeText: '取消',
    onPositiveClick: async () => {
      isSaving.value = true
      try {
        // 按记录分组修改
        const updatesByRow = new Map<number, Record<string, unknown>>()
        modifications.value.forEach((mod) => {
          if (!updatesByRow.has(mod.rowIndex)) {
            updatesByRow.set(mod.rowIndex, {})
          }
          updatesByRow.get(mod.rowIndex)![mod.column] = mod.newValue
        })

        let successCount = 0
        let failCount = 0

        for (const [rowIndex, updates] of updatesByRow.entries()) {
          const rowData = rows.value[rowIndex]
          const pkValue = rowData[pkCol.name]

          try {
            await mysqlApi.updateRecord(
              database.value,
              table.value,
              pkCol.name,
              pkValue,
              updates
            )
            successCount++
          } catch (e) {
            console.error(`Failed to update row ${rowIndex}:`, e)
            failCount++
          }
        }

        if (failCount > 0) {
          message.warning(`保存完成：成功 ${successCount} 条，失败 ${failCount} 条`)
        } else {
          message.success(`成功保存 ${successCount} 条修改`)
        }

        modifications.value.clear()
        fetchData()
      } catch (e) {
        message.error((e as Error).message)
      } finally {
        isSaving.value = false
      }
    }
  })
}

const tableColumns = computed<DataTableColumns<Record<string, unknown>>>(() => {
  const cols: DataTableColumns<Record<string, unknown>> = columns.value.map(col => ({
    title: col,
    key: col,
    ellipsis: { tooltip: true },
    render(row, index) {
      const isEditing = editingCell.value?.rowIndex === index && editingCell.value?.column === col
      const isModified = isCellModified(index, col)
      const val = row[col]

      if (isEditing) {
        return h(NInput, {
          size: 'tiny',
          value: String(editValue.value ?? ''),
          onUpdateValue: (v: string) => { editValue.value = v },
          onBlur: saveEdit,
          onKeyup: (e: KeyboardEvent) => {
            if (e.key === 'Enter') saveEdit()
            if (e.key === 'Escape') cancelEdit()
          },
          autofocus: true
        })
      }

      if (isModified) {
        const mod = getCellModification(index, col)!
        return h('div', { 
          style: { display: 'flex', alignItems: 'center', gap: '4px', cursor: 'pointer' },
          onClick: () => startEdit(index, col)
        }, [
          h('span', { style: { textDecoration: 'line-through', opacity: 0.5, fontSize: '11px' } }, String(mod.oldValue ?? 'NULL')),
          h('span', { style: { color: '#f0a020', fontWeight: 'bold' } }, String(mod.newValue ?? 'NULL')),
          h(NButton, {
            size: 'tiny',
            quaternary: true,
            circle: true,
            onClick: (e: Event) => {
              e.stopPropagation()
              revertCell(index, col)
            }
          }, { icon: () => h(NIcon, { size: 12 }, { default: () => h(CloseCircleOutline) }) })
        ])
      }

      return h('span', {
        onClick: () => startEdit(index, col),
        style: { cursor: 'pointer', display: 'block', width: '100%', minHeight: '1.5em' }
      }, val === null ? 'NULL' : (typeof val === 'object' ? JSON.stringify(val) : String(val)))
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
  if (hasModifications.value) {
    dialog.warning({
      title: '未保存的修改',
      content: '当前页面有未保存的修改，切换页面将丢失这些修改。确定要继续吗？',
      positiveText: '确定',
      negativeText: '取消',
      onPositiveClick: () => {
        modifications.value.clear()
        page.value = newPage
        fetchData()
      }
    })
    return
  }
  page.value = newPage
  fetchData()
}

function handleRefresh() {
  if (hasModifications.value) {
    dialog.warning({
      title: '未保存的修改',
      content: '当前页面有未保存的修改，刷新将丢失这些修改。确定要继续吗？',
      positiveText: '确定',
      negativeText: '取消',
      onPositiveClick: () => {
        modifications.value.clear()
        fetchData()
        message.success('已刷新')
      }
    })
    return
  }
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
    modifications.value.clear()
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
        <!-- Modification Action Bar -->
        <div v-if="hasModifications" class="action-bar-top">
          <NSpace align="center" justify="space-between">
            <NSpace align="center">
              <NIcon size="16" color="#f0a020">
                <SaveOutline />
              </NIcon>
              <NText style="font-size: 13px">有 {{ modifiedCount }} 处修改未保存</NText>
            </NSpace>
            <NSpace :size="8">
              <NButton size="tiny" ghost @click="discardAllChanges">放弃</NButton>
              <NButton size="tiny" type="primary" :loading="isSaving" @click="handleSaveAllChanges">
                <template #icon>
                  <NIcon size="14"><CheckmarkCircleOutline /></NIcon>
                </template>
                保存修改
              </NButton>
            </NSpace>
          </NSpace>
        </div>

        <NCard class="glass-card">
          <template #header>
            <NSpace align="center" justify="space-between">
              <NSpace align="center" :size="8">
                <span style="font-size: 12px">共 {{ total.toLocaleString() }} 行</span>
                <NText v-if="hasModifications" depth="3" style="font-size: 11px">
                  (提示：点击单元格编辑，黄色表示已修改)
                </NText>
              </NSpace>
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

.action-bar-top {
  padding: 8px 16px;
  background: rgba(240, 160, 32, 0.1);
  border: 1px solid rgba(240, 160, 32, 0.2);
  border-radius: 4px;
  margin-bottom: 12px;
}

.pagination {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
</style>
