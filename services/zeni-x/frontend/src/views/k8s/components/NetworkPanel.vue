<script setup lang="ts">
import { ref, watch, h } from 'vue'
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
import type { K8sServiceInfo, K8sIngressInfo } from '@/api/types'

const props = defineProps<{
  clusterId: number
  namespace: string
}>()

const message = useMessage()

// State
const services = ref<K8sServiceInfo[]>([])
const ingresses = ref<K8sIngressInfo[]>([])
const servicesLoading = ref(false)
const ingressesLoading = ref(false)

// Service columns
const serviceColumns: DataTableColumns<K8sServiceInfo> = [
  { title: 'Name', key: 'name', width: 200, ellipsis: { tooltip: true } },
  {
    title: 'Type',
    key: 'service_type',
    width: 120,
    render(row) {
      const typeColors: Record<string, string> = {
        ClusterIP: 'default',
        NodePort: 'info',
        LoadBalancer: 'success',
        ExternalName: 'warning'
      }
      const color = typeColors[row.service_type] || 'default'
      return h(NTag, { type: color as any, size: 'small' }, () => row.service_type)
    }
  },
  { title: 'Cluster IP', key: 'cluster_ip', width: 140 },
  {
    title: 'External IP',
    key: 'external_ip',
    width: 140,
    render(row) {
      return row.external_ip || '-'
    }
  },
  {
    title: 'Ports',
    key: 'ports',
    ellipsis: { tooltip: true },
    render(row) {
      const ports = row.ports || []
      return ports.join(', ') || '-'
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

// Ingress columns
const ingressColumns: DataTableColumns<K8sIngressInfo> = [
  { title: 'Name', key: 'name', width: 200, ellipsis: { tooltip: true } },
  {
    title: 'Hosts',
    key: 'hosts',
    ellipsis: { tooltip: true },
    render(row) {
      const hosts = row.hosts || []
      if (hosts.length === 0) return '*'
      return hosts.join(', ')
    }
  },
  {
    title: 'Address',
    key: 'address',
    width: 180,
    render(row) {
      return row.address || '-'
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
async function fetchServices() {
  servicesLoading.value = true
  try {
    services.value = await api.k8s.listServices(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch services: ' + (error as Error).message)
    services.value = []
  } finally {
    servicesLoading.value = false
  }
}

async function fetchIngresses() {
  ingressesLoading.value = true
  try {
    ingresses.value = await api.k8s.listIngresses(props.clusterId, props.namespace)
  } catch (error) {
    message.error('Failed to fetch ingresses: ' + (error as Error).message)
    ingresses.value = []
  } finally {
    ingressesLoading.value = false
  }
}

async function refresh() {
  await Promise.all([fetchServices(), fetchIngresses()])
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
    <!-- Services -->
    <NCard title="Services" size="small">
      <NSpin :show="servicesLoading">
        <NDataTable
          v-if="services.length > 0"
          :columns="serviceColumns"
          :data="services"
          :bordered="false"
          size="small"
          max-height="300"
        />
        <NEmpty v-else description="No services found" />
      </NSpin>
    </NCard>

    <!-- Ingresses -->
    <NCard title="Ingresses" size="small">
      <NSpin :show="ingressesLoading">
        <NDataTable
          v-if="ingresses.length > 0"
          :columns="ingressColumns"
          :data="ingresses"
          :bordered="false"
          size="small"
          max-height="300"
        />
        <NEmpty v-else description="No ingresses found" />
      </NSpin>
    </NCard>
  </NSpace>
</template>
