<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import {
  NLayout,
  NLayoutContent,
  NSpace,
  NIcon,
  NTag,
  NSelect,
  NEmpty,
  NButton,
  NTabs,
  NTabPane,
  NSpin,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NUpload,
  NRadioGroup,
  NRadio,
  NCheckboxGroup,
  NCheckbox,
  NDivider,
  useMessage
} from 'naive-ui'
import { CloudOutline, CheckmarkCircleOutline, CloseCircleOutline, RefreshOutline, AddOutline } from '@vicons/ionicons5'
import { useRouter } from 'vue-router'
import { api } from '@/api/adapter'
import type { Cluster } from '@/api'
import WorkloadPanel from './components/WorkloadPanel.vue'
import ConfigPanel from './components/ConfigPanel.vue'
import NetworkPanel from './components/NetworkPanel.vue'

const router = useRouter()
const message = useMessage()

// State
const clusters = ref<Cluster[]>([])
const selectedClusterId = ref<number | null>(null)
const namespaces = ref<string[]>([])
const selectedNamespace = ref<string | null>(null)
const loading = ref(false)
const namespacesLoading = ref(false)
const activeTab = ref('workload')

// Add Cluster Modal State
const showAddClusterModal = ref(false)
const kubeconfigContent = ref('')
const availableContexts = ref<string[]>([])
const selectedContexts = ref<string[]>([])
const configSource = ref<'upload' | 'local'>('upload')
const loadingLocalConfig = ref(false)
const clusterForm = ref({
  name: '',
  context: '',
  environment: 'development'
})
const saving = ref(false)

// Computed
const selectedCluster = computed(() =>
  clusters.value.find(c => c.id === selectedClusterId.value)
)

const clusterOptions = computed(() =>
  clusters.value.map(c => ({
    label: c.name,
    value: c.id
  }))
)

const namespaceOptions = computed(() =>
  namespaces.value.map(ns => ({
    label: ns,
    value: ns
  }))
)

const contextOptions = computed(() =>
  availableContexts.value.map(ctx => ({
    label: ctx,
    value: ctx
  }))
)

const envOptions = [
  { label: 'Development', value: 'development' },
  { label: 'Staging', value: 'staging' },
  { label: 'Production', value: 'production' }
]

// Methods
async function fetchClusters() {
  loading.value = true
  try {
    clusters.value = await api.cluster.getAll()
    if (clusters.value.length > 0 && !selectedClusterId.value) {
      selectedClusterId.value = clusters.value[0].id!
    }
  } catch (error) {
    message.error('Failed to fetch clusters: ' + (error as Error).message)
  } finally {
    loading.value = false
  }
}

async function fetchNamespaces() {
  if (!selectedClusterId.value) {
    namespaces.value = []
    return
  }

  namespacesLoading.value = true
  try {
    namespaces.value = await api.k8s.listNamespaces(selectedClusterId.value)
    if (namespaces.value.length > 0 && !selectedNamespace.value) {
      // Default to 'default' namespace if available
      if (namespaces.value.includes('default')) {
        selectedNamespace.value = 'default'
      } else {
        selectedNamespace.value = namespaces.value[0]
      }
    }
  } catch (error) {
    message.error('Failed to fetch namespaces: ' + (error as Error).message)
    namespaces.value = []
  } finally {
    namespacesLoading.value = false
  }
}

function handleClusterChange(clusterId: number) {
  selectedClusterId.value = clusterId
  selectedNamespace.value = null
}

function handleNamespaceChange(namespace: string) {
  selectedNamespace.value = namespace
}

function refresh() {
  if (selectedClusterId.value) {
    fetchNamespaces()
  }
}

function goToConnections() {
  router.push('/connections')
}

// Add Cluster Modal Functions
function openAddClusterModal() {
  showAddClusterModal.value = true
  resetClusterForm()
}

function resetClusterForm() {
  kubeconfigContent.value = ''
  availableContexts.value = []
  selectedContexts.value = []
  configSource.value = 'upload'
  clusterForm.value = {
    name: '',
    context: '',
    environment: 'development'
  }
}

async function handleLoadLocalConfig() {
  loadingLocalConfig.value = true
  try {
    const content = await api.k8s.readLocalKubeconfig()
    kubeconfigContent.value = content

    // Parse kubeconfig to get available contexts
    const result = await api.k8s.listClusters(content)
    availableContexts.value = result.clusters || []
    selectedContexts.value = []
    message.success(`Found ${availableContexts.value.length} context(s) in local kubeconfig`)
  } catch (e) {
    message.error('Failed to read local kubeconfig: ' + (e as Error).message)
    kubeconfigContent.value = ''
    availableContexts.value = []
  } finally {
    loadingLocalConfig.value = false
  }
}

