<script setup lang="ts">
import { ref, watch, h } from 'vue'
import {
  NDataTable,
  NSpace,
  NTag,
  NSpin,
  NEmpty,
  NButton,
  NIcon,
  NCollapse,
  NCollapseItem,
  NBadge,
  useMessage
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { RefreshOutline } from '@vicons/ionicons5'
import { api } from '@/api/adapter'
import type { K8sDeployment, K8sStatefulSet, K8sDaemonSet, K8sJob, K8sCronJob, K8sReplicaSet } from '@/api/types'

const props = defineProps<{
  clusterId: number
  namespace: string
}>()

const message = useMessage()

// State
const deployments = ref<K8sDeployment[]>([])
const statefulsets = ref<K8sStatefulSet[]>([])
const daemonsets = ref<K8sDaemonSet[]>([])
const jobs = ref<K8sJob[]>([])
const cronjobs = ref<K8sCronJob[]>([])
const replicasets = ref<K8sReplicaSet[]>([])

const deploymentsLoading = ref(false)
const statefulsetsLoading = ref(false)
const daemonsetsLoading = ref(false)
const jobsLoading = ref(false)
const cronjobsLoading = ref(false)
const replicasetsLoading = ref(false)

// Collapse expanded state
const expandedNames = ref<string[]>(['deployments'])

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

// StatefulSet columns
const statefulsetColumns: DataTableColumns<K8sStatefulSet> = [
  { title: 'Name', key: 'name', width: 200, ellipsis: { tooltip: true } },
  {
    title: 'Ready',
    key: 'ready_replicas',
    width: 100,
    render(row) {
      const ready = row.ready_replicas || 0
      const total = row.replicas || 0
      const color = ready === total ? 'success' : ready > 0 ? 'warning' : 'error'
      return h(NTag, { type: color, size: 'small' }, () => `${ready}/${total}`)
    }
  },
  { title: 'Current', key: 'current_replicas', width: 80 },
  { title: 'Updated', key: 'updated_replicas', width: 80 },
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

// DaemonSet columns
const daemonsetColumns: DataTableColumns<K8sDaemonSet> = [
  { title: 'Name', key: 'name', width: 200, ellipsis: { tooltip: true } },
  {
    title: 'Ready',
    key: 'number_ready',
    width: 100,
    render(row) {
      const ready = row.number_ready || 0
      const desired = row.desired_number_scheduled || 0
      const color = ready === desired ? 'success' : ready > 0 ? 'warning' : 'error'
      return h(NTag, { type: color, size: 'small' }, () => `${ready}/${desired}`)
    }
  },
  { title: 'Current', key: 'current_number_scheduled', width: 80 },
  { title: 'Available', key: 'number_available', width: 80 },
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

// Job columns
const jobColumns: DataTableColumns<K8sJob> = [
  { title: 'Name', key: 'name', width: 200, ellipsis: { tooltip: true } },
  {
    title: 'Status',
    key: 'status',
    width: 120,
    render(row) {
      const succeeded = row.succeeded || 0
      const failed = row.failed || 0
      const active = row.active || 0
      const completions = row.completions || 1

      if (succeeded >= completions) {
        return h(NTag, { type: 'success', size: 'small' }, () => 'Complete')
      } else if (failed > 0) {
        return h(NTag, { type: 'error', size: 'small' }, () => `Failed: ${failed}`)
      } else if (active > 0) {
        return h(NTag, { type: 'info', size: 'small' }, () => `Running: ${active}`)
      }
      return h(NTag, { type: 'default', size: 'small' }, () => 'Pending')
    }
  },
  {
    title: 'Progress',
    key: 'progress',
    width: 100,
    render(row) {
      const succeeded = row.succeeded || 0
      const completions = row.completions || 1
      return `${succeeded}/${completions}`
    }
  },
  {
    title: 'Duration',
    key: 'duration',
    width: 120,
    render(row) {
      if (!row.start_time) return '-'
      const start = new Date(row.start_time)
      const end = row.completion_time ? new Date(row.completion_time) : new Date()
      const duration = Math.floor((end.getTime() - start.getTime()) / 1000)
      if (duration < 60) return `${duration}s`
      if (duration < 3600) return `${Math.floor(duration / 60)}m ${duration % 60}s`
      return `${Math.floor(duration / 3600)}h ${Math.floor((duration % 3600) / 60)}m`
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

// CronJob columns
const cronjobColumns: DataTableColumns<K8sCronJob> = [
  { title: 'Name', key: 'name', width: 200, ellipsis: { tooltip: true } },
  { title: 'Schedule', key: 'schedule', width: 120 },
  {
    title: 'Suspend',
    key: 'suspend',
    width: 80,
    render(row) {
      return h(NTag, {
        type: row.suspend ? 'warning' : 'success',
        size: 'small'
      }, () => row.suspend ? 'Yes' : 'No')
    }
  },
  { title: 'Active', key: 'active', width: 70 },
  {
    title: 'Last Schedule',
    key: 'last_schedule_time',
    width: 160,
    render(row) {
      if (!row.last_schedule_time) return '-'
      return new Date(row.last_schedule_time).toLocaleString()
    }
  }
]

// ReplicaSet columns
const replicasetColumns: DataTableColumns<K8sReplicaSet> = [
  { title: 'Name', key: 'name', width: 280, ellipsis: { tooltip: true } },
  {
    title: 'Ready',
    key: 'ready_replicas',
    width: 100,
    render(row) {
      const ready = row.ready_replicas || 0
      const total = row.replicas || 0
      const color = ready === total ? 'success' : ready > 0 ? 'warning' : 'error'
      return h(NTag, { type: color, size: 'small' }, () => `${ready}/${total}`)
    }
  },
  { title: 'Available', key: 'available_replicas', width: 80 },
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

async function fetchStatefulsets() {
  statefulsetsLoading.value = true
  try {
    statefulsets.value = await api.k8s.listStatefulSets(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch statefulsets: ' + (error as Error).message)
    statefulsets.value = []
  } finally {
    statefulsetsLoading.value = false
  }
}

async function fetchDaemonsets() {
  daemonsetsLoading.value = true
  try {
    daemonsets.value = await api.k8s.listDaemonSets(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch daemonsets: ' + (error as Error).message)
    daemonsets.value = []
  } finally {
    daemonsetsLoading.value = false
  }
}

async function fetchJobs() {
  jobsLoading.value = true
  try {
    jobs.value = await api.k8s.listJobs(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch jobs: ' + (error as Error).message)
    jobs.value = []
  } finally {
    jobsLoading.value = false
  }
}

async function fetchCronjobs() {
  cronjobsLoading.value = true
  try {
    cronjobs.value = await api.k8s.listCronJobs(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch cronjobs: ' + (error as Error).message)
    cronjobs.value = []
  } finally {
    cronjobsLoading.value = false
  }
}

async function fetchReplicasets() {
  replicasetsLoading.value = true
  try {
    replicasets.value = await api.k8s.listReplicaSets(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch replicasets: ' + (error as Error).message)
    replicasets.value = []
  } finally {
    replicasetsLoading.value = false
  }
}

async function refreshAll() {
  await Promise.all([
    fetchDeployments(),
    fetchStatefulsets(),
    fetchDaemonsets(),
    fetchJobs(),
    fetchCronjobs(),
    fetchReplicasets()
  ])
}

// Watch for prop changes
watch(
  () => [props.clusterId, props.namespace],
  () => {
    refreshAll()
  },
  { immediate: true }
)
</script>

<template>
  <NSpace vertical :size="12">
    <NSpace justify="end">
      <NButton size="small" quaternary @click="refreshAll">
        <template #icon>
          <NIcon><RefreshOutline /></NIcon>
        </template>
        Refresh All
      </NButton>
    </NSpace>

    <NCollapse v-model:expanded-names="expandedNames">
      <!-- Deployments -->
      <NCollapseItem name="deployments">
        <template #header>
          <NSpace :size="8" align="center">
            <span>Deployments</span>
            <NBadge :value="deployments.length" :max="99" />
          </NSpace>
        </template>
        <NSpin :show="deploymentsLoading">
          <NDataTable
            v-if="deployments.length > 0"
            :columns="deploymentColumns"
            :data="deployments"
            :bordered="false"
            size="small"
            max-height="300"
          />
          <NEmpty v-else description="No deployments found" size="small" />
        </NSpin>
      </NCollapseItem>

      <!-- StatefulSets -->
      <NCollapseItem name="statefulsets">
        <template #header>
          <NSpace :size="8" align="center">
            <span>StatefulSets</span>
            <NBadge :value="statefulsets.length" :max="99" />
          </NSpace>
        </template>
        <NSpin :show="statefulsetsLoading">
          <NDataTable
            v-if="statefulsets.length > 0"
            :columns="statefulsetColumns"
            :data="statefulsets"
            :bordered="false"
            size="small"
            max-height="300"
          />
          <NEmpty v-else description="No statefulsets found" size="small" />
        </NSpin>
      </NCollapseItem>

      <!-- DaemonSets -->
      <NCollapseItem name="daemonsets">
        <template #header>
          <NSpace :size="8" align="center">
            <span>DaemonSets</span>
            <NBadge :value="daemonsets.length" :max="99" />
          </NSpace>
        </template>
        <NSpin :show="daemonsetsLoading">
          <NDataTable
            v-if="daemonsets.length > 0"
            :columns="daemonsetColumns"
            :data="daemonsets"
            :bordered="false"
            size="small"
            max-height="300"
          />
          <NEmpty v-else description="No daemonsets found" size="small" />
        </NSpin>
      </NCollapseItem>

      <!-- Jobs -->
      <NCollapseItem name="jobs">
        <template #header>
          <NSpace :size="8" align="center">
            <span>Jobs</span>
            <NBadge :value="jobs.length" :max="99" />
          </NSpace>
        </template>
        <NSpin :show="jobsLoading">
          <NDataTable
            v-if="jobs.length > 0"
            :columns="jobColumns"
            :data="jobs"
            :bordered="false"
            size="small"
            max-height="300"
          />
          <NEmpty v-else description="No jobs found" size="small" />
        </NSpin>
      </NCollapseItem>

      <!-- CronJobs -->
      <NCollapseItem name="cronjobs">
        <template #header>
          <NSpace :size="8" align="center">
            <span>CronJobs</span>
            <NBadge :value="cronjobs.length" :max="99" />
          </NSpace>
        </template>
        <NSpin :show="cronjobsLoading">
          <NDataTable
            v-if="cronjobs.length > 0"
            :columns="cronjobColumns"
            :data="cronjobs"
            :bordered="false"
            size="small"
            max-height="300"
          />
          <NEmpty v-else description="No cronjobs found" size="small" />
        </NSpin>
      </NCollapseItem>

      <!-- ReplicaSets -->
      <NCollapseItem name="replicasets">
        <template #header>
          <NSpace :size="8" align="center">
            <span>ReplicaSets</span>
            <NBadge :value="replicasets.length" :max="99" />
          </NSpace>
        </template>
        <NSpin :show="replicasetsLoading">
          <NDataTable
            v-if="replicasets.length > 0"
            :columns="replicasetColumns"
            :data="replicasets"
            :bordered="false"
            size="small"
            max-height="300"
          />
          <NEmpty v-else description="No replicasets found" size="small" />
        </NSpin>
      </NCollapseItem>
    </NCollapse>
  </NSpace>
</template>
