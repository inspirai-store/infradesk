<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import {
  NSpace,
  NTabs,
  NTabPane,
  NList,
  NListItem,
  NText,
  NButton,
  NIcon,
  NTooltip,
  NTag,
  NInput,
  NSelect,
  NSpin,
  NEmpty,
  useMessage,
  NPopconfirm,
  NModal,
  NForm,
  NFormItem,
  NDivider,
} from 'naive-ui'
import {
  TimeOutline,
  TrashOutline,
  StarOutline,
  Star,
  RefreshOutline,
  SearchOutline,
} from '@vicons/ionicons5'
import { useHistoryStore } from '@/stores/history'
import { useMySQLStore } from '@/stores/mysql'
import { getActiveConnectionId } from '@/api'

interface Props {
  show: boolean
  onLoadQuery?: (query: string) => void
}

interface Emits {
  (e: 'update:show', value: boolean): void
}

const props = withDefaults(defineProps<Props>(), {
  show: false
})

const emit = defineEmits<Emits>()

const historyStore = useHistoryStore()
const mysqlStore = useMySQLStore()
const message = useMessage()

// 本地状态
const activeTab = ref<'history' | 'saved'>('history')
const searchKeyword = ref('')
const statusFilter = ref<string | null>(null)
const typeFilter = ref<string | null>(null)
const categoryFilter = ref<string | null>(null)

// 收藏相关
const showSaveModal = ref(false)
const saveForm = ref({
  name: '',
  description: '',
  category: ''
})

// 状态选项
const statusOptions = [
  { label: '成功', value: 'success' },
  { label: '失败', value: 'error' }
]

const typeOptions = [
  { label: 'SELECT', value: 'SELECT' },
  { label: 'INSERT', value: 'INSERT' },
  { label: 'UPDATE', value: 'UPDATE' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'DDL', value: 'DDL' }
]

// 计算属性：过滤后的历史记录
const filteredHistory = computed(() => {
  let result = historyStore.history

  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase()
    result = result.filter(h =>
      h.query_text.toLowerCase().includes(keyword) ||
      h.database.toLowerCase().includes(keyword)
    )
  }

  if (statusFilter.value) {
    result = result.filter(h => h.status === statusFilter.value)
  }

  if (typeFilter.value) {
    result = result.filter(h => h.query_type === typeFilter.value)
  }

  return result
})

// 计算属性：过滤后的收藏查询
const filteredSavedQueries = computed(() => {
  let result = historyStore.savedQueries

  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase()
    result = result.filter(q =>
      q.name.toLowerCase().includes(keyword) ||
      q.query_text.toLowerCase().includes(keyword) ||
      (q.description && q.description.toLowerCase().includes(keyword))
    )
  }

  if (categoryFilter.value) {
    result = result.filter(q => q.category === categoryFilter.value)
  }

  return result
})

// 分类选项
const categoryOptions = computed(() => {
  return historyStore.categories.map(cat => ({ label: cat, value: cat }))
})