async function handleKubeconfigUpload(options: { file: { file: File } }) {
  const file = options.file.file
  if (!file) return

  const reader = new FileReader()
  reader.onload = async (e) => {
    kubeconfigContent.value = e.target?.result as string

    // Parse kubeconfig to get available contexts
    try {
      const result = await api.k8s.listClusters(kubeconfigContent.value)
      availableContexts.value = result.clusters || []
      selectedContexts.value = []
      if (availableContexts.value.length > 0) {
        clusterForm.value.context = availableContexts.value[0]
        clusterForm.value.name = availableContexts.value[0]
      }
      message.success('Kubeconfig 解析成功')
    } catch (e) {
      message.error('解析 kubeconfig 失败: ' + (e as Error).message)
      kubeconfigContent.value = ''
    }
  }
  reader.onerror = () => {
    message.error('读取文件失败')
  }
  reader.readAsText(file)
}

async function handleSaveCluster() {
  if (!kubeconfigContent.value) {
    message.warning('请上传 kubeconfig 文件或读取本地配置')
    return
  }

  // Multi-select mode (local config with checkbox selection)
  if (configSource.value === 'local' && selectedContexts.value.length > 0) {
    saving.value = true
    try {
      let lastClusterId: number | undefined
      let successCount = 0

      for (const ctx of selectedContexts.value) {
        try {
          const newCluster = await api.cluster.create({
            name: ctx,
            context: ctx,
            environment: clusterForm.value.environment,
            kubeconfig: kubeconfigContent.value,
            is_active: true
          })
          successCount++
          if (newCluster.id) {
            lastClusterId = newCluster.id
          }
        } catch (e) {
          message.warning(`添加 ${ctx} 失败: ${(e as Error).message}`)
        }
      }

      if (successCount > 0) {
        message.success(`成功添加 ${successCount} 个集群`)
        showAddClusterModal.value = false
        await fetchClusters()
        if (lastClusterId) {
          selectedClusterId.value = lastClusterId
        }
      }
    } finally {
      saving.value = false
    }
    return
  }

  // Single mode (upload file or local config with single selection)
  if (!clusterForm.value.name) {
    message.warning('请填写集群名称')
    return
  }

  saving.value = true
  try {
    const newCluster = await api.cluster.create({
      name: clusterForm.value.name,
      context: clusterForm.value.context || undefined,
      environment: clusterForm.value.environment,
      kubeconfig: kubeconfigContent.value,
      is_active: true
    })
    message.success('集群添加成功')
    showAddClusterModal.value = false
    await fetchClusters()
    // Auto-select the new cluster
    if (newCluster.id) {
      selectedClusterId.value = newCluster.id
    }
  } catch (e) {
    message.error('添加失败: ' + (e as Error).message)
  } finally {
    saving.value = false
  }
}

// Lifecycle
onMounted(() => {
  fetchClusters()
})

// Watch for cluster changes
watch(selectedClusterId, () => {
  fetchNamespaces()
})
</script>

