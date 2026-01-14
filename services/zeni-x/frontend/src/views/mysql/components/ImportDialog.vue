<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  NModal,
  NForm,
  NFormItem,
  NRadioGroup,
  NRadioButton,
  NInput,
  NInputNumber,
  NSelect,
  NButton,
  NSpace,
  NUpload,
  NAlert,
  NSpin,
  NDataTable,
  NScrollbar,
  useMessage,
  type UploadFileInfo,
} from 'naive-ui'
import { mysqlApi } from '@/api'
import type { ImportDataRequest, ImportResult } from '@/api/types'

const props = defineProps<{
  show: boolean
  database: string
  table: string
  columns: string[]
}>()

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
  (e: 'imported'): void
}>()

const message = useMessage()

// Import options
const importFormat = ref<'csv' | 'json'>('csv')
const importData = ref('')
const skipRows = ref(0)
const onDuplicate = ref<'ignore' | 'update' | 'error'>('error')
const columnMapping = ref<Record<string, string>>({})

// Import state
const loading = ref(false)
const importResult = ref<ImportResult | null>(null)
const error = ref<string | null>(null)
const previewData = ref<Record<string, unknown>[]>([])

const formatOptions = [
  { label: 'CSV', value: 'csv' },
  { label: 'JSON', value: 'json' },
]

const duplicateOptions = [
  { label: '报错', value: 'error' },
  { label: '忽略', value: 'ignore' },
  { label: '更新', value: 'update' },
]

const showModal = computed({
  get: () => props.show,
  set: (value) => emit('update:show', value),
})

// Reset state when modal opens
watch(() => props.show, (newVal) => {
  if (newVal) {
    importData.value = ''
    skipRows.value = 0
    onDuplicate.value = 'error'
    importResult.value = null
    error.value = null
    previewData.value = []
    columnMapping.value = {}
  }
})

