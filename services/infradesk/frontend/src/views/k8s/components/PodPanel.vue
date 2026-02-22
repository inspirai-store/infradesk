<script setup lang="ts">
import { ref, watch, h } from 'vue'
import {
  NDataTable,
  NCard,
  NSpace,
  NTag,
  NSpin,
  NEmpty,
  NButton,
  NIcon,
  NModal,
  NDescriptions,
  NDescriptionsItem,
  NTabs,
  NTabPane,
  NInputNumber,
  NCode,
  NCollapse,
  NCollapseItem,
  NText,
  useMessage
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { EyeOutline, DocumentTextOutline, CopyOutline, RefreshOutline } from '@vicons/ionicons5'
import { api } from '@/api/adapter'
import type { K8sPod, K8sPodDetail } from '@/api/types'

const props = defineProps<{
  clusterId: number
  namespace: string
}>()

const message = useMessage()

// State
const pods = ref<K8sPod[]>([])
const loading = ref(false)

// Detail Modal State
const showDetailModal = ref(false)
const detailLoading = ref(false)
const selectedPodDetail = ref<K8sPodDetail | null>(null)

// Logs Modal State
const showLogsModal = ref(false)
const logsLoading = ref(false)
const podLogs = ref('')
const selectedPodForLogs = ref<K8sPod | null>(null)
const selectedContainer = ref<string | null>(null)
const tailLines = ref(100)

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
  },
  {
    title: 'Actions',
    key: 'actions',
    width: 120,
    fixed: 'right',
    render(row) {
      return h(NSpace, { size: 4 }, () => [
        h(NButton, {
          size: 'tiny',
          quaternary: true,
          type: 'primary',
          onClick: () => viewPodDetail(row)
        }, {
          icon: () => h(NIcon, null, () => h(EyeOutline)),
          default: () => 'Detail'
        }),
        h(NButton, {
          size: 'tiny',
          quaternary: true,
          type: 'info',
          onClick: () => viewPodLogs(row)
        }, {
          icon: () => h(NIcon, null, () => h(DocumentTextOutline)),
          default: () => 'Logs'
        })
      ])
    }
  }
]

