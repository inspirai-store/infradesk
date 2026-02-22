<script setup lang="ts">
import { ref, computed, h, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { 
  NCard, 
  NSpace, 
  NButton, 
  NIcon, 
  NDataTable,
  NInput,
  NTag,
  NModal,
  NForm,
  NFormItem,
  NSelect,
  NInputNumber,
  useMessage,
  useDialog,
} from 'naive-ui'
import { 
  AddOutline, 
  TrashOutline, 
  RefreshOutline,
  SearchOutline,
  DownloadOutline,
} from '@vicons/ionicons5'
import { useRedisStore } from '@/stores/redis'
import { redisApi } from '@/api'
import type { DataTableColumns, DataTableRowKey } from 'naive-ui'

const router = useRouter()
const store = useRedisStore()
const message = useMessage()
const dialog = useDialog()

const searchPattern = ref('*')
const selectedKeys = ref<string[]>([])

const showCreateModal = ref(false)
const newKeyData = ref({
  key: '',
  type: 'string' as 'string' | 'hash' | 'list' | 'set' | 'zset',
  value: '',
  ttl: -1,
})

const typeOptions = [
  { label: 'String', value: 'string' },
  { label: 'Hash', value: 'hash' },
  { label: 'List', value: 'list' },
  { label: 'Set', value: 'set' },
  { label: 'Sorted Set', value: 'zset' },
]

const typeColors: Record<string, 'default' | 'error' | 'info' | 'success' | 'warning' | 'primary'> = {
  string: 'success',
  hash: 'warning',
  list: 'info',
  set: 'error',
  zset: 'default',
}

const newKeyValuePlaceholder = computed(() => {
  switch (newKeyData.value.type) {
    case 'string':
      return '输入值'
    case 'hash':
      return '输入 JSON 对象，如 {"field": "value"}'
    case 'list':
    case 'set':
      return '输入 JSON 数组，如 ["item1", "item2"]'
    case 'zset':
      return '输入 JSON 数组，如 [{"member": "a", "score": 1}]'
    default:
      return '输入值'
  }
})

const columns: DataTableColumns<{ key: string; type: string; ttl: number }> = [
  {
    type: 'selection',
  },
  {
    title: 'Key',
    key: 'key',
    render(row) {
      return h(
        'a',
        {
          style: 'color: var(--zx-accent-cyan); cursor: pointer; font-size: 12px;',
          onClick: () => router.push(`/redis/key/${encodeURIComponent(row.key)}`),
        },
        row.key
      )
    },
  },
  {
    title: '类型',
    key: 'type',
    width: 80,
    render(row) {
      return h(
        NTag,
        { type: typeColors[row.type] || 'default', size: 'tiny' },
        { default: () => row.type.toUpperCase() }
      )
    },
  },
  {
    title: '过期时间',
    key: 'ttl',
    width: 90,
    render(row) {
      if (row.ttl < 0) return '永不'
      return formatTTL(row.ttl)
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
          onClick: () => handleDelete(row.key),
        },
        { icon: () => h(NIcon, { size: 14 }, { default: () => h(TrashOutline) }) }
      )
    },
  },
]

