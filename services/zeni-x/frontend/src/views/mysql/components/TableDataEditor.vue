<script setup lang="ts">
import { ref, computed, h } from 'vue'
import {
  NSpace,
  NDataTable,
  NButton,
  NText,
  NInput,
  NIcon,
  NModal,
  NAlert,
  useMessage,
  NEmpty,
  type DataTableColumns,
} from 'naive-ui'
import {
  CheckmarkCircleOutline,
  CloseCircleOutline,
  SaveOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
} from '@vicons/ionicons5'
import { mysqlApi } from '@/api'

interface Props {
  database: string
  sql: string
  columns: string[]
  data: Record<string, unknown>[]
  loading?: boolean
}

interface Emits {
  (e: 'refresh'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
const message = useMessage()

// 编辑状态
const editingCell = ref<{ rowIndex: number; column: string } | null>(null)
const editValue = ref<unknown>('')

// 修改追踪
interface CellModification {
  rowIndex: number
  column: string
  oldValue: unknown
  newValue: unknown
}
const modifications = ref<Map<string, CellModification>>(new Map())

// 分页状态
const currentPage = ref(1)
const pageSize = ref(100)
const totalCount = ref(props.data.length)

// 加载状态
const isSaving = ref(false)

// 显示保存确认
const showSaveModal = ref(false)
const pendingUpdates = ref<Array<{ rowIndex: number; updates: Record<string, unknown> }>>([])

// 计算属性
const modifiedCount = computed(() => modifications.value.size)
const hasModifications = computed(() => modifiedCount.value > 0)

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
  editingCell.value = { rowIndex, column }
  editValue.value = props.data[rowIndex][column]
}

// 保存单元格编辑
function saveEdit() {
  if (!editingCell.value) return
  const { rowIndex, column } = editingCell.value
  const oldValue = props.data[rowIndex][column]
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

// 从 SQL 中提取表名
function extractTableName(sql: string): string {
  const match = sql.match(/FROM\s+`?(\w+)`?\s*/i)
  return match ? match[1] : 'table_name'
}

// 保存所有修改
async function saveAllChanges() {
  if (modifications.value.size === 0) {
    message.warning('没有需要保存的修改')
    return
  }

  // 按记录分组修改
  const updatesByRow = new Map<number, Record<string, unknown>>()
  modifications.value.forEach((mod) => {
    if (!updatesByRow.has(mod.rowIndex)) {
      updatesByRow.set(mod.rowIndex, {})
    }
    updatesByRow.get(mod.rowIndex)![mod.column] = mod.newValue
  })

  // 准备更新列表
  const updates: Array<{ rowIndex: number; updates: Record<string, unknown> }> = []
  updatesByRow.forEach((vals, rowIndex) => {
    updates.push({ rowIndex, updates: vals })
  })

  pendingUpdates.value = updates
  showSaveModal.value = true
}

// 确认保存
async function confirmSave() {
  showSaveModal.value = false
  isSaving.value = true

  try {
    const tableName = extractTableName(props.sql)
    let successCount = 0
    let failCount = 0

    for (const { rowIndex, updates } of pendingUpdates.value) {
      const rowData = props.data[rowIndex]
      // 尝试使用第一个列作为主键（简化版）
      const primaryKey = props.columns[0]
      const pkValue = rowData[primaryKey]

      try {
        await mysqlApi.updateRecord(
          props.database,
          tableName,
          primaryKey,
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
    emit('refresh')
  } catch (e) {
    message.error(`保存失败: ${(e as Error).message}`)
  } finally {
    isSaving.value = false
  }
}

// 列定义
const tableColumns = computed<DataTableColumns<Record<string, unknown>>>(() => {
  return props.columns.map((col) => ({
    title: col,
    key: col,
    render: (row: Record<string, unknown>, index: number) => {
      const value = row[col]
      const isEditing = editingCell.value?.rowIndex === index && editingCell.value?.column === col
      const isModified = isCellModified(index, col)

      if (isEditing) {
        return h(NInput, {
          value: String(editValue.value || ''),
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
        return h('div', { style: { display: 'flex', alignItems: 'center', gap: '4px' } }, [
          h('span', { style: { textDecoration: 'line-through', opacity: 0.5 } }, String(mod.oldValue ?? 'NULL')),
          h('span', { style: { color: '#f0a020', fontWeight: 'bold' } }, String(mod.newValue ?? 'NULL')),
          h(NButton, {
            size: 'tiny',
            quaternary: true,
            onClick: (e: Event) => {
              e.stopPropagation()
              revertCell(index, col)
            }
          }, { icon: () => h(NIcon, null, { default: () => h(CloseCircleOutline) }) })
        ])
      }

      return h('span', {
        onClick: () => startEdit(index, col),
        style: { cursor: 'pointer' }
      }, String(value ?? 'NULL'))
    }
  }))
})
</script>

<template>
  <div class="table-data-editor">
    <!-- 操作栏 -->
    <div v-if="hasModifications" class="action-bar">
      <NSpace align="center" justify="space-between">
        <NSpace align="center">
          <NIcon size="16" color="#f0a020">
            <SaveOutline />
          </NIcon>
          <NText>有 {{ modifiedCount }} 处修改未保存</NText>
        </NSpace>
        <NSpace :size="8">
          <NButton size="small" ghost @click="discardAllChanges">放弃修改</NButton>
          <NButton size="small" type="primary" :loading="isSaving" @click="saveAllChanges">
            <template #icon>
              <NIcon size="14"><CheckmarkCircleOutline /></NIcon>
            </template>
            保存修改
          </NButton>
        </NSpace>
      </NSpace>
    </div>

    <!-- 数据表格 -->
    <NDataTable
      :columns="tableColumns"
      :data="props.data"
      :loading="props.loading"
      :bordered="false"
      size="small"
      striped
      :max-height="400"
      :scroll-x="columns.length * 120"
    />

    <!-- 分页控件 -->
    <div v-if="totalCount > pageSize" class="pagination">
      <NSpace align="center" justify="center">
        <NButton size="small" :disabled="currentPage === 1">
          <template #icon><NIcon size="14"><ChevronBackOutline /></NIcon></template>
          上一页
        </NButton>
        <NText style="font-size: 12px">第 {{ currentPage }} 页</NText>
        <NButton size="small" :disabled="currentPage * pageSize >= totalCount">
          下一页
          <template #icon><NIcon size="14"><ChevronForwardOutline /></NIcon></template>
        </NButton>
        <NText depth="3" style="font-size: 11px">共 {{ totalCount }} 条</NText>
      </NSpace>
    </div>

    <!-- 保存确认对话框 -->
    <NModal v-model:show="showSaveModal" preset="card" title="确认保存" style="width: 600px">
      <NSpace vertical :size="16">
        <NAlert type="warning" :bordered="false">
          即将保存 {{ pendingUpdates.length }} 条记录的修改
        </NAlert>
        <NSpace justify="end">
          <NButton @click="showSaveModal = false">取消</NButton>
          <NButton type="primary" :loading="isSaving" @click="confirmSave">确认保存</NButton>
        </NSpace>
      </NSpace>
    </NModal>

    <!-- 空状态 -->
    <div v-if="!props.loading && props.data.length === 0" class="empty-state">
      <NEmpty description="无数据" />
    </div>

    <!-- 提示 -->
    <div v-if="props.data.length > 0" class="hint-text">
      <NText depth="3" style="font-size: 11px">提示：点击单元格编辑，黄色显示表示已修改</NText>
    </div>
  </div>
</template>

<style scoped>
.table-data-editor {
  width: 100%;
}

.action-bar {
  padding: 8px 12px;
  background: rgba(240, 160, 32, 0.1);
  border: 1px solid rgba(240, 160, 32, 0.3);
  border-radius: 4px;
  margin-bottom: 12px;
}

.pagination {
  padding: 12px;
  border-top: 1px solid var(--n-border-color);
  margin-top: 12px;
}

.empty-state {
  padding: 32px;
  text-align: center;
}

.hint-text {
  margin-top: 8px;
}
</style>
