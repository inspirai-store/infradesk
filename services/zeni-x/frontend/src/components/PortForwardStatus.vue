<template>
  <div class="port-forward-status">
    <NSpace :size="8" align="center">
      <!-- 状态图标 -->
      <NIcon :color="statusColor" :size="16">
        <component :is="statusIcon" />
      </NIcon>
      
      <!-- 状态文本 -->
      <span class="status-text" :style="{ color: statusColor }">
        {{ statusText }}
      </span>
      
      <!-- 端口信息 -->
      <span v-if="forwardInfo && forwardInfo.local_port" class="port-info">
        localhost:{{ forwardInfo.local_port }}
      </span>
      
      <!-- 重连按钮 -->
      <NButton
        v-if="forwardInfo && forwardInfo.status === 'error'"
        size="tiny"
        type="warning"
        quaternary
        :loading="reconnecting"
        @click="handleReconnect"
      >
        重连
      </NButton>
      
      <!-- 停止按钮 -->
      <NButton
        v-if="forwardInfo"
        size="tiny"
        type="error"
        quaternary
        :loading="stopping"
        @click="handleStop"
      >
        停止
      </NButton>
      
      <!-- 错误提示 -->
      <NTooltip v-if="forwardInfo && forwardInfo.error_message" placement="top">
        <template #trigger>
          <NIcon color="#EF4444" :size="16">
            <AlertCircleOutline />
          </NIcon>
        </template>
        {{ forwardInfo.error_message }}
      </NTooltip>
    </NSpace>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { NIcon, NSpace, NButton, NTooltip, useMessage } from 'naive-ui'
import {
  CheckmarkCircleOutline,
  CloseCircleOutline,
  TimeOutline,
  AlertCircleOutline,
} from '@vicons/ionicons5'
import { portForwardApi, type ForwardInfo } from '@/api'

const props = defineProps<{
  connectionId: number
  autoRefresh?: boolean
  refreshInterval?: number
}>()

const message = useMessage()
const forwardInfo = ref<ForwardInfo | null>(null)
const loading = ref(false)
const reconnecting = ref(false)
const stopping = ref(false)
let intervalId: number | null = null

// 状态颜色
const statusColor = computed(() => {
  if (!forwardInfo.value) return '#9CA3AF'
  switch (forwardInfo.value.status) {
    case 'active':
      return '#10B981'
    case 'error':
      return '#EF4444'
    case 'idle':
      return '#F59E0B'
    default:
      return '#9CA3AF'
  }
})

// 状态图标
const statusIcon = computed(() => {
  if (!forwardInfo.value) return TimeOutline
  switch (forwardInfo.value.status) {
    case 'active':
      return CheckmarkCircleOutline
    case 'error':
      return CloseCircleOutline
    case 'idle':
      return TimeOutline
    default:
      return TimeOutline
  }
})

// 状态文本
const statusText = computed(() => {
  if (!forwardInfo.value) return '未转发'
  switch (forwardInfo.value.status) {
    case 'active':
      return '已转发'
    case 'error':
      return '转发错误'
    case 'idle':
      return '空闲'
    default:
      return '未知'
  }
})

// 加载转发信息
async function loadForwardInfo() {
  if (loading.value) return

  try {
    loading.value = true
    const data = await portForwardApi.getByConnection(props.connectionId)
    forwardInfo.value = data
  } catch (error: any) {
    // 404 表示没有转发，这是正常的
    if (error.response?.status !== 404) {
      console.error('Failed to load forward info:', error)
    }
    forwardInfo.value = null
  } finally {
    loading.value = false
  }
}

// 重新连接
async function handleReconnect() {
  if (!forwardInfo.value) return

  reconnecting.value = true
  try {
    const data = await portForwardApi.reconnect(forwardInfo.value.id)
    forwardInfo.value = data
    message.success('端口转发已重新连接')
  } catch (error: any) {
    message.error(`重新连接失败: ${error.message}`)
  } finally {
    reconnecting.value = false
  }
}

// 停止转发
async function handleStop() {
  if (!forwardInfo.value) return
  
  stopping.value = true
  try {
    await portForwardApi.stop(forwardInfo.value.id)
    forwardInfo.value = null
    message.success('端口转发已停止')
  } catch (error: any) {
    message.error(`停止转发失败: ${error.message}`)
  } finally {
    stopping.value = false
  }
}

// 启动自动刷新
function startAutoRefresh() {
  if (!props.autoRefresh) return
  
  const interval = props.refreshInterval || 5000
  intervalId = window.setInterval(() => {
    loadForwardInfo()
  }, interval)
}

// 停止自动刷新
function stopAutoRefresh() {
  if (intervalId !== null) {
    clearInterval(intervalId)
    intervalId = null
  }
}

// 监听连接ID变化
watch(() => props.connectionId, () => {
  loadForwardInfo()
})

onMounted(() => {
  loadForwardInfo()
  if (props.autoRefresh) {
    startAutoRefresh()
  }
})

onUnmounted(() => {
  stopAutoRefresh()
})
</script>

<style scoped>
.port-forward-status {
  display: inline-flex;
  align-items: center;
}

.status-text {
  font-size: 13px;
  font-weight: 500;
}

.port-info {
  font-size: 12px;
  color: #6B7280;
  font-family: 'Monaco', 'Courier New', monospace;
}
</style>

