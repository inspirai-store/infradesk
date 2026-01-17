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
  NDropdown,
  NTag,
  useMessage,
  useDialog,
} from 'naive-ui'
import {
  AddOutline,
  TrashOutline,
  RefreshOutline,
  DownloadOutline,
  CloudUploadOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  SaveOutline,
  EyeOutline,
  CreateOutline,
  EllipsisVertical,
} from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import { mysqlApi } from '@/api'
import type { DataTableColumns, DropdownOption } from 'naive-ui'
import IndexManager from './components/IndexManager.vue'
import ForeignKeyManager from './components/ForeignKeyManager.vue'
import ExportDialog from './components/ExportDialog.vue'
import ImportDialog from './components/ImportDialog.vue'
import CellDetailModal from './components/CellDetailModal.vue'

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

// Export/Import dialog state
const showExportDialog = ref(false)
const showImportDialog = ref(false)

// Cell detail modal state
const showCellDetail = ref(false)
const cellDetailValue = ref<unknown>(null)
const cellDetailColumn = ref('')
const cellDetailColumnType = ref('')
const cellDetailMode = ref<'view' | 'edit'>('view')
const cellDetailRowIndex = ref(-1)

// 数据显示阈值
const MAX_CELL_DISPLAY_LENGTH = 100

// 格式化单元格显示值
function formatCellValue(val: unknown, columnType?: string): { display: string; isLong: boolean; isBlob: boolean; isObject: boolean } {
  if (val === null || val === undefined) {
    return { display: 'NULL', isLong: false, isBlob: false, isObject: false }
  }

  const colType = (columnType || '').toLowerCase()
  const isBlob = colType.includes('blob') || colType.includes('binary') || colType.includes('varbinary')

  if (isBlob) {
    const size = typeof val === 'string' ? val.length : 0
    return { display: `[BLOB: ${formatSize(size)}]`, isLong: true, isBlob: true, isObject: false }
  }

  if (typeof val === 'object') {
    try {
      const jsonStr = JSON.stringify(val)
      if (jsonStr.length > MAX_CELL_DISPLAY_LENGTH) {
        return { display: jsonStr.substring(0, MAX_CELL_DISPLAY_LENGTH) + '...', isLong: true, isBlob: false, isObject: true }
      }
      return { display: jsonStr, isLong: false, isBlob: false, isObject: true }
    } catch {
      return { display: '[Object]', isLong: true, isBlob: false, isObject: true }
    }
  }

  const strVal = String(val)
  if (strVal.length > MAX_CELL_DISPLAY_LENGTH) {
    return { display: strVal.substring(0, MAX_CELL_DISPLAY_LENGTH) + '...', isLong: true, isBlob: false, isObject: false }
  }
  return { display: strVal, isLong: false, isBlob: false, isObject: false }
}

// 格式化大小
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

// 获取列类型
function getColumnType(columnName: string): string {
  const col = store.tableSchema?.columns.find(c => c.name === columnName)
  return col?.type || 'unknown'
}

// 打开单元格详情
function openCellDetail(rowIndex: number, column: string, mode: 'view' | 'edit') {
  cellDetailRowIndex.value = rowIndex
  cellDetailColumn.value = column
  cellDetailValue.value = rows.value[rowIndex][column]
  cellDetailColumnType.value = getColumnType(column)
  cellDetailMode.value = mode
  showCellDetail.value = true
}

// 保存单元格详情编辑
function handleCellDetailSave(newValue: unknown) {
  if (cellDetailRowIndex.value < 0) return

  const rowIndex = cellDetailRowIndex.value
  const column = cellDetailColumn.value
  const oldValue = rows.value[rowIndex][column]

  if (newValue === oldValue) return

  const key = `${rowIndex}-${column}`
  modifications.value.set(key, { rowIndex, column, oldValue, newValue })
  message.success('修改已暂存，点击"保存全部"提交')
}

// 导出单元格数据
function exportCellData(rowIndex: number, column: string) {
  const val = rows.value[rowIndex][column]
  let content: string
  let filename: string
  let mimeType: string

  if (typeof val === 'object' && val !== null) {
    content = JSON.stringify(val, null, 2)
    filename = `${table.value}_${column}_row${rowIndex + 1}.json`
    mimeType = 'application/json'
  } else {
    content = String(val ?? '')
    filename = `${table.value}_${column}_row${rowIndex + 1}.txt`
    mimeType = 'text/plain'
  }

  const blob = new Blob([content], { type: mimeType })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)

  message.success('导出成功')
}