<template>
  <NLayout class="k8s-layout">
    <!-- Header -->
    <div class="k8s-header">
      <NSpace align="center" justify="space-between">
        <NSpace align="center" :size="12">
          <NIcon size="20" color="#326CE5">
            <CloudOutline />
          </NIcon>
          <span class="title-font" style="font-size: 15px">K8s Resources</span>
          <NTag
            v-if="selectedCluster"
            :type="selectedCluster.is_active ? 'success' : 'warning'"
            size="tiny"
            round
          >
            <template #icon>
              <NIcon size="10">
                <CheckmarkCircleOutline v-if="selectedCluster.is_active" />
                <CloseCircleOutline v-else />
              </NIcon>
            </template>
            {{ selectedCluster.is_active ? 'Active' : 'Inactive' }}
          </NTag>
        </NSpace>

        <!-- Cluster & Namespace Selectors -->
        <NSpace align="center" :size="12">
          <NSelect
            v-model:value="selectedClusterId"
            :options="clusterOptions"
            placeholder="Select Cluster"
            style="width: 180px"
            :loading="loading"
            @update:value="handleClusterChange"
          />
          <NButton quaternary circle @click="openAddClusterModal" title="Add Cluster">
            <template #icon>
              <NIcon><AddOutline /></NIcon>
            </template>
          </NButton>
          <NSelect
            v-model:value="selectedNamespace"
            :options="namespaceOptions"
            placeholder="Select Namespace"
            style="width: 180px"
            :loading="namespacesLoading"
            :disabled="!selectedClusterId"
            @update:value="handleNamespaceChange"
          />
          <NButton quaternary circle @click="refresh" :disabled="!selectedClusterId">
            <template #icon>
              <NIcon><RefreshOutline /></NIcon>
            </template>
          </NButton>
        </NSpace>
      </NSpace>
    </div>

    <!-- Main Content -->
    <NLayoutContent class="main-content">
      <template v-if="clusters.length === 0 && !loading">
        <div class="no-connection-content">
          <NEmpty description="No K8s clusters configured" size="large">
            <template #extra>
              <NButton type="primary" @click="goToConnections">
                <template #icon>
                  <NIcon><CloudOutline /></NIcon>
                </template>
                Configure Cluster
              </NButton>
            </template>
          </NEmpty>
        </div>
      </template>

      <template v-else-if="!selectedClusterId || !selectedNamespace">
        <div class="no-connection-content">
          <NEmpty description="Please select a cluster and namespace" size="large" />
        </div>
      </template>

      <template v-else>
        <NSpin :show="loading || namespacesLoading">
          <NTabs v-model:value="activeTab" type="line" animated>
            <NTabPane name="workload" tab="Workload">
              <WorkloadPanel
                :cluster-id="selectedClusterId"
                :namespace="selectedNamespace"
              />
            </NTabPane>
            <NTabPane name="config" tab="Config">
              <ConfigPanel
                :cluster-id="selectedClusterId"
                :namespace="selectedNamespace"
              />
            </NTabPane>
            <NTabPane name="network" tab="Network">
              <NetworkPanel
                :cluster-id="selectedClusterId"
                :namespace="selectedNamespace"
              />
            </NTabPane>
          </NTabs>
        </NSpin>
      </template>
    </NLayoutContent>

    <!-- Add Cluster Modal -->
    <NModal
      v-model:show="showAddClusterModal"
      preset="card"
      title="Add K8s Cluster"
      style="width: 520px"
      :mask-closable="false"
    >
      <NForm :model="clusterForm" label-placement="top">
        <!-- Config Source Selection -->
        <NFormItem label="Config Source">
          <NRadioGroup v-model:value="configSource">
            <NSpace>
              <NRadio value="upload">Upload File</NRadio>
              <NRadio value="local">Local ~/.kube/config</NRadio>
            </NSpace>
          </NRadioGroup>
        </NFormItem>

        <!-- Upload Mode -->
        <NFormItem v-if="configSource === 'upload'" label="Kubeconfig File" required>
          <NUpload
            :max="1"
            :show-file-list="false"
            accept=".yaml,.yml,application/x-yaml"
            :custom-request="handleKubeconfigUpload"
          >
            <NButton block :type="kubeconfigContent ? 'success' : 'default'">
              {{ kubeconfigContent ? 'Uploaded' : 'Click to Upload' }}
            </NButton>
          </NUpload>
        </NFormItem>

        <!-- Local Config Mode -->
        <NFormItem v-if="configSource === 'local'" label="Local Config">
          <NButton
            block
            :type="kubeconfigContent ? 'success' : 'default'"
            :loading="loadingLocalConfig"
            @click="handleLoadLocalConfig"
          >
            {{ kubeconfigContent ? `Loaded (${availableContexts.length} contexts)` : 'Read Local Config' }}
          </NButton>
        </NFormItem>

        <!-- Multi-select Contexts (Local Mode) -->
        <NFormItem v-if="configSource === 'local' && availableContexts.length > 0" label="Select Contexts">
          <NCheckboxGroup v-model:value="selectedContexts" style="width: 100%">
            <NSpace vertical>
              <NCheckbox
                v-for="ctx in availableContexts"
                :key="ctx"
                :value="ctx"
                :label="ctx"
              />
            </NSpace>
          </NCheckboxGroup>
          <div style="margin-top: 8px; font-size: 12px; color: var(--n-text-color-3)">
            Select one or more contexts to add as clusters
          </div>
        </NFormItem>

        <!-- Single Select Context (Upload Mode) -->
        <NFormItem v-if="configSource === 'upload' && availableContexts.length > 0" label="Select Context">
          <NSelect
            v-model:value="clusterForm.context"
            :options="contextOptions"
            placeholder="Select a context"
          />
        </NFormItem>

        <NDivider v-if="configSource === 'upload' || (configSource === 'local' && selectedContexts.length === 0)" />

        <!-- Single Cluster Name (Upload Mode or Local without multi-select) -->
        <NFormItem
          v-if="configSource === 'upload' || (configSource === 'local' && selectedContexts.length === 0)"
          label="Cluster Name"
          required
        >
          <NInput
            v-model:value="clusterForm.name"
            placeholder="e.g. production-cluster"
          />
        </NFormItem>

        <NFormItem label="Environment">
          <NSelect
            v-model:value="clusterForm.environment"
            :options="envOptions"
          />
        </NFormItem>
      </NForm>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showAddClusterModal = false">Cancel</NButton>
          <NButton type="primary" :loading="saving" @click="handleSaveCluster">
            {{ configSource === 'local' && selectedContexts.length > 1 ? `Save ${selectedContexts.length} Clusters` : 'Save' }}
          </NButton>
        </NSpace>
      </template>
    </NModal>
  </NLayout>
</template>

<style scoped>
.k8s-layout {
  height: 100%;
  background: transparent;
}

.k8s-header {
  padding: 12px 16px;
  background: var(--n-color);
  border-bottom: 1px solid var(--n-border-color);
}

.title-font {
  font-weight: 600;
  color: var(--n-text-color-1);
}

.main-content {
  padding: 16px;
  background: transparent;
  height: calc(100% - 60px);
  overflow: auto;
}

.no-connection-content {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 300px;
}
</style>