// 格式化时间
function formatTime(isoString: string): string {
  const date = new Date(isoString)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)

  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes} 分钟前`
  if (hours < 24) return `${hours} 小时前`
  if (days < 7) return `${days} 天前`

  return date.toLocaleDateString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 获取查询类型标签类型
function getQueryTypeTag(type: string): 'default' | 'info' | 'success' | 'warning' | 'error' {
  switch (type) {
    case 'SELECT': return 'info'
    case 'INSERT': return 'success'
    case 'UPDATE': return 'warning'
    case 'DELETE': return 'error'
    default: return 'default'
  }
}

// 加载查询到编辑器
function loadQuery(queryText: string) {
  if (props.onLoadQuery) {
    props.onLoadQuery(queryText)
    emit('update:show', false)
  }
}

// 删除历史记录
async function deleteHistoryItem(id: number) {
  try {
    await historyStore.deleteHistoryItem(id)
    message.success('已删除')
  } catch (e) {
    message.error((e as Error).message)
  }
}

// 刷新历史
async function refreshHistory() {
  await historyStore.fetchHistory(true)
}

// 保存为收藏
function openSaveModal(queryText: string) {
  saveForm.value = {
    name: '',
    description: '',
    category: categoryFilter.value || ''
  }
  // 存储当前查询文本
  ;(window as any).__pendingSaveQuery = queryText
  showSaveModal.value = true
}

async function confirmSave() {
  const queryText = (window as any).__pendingSaveQuery
  if (!queryText) return

  if (!saveForm.value.name.trim()) {
    message.warning('请输入名称')
    return
  }

  const connectionId = getActiveConnectionId('mysql')
  if (!connectionId) {
    message.error('未选择连接')
    return
  }

  try {
    await historyStore.createSavedQuery({
      connection_id: connectionId,
      database: mysqlStore.currentDatabase,
      name: saveForm.value.name,
      query_text: queryText,
      description: saveForm.value.description || undefined,
      category: saveForm.value.category || undefined
    })
    message.success('已保存到收藏')
    showSaveModal.value = false
    delete (window as any).__pendingSaveQuery
  } catch (e) {
    message.error((e as Error).message)
  }
}

// 删除收藏
async function deleteSavedQuery(id: number) {
  try {
    await historyStore.deleteSavedQueryItem(id)
    message.success('已删除')
  } catch (e) {
    message.error((e as Error).message)
  }
}

// 清理旧历史
async function cleanupOldHistory() {
  try {
    const deleted = await historyStore.cleanupOldHistory(30) // 清理 30 天前的
    message.success(`已清理 ${deleted} 条历史记录`)
  } catch (e) {
    message.error((e as Error).message)
  }
}

// 监听显示状态变化，加载数据
watch(() => props.show, (show) => {
  if (show) {
    historyStore.fetchHistory()
    historyStore.fetchSavedQueries()
  }
})

onMounted(() => {
  if (props.show) {
    historyStore.fetchHistory()
    historyStore.fetchSavedQueries()
  }
})
</script>

<template>
  <div class="history-panel-container">
    <NModal
      :show="show"
      :mask-closable="true"
      @update:show="emit('update:show', $event)"
      preset="card"
      :style="{ width: '520px', height: '640px' }"
      :bordered="false"
      :segmented="{ content: 'soft' }"
    >
      <template #header>
        <NSpace align="center" justify="space-between" style="width: 100%">
          <span class="title-font neon-text">查询历史 & 收藏</span>
          <NSpace :size="8">
            <NTooltip placement="bottom">
              <template #trigger>
                <NButton size="tiny" quaternary circle @click="refreshHistory">
                  <template #icon><NIcon size="14"><RefreshOutline /></NIcon></template>
                </NButton>
              </template>
              刷新
            </NTooltip>
          </NSpace>
        </NSpace>
      </template>

      <NTabs v-model:value="activeTab" type="line" size="small">
        <!-- 历史记录 Tab -->
        <NTabPane name="history" tab="历史记录">
          <NSpace vertical :size="12">
            <!-- 搜索和过滤 -->
            <NSpace :size="8">
              <NInput
                v-model:value="searchKeyword"
                placeholder="搜索查询或数据库"
                size="small"
                clearable
                style="flex: 1"
              >
                <template #prefix>
                  <NIcon size="14"><SearchOutline /></NIcon>
                </template>
              </NInput>
              <NSelect
                v-model:value="statusFilter"
                :options="statusOptions"
                placeholder="状态"
                size="small"
                clearable
                style="width: 80px"
              />
              <NSelect
                v-model:value="typeFilter"
                :options="typeOptions"
                placeholder="类型"
                size="small"
                clearable
                style="width: 90px"
              />
            </NSpace>

            <!-- 历史记录列表 -->
            <NSpin :show="historyStore.loading">
              <div class="history-list">
                <NEmpty v-if="filteredHistory.length === 0 && !historyStore.loading" description="暂无历史记录" size="small" />

                <NList v-else hoverable clickable bordered size="small" style="--n-item-text-color: var(--zx-text-2)">
                  <NListItem v-for="item in filteredHistory" :key="item.id">
                    <template #prefix>
                      <NIcon
                        size="16"
                        :color="item.status === 'success' ? 'var(--zx-success)' : 'var(--zx-error)'"
                      >
                        <TimeOutline />
                      </NIcon>
                    </template>

                    <div class="history-item" @click="loadQuery(item.query_text)">
                      <NSpace vertical :size="4">
                        <NSpace align="center" :size="8">
                          <NTag :type="getQueryTypeTag(item.query_type)" size="tiny" round>
                            {{ item.query_type }}
                          </NTag>
                          <NText depth="3" style="font-size: 11px">{{ item.database }}</NText>
                          <NText depth="3" style="font-size: 11px">{{ formatTime(item.executed_at) }}</NText>
                          <NText depth="3" style="font-size: 11px">{{ item.duration_ms }}ms</NText>
                        </NSpace>
                        <NText
                          :line-clamp="2"
                          style="font-size: 12px; font-family: 'JetBrains Mono', monospace; word-break: break-all"
                        >
                          {{ item.query_text }}
                        </NText>
                        <NText v-if="item.error_message" type="error" style="font-size: 11px">
                          {{ item.error_message }}
                        </NText>
                      </NSpace>
                    </div>

                    <template #suffix>
                      <NSpace :size="4">
                        <NTooltip placement="left">
                          <template #trigger>
                            <NButton
                              size="tiny"
                              quaternary
                              @click.stop="openSaveModal(item.query_text)"
                            >
                              <template #icon><NIcon size="14"><StarOutline /></NIcon></template>
                            </NButton>
                          </template>
                          保存为收藏
                        </NTooltip>
                        <NPopconfirm @positive-click.stop="deleteHistoryItem(item.id)">
                          <template #trigger>
                            <NButton size="tiny" quaternary type="error">
                              <template #icon><NIcon size="14"><TrashOutline /></NIcon></template>
                            </NButton>
                          </template>
                          确认删除此历史记录？
                        </NPopconfirm>
                      </NSpace>
                    </template>
                  </NListItem>
                </NList>
              </div>
            </NSpin>
          </NSpace>
        </NTabPane>

        <!-- 收藏查询 Tab -->
        <NTabPane name="saved" tab="收藏查询">
          <NSpace vertical :size="12">
            <!-- 搜索和过滤 -->
            <NSpace :size="8" align="center" justify="space-between">
              <NSpace :size="8" style="flex: 1">
                <NInput
                  v-model:value="searchKeyword"
                  placeholder="搜索名称或查询"
                  size="small"
                  clearable
                >
                  <template #prefix>
                    <NIcon size="14"><SearchOutline /></NIcon>
                  </template>
                </NInput>
                <NSelect
                  v-model:value="categoryFilter"
                  :options="categoryOptions"
                  placeholder="分类"
                  size="small"
                  clearable
                  style="width: 100px"
                />
              </NSpace>
              <NTooltip placement="bottom">
                <template #trigger>
                  <NButton size="tiny" quaternary @click="cleanupOldHistory">
                    <template #icon><NIcon size="14"><TrashOutline /></NIcon></template>
                    清理旧历史
                  </NButton>
                </template>
                清理 30 天前的历史记录
              </NTooltip>
            </NSpace>

            <!-- 收藏查询列表 -->
            <NSpin :show="historyStore.loading">
              <div class="saved-list">
                <NEmpty v-if="filteredSavedQueries.length === 0 && !historyStore.loading" description="暂无收藏查询" size="small" />

                <NList v-else hoverable clickable bordered size="small">
                  <NListItem v-for="item in filteredSavedQueries" :key="item.id">
                    <template #prefix>
                      <NIcon size="16" color="var(--zx-accent-yellow)">
                        <Star />
                      </NIcon>
                    </template>

                    <div class="saved-item" @click="loadQuery(item.query_text)">
                      <NSpace vertical :size="4">
                        <NSpace align="center" :size="8">
                          <NText strong style="font-size: 13px">{{ item.name }}</NText>
                          <NTag v-if="item.category" size="tiny" type="info" round>
                            {{ item.category }}
                          </NTag>
                        </NSpace>
                        <NText
                          :line-clamp="2"
                          style="font-size: 12px; font-family: 'JetBrains Mono', monospace; word-break: break-all"
                        >
                          {{ item.query_text }}
                        </NText>
                        <NText v-if="item.description" depth="3" style="font-size: 11px" :line-clamp="1">
                          {{ item.description }}
                        </NText>
                        <NText depth="3" style="font-size: 10px">
                          {{ item.database }} · {{ formatTime(item.created_at || '') }}
                        </NText>
                      </NSpace>
                    </div>

                    <template #suffix>
                      <NPopconfirm @positive-click.stop="deleteSavedQuery(item.id!)">
                        <template #trigger>
                          <NButton size="tiny" quaternary type="error">
                            <template #icon><NIcon size="14"><TrashOutline /></NIcon></template>
                          </NButton>
                        </template>
                        确认删除此收藏？
                      </NPopconfirm>
                    </template>
                  </NListItem>
                </NList>
              </div>
            </NSpin>
          </NSpace>
        </NTabPane>
      </NTabs>
    </NModal>

    <!-- 保存收藏对话框 -->
    <NModal
      v-model:show="showSaveModal"
      preset="card"
      title="保存为收藏"
      :style="{ width: '420px' }"
      :bordered="false"
    >
      <NForm label-placement="left" label-width="60px" :show-feedback="false">
        <NFormItem label="名称">
          <NInput v-model:value="saveForm.name" placeholder="给查询起个名字" />
        </NFormItem>
        <NFormItem label="分类">
          <NInput v-model:value="saveForm.category" placeholder="可选，如：报表、运维" />
        </NFormItem>
        <NFormItem label="描述">
          <NInput
            v-model:value="saveForm.description"
            type="textarea"
            placeholder="可选，添加描述"
            :autosize="{ minRows: 2, maxRows: 4 }"
          />
        </NFormItem>
      </NForm>
      <NDivider style="margin: 12px 0" />
      <NSpace justify="end">
        <NButton size="small" @click="showSaveModal = false">取消</NButton>
        <NButton size="small" type="primary" @click="confirmSave">保存</NButton>
      </NSpace>
    </NModal>
  </div>
</template>

<style scoped>
.history-panel-container :deep(.n-card__content) {
  padding: 12px;
}

.history-list,
.saved-list {
  max-height: 480px;
  overflow-y: auto;
}

.history-list::-webkit-scrollbar,
.saved-list::-webkit-scrollbar {
  width: 6px;
}

.history-list::-webkit-scrollbar-thumb,
.saved-list::-webkit-scrollbar-thumb {
  background: var(--zx-border);
  border-radius: 3px;
}

.history-list::-webkit-scrollbar-thumb:hover,
.saved-list::-webkit-scrollbar-thumb:hover {
  background: var(--zx-text-3);
}

.history-item,
.saved-item {
  cursor: pointer;
  transition: opacity 0.2s;
  width: 100%;
}

.history-item:hover,
.saved-item:hover {
  opacity: 0.8;
}
</style>
