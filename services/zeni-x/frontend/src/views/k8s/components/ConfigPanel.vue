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
  NInput,
  NPopconfirm,
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
const configMapEditing = ref(false)
const configMapEditData = ref<Record<string, string>>({})
const configMapSaving = ref(false)

// Secret data modal
const showSecretData = ref(false)
const selectedSecretName = ref('')
const secretData = ref<Record<string, string>>({})
const secretDataLoading = ref(false)
const secretEditing = ref(false)
const secretEditData = ref<Record<string, string>>({})
const secretSaving = ref(false)
const secretVisibility = ref<Record<string, boolean>>({})

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
          type: 'warning',
          onClick: () => viewSecretData(row.name)
        },
        () => 'View'
      )
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
  configMapEditing.value = false

  try {
    configMapData.value = await api.k8s.getConfigMapData(props.clusterId, props.namespace, name)
    configMapEditData.value = { ...configMapData.value }
  } catch (error) {
    message.error('Failed to fetch configmap data: ' + (error as Error).message)
    configMapData.value = {}
  } finally {
    configMapDataLoading.value = false
  }
}

async function viewSecretData(name: string) {
  selectedSecretName.value = name
  showSecretData.value = true
  secretDataLoading.value = true
  secretEditing.value = false
  secretVisibility.value = {}

  try {
    secretData.value = await api.k8s.getSecretData(props.clusterId, props.namespace, name)
    secretEditData.value = { ...secretData.value }
  } catch (error) {
    message.error('Failed to fetch secret data: ' + (error as Error).message)
    secretData.value = {}
  } finally {
    secretDataLoading.value = false
  }
}

function startConfigMapEdit() {
  configMapEditData.value = { ...configMapData.value }
  configMapEditing.value = true
}

function cancelConfigMapEdit() {
  configMapEditing.value = false
  configMapEditData.value = { ...configMapData.value }
}

async function saveConfigMap() {
  configMapSaving.value = true
  try {
    await api.k8s.updateConfigMap(props.clusterId, props.namespace, selectedConfigMapName.value, configMapEditData.value)
    message.success('ConfigMap updated successfully')
    configMapData.value = { ...configMapEditData.value }
    configMapEditing.value = false
  } catch (error) {
    message.error('Failed to update configmap: ' + (error as Error).message)
  } finally {
    configMapSaving.value = false
  }
}

function startSecretEdit() {
  secretEditData.value = { ...secretData.value }
  secretEditing.value = true
}

function cancelSecretEdit() {
  secretEditing.value = false
  secretEditData.value = { ...secretData.value }
}

async function saveSecret() {
  secretSaving.value = true
  try {
    await api.k8s.updateSecret(props.clusterId, props.namespace, selectedSecretName.value, secretEditData.value)
    message.success('Secret updated successfully')
    secretData.value = { ...secretEditData.value }
    secretEditing.value = false
  } catch (error) {
    message.error('Failed to update secret: ' + (error as Error).message)
  } finally {
    secretSaving.value = false
  }
}

function toggleSecretVisibility(key: string) {
  secretVisibility.value[key] = !secretVisibility.value[key]
}

function copyToClipboard(value: string, label: string) {
  navigator.clipboard.writeText(value).then(() => {
    message.success(`${label} copied to clipboard`)
  }).catch(() => {
    message.error('Failed to copy to clipboard')
  })
}

function getMaskedValue(value: string): string {
  return '*'.repeat(Math.min(value.length, 20))
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
      <template #header-extra>
        <NSpace>
          <NButton
            v-if="!configMapEditing"
            size="small"
            type="primary"
            @click="startConfigMapEdit"
          >
            Edit
          </NButton>
          <template v-else>
            <NButton size="small" @click="cancelConfigMapEdit">Cancel</NButton>
            <NButton
              size="small"
              type="primary"
              :loading="configMapSaving"
              @click="saveConfigMap"
            >
              Save
            </NButton>
          </template>
        </NSpace>
      </template>
      <NSpin :show="configMapDataLoading">
        <NSpace vertical :size="12">
          <template v-for="(value, key) in (configMapEditing ? configMapEditData : configMapData)" :key="key">
            <NCard :title="String(key)" size="small">
              <template #header-extra>
                <NButton size="tiny" quaternary @click="copyToClipboard(String(value), String(key))">
                  Copy
                </NButton>
              </template>
              <NInput
                v-if="configMapEditing"
                v-model:value="configMapEditData[key]"
                type="textarea"
                :autosize="{ minRows: 3, maxRows: 15 }"
              />
              <NCode v-else :code="value" language="yaml" word-wrap />
            </NCard>
          </template>
          <NEmpty v-if="Object.keys(configMapData).length === 0 && !configMapDataLoading" description="No data" />
        </NSpace>
      </NSpin>
    </NModal>

    <!-- Secret Data Modal -->
    <NModal
      v-model:show="showSecretData"
      preset="card"
      :title="`Secret: ${selectedSecretName}`"
      style="width: 800px; max-width: 90vw"
    >
      <template #header-extra>
        <NSpace>
          <NTag type="warning" size="small">Sensitive Data</NTag>
          <NButton
            v-if="!secretEditing"
            size="small"
            type="primary"
            @click="startSecretEdit"
          >
            Edit
          </NButton>
          <template v-else>
            <NButton size="small" @click="cancelSecretEdit">Cancel</NButton>
            <NPopconfirm @positive-click="saveSecret">
              <template #trigger>
                <NButton size="small" type="warning" :loading="secretSaving">
                  Save
                </NButton>
              </template>
              Are you sure you want to update this secret?
            </NPopconfirm>
          </template>
        </NSpace>
      </template>
      <NSpin :show="secretDataLoading">
        <NSpace vertical :size="12">
          <template v-for="(value, key) in (secretEditing ? secretEditData : secretData)" :key="key">
            <NCard :title="String(key)" size="small">
              <template #header-extra>
                <NSpace :size="4">
                  <NButton
                    size="tiny"
                    quaternary
                    @click="toggleSecretVisibility(String(key))"
                  >
                    {{ secretVisibility[key] ? 'Hide' : 'Show' }}
                  </NButton>
                  <NButton
                    size="tiny"
                    quaternary
                    @click="copyToClipboard(String(value), String(key))"
                  >
                    Copy
                  </NButton>
                </NSpace>
              </template>
              <NInput
                v-if="secretEditing"
                v-model:value="secretEditData[key]"
                type="textarea"
                :autosize="{ minRows: 2, maxRows: 10 }"
                placeholder="Enter value..."
              />
              <div v-else>
                <NCode
                  v-if="secretVisibility[key]"
                  :code="value"
                  language="text"
                  word-wrap
                />
                <span v-else class="masked-value">{{ getMaskedValue(String(value)) }}</span>
              </div>
            </NCard>
          </template>
          <NEmpty v-if="Object.keys(secretData).length === 0 && !secretDataLoading" description="No data" />
        </NSpace>
      </NSpin>
    </NModal>
  </NSpace>
</template>

<style scoped>
.masked-value {
  font-family: monospace;
  color: var(--n-text-color-3);
  letter-spacing: 2px;
}
</style>