// Parse and preview data when content changes
watch([importData, importFormat], () => {
  if (!importData.value.trim()) {
    previewData.value = []
    return
  }
  try {
    if (importFormat.value === 'json') {
      const parsed = JSON.parse(importData.value)
      previewData.value = Array.isArray(parsed) ? parsed.slice(0, 5) : [parsed]
    } else {
      // Simple CSV parsing for preview
      const lines = importData.value.split('\n').filter(l => l.trim())
      if (lines.length > 0) {
        const headers = lines[0].split(',').map(h => h.trim().replace(/^["']|["']$/g, ''))
        previewData.value = lines.slice(1, 6).map(line => {
          const values = line.split(',').map(v => v.trim().replace(/^["']|["']$/g, ''))
          const row: Record<string, unknown> = {}
          headers.forEach((h, i) => {
            row[h] = values[i] || ''
          })
          return row
        })
      }
    }
  } catch {
    previewData.value = []
  }
})

// Preview table columns
const previewColumns = computed(() => {
  if (previewData.value.length === 0) return []
  const keys = Object.keys(previewData.value[0])
  return keys.map(key => ({
    title: key,
    key,
    ellipsis: { tooltip: true },
  }))
})

// Handle file upload
function handleFileChange(options: { fileList: UploadFileInfo[] }) {
  const file = options.fileList[0]?.file
  if (!file) return

  const reader = new FileReader()
  reader.onload = (e) => {
    importData.value = e.target?.result as string || ''
  }
  reader.readAsText(file)
}

async function handleImport() {
  if (!importData.value.trim()) {
    message.warning('请输入或上传数据')
    return
  }

  loading.value = true
  error.value = null
  importResult.value = null

  try {
    const request: ImportDataRequest = {
      data: importData.value,
      format: importFormat.value,
      skip_rows: skipRows.value,
      on_duplicate: onDuplicate.value,
      column_mapping: Object.keys(columnMapping.value).length > 0 ? columnMapping.value : undefined,
    }

    const result = await mysqlApi.importData(props.database, props.table, request)
    importResult.value = result

    if (result.failed > 0) {
      message.warning(`导入完成: ${result.imported} 成功, ${result.skipped} 跳过, ${result.failed} 失败`)
    } else {
      message.success(`导入成功: ${result.imported} 行`)
    }

    emit('imported')
  } catch (err) {
    error.value = err instanceof Error ? err.message : '导入失败'
    message.error(error.value)
  } finally {
    loading.value = false
  }
}

function handleClose() {
  showModal.value = false
}
</script>

<template>
  <NModal
    v-model:show="showModal"
    preset="card"
    title="导入数据"
    :style="{ width: '800px' }"
    :mask-closable="false"
  >
    <NSpin :show="loading">
      <NForm label-placement="left" label-width="100">
        <!-- Format selection -->
        <NFormItem label="数据格式">
          <NRadioGroup v-model:value="importFormat" :disabled="!!importResult">
            <NRadioButton
              v-for="opt in formatOptions"
              :key="opt.value"
              :value="opt.value"
              :label="opt.label"
            />
          </NRadioGroup>
        </NFormItem>

        <!-- File upload -->
        <NFormItem label="上传文件">
          <NUpload
            :max="1"
            :show-file-list="false"
            :accept="importFormat === 'csv' ? '.csv' : '.json'"
            :disabled="!!importResult"
            @change="handleFileChange"
          >
            <NButton :disabled="!!importResult">选择文件</NButton>
          </NUpload>
        </NFormItem>

        <!-- Data input -->
        <NFormItem label="数据内容">
          <NInput
            v-model:value="importData"
            type="textarea"
            :rows="8"
            :placeholder="importFormat === 'csv'
              ? 'id,name,email\n1,John,john@example.com\n2,Jane,jane@example.com'
              : '[{&quot;id&quot;: 1, &quot;name&quot;: &quot;John&quot;}, {&quot;id&quot;: 2, &quot;name&quot;: &quot;Jane&quot;}]'"
            :disabled="!!importResult"
          />
        </NFormItem>

        <!-- Skip rows (CSV only) -->
        <NFormItem v-if="importFormat === 'csv'" label="跳过行数">
          <NInputNumber
            v-model:value="skipRows"
            :min="0"
            :max="100"
            placeholder="0"
            style="width: 120px"
            :disabled="!!importResult"
          />
          <span style="margin-left: 8px; color: #999; font-size: 12px">跳过前 N 行（不包括表头）</span>
        </NFormItem>

        <!-- On duplicate -->
        <NFormItem label="重复处理">
          <NSelect
            v-model:value="onDuplicate"
            :options="duplicateOptions"
            style="width: 150px"
            :disabled="!!importResult"
          />
          <span style="margin-left: 8px; color: #999; font-size: 12px">
            当主键或唯一索引冲突时
          </span>
        </NFormItem>
      </NForm>

      <!-- Preview -->
      <template v-if="previewData.length > 0 && !importResult">
        <div style="margin: 16px 0 8px; font-weight: 500">数据预览 (前 5 行)</div>
        <NScrollbar style="max-height: 200px">
          <NDataTable
            :columns="previewColumns"
            :data="previewData"
            :bordered="true"
            size="small"
          />
        </NScrollbar>
      </template>

      <!-- Error display -->
      <NAlert v-if="error" type="error" :title="error" style="margin-top: 16px" />

      <!-- Import result -->
      <template v-if="importResult">
        <NAlert
          :type="importResult.failed > 0 ? 'warning' : 'success'"
          style="margin-top: 16px"
        >
          <template #header>
            导入完成
          </template>
          <div>成功导入: {{ importResult.imported }} 行</div>
          <div>跳过: {{ importResult.skipped }} 行</div>
          <div>失败: {{ importResult.failed }} 行</div>
          <template v-if="importResult.errors.length > 0">
            <div style="margin-top: 8px; color: #d03050">
              错误信息:
              <ul style="margin: 4px 0 0 16px">
                <li v-for="(err, idx) in importResult.errors.slice(0, 5)" :key="idx">
                  {{ err }}
                </li>
                <li v-if="importResult.errors.length > 5">
                  ... 还有 {{ importResult.errors.length - 5 }} 个错误
                </li>
              </ul>
            </div>
          </template>
        </NAlert>
      </template>
    </NSpin>

    <template #footer>
      <NSpace justify="end">
        <NButton @click="handleClose">关闭</NButton>
        <template v-if="!importResult">
          <NButton
            type="primary"
            :loading="loading"
            :disabled="!importData.trim()"
            @click="handleImport"
          >
            导入
          </NButton>
        </template>
        <template v-else>
          <NButton @click="importResult = null; error = null; importData = ''">重新导入</NButton>
        </template>
      </NSpace>
    </template>
  </NModal>
</template>
