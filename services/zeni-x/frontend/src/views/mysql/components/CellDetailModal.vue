<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  NModal,
  NCard,
  NSpace,
  NButton,
  NInput,
  NText,
  NTag,
  NScrollbar,
  useMessage,
} from 'naive-ui'

interface Props {
  show: boolean
  value: unknown
  column: string
  columnType?: string
  mode: 'view' | 'edit'
  readonly?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  columnType: 'unknown',
  readonly: false,
})

const emit = defineEmits<{
  'update:show': [value: boolean]
  'save': [value: unknown]
  'export': [value: unknown]
}>()

const message = useMessage()
const editValue = ref('')
const currentMode = ref<'view' | 'edit'>('view')

// 数据大小限制 (1MB)
const MAX_DISPLAY_SIZE = 1024 * 1024

// 判断是否为 BLOB 类型
const isBlobType = computed(() => {
  const type = props.columnType.toLowerCase()
  return type.includes('blob') || type.includes('binary') || type.includes('varbinary')
})

// 判断数据是否过大
const isDataTooLarge = computed(() => {
  if (typeof props.value === 'string') {
    return props.value.length > MAX_DISPLAY_SIZE
  }
  if (props.value instanceof ArrayBuffer) {
    return props.value.byteLength > MAX_DISPLAY_SIZE
  }
  if (typeof props.value === 'object' && props.value !== null) {
    const str = JSON.stringify(props.value)
    return str.length > MAX_DISPLAY_SIZE
  }
  return false
})

// 计算数据大小
const dataSize = computed(() => {
  if (typeof props.value === 'string') {
    return formatSize(props.value.length)
  }
  if (props.value instanceof ArrayBuffer) {
    return formatSize(props.value.byteLength)
  }
  if (typeof props.value === 'object' && props.value !== null) {
    return formatSize(JSON.stringify(props.value).length)
  }
  return '0 B'
})

// 格式化大小
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

// 格式化显示值
const displayValue = computed(() => {
  if (props.value === null || props.value === undefined) {
    return 'NULL'
  }

  if (isDataTooLarge.value) {
    return `[数据过大，无法显示] 大小: ${dataSize.value}`
  }

  // BLOB 数据显示为十六进制
  if (isBlobType.value) {
    if (typeof props.value === 'string') {
      // 如果已经是字符串，尝试转为十六进制显示
      return formatAsHex(props.value)
    }
    if (props.value instanceof ArrayBuffer) {
      return arrayBufferToHex(props.value)
    }
    // 尝试将对象转为二进制表示
    return formatAsHex(String(props.value))
  }

  // JSON 对象格式化显示
  if (typeof props.value === 'object') {
    try {
      return JSON.stringify(props.value, null, 2)
    } catch {
      return String(props.value)
    }
  }

  return String(props.value)
})

// 数据类型标签
const dataTypeTag = computed(() => {
  if (props.value === null) return { type: 'default' as const, text: 'NULL' }
  if (isBlobType.value) return { type: 'warning' as const, text: 'BLOB' }
  if (typeof props.value === 'object') return { type: 'info' as const, text: 'JSON' }
  if (typeof props.value === 'number') return { type: 'success' as const, text: 'Number' }
  if (typeof props.value === 'boolean') return { type: 'primary' as const, text: 'Boolean' }
  return { type: 'default' as const, text: 'String' }
})

// 转换为十六进制显示
function formatAsHex(str: string): string {
  const bytes: string[] = []
  for (let i = 0; i < str.length && i < 1000; i++) {
    bytes.push(str.charCodeAt(i).toString(16).padStart(2, '0'))
  }
  if (str.length > 1000) {
    return bytes.join(' ') + ` ... (${str.length - 1000} more bytes)`
  }
  return bytes.join(' ')
}

// ArrayBuffer 转十六进制
function arrayBufferToHex(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer)
  const hexParts: string[] = []
  for (let i = 0; i < bytes.length && i < 1000; i++) {
    hexParts.push(bytes[i].toString(16).padStart(2, '0'))
  }
  if (bytes.length > 1000) {
    return hexParts.join(' ') + ` ... (${bytes.length - 1000} more bytes)`
  }
  return hexParts.join(' ')
}

