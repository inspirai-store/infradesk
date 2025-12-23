<script setup lang="ts">
import { onMounted, computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { 
  NCard, 
  NSpace, 
  NButton, 
  NIcon, 
  NBreadcrumb,
  NBreadcrumbItem,
  NTag,
  NDescriptions,
  NDescriptionsItem,
  NInput,
  NInputNumber,
  NDataTable,
  NModal,
  NForm,
  NFormItem,
  useMessage,
  useDialog,
} from 'naive-ui'
import { 
  SaveOutline, 
  TrashOutline, 
  RefreshOutline,
  TimeOutline,
} from '@vicons/ionicons5'
import { useRedisStore } from '@/stores/redis'
import type { DataTableColumns } from 'naive-ui'

const route = useRoute()
const router = useRouter()
const store = useRedisStore()
const message = useMessage()
const dialog = useDialog()

const key = computed(() => decodeURIComponent(route.params.key as string))
const editValue = ref('')
const showTTLModal = ref(false)
const newTTL = ref(-1)

const typeColors: Record<string, string> = {
  string: 'success',
  hash: 'warning',
  list: 'info',
  set: 'error',
  zset: 'default',
}

const hashColumns: DataTableColumns<{ field: string; value: string }> = [
  { title: '字段', key: 'field' },
  { title: '值', key: 'value', ellipsis: { tooltip: true } },
]

const listColumns: DataTableColumns<{ index: number; value: string }> = [
  { title: '索引', key: 'index', width: 60 },
  { title: '值', key: 'value', ellipsis: { tooltip: true } },
]

const zsetColumns: DataTableColumns<{ member: string; score: number }> = [
  { title: '分数', key: 'score', width: 80 },
  { title: '成员', key: 'member', ellipsis: { tooltip: true } },
]

const hashData = computed(() => {
  if (store.currentKey?.type !== 'hash' || !store.currentKey.value) return []
  const val = store.currentKey.value as Record<string, string>
  return Object.entries(val).map(([field, value]) => ({ field, value }))
})

const listData = computed(() => {
  if (!['list', 'set'].includes(store.currentKey?.type || '') || !store.currentKey?.value) return []
  const val = store.currentKey.value as string[]
  return val.map((value, index) => ({ index, value }))
})

const zsetData = computed(() => {
  if (store.currentKey?.type !== 'zset' || !store.currentKey.value) return []
  const val = store.currentKey.value as Array<{ Score: number; Member: string }>
  return val.map(item => ({ score: item.Score, member: item.Member }))
})

function formatTTL(seconds: number): string {
  if (seconds < 0) return '永不过期'
  if (seconds < 60) return `${seconds} 秒`
  if (seconds < 3600) return `${Math.floor(seconds / 60)} 分钟`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)} 小时`
  return `${Math.floor(seconds / 86400)} 天`
}

async function handleSave() {
  if (!store.currentKey) return
  
  try {
    let value: unknown = editValue.value
    
    if (store.currentKey.type !== 'string') {
      try {
        value = JSON.parse(editValue.value)
      } catch {
        message.error('无效的 JSON')
        return
      }
    }
    
    await store.setKey(key.value, store.currentKey.type, value)
    message.success('保存成功')
    store.fetchKey(key.value)
  } catch (e) {
    message.error((e as Error).message)
  }
}

async function handleDelete() {
  dialog.warning({
    title: '删除 Key',
    content: `确定要删除 "${key.value}" 吗？`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await store.deleteKey(key.value)
        message.success('删除成功')
        router.push('/redis')
      } catch (e) {
        message.error((e as Error).message)
      }
    },
  })
}

async function handleSetTTL() {
  try {
    await store.setTTL(key.value, newTTL.value)
    message.success('TTL 已更新')
    showTTLModal.value = false
    store.fetchKey(key.value)
  } catch (e) {
    message.error((e as Error).message)
  }
}

function handleRefresh() {
  store.fetchKey(key.value)
  message.success('已刷新')
}

watch(() => store.currentKey, (newKey) => {
  if (newKey) {
    if (newKey.type === 'string') {
      editValue.value = newKey.value as string
    } else {
      editValue.value = JSON.stringify(newKey.value, null, 2)
    }
    newTTL.value = newKey.ttl
  }
}, { immediate: true })

onMounted(() => {
  store.fetchKey(key.value)
})
</script>

<template>
  <div class="key-detail">
    <!-- Breadcrumb -->
    <NBreadcrumb class="breadcrumb">
      <NBreadcrumbItem @click="router.push('/redis')">Redis</NBreadcrumbItem>
      <NBreadcrumbItem>{{ key }}</NBreadcrumbItem>
    </NBreadcrumb>
    
    <NCard v-if="store.currentKey" class="glass-card">
      <template #header>
        <NSpace align="center" justify="space-between">
          <NSpace align="center" :size="8">
            <span class="title-font" style="font-size: 13px; max-width: 300px; overflow: hidden; text-overflow: ellipsis;">{{ key }}</span>
            <NTag :type="typeColors[store.currentKey.type]" size="tiny">
              {{ store.currentKey.type.toUpperCase() }}
            </NTag>
          </NSpace>
          <NSpace :size="4">
            <NButton size="tiny" @click="handleRefresh">
              <template #icon>
                <NIcon size="14"><RefreshOutline /></NIcon>
              </template>
            </NButton>
            <NButton size="tiny" @click="showTTLModal = true">
              <template #icon>
                <NIcon size="14"><TimeOutline /></NIcon>
              </template>
              TTL
            </NButton>
            <NButton size="tiny" type="primary" @click="handleSave">
              <template #icon>
                <NIcon size="14"><SaveOutline /></NIcon>
              </template>
              保存
            </NButton>
            <NButton size="tiny" type="error" @click="handleDelete">
              <template #icon>
                <NIcon size="14"><TrashOutline /></NIcon>
              </template>
              删除
            </NButton>
          </NSpace>
        </NSpace>
      </template>
      
      <!-- Key Info -->
      <NDescriptions :column="3" class="key-info" size="small">
        <NDescriptionsItem label="类型">
          {{ store.currentKey.type }}
        </NDescriptionsItem>
        <NDescriptionsItem label="过期时间">
          {{ formatTTL(store.currentKey.ttl) }}
        </NDescriptionsItem>
        <NDescriptionsItem v-if="store.currentKey.type !== 'string'" label="元素数">
          {{ Array.isArray(store.currentKey.value) 
              ? store.currentKey.value.length 
              : typeof store.currentKey.value === 'object' 
                ? Object.keys(store.currentKey.value as object).length 
                : 0 
          }}
        </NDescriptionsItem>
      </NDescriptions>
      
      <!-- Value Editor -->
      <div class="value-section">
        <!-- String -->
        <template v-if="store.currentKey.type === 'string'">
          <NInput
            v-model:value="editValue"
            type="textarea"
            :rows="8"
            placeholder="输入值"
            class="code-editor"
          />
        </template>
        
        <!-- Hash -->
        <template v-else-if="store.currentKey.type === 'hash'">
          <NDataTable
            :columns="hashColumns"
            :data="hashData"
            :bordered="false"
            size="small"
            striped
            :max-height="240"
          />
          <NInput
            v-model:value="editValue"
            type="textarea"
            :rows="4"
            placeholder='{"field": "value"}'
            class="code-editor"
            style="margin-top: 12px"
          />
        </template>
        
        <!-- List/Set -->
        <template v-else-if="['list', 'set'].includes(store.currentKey.type)">
          <NDataTable
            :columns="listColumns"
            :data="listData"
            :bordered="false"
            size="small"
            striped
            :max-height="240"
          />
          <NInput
            v-model:value="editValue"
            type="textarea"
            :rows="4"
            placeholder='["item1", "item2"]'
            class="code-editor"
            style="margin-top: 12px"
          />
        </template>
        
        <!-- ZSet -->
        <template v-else-if="store.currentKey.type === 'zset'">
          <NDataTable
            :columns="zsetColumns"
            :data="zsetData"
            :bordered="false"
            size="small"
            striped
            :max-height="240"
          />
          <NInput
            v-model:value="editValue"
            type="textarea"
            :rows="4"
            placeholder='[{"member": "a", "score": 1}]'
            class="code-editor"
            style="margin-top: 12px"
          />
        </template>
      </div>
    </NCard>
    
    <!-- Loading State -->
    <NCard v-else-if="store.loading" class="glass-card">
      <div class="loading">加载中...</div>
    </NCard>
    
    <!-- Error State -->
    <NCard v-else class="glass-card">
      <div class="error">Key 不存在</div>
    </NCard>
    
    <!-- TTL Modal -->
    <NModal
      v-model:show="showTTLModal"
      title="设置 TTL"
      preset="card"
      style="width: 360px"
    >
      <NForm size="small">
        <NFormItem label="TTL（秒，-1 表示永不过期）">
          <NInputNumber v-model:value="newTTL" :min="-1" style="width: 100%" />
        </NFormItem>
      </NForm>
      
      <template #footer>
        <NSpace justify="end" :size="8">
          <NButton size="small" @click="showTTLModal = false">取消</NButton>
          <NButton size="small" type="primary" @click="handleSetTTL">确定</NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>

<style scoped>
.key-detail {
  padding: 16px;
}

.breadcrumb {
  margin-bottom: 12px;
}

.key-info {
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--zx-border);
}

.value-section {
  margin-top: 12px;
}

.code-editor {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
}

.loading, .error {
  padding: 32px;
  text-align: center;
  color: var(--zx-text-secondary);
  font-size: 12px;
}
</style>
