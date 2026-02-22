<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  NModal,
  NForm,
  NFormItem,
  NRadioGroup,
  NRadioButton,
  NCheckboxGroup,
  NCheckbox,
  NInput,
  NInputNumber,
  NSwitch,
  NButton,
  NSpace,
  NCode,
  NScrollbar,
  NSpin,
  NAlert,
  useMessage,
} from 'naive-ui'
import { mysqlApi } from '@/api'
import type { ExportFormat, ExportTableRequest, ExportTableResponse } from '@/api/types'

const props = defineProps<{
  show: boolean
  database: string
  table: string
  columns: string[]
}>()

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
}>()

const message = useMessage()

// Export options
const exportFormat = ref<ExportFormat>('csv')
const selectedColumns = ref<string[]>([])
const whereClause = ref('')
const limit = ref<number | null>(null)
const includeHeaders = ref(true)

// Export state
const loading = ref(false)
const exportResult = ref<ExportTableResponse | null>(null)
const error = ref<string | null>(null)

const formatOptions = [
  { label: 'CSV', value: 'csv' },
  { label: 'JSON', value: 'json' },
  { label: 'SQL', value: 'sql' },
]

const showModal = computed({
  get: () => props.show,
  set: (value) => emit('update:show', value),
})

// Reset state when modal opens
watch(() => props.show, (newVal) => {
  if (newVal) {
    selectedColumns.value = [...props.columns]
    exportResult.value = null
    error.value = null
    whereClause.value = ''
    limit.value = null
  }
})

async function handleExport() {
  loading.value = true
  error.value = null
  exportResult.value = null

  try {
    const request: ExportTableRequest = {
      format: exportFormat.value,
      columns: selectedColumns.value.length === props.columns.length
        ? undefined
        : selectedColumns.value,
      where_clause: whereClause.value.trim() || undefined,
      limit: limit.value || undefined,
      include_headers: includeHeaders.value,
    }

    const result = await mysqlApi.exportTable(props.database, props.table, request)
    exportResult.value = result
    message.success(`导出成功，共 ${result.row_count} 行`)
  } catch (err) {
    error.value = err instanceof Error ? err.message : '导出失败'
    message.error(error.value)
  } finally {
    loading.value = false
  }
}

function handleDownload() {
  if (!exportResult.value) return

  const mimeTypes: Record<string, string> = {
    csv: 'text/csv',
    json: 'application/json',
    sql: 'text/plain',
  }
  const extensions: Record<string, string> = {
    csv: 'csv',
    json: 'json',
    sql: 'sql',
  }

  const blob = new Blob([exportResult.value.data], {
    type: `${mimeTypes[exportResult.value.format]};charset=utf-8`
  })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${props.table}_${new Date().toISOString().slice(0, 10)}.${extensions[exportResult.value.format]}`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
  message.success('文件已下载')
}

function handleCopy() {
  if (!exportResult.value) return
  navigator.clipboard.writeText(exportResult.value.data)
  message.success('已复制到剪贴板')
}

function handleClose() {
  showModal.value = false
}
</script>

<template>
  <NModal
    v-model:show="showModal"
    preset="card"
    title="导出数据"
    :style="{ width: '700px' }"
    :mask-closable="false"
  >
    <NSpin :show="loading">
      <NForm label-placement="left" label-width="100">
        <!-- Format selection -->
        <NFormItem label="导出格式">
          <NRadioGroup v-model:value="exportFormat" :disabled="!!exportResult">
            <NRadioButton
              v-for="opt in formatOptions"
              :key="opt.value"
              :value="opt.value"
              :label="opt.label"
            />
          </NRadioGroup>
        </NFormItem>

        <!-- Column selection -->
        <NFormItem label="导出列">
          <NCheckboxGroup v-model:value="selectedColumns" :disabled="!!exportResult">
            <NSpace>
              <NCheckbox
                v-for="col in columns"
                :key="col"
                :value="col"
                :label="col"
              />
            </NSpace>
          </NCheckboxGroup>
        </NFormItem>

        <!-- WHERE clause -->
        <NFormItem label="筛选条件">
          <NInput
            v-model:value="whereClause"
            placeholder="可选，如: status = 1 AND created_at > '2024-01-01'"
            :disabled="!!exportResult"
          />
        </NFormItem>

        <!-- Limit -->
        <NFormItem label="行数限制">
          <NInputNumber
            v-model:value="limit"
            :min="1"
            :max="1000000"
            placeholder="不限制"
            style="width: 150px"
            :disabled="!!exportResult"
          />
        </NFormItem>

        <!-- Include headers (CSV only) -->
        <NFormItem v-if="exportFormat === 'csv'" label="包含表头">
          <NSwitch v-model:value="includeHeaders" :disabled="!!exportResult" />
        </NFormItem>
      </NForm>

      <!-- Error display -->
      <NAlert v-if="error" type="error" :title="error" style="margin-bottom: 16px" />

      <!-- Export result preview -->
      <template v-if="exportResult">
        <div style="margin-bottom: 12px; color: #999; font-size: 12px">
          导出格式: {{ exportResult.format.toUpperCase() }} | 行数: {{ exportResult.row_count }}
        </div>
        <NScrollbar style="max-height: 300px; border: 1px solid #333; border-radius: 4px; padding: 8px; background: #1e1e1e">
          <NCode :code="exportResult.data" :language="exportFormat === 'json' ? 'json' : 'text'" />
        </NScrollbar>
      </template>
    </NSpin>

    <template #footer>
      <NSpace justify="end">
        <NButton @click="handleClose">关闭</NButton>
        <template v-if="!exportResult">
          <NButton type="primary" :loading="loading" :disabled="selectedColumns.length === 0" @click="handleExport">
            导出
          </NButton>
        </template>
        <template v-else>
          <NButton @click="exportResult = null; error = null">重新导出</NButton>
          <NButton type="info" @click="handleCopy">复制</NButton>
          <NButton type="primary" @click="handleDownload">下载</NButton>
        </template>
      </NSpace>
    </template>
  </NModal>
</template>