// 右键菜单选项
const cellMenuOptions: DropdownOption[] = [
  { label: '查看', key: 'view', icon: () => h(NIcon, { size: 14 }, { default: () => h(EyeOutline) }) },
  { label: '编辑', key: 'edit', icon: () => h(NIcon, { size: 14 }, { default: () => h(CreateOutline) }) },
  { label: '导出', key: 'export', icon: () => h(NIcon, { size: 14 }, { default: () => h(DownloadOutline) }) },
  { type: 'divider', key: 'd1' },
  { label: '取消', key: 'cancel' },
]

// 处理右键菜单选择
function handleCellMenuSelect(key: string, rowIndex: number, column: string) {
  switch (key) {
    case 'view':
      openCellDetail(rowIndex, column, 'view')
      break
    case 'edit':
      openCellDetail(rowIndex, column, 'edit')
      break
    case 'export':
      exportCellData(rowIndex, column)
      break
  }
}

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
  
  // 如果值没有变化，直接取消编辑状态
  if (newValue === oldValue) {
    editingCell.value = null
    return
  }
  
  // 仅在此时将修改放入待保存队列（Staging）
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

// 保存单行修改
async function handleSaveRow(rowIndex: number) {
  const pkCol = store.tableSchema?.columns.find(c => c.key === 'PRI')
  if (!pkCol) {
    message.error('无法保存：未找到主键')
    return
  }

  const rowMods = Array.from(modifications.value.values()).filter(m => m.rowIndex === rowIndex)
  if (rowMods.length === 0) return

  const updates: Record<string, unknown> = {}
  rowMods.forEach(m => { updates[m.column] = m.newValue })

  const rowData = rows.value[rowIndex]
  const pkValue = rowData[pkCol.name]

  isSaving.value = true
  try {
    await mysqlApi.updateRecord(database.value, table.value, pkCol.name, pkValue, updates)
    message.success('行保存成功')
    
    // 清除该行的修改追踪
    rowMods.forEach(m => {
      const key = `${m.rowIndex}-${m.column}`
      modifications.value.delete(key)
    })
    
    fetchData()
  } catch (e) {
    message.error((e as Error).message)
  } finally {
    isSaving.value = false
  }
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
    ellipsis: { tooltip: false }, // 禁用默认 tooltip，使用自定义显示
    render(row, index) {
      const isEditing = editingCell.value?.rowIndex === index && editingCell.value?.column === col
      const isModified = isCellModified(index, col)
      const val = row[col]
      const columnType = getColumnType(col)
      const formatted = formatCellValue(val, columnType)

      if (isEditing) {
        return h('div', { style: { display: 'flex', alignItems: 'center', gap: '4px' } }, [
          h(NInput, {
            size: 'tiny',
            value: String(editValue.value ?? ''),
            onUpdateValue: (v: string) => { editValue.value = v },
            onKeyup: (e: KeyboardEvent) => {
              if (e.key === 'Enter') saveEdit()
              if (e.key === 'Escape') cancelEdit()
            },
            autofocus: true,
            style: { flex: 1 }
          }),
          h(NButton, {
            size: 'tiny',
            quaternary: true,
            circle: true,
            type: 'success',
            onClick: (e: Event) => { e.stopPropagation(); saveEdit() }
          }, { icon: () => h(NIcon, { size: 14 }, { default: () => h(CheckmarkCircleOutline) }) }),
          h(NButton, {
            size: 'tiny',
            quaternary: true,
            circle: true,
            type: 'error',
            onClick: (e: Event) => { e.stopPropagation(); cancelEdit() }
          }, { icon: () => h(NIcon, { size: 14 }, { default: () => h(CloseCircleOutline) }) })
        ])
      }

      if (isModified) {
        const mod = getCellModification(index, col)!
        const oldFormatted = formatCellValue(mod.oldValue, columnType)
        const newFormatted = formatCellValue(mod.newValue, columnType)
        return h('div', {
          style: { display: 'flex', alignItems: 'center', gap: '4px', cursor: 'pointer' },
          onClick: () => startEdit(index, col)
        }, [
          h('span', { style: { textDecoration: 'line-through', opacity: 0.5, fontSize: '11px' } }, oldFormatted.display),
          h('span', { style: { color: '#f0a020', fontWeight: 'bold' } }, newFormatted.display),
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

      // 构建单元格内容
      const cellContent: any[] = []

      // 数据类型标签（仅对 BLOB 和 JSON 显示）
      if (formatted.isBlob) {
        cellContent.push(h(NTag, { size: 'tiny', type: 'warning', style: { marginRight: '4px' } }, { default: () => 'BLOB' }))
      } else if (formatted.isObject) {
        cellContent.push(h(NTag, { size: 'tiny', type: 'info', style: { marginRight: '4px' } }, { default: () => 'JSON' }))
      }

      // 数据内容
      cellContent.push(h('span', {
        class: val === null ? 'null-value' : '',
        style: {
          flex: 1,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
          cursor: formatted.isLong ? 'pointer' : 'text',
          opacity: val === null ? 0.5 : 1,
          fontStyle: val === null ? 'italic' : 'normal',
        },
        onClick: () => {
          if (!formatted.isLong && !formatted.isBlob) {
            startEdit(index, col)
          }
        }
      }, formatted.display))

      // 长数据或特殊数据添加操作下拉菜单
      if (formatted.isLong || formatted.isBlob || formatted.isObject) {
        cellContent.push(
          h(NDropdown, {
            trigger: 'click',
            options: cellMenuOptions,
            onSelect: (key: string) => handleCellMenuSelect(key, index, col)
          }, {
            default: () => h(NButton, {
              size: 'tiny',
              quaternary: true,
              circle: true,
              style: { marginLeft: '4px', flexShrink: 0 },
              onClick: (e: Event) => e.stopPropagation()
            }, { icon: () => h(NIcon, { size: 12 }, { default: () => h(EllipsisVertical) }) })
          })
        )
      }

      return h('div', {
        style: {
          display: 'flex',
          alignItems: 'center',
          width: '100%',
          minHeight: '1.5em',
        }
      }, cellContent)
    },
  }))
  
  cols.push({
    title: '操作',
    key: 'actions',
    width: 80,
    fixed: 'right',
    render(row, index) {
      const rowMods = Array.from(modifications.value.values()).filter(m => m.rowIndex === index)
      const hasRowMods = rowMods.length > 0
      
      return h(NSpace, { size: 4, justify: 'center' }, {
        default: () => [
          hasRowMods && h(
            NButton,
            {
              size: 'tiny',
              type: 'primary',
              quaternary: true,
              circle: true,
              loading: isSaving.value,
              onClick: () => handleSaveRow(index),
              title: '保存行修改'
            },
            { icon: () => h(NIcon, { size: 14 }, { default: () => h(SaveOutline) }) }
          ),
          h(
            NButton,
            {
              size: 'tiny',
              type: 'error',
              quaternary: true,
              circle: true,
              onClick: () => handleDeleteRow(row),
              title: '删除行'
            },
            { icon: () => h(NIcon, { size: 14 }, { default: () => h(TrashOutline) }) }
          ),
        ]
      })
    },
  })
  
  return cols
})

