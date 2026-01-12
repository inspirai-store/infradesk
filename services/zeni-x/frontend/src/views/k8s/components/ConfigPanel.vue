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
  NModal,
  NCode,
  useMessage
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { api } from '@/api/adapter'
import type { K8sConfigMapInfo, K8sSecretInfo } from '@/api/types'

const props = defineProps<{
  clusterId: number
  namespace: string
}>()

const message = useMessage()

// State
const configmaps = ref<K8sConfigMapInfo[]>([])
const secrets = ref<K8sSecretInfo[]>([])
const configmapsLoading = ref(false)
const secretsLoading = ref(false)

// ConfigMap data modal
const showConfigMapData = ref(false)
const selectedConfigMapName = ref('')
const configMapData = ref<Record<string, string>>({})
const configMapDataLoading = ref(false)

// ConfigMap columns
const configMapColumns: DataTableColumns<K8sConfigMapInfo> = [
  { title: 'Name', key: 'name', width: 250, ellipsis: { tooltip: true } },
  {
    title: 'Keys',
    key: 'data_keys',
    render(row) {
      const keys = row.data_keys || []
      return h(NTag, { type: 'info', size: 'small' }, () => `${keys.length} keys`)
    }
  },
  {
    title: 'Key Names',
    key: 'key_names',
    ellipsis: { tooltip: true },
    render(row) {
      const keys = row.data_keys || []
      return keys.slice(0, 5).join(', ') + (keys.length > 5 ? '...' : '')
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
  },
  {
    title: 'Action',
    key: 'action',
    width: 100,
    render(row) {
      return h(
        NButton,
        {
          size: 'tiny',
          onClick: () => viewConfigMapData(row.name)
        },
        () => 'View'
      )
    }
  }
]

// Secret columns
const secretColumns: DataTableColumns<K8sSecretInfo> = [
  { title: 'Name', key: 'name', width: 250, ellipsis: { tooltip: true } },
  {
    title: 'Type',
    key: 'secret_type',
    width: 180,
    render(row) {
      return h(NTag, { size: 'small' }, () => row.secret_type || 'Opaque')
    }
  },
  {
    title: 'Keys',
    key: 'data_keys',
    render(row) {
      const keys = row.data_keys || []
      return h(NTag, { type: 'warning', size: 'small' }, () => `${keys.length} keys`)
    }
  },
  {
    title: 'Key Names',
    key: 'key_names',
    ellipsis: { tooltip: true },
    render(row) {
      const keys = row.data_keys || []
      return keys.slice(0, 5).join(', ') + (keys.length > 5 ? '...' : '')
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

// Methods
async function fetchConfigMaps() {
  configmapsLoading.value = true
  try {
    configmaps.value = await api.k8s.listConfigMaps(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch configmaps: ' + (error as Error).message)
    configmaps.value = []
  } finally {
    configmapsLoading.value = false
  }
}

async function fetchSecrets() {
  secretsLoading.value = true
  try {
    secrets.value = await api.k8s.listSecrets(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch secrets: ' + (error as Error).message)
    secrets.value = []
  } finally {
    secretsLoading.value = false
  }
}

async function viewConfigMapData(name: string) {
  selectedConfigMapName.value = name
  showConfigMapData.value = true
  configMapDataLoading.value = true

  try {
    configMapData.value = await api.k8s.getConfigMapData(props.clusterId, props.namespace, name)
  } catch (error) {
    message.error('Failed to fetch configmap data: ' + (error as Error).message)
    configMapData.value = {}
  } finally {
    configMapDataLoading.value = false
  }
}

async function refresh() {
  await Promise.all([fetchConfigMaps(), fetchSecrets()])
}

// Watch for prop changes
watch(
  () => [props.clusterId, props.namespace],
  () => {
    refresh()
  },
  { immediate: true }
)
</script>

<template>
  <NSpace vertical :size="16">
    <!-- ConfigMaps -->
    <NCard title="ConfigMaps" size="small">
      <NSpin :show="configmapsLoading">
        <NDataTable
          v-if="configmaps.length > 0"
          :columns="configMapColumns"
          :data="configmaps"
          :bordered="false"
          size="small"
          max-height="300"
        />
        <NEmpty v-else description="No configmaps found" />
      </NSpin>
    </NCard>

    <!-- Secrets -->
    <NCard title="Secrets" size="small">
      <template #header-extra>
        <NTag type="warning" size="small">Values hidden for security</NTag>
      </template>
      <NSpin :show="secretsLoading">
        <NDataTable
          v-if="secrets.length > 0"
          :columns="secretColumns"
          :data="secrets"
          :bordered="false"
          size="small"
          max-height="300"
        />
        <NEmpty v-else description="No secrets found" />
      </NSpin>
    </NCard>

    <!-- ConfigMap Data Modal -->
    <NModal
      v-model:show="showConfigMapData"
      preset="card"
      :title="`ConfigMap: ${selectedConfigMapName}`"
      style="width: 800px; max-width: 90vw"
    >
      <NSpin :show="configMapDataLoading">
        <NSpace vertical :size="12">
          <template v-for="(value, key) in configMapData" :key="key">
            <NCard :title="String(key)" size="small">
              <NCode :code="value" language="yaml" word-wrap />
            </NCard>
          </template>
          <NEmpty v-if="Object.keys(configMapData).length === 0 && !configMapDataLoading" description="No data" />
        </NSpace>
      </NSpin>
    </NModal>
  </NSpace>
</template>