function formatTTL(seconds: number): string {
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`
  return `${Math.floor(seconds / 86400)}d`
}

function handleSearch() {
  store.fetchKeys(searchPattern.value)
}

function handleRefresh() {
  store.fetchKeys(searchPattern.value)
  message.success('已刷新')
}

function handleRowSelect(keys: DataTableRowKey[]) {
  selectedKeys.value = keys as string[]
}

async function handleDelete(key: string) {
  dialog.warning({
    title: '删除 Key',
    content: `确定要删除 "${key}" 吗？`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await store.deleteKey(key)
        message.success(`"${key}" 已删除`)
      } catch (e) {
        message.error((e as Error).message)
      }
    },
  })
}

async function handleBatchDelete() {
  if (selectedKeys.value.length === 0) {
    message.warning('请选择要删除的 Key')
    return
  }
  
  dialog.warning({
    title: '批量删除',
    content: `确定要删除选中的 ${selectedKeys.value.length} 个 Key 吗？`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        for (const key of selectedKeys.value) {
          await store.deleteKey(key)
        }
        selectedKeys.value = []
        message.success('删除成功')
      } catch (e) {
        message.error((e as Error).message)
      }
    },
  })
}

async function handleCreate() {
  if (!newKeyData.value.key.trim()) {
    message.warning('请输入 Key 名称')
    return
  }
  
  try {
    let value: unknown = newKeyData.value.value
    
    if (newKeyData.value.type === 'hash') {
      try {
        value = JSON.parse(newKeyData.value.value)
      } catch {
        message.error('Hash 值必须是有效的 JSON 对象')
        return
      }
    } else if (newKeyData.value.type === 'list' || newKeyData.value.type === 'set') {
      try {
        value = JSON.parse(newKeyData.value.value)
        if (!Array.isArray(value)) {
          message.error('值必须是 JSON 数组')
          return
        }
      } catch {
        message.error('无效的 JSON 数组')
        return
      }
    } else if (newKeyData.value.type === 'zset') {
      try {
        value = JSON.parse(newKeyData.value.value)
      } catch {
        message.error('Sorted Set 值必须是有效的 JSON')
        return
      }
    }
    
    await store.setKey(
      newKeyData.value.key,
      newKeyData.value.type,
      value,
      newKeyData.value.ttl > 0 ? newKeyData.value.ttl : undefined
    )
    
    message.success(`"${newKeyData.value.key}" 创建成功`)
    showCreateModal.value = false
    newKeyData.value = { key: '', type: 'string', value: '', ttl: -1 }
  } catch (e) {
    message.error((e as Error).message)
  }
}

async function handleExport() {
  const keys = selectedKeys.value.length > 0
    ? selectedKeys.value
    : store.keys.map(k => k.key)

  if (keys.length === 0) {
    message.warning('没有可导出的 Key')
    return
  }

  try {
    const data = await redisApi.exportKeys(keys)
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'redis-export.json'
    a.click()
    URL.revokeObjectURL(url)
    message.success('导出完成')
  } catch (e) {
    message.error((e as Error).message)
  }
}

function handleLoadMore() {
  if (store.cursor > 0) {
    store.fetchKeys(searchPattern.value, store.cursor)
  }
}

onMounted(() => {
  if (store.keys.length === 0) {
    store.fetchKeys()
  }
})
</script>

<template>
  <div class="key-browser">
    <NCard class="glass-card">
      <template #header>
        <NSpace align="center" justify="space-between">
          <span class="title-font" style="font-size: 14px">Key 浏览器</span>
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
            <NButton 
              v-if="selectedKeys.length > 0"
              size="tiny" 
              type="error"
              @click="handleBatchDelete"
            >
              <template #icon>
                <NIcon size="14"><TrashOutline /></NIcon>
              </template>
              删除 ({{ selectedKeys.length }})
            </NButton>
            <NButton size="tiny" type="primary" @click="showCreateModal = true">
              <template #icon>
                <NIcon size="14"><AddOutline /></NIcon>
              </template>
              新建
            </NButton>
          </NSpace>
        </NSpace>
      </template>
      
      <!-- Search -->
      <NSpace class="search-section" :size="8">
        <NInput 
          v-model:value="searchPattern" 
          placeholder="搜索模式（如 user:*）"
          size="small"
          style="width: 240px"
          @keyup.enter="handleSearch"
        >
          <template #prefix>
            <NIcon size="14"><SearchOutline /></NIcon>
          </template>
        </NInput>
        <NButton size="small" @click="handleSearch">搜索</NButton>
      </NSpace>
      
      <!-- Keys Table -->
      <NDataTable
        :columns="columns"
        :data="store.keys"
        :loading="store.loading"
        :bordered="false"
        :row-key="(row: any) => row.key"
        size="small"
        striped
        @update:checked-row-keys="handleRowSelect"
      />
      
      <!-- Load More -->
      <NSpace v-if="store.cursor > 0" justify="center" style="margin-top: 12px">
        <NButton size="small" @click="handleLoadMore">加载更多</NButton>
      </NSpace>
    </NCard>
    
    <!-- Create Key Modal -->
    <NModal
      v-model:show="showCreateModal"
      title="新建 Key"
      preset="card"
      style="width: 500px"
    >
      <NForm size="small">
        <NFormItem label="Key 名称">
          <NInput v-model:value="newKeyData.key" placeholder="输入 Key 名称" />
        </NFormItem>
        
        <NFormItem label="类型">
          <NSelect v-model:value="newKeyData.type" :options="typeOptions" />
        </NFormItem>
        
        <NFormItem label="值">
          <NInput 
            v-model:value="newKeyData.value" 
            type="textarea"
            :placeholder="newKeyValuePlaceholder"
            :rows="3"
          />
        </NFormItem>
        
        <NFormItem label="TTL（秒，-1 表示永不过期）">
          <NInputNumber v-model:value="newKeyData.ttl" :min="-1" style="width: 100%" />
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
.key-browser {
  padding: 16px;
}

.search-section {
  margin-bottom: 12px;
}
</style>