// 初始化编辑值
watch(() => props.show, (newVal) => {
  if (newVal) {
    currentMode.value = props.mode
    if (typeof props.value === 'object' && props.value !== null) {
      editValue.value = JSON.stringify(props.value, null, 2)
    } else {
      editValue.value = props.value === null ? '' : String(props.value)
    }
  }
})

// 关闭弹窗
function handleClose() {
  emit('update:show', false)
}

// 保存修改
function handleSave() {
  let parsedValue: unknown = editValue.value

  // 尝试解析 JSON
  if (typeof props.value === 'object' && props.value !== null) {
    try {
      parsedValue = JSON.parse(editValue.value)
    } catch {
      message.warning('JSON 格式无效，将作为字符串保存')
    }
  }

  emit('save', parsedValue)
  emit('update:show', false)
}

// 导出数据
function handleExport() {
  let content: string
  let filename: string
  let mimeType: string

  if (typeof props.value === 'object' && props.value !== null) {
    content = JSON.stringify(props.value, null, 2)
    filename = `${props.column}.json`
    mimeType = 'application/json'
  } else if (isBlobType.value) {
    content = displayValue.value
    filename = `${props.column}.hex`
    mimeType = 'text/plain'
  } else {
    content = String(props.value)
    filename = `${props.column}.txt`
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

// 复制到剪贴板
async function handleCopy() {
  try {
    await navigator.clipboard.writeText(displayValue.value)
    message.success('已复制到剪贴板')
  } catch {
    message.error('复制失败')
  }
}

// 切换模式
function toggleMode() {
  if (props.readonly || isBlobType.value || isDataTooLarge.value) {
    message.warning('当前数据不支持编辑')
    return
  }
  currentMode.value = currentMode.value === 'view' ? 'edit' : 'view'
}
</script>

<template>
  <NModal
    :show="show"
    :mask-closable="true"
    @update:show="handleClose"
  >
    <NCard
      :title="`${column} - ${currentMode === 'view' ? '查看' : '编辑'}`"
      :bordered="false"
      size="small"
      style="width: 700px; max-width: 90vw"
      :segmented="{ content: true, footer: 'soft' }"
    >
      <template #header-extra>
        <NSpace :size="8">
          <NTag :type="dataTypeTag.type" size="small">{{ dataTypeTag.type }}</NTag>
          <NText depth="3" style="font-size: 12px">{{ dataSize }}</NText>
        </NSpace>
      </template>

      <div class="content-area">
        <template v-if="isDataTooLarge">
          <div class="large-data-warning">
            <NText type="warning">
              数据过大 ({{ dataSize }})，无法直接显示
            </NText>
            <NButton size="small" @click="handleExport" style="margin-top: 12px">
              导出查看
            </NButton>
          </div>
        </template>

        <template v-else-if="currentMode === 'view'">
          <NScrollbar style="max-height: 400px">
            <pre class="data-display">{{ displayValue }}</pre>
          </NScrollbar>
        </template>

        <template v-else>
          <NInput
            v-model:value="editValue"
            type="textarea"
            :rows="15"
            :placeholder="isBlobType ? 'BLOB 数据不支持编辑' : '编辑数据'"
            :disabled="isBlobType"
            style="font-family: 'Monaco', 'Menlo', 'Consolas', monospace; font-size: 13px"
          />
        </template>
      </div>

      <template #footer>
        <NSpace justify="space-between">
          <NSpace :size="8">
            <NButton size="small" @click="handleCopy">复制</NButton>
            <NButton size="small" @click="handleExport">导出</NButton>
          </NSpace>
          <NSpace :size="8">
            <NButton size="small" @click="handleClose">关闭</NButton>
            <NButton
              v-if="!readonly && !isBlobType && !isDataTooLarge"
              size="small"
              :type="currentMode === 'edit' ? 'default' : 'primary'"
              @click="toggleMode"
            >
              {{ currentMode === 'view' ? '编辑' : '取消编辑' }}
            </NButton>
            <NButton
              v-if="currentMode === 'edit'"
              size="small"
              type="primary"
              @click="handleSave"
            >
              保存
            </NButton>
          </NSpace>
        </NSpace>
      </template>
    </NCard>
  </NModal>
</template>

<style scoped>
.content-area {
  min-height: 200px;
}

.data-display {
  margin: 0;
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 6px;
  font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--n-text-color);
}

.large-data-warning {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  text-align: center;
}
</style>
