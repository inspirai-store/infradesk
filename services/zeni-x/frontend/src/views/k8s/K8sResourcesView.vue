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
  useMessage
} from 'naive-ui'
import { CloudOutline, CheckmarkCircleOutline, CloseCircleOutline, RefreshOutline } from '@vicons/ionicons5'
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
