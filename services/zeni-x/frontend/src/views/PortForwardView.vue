<template>
  <div class="port-forward-view">
    <NCard title="端口转发管理" :bordered="false">
      <template #header-extra>
        <NSpace>
          <NButton
            type="primary"
            :loading="refreshing"
            @click="handleRefresh"
          >
            <template #icon>
              <NIcon><RefreshOutline /></NIcon>
            </template>
            刷新
          </NButton>
        </NSpace>
      </template>
      
      <!-- 统计信息 -->
      <NSpace :size="16" style="margin-bottom: 16px">
        <NStatistic label="总计" :value="stats.total" />
        <NStatistic label="活跃" :value="stats.active">
          <template #prefix>
            <NIcon color="#10B981"><CheckmarkCircleOutline /></NIcon>
          </template>
        </NStatistic>
        <NStatistic label="错误" :value="stats.error">
          <template #prefix>
            <NIcon color="#EF4444"><CloseCircleOutline /></NIcon>
          </template>
        </NStatistic>
        <NStatistic label="空闲" :value="stats.idle">
          <template #prefix>
            <NIcon color="#F59E0B"><TimeOutline /></NIcon>
          </template>
        </NStatistic>
      </NSpace>
      
      <!-- 转发列表 -->
      <NDataTable
        :columns="columns"
        :data="forwards"
        :loading="loading"
        :pagination="pagination"
        :row-key="(row: ForwardInfo) => row.id"
      />
    </NCard>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h, onMounted, onUnmounted } from 'vue'
import type { DataTableColumns } from 'naive-ui'
import {
  NCard,
  NSpace,
  NButton,
  NIcon,
  NDataTable,
  NStatistic,
  NTag,
  NTooltip,
  useMessage,
} from 'naive-ui'
import {
  RefreshOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  TimeOutline,
} from '@vicons/ionicons5'
import { portForwardApi, type ForwardInfo } from '@/api'

const message = useMessage()
const loading = ref(false)
const refreshing = ref(false)
const forwards = ref<ForwardInfo[]>([])
let intervalId: number | null = null

// 统计信息
const stats = computed(() => {
  const result = {
    total: forwards.value.length,
    active: 0,
    error: 0,
    idle: 0,
  }
  
  forwards.value.forEach(fwd => {
    if (fwd.status === 'active') result.active++
    else if (fwd.status === 'error') result.error++
    else if (fwd.status === 'idle') result.idle++
  })
  
  return result
})

// 分页配置
const pagination = {
  pageSize: 20,
}

// 表格列配置
const columns: DataTableColumns<ForwardInfo> = [
  {
    title: '转发ID',
    key: 'id',
    width: 280,
    ellipsis: {
      tooltip: true,
    },
    render: (row) => h('code', { style: 'font-size: 12px' }, row.id),
  },
  {
    title: '连接ID',
    key: 'connection_id',
    width: 80,
  },
  {
    title: '服务地址',
    key: 'remote_host',
    width: 260,
    ellipsis: {
      tooltip: true,
    },
    render: (row) => h(
      'span',
      { style: 'font-family: Monaco, monospace; font-size: 12px' },
      `${row.remote_host}:${row.remote_port}`
    ),
  },
  {
    title: '本地端口',
    key: 'local_port',
    width: 100,
    render: (row) => h(
      'span',
      { style: 'font-family: Monaco, monospace; font-size: 12px' },
      `localhost:${row.local_port}`
    ),
  },
  {
    title: '状态',
    key: 'status',
    width: 100,
    render: (row) => {
      const statusConfig = {
        active: { type: 'success' as const, text: '活跃' },
        error: { type: 'error' as const, text: '错误' },
        idle: { type: 'warning' as const, text: '空闲' },
      }
      const config = statusConfig[row.status] || { type: 'default' as const, text: row.status }
      
      if (row.status === 'error' && row.error_message) {
        return h(
          NTooltip,
          {},
          {
            trigger: () => h(NTag, { type: config.type, size: 'small' }, { default: () => config.text }),
            default: () => row.error_message,
          }
        )
      }
      
      return h(NTag, { type: config.type, size: 'small' }, { default: () => config.text })
    },
  },
  {
    title: '创建时间',
    key: 'created_at',
    width: 160,
    render: (row) => new Date(row.created_at).toLocaleString('zh-CN'),
  },
  {
    title: '最后使用',
    key: 'last_used_at',
    width: 160,
    render: (row) => new Date(row.last_used_at).toLocaleString('zh-CN'),
  },
  {
    title: '操作',
    key: 'actions',
    width: 150,
    fixed: 'right',
    render: (row) => {
      return h(NSpace, { size: 4 }, {
        default: () => [
          row.status === 'error' && h(
            NButton,
            {
              size: 'small',
              type: 'warning',
              quaternary: true,
              onClick: () => handleReconnect(row.id),
            },
            { default: () => '重连' }
          ),
          h(
            NButton,
            {
              size: 'small',
              type: 'error',
              quaternary: true,
              onClick: () => handleStop(row.id),
            },
            { default: () => '停止' }
          ),
        ],
      })
    },
  },
]

// 加载转发列表
async function loadForwards() {
  loading.value = true
  try {
    const response = await portForwardApi.list()
    forwards.value = response.data.forwards || []
  } catch (error: any) {
    message.error(`加载转发列表失败: ${error.message}`)
  } finally {
    loading.value = false
  }
}

// 刷新
async function handleRefresh() {
  refreshing.value = true
  await loadForwards()
  refreshing.value = false
  message.success('刷新成功')
}

// 重新连接
async function handleReconnect(id: string) {
  try {
    await portForwardApi.reconnect(id)
    message.success('重新连接成功')
    await loadForwards()
  } catch (error: any) {
    message.error(`重新连接失败: ${error.message}`)
  }
}

// 停止转发
async function handleStop(id: string) {
  try {
    await portForwardApi.stop(id)
    message.success('转发已停止')
    await loadForwards()
  } catch (error: any) {
    message.error(`停止转发失败: ${error.message}`)
  }
}

// 启动自动刷新
function startAutoRefresh() {
  intervalId = window.setInterval(() => {
    if (!loading.value) {
      loadForwards()
    }
  }, 10000) // 每10秒刷新一次
}

// 停止自动刷新
function stopAutoRefresh() {
  if (intervalId !== null) {
    clearInterval(intervalId)
    intervalId = null
  }
}

onMounted(() => {
  loadForwards()
  startAutoRefresh()
})

onUnmounted(() => {
  stopAutoRefresh()
})
</script>

<style scoped>
.port-forward-view {
  padding: 16px;
}
</style>