async function fetchData() {
  loading.value = true
  try {
    const result = await mysqlApi.getRows(database.value, table.value, page.value, pageSize.value) as { rows?: Record<string, unknown>[]; columns?: string[]; total: number }
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
        modifications.value.clear()
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
    modifications.value.clear()
    fetchData()
  } catch (e) {
    message.error((e as Error).message)
  }
}

function handleExport() {
  showExportDialog.value = true
}

function handleImport() {
  showImportDialog.value = true
}

function onImported() {
  fetchData()
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
        <NCard class="glass-card">
          <template #header>
            <NSpace align="center" justify="space-between">
              <NSpace align="center" :size="8">
                <span style="font-size: 12px">共 {{ total.toLocaleString() }} 行</span>
                <NText v-if="hasModifications" type="warning" strong style="font-size: 11px">
                  (有 {{ modifiedCount }} 处修改待保存)
                </NText>
              </NSpace>
              <NSpace :size="4">
                <template v-if="hasModifications">
                  <NButton size="tiny" ghost @click="discardAllChanges">放弃修改</NButton>
                  <NButton size="tiny" type="primary" :loading="isSaving" @click="handleSaveAllChanges">
                    <template #icon>
                      <NIcon size="14"><CheckmarkCircleOutline /></NIcon>
                    </template>
                    保存全部
                  </NButton>
                </template>
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
                <NButton size="tiny" @click="handleImport">
                  <template #icon>
                    <NIcon size="14"><CloudUploadOutline /></NIcon>
                  </template>
                  导入
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

      <!-- Indexes Tab -->
      <NTabPane name="indexes" tab="索引">
        <NCard class="glass-card">
          <IndexManager :database="database" :table="table" />
        </NCard>
      </NTabPane>

      <!-- Foreign Keys Tab -->
      <NTabPane name="foreign-keys" tab="外键">
        <NCard class="glass-card">
          <ForeignKeyManager :database="database" :table="table" />
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

    <!-- Export Dialog -->
    <ExportDialog
      v-model:show="showExportDialog"
      :database="database"
      :table="table"
      :columns="columns"
    />

    <!-- Import Dialog -->
    <ImportDialog
      v-model:show="showImportDialog"
      :database="database"
      :table="table"
      :columns="columns"
      @imported="onImported"
    />

    <!-- Cell Detail Modal -->
    <CellDetailModal
      v-model:show="showCellDetail"
      :value="cellDetailValue"
      :column="cellDetailColumn"
      :column-type="cellDetailColumnType"
      :mode="cellDetailMode"
      @save="handleCellDetailSave"
    />
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
