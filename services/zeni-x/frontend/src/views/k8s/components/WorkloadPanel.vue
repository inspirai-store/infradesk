<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NDataTable,
  NCard,
  NSpace,
  NTag,
  NSpin,
  NEmpty,
  useMessage
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { api } from '@/api/adapter'
import type { K8sDeployment, K8sPod } from '@/api/types'

const props = defineProps<{
  clusterId: number
  namespace: string
}>()

const message = useMessage()

// State
const deployments = ref<K8sDeployment[]>([])
const pods = ref<K8sPod[]>([])
const deploymentsLoading = ref(false)
const podsLoading = ref(false)

// Deployment columns
const deploymentColumns: DataTableColumns<K8sDeployment> = [
  { title: 'Name', key: 'name', width: 200, ellipsis: { tooltip: true } },
  {
    title: 'Replicas',
    key: 'replicas',
    width: 120,
    render(row) {
      const ready = row.ready_replicas || 0
      const total = row.replicas || 0
      const color = ready === total ? 'success' : ready > 0 ? 'warning' : 'error'
      return h(NTag, { type: color, size: 'small' }, () => `${ready}/${total}`)
    }
  },
  {
    title: 'Available',
    key: 'available_replicas',
    width: 100,
    render(row) {
      return row.available_replicas || 0
    }
  },
  {
    title: 'Labels',
    key: 'labels',
    ellipsis: { tooltip: true },
    render(row) {
      const labels = row.labels || {}
      const entries = Object.entries(labels).slice(0, 3)
      return entries.map(([k, v]) => `${k}=${v}`).join(', ')
    }
  },
  {
    title: 'Created',
    key: 'created_at',
    width: 160,
    render(row) {
      if (!row.created_at) return '-'
      return new Date(row.created_at).toLocaleString()
    }
  }
]

// Pod columns
const podColumns: DataTableColumns<K8sPod> = [
  { title: 'Name', key: 'name', width: 280, ellipsis: { tooltip: true } },
  {
    title: 'Status',
    key: 'status',
    width: 100,
    render(row) {
      const statusColors: Record<string, string> = {
        Running: 'success',
        Pending: 'warning',
        Succeeded: 'info',
        Failed: 'error',
        Unknown: 'default'
      }
      const color = statusColors[row.status] || 'default'
      return h(NTag, { type: color as any, size: 'small' }, () => row.status)
    }
  },
  { title: 'Ready', key: 'ready', width: 80 },
  { title: 'Restarts', key: 'restarts', width: 80 },
  { title: 'Node', key: 'node', width: 150, ellipsis: { tooltip: true } },
  { title: 'IP', key: 'ip', width: 130 },
  {
    title: 'Created',
    key: 'created_at',
    width: 160,
    render(row) {
      if (!row.created_at) return '-'
      return new Date(row.created_at).toLocaleString()
    }
  }
]

// Methods
async function fetchDeployments() {
  deploymentsLoading.value = true
  try {
    deployments.value = await api.k8s.listDeployments(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch deployments: ' + (error as Error).message)
    deployments.value = []
  } finally {
    deploymentsLoading.value = false
  }
}

async function fetchPods() {
  podsLoading.value = true
  try {
    pods.value = await api.k8s.listPods(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch pods: ' + (error as Error).message)
    pods.value = []
  } finally {
    podsLoading.value = false
  }
}

async function refresh() {
  await Promise.all([fetchDeployments(), fetchPods()])
}

// Watch for prop changes
watch(
  () => [props.clusterId, props.namespace],
  () => {
    refresh()
  },
  { immediate: true }
)

// For h() render function
import { h } from 'vue'
</script>

<template>
  <NSpace vertical :size="16">
    <!-- Deployments -->
    <NCard title="Deployments" size="small">
      <NSpin :show="deploymentsLoading">
        <NDataTable
          v-if="deployments.length > 0"
          :columns="deploymentColumns"
          :data="deployments"
          :bordered="false"
          size="small"
          max-height="300"
        />
        <NEmpty v-else description="No deployments found" />
      </NSpin>
    </NCard>

    <!-- Pods -->
    <NCard title="Pods" size="small">
      <NSpin :show="podsLoading">
        <NDataTable
          v-if="pods.length > 0"
          :columns="podColumns"
          :data="pods"
          :bordered="false"
          size="small"
          max-height="400"
        />
        <NEmpty v-else description="No pods found" />
      </NSpin>
    </NCard>
  </NSpace>
</template>