// Methods
async function fetchPods() {
  loading.value = true
  try {
    pods.value = await api.k8s.listPods(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch pods: ' + (error as Error).message)
    pods.value = []
  } finally {
    loading.value = false
  }
}

async function viewPodDetail(pod: K8sPod) {
  showDetailModal.value = true
  detailLoading.value = true
  selectedPodDetail.value = null

  try {
    selectedPodDetail.value = await api.k8s.getPodDetail(props.clusterId, props.namespace, pod.name)
  } catch (error) {
    message.error('Failed to fetch pod detail: ' + (error as Error).message)
  } finally {
    detailLoading.value = false
  }
}

async function viewPodLogs(pod: K8sPod) {
  selectedPodForLogs.value = pod
  selectedContainer.value = null
  podLogs.value = ''
  showLogsModal.value = true
  await fetchLogs()
}

async function fetchLogs() {
  if (!selectedPodForLogs.value) return

  logsLoading.value = true
  try {
    podLogs.value = await api.k8s.getPodLogs(
      props.clusterId,
      props.namespace,
      selectedPodForLogs.value.name,
      selectedContainer.value || undefined,
      tailLines.value
    )
  } catch (error) {
    message.error('Failed to fetch logs: ' + (error as Error).message)
    podLogs.value = ''
  } finally {
    logsLoading.value = false
  }
}

function copyLogs() {
  navigator.clipboard.writeText(podLogs.value)
  message.success('Logs copied to clipboard')
}

function formatEnvValue(env: { name: string; value?: string; value_from?: string }) {
  if (env.value) return env.value
  if (env.value_from) return `<${env.value_from}>`
  return '-'
}

function formatPorts(ports: Array<{ name?: string; container_port: number; protocol: string }>) {
  return ports.map(p => `${p.container_port}/${p.protocol}${p.name ? ` (${p.name})` : ''}`).join(', ')
}

function getContainerStateType(state: string): 'success' | 'warning' | 'error' | 'default' {
  switch (state.toLowerCase()) {
    case 'running': return 'success'
    case 'waiting': return 'warning'
    case 'terminated': return 'error'
    default: return 'default'
  }
}

// Watch for prop changes
watch(
  () => [props.clusterId, props.namespace],
  () => {
    fetchPods()
  },
  { immediate: true }
)
</script>

<template>
  <NSpace vertical :size="16">
    <!-- Pods Table -->
    <NCard title="Pods" size="small">
      <template #header-extra>
        <NButton size="small" quaternary @click="fetchPods">
          <template #icon>
            <NIcon><RefreshOutline /></NIcon>
          </template>
          Refresh
        </NButton>
      </template>
      <NSpin :show="loading">
        <NDataTable
          v-if="pods.length > 0"
          :columns="podColumns"
          :data="pods"
          :bordered="false"
          size="small"
          max-height="600"
          :scroll-x="1200"
        />
        <NEmpty v-else description="No pods found" />
      </NSpin>
    </NCard>

    <!-- Pod Detail Modal -->
    <NModal
      v-model:show="showDetailModal"
      preset="card"
      title="Pod Details"
      style="width: 800px; max-width: 90vw"
    >
      <NSpin :show="detailLoading">
        <template v-if="selectedPodDetail">
          <NTabs type="line" animated>
            <!-- Overview Tab -->
            <NTabPane name="overview" tab="Overview">
              <NDescriptions :column="2" label-placement="left" bordered size="small">
                <NDescriptionsItem label="Name">{{ selectedPodDetail.name }}</NDescriptionsItem>
                <NDescriptionsItem label="Namespace">{{ selectedPodDetail.namespace }}</NDescriptionsItem>
                <NDescriptionsItem label="Status">
                  <NTag :type="selectedPodDetail.status === 'Running' ? 'success' : 'warning'" size="small">
                    {{ selectedPodDetail.status }}
                  </NTag>
                </NDescriptionsItem>
                <NDescriptionsItem label="Phase">{{ selectedPodDetail.phase }}</NDescriptionsItem>
                <NDescriptionsItem label="Node">{{ selectedPodDetail.node || '-' }}</NDescriptionsItem>
                <NDescriptionsItem label="Pod IP">{{ selectedPodDetail.ip || '-' }}</NDescriptionsItem>
                <NDescriptionsItem label="Host IP">{{ selectedPodDetail.host_ip || '-' }}</NDescriptionsItem>
                <NDescriptionsItem label="Start Time">
                  {{ selectedPodDetail.start_time ? new Date(selectedPodDetail.start_time).toLocaleString() : '-' }}
                </NDescriptionsItem>
              </NDescriptions>

              <!-- Labels -->
              <div v-if="Object.keys(selectedPodDetail.labels || {}).length > 0" style="margin-top: 16px">
                <NText strong>Labels</NText>
                <div style="margin-top: 8px">
                  <NTag
                    v-for="(value, key) in selectedPodDetail.labels"
                    :key="key"
                    size="small"
                    style="margin-right: 4px; margin-bottom: 4px"
                  >
                    {{ key }}={{ value }}
                  </NTag>
                </div>
              </div>

              <!-- Conditions -->
              <div v-if="selectedPodDetail.conditions.length > 0" style="margin-top: 16px">
                <NText strong>Conditions</NText>
                <NDataTable
                  :columns="[
                    { title: 'Type', key: 'type', width: 150 },
                    { title: 'Status', key: 'status', width: 80 },
                    { title: 'Reason', key: 'reason', width: 150 },
                    { title: 'Message', key: 'message', ellipsis: { tooltip: true } }
                  ]"
                  :data="selectedPodDetail.conditions"
                  :bordered="false"
                  size="small"
                  style="margin-top: 8px"
                />
              </div>
            </NTabPane>

            <!-- Containers Tab -->
            <NTabPane name="containers" tab="Containers">
              <NCollapse>
                <NCollapseItem
                  v-for="container in selectedPodDetail.containers"
                  :key="container.name"
                  :title="container.name"
                  :name="container.name"
                >
                  <template #header-extra>
                    <NSpace :size="8">
                      <NTag :type="getContainerStateType(container.state)" size="small">
                        {{ container.state }}
                      </NTag>
                      <NTag :type="container.ready ? 'success' : 'warning'" size="small">
                        {{ container.ready ? 'Ready' : 'Not Ready' }}
                      </NTag>
                      <NTag v-if="container.restart_count > 0" type="warning" size="small">
                        Restarts: {{ container.restart_count }}
                      </NTag>
                    </NSpace>
                  </template>

                  <NDescriptions :column="1" label-placement="left" bordered size="small">
                    <NDescriptionsItem label="Image">
                      <NText code>{{ container.image }}</NText>
                    </NDescriptionsItem>
                    <NDescriptionsItem v-if="container.image_pull_policy" label="Pull Policy">
                      {{ container.image_pull_policy }}
                    </NDescriptionsItem>
                    <NDescriptionsItem v-if="container.ports.length > 0" label="Ports">
                      {{ formatPorts(container.ports) }}
                    </NDescriptionsItem>
                    <NDescriptionsItem v-if="container.resources" label="Resources">
                      <div v-if="container.resources.cpu_request || container.resources.memory_request">
                        <strong>Requests:</strong>
                        CPU: {{ container.resources.cpu_request || '-' }},
                        Memory: {{ container.resources.memory_request || '-' }}
                      </div>
                      <div v-if="container.resources.cpu_limit || container.resources.memory_limit">
                        <strong>Limits:</strong>
                        CPU: {{ container.resources.cpu_limit || '-' }},
                        Memory: {{ container.resources.memory_limit || '-' }}
                      </div>
                    </NDescriptionsItem>
                  </NDescriptions>

                  <!-- Environment Variables -->
                  <div v-if="container.env.length > 0" style="margin-top: 12px">
                    <NText strong>Environment Variables</NText>
                    <NDataTable
                      :columns="[
                        { title: 'Name', key: 'name', width: 200 },
                        { title: 'Value', key: 'value', render: (row: any) => formatEnvValue(row), ellipsis: { tooltip: true } }
                      ]"
                      :data="container.env"
                      :bordered="false"
                      size="small"
                      max-height="200"
                      style="margin-top: 8px"
                    />
                  </div>
                </NCollapseItem>
              </NCollapse>

              <!-- Init Containers -->
              <div v-if="selectedPodDetail.init_containers.length > 0" style="margin-top: 16px">
                <NText strong>Init Containers</NText>
                <NCollapse style="margin-top: 8px">
                  <NCollapseItem
                    v-for="container in selectedPodDetail.init_containers"
                    :key="container.name"
                    :title="container.name"
                    :name="container.name"
                  >
                    <template #header-extra>
                      <NTag :type="getContainerStateType(container.state)" size="small">
                        {{ container.state }}
                      </NTag>
                    </template>

                    <NDescriptions :column="1" label-placement="left" bordered size="small">
                      <NDescriptionsItem label="Image">
                        <NText code>{{ container.image }}</NText>
                      </NDescriptionsItem>
                    </NDescriptions>
                  </NCollapseItem>
                </NCollapse>
              </div>
            </NTabPane>
          </NTabs>
        </template>
        <NEmpty v-else description="Loading pod details..." />
      </NSpin>
    </NModal>

    <!-- Logs Modal -->
    <NModal
      v-model:show="showLogsModal"
      preset="card"
      :title="`Logs: ${selectedPodForLogs?.name || ''}`"
      style="width: 900px; max-width: 95vw"
    >
      <template #header-extra>
        <NSpace :size="8">
          <NInputNumber
            v-model:value="tailLines"
            :min="10"
            :max="5000"
            :step="100"
            size="small"
            style="width: 120px"
          >
            <template #prefix>Tail:</template>
          </NInputNumber>
          <NButton size="small" @click="fetchLogs" :loading="logsLoading">
            <template #icon>
              <NIcon><RefreshOutline /></NIcon>
            </template>
            Refresh
          </NButton>
          <NButton size="small" @click="copyLogs" :disabled="!podLogs">
            <template #icon>
              <NIcon><CopyOutline /></NIcon>
            </template>
            Copy
          </NButton>
        </NSpace>
      </template>

      <NSpin :show="logsLoading">
        <div class="logs-container">
          <NCode
            v-if="podLogs"
            :code="podLogs"
            language="log"
            style="white-space: pre-wrap; word-break: break-all"
          />
          <NEmpty v-else description="No logs available" />
        </div>
      </NSpin>
    </NModal>
  </NSpace>
</template>

<style scoped>
.logs-container {
  max-height: 500px;
  overflow: auto;
  background: var(--n-color-embedded);
  border-radius: 4px;
  padding: 12px;
  font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.5;
}

.logs-container :deep(.n-code) {
  background: transparent;
}
</style>
