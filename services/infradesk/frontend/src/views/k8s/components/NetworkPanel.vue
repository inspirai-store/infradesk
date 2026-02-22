<script setup lang="ts">
import { ref, watch, h, computed } from 'vue'
import {
  NDataTable,
  NCard,
  NSpace,
  NTag,
  NSpin,
  NEmpty,
  NButton,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NRadioGroup,
  NRadio,
  NSelect,
  NTooltip,
  useMessage
} from 'naive-ui'
import type { DataTableColumns, SelectOption } from 'naive-ui'
import { api } from '@/api/adapter'
import type { K8sServiceInfo, K8sIngressInfo, ProxyPodInfo } from '@/api/types'

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

// Proxy state
const proxies = ref<ProxyPodInfo[]>([])
const proxiesLoading = ref(false)
const showProxyModal = ref(false)
const proxyModalLoading = ref(false)
const selectedService = ref<K8sServiceInfo | null>(null)
const proxyMode = ref<'existing' | 'create'>('existing')
const selectedProxyName = ref<string | null>(null)
const newProxyForm = ref({
  name: '',
  targetPort: 3306,
  image: '' // 可选，默认使用 alpine/socat
})

// Computed proxy options
const proxyOptions = computed<SelectOption[]>(() => {
  return proxies.value
    .filter((p) => p.status === 'Running')
    .map((p) => ({
      label: `${p.name} (${p.status})${p.target_host ? ` → ${p.target_host}:${p.target_port}` : ''}`,
      value: p.name
    }))
})

// Service columns
const serviceColumns: DataTableColumns<K8sServiceInfo> = [
  { title: 'Name', key: 'name', width: 180, ellipsis: { tooltip: true } },
  {
    title: 'Type',
    key: 'service_type',
    width: 110,
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
  { title: 'Cluster IP', key: 'cluster_ip', width: 130 },
  {
    title: 'Target / External IP',
    key: 'target',
    width: 200,
    ellipsis: { tooltip: true },
    render(row) {
      // ExternalName 显示目标域名
      if (row.service_type === 'ExternalName' && row.external_name) {
        return h(
          NTooltip,
          { trigger: 'hover' },
          {
            trigger: () =>
              h('span', { style: { color: '#f0a020', cursor: 'pointer' } }, row.external_name),
            default: () => `ExternalName 目标: ${row.external_name}`
          }
        )
      }
      return row.external_ip || '-'
    }
  },
  {
    title: 'Ports',
    key: 'ports',
    width: 120,
    ellipsis: { tooltip: true },
    render(row) {
      const ports = row.ports || []
      return ports.join(', ') || '-'
    }
  },
  {
    title: 'Created',
    key: 'created_at',
    width: 150,
    render(row) {
      if (!row.created_at) return '-'
      return new Date(row.created_at).toLocaleString()
    }
  },
  {
    title: 'Actions',
    key: 'actions',
    width: 100,
    render(row) {
      // 仅对 ExternalName 类型显示代理连接按钮
      if (row.service_type === 'ExternalName' && row.external_name) {
        return h(
          NButton,
          {
            size: 'small',
            type: 'primary',
            ghost: true,
            onClick: () => openProxyModal(row)
          },
          () => '代理连接'
        )
      }
      return null
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

async function fetchProxies() {
  proxiesLoading.value = true
  try {
    proxies.value = await api.k8s.listProxies(props.clusterId, props.namespace)
  } catch (error) {
    console.error('Failed to fetch proxies:', error)
    proxies.value = []
  } finally {
    proxiesLoading.value = false
  }
}

async function refresh() {
  await Promise.all([fetchServices(), fetchIngresses(), fetchProxies()])
}

// Proxy modal methods
function openProxyModal(service: K8sServiceInfo) {
  selectedService.value = service
  proxyMode.value = proxies.value.length > 0 ? 'existing' : 'create'
  selectedProxyName.value = null
  newProxyForm.value = {
    name: `${service.name}-proxy`,
    targetPort: 3306, // 默认 MySQL 端口
    image: '' // 留空使用默认镜像 alpine/socat
  }
  showProxyModal.value = true
}

function closeProxyModal() {
  showProxyModal.value = false
  selectedService.value = null
}

async function handleProxySubmit() {
  if (!selectedService.value?.external_name) {
    message.error('目标服务无效')
    return
  }

  proxyModalLoading.value = true
  try {
    if (proxyMode.value === 'create') {
      // 创建新代理
      if (!newProxyForm.value.name) {
        message.error('请输入代理名称')
        return
      }
      await api.k8s.createProxy(props.clusterId, props.namespace, {
        proxy_name: newProxyForm.value.name,
        target_host: selectedService.value.external_name,
        target_port: newProxyForm.value.targetPort,
        target_type: 'mysql', // 可扩展支持其他类型
        image: newProxyForm.value.image || undefined // 留空使用默认镜像
      })
      message.success('代理创建成功，请等待 Pod 就绪后使用')
      await fetchProxies()
    } else {
      // 使用已有代理
      if (!selectedProxyName.value) {
        message.error('请选择一个代理')
        return
      }
      message.success(`已选择代理: ${selectedProxyName.value}，可在端口转发中使用`)
    }
    closeProxyModal()
  } catch (error) {
    message.error('操作失败: ' + (error as Error).message)
  } finally {
    proxyModalLoading.value = false
  }
}

async function handleDeleteProxy(proxyName: string) {
  try {
    await api.k8s.deleteProxy(props.clusterId, props.namespace, proxyName)
    message.success('代理已删除')
    await fetchProxies()
  } catch (error) {
    message.error('删除失败: ' + (error as Error).message)
  }
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

    <!-- Proxies (zeni-x managed) -->
    <NCard title="Zeni-X Proxies" size="small">
      <template #header-extra>
        <NButton size="small" @click="fetchProxies">刷新</NButton>
      </template>
      <NSpin :show="proxiesLoading">
        <div v-if="proxies.length > 0" class="proxy-list">
          <div v-for="proxy in proxies" :key="proxy.name" class="proxy-item">
            <NSpace align="center" justify="space-between" style="width: 100%">
              <NSpace align="center">
                <NTag :type="proxy.status === 'Running' ? 'success' : 'warning'" size="small">
                  {{ proxy.status }}
                </NTag>
                <span class="proxy-name">{{ proxy.name }}</span>
                <span v-if="proxy.target_host" class="proxy-target">
                  → {{ proxy.target_host }}:{{ proxy.target_port }}
                </span>
              </NSpace>
              <NButton size="tiny" type="error" ghost @click="handleDeleteProxy(proxy.name)">
                删除
              </NButton>
            </NSpace>
          </div>
        </div>
        <NEmpty v-else description="暂无 Zeni-X 代理，可点击 ExternalName 服务的代理连接按钮创建" />
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

    <!-- Proxy Modal -->
    <NModal
      v-model:show="showProxyModal"
      preset="dialog"
      title="连接 ExternalName 服务"
      positive-text="确定"
      negative-text="取消"
      :loading="proxyModalLoading"
      @positive-click="handleProxySubmit"
      @negative-click="closeProxyModal"
    >
      <div v-if="selectedService" class="proxy-modal-content">
        <p class="target-info">
          目标服务: <strong>{{ selectedService.name }}</strong>
        </p>
        <p class="target-host">
          目标地址: <code>{{ selectedService.external_name }}</code>
        </p>
        <p class="hint">
          此服务指向外部地址（如 VPC 内网 RDS），需要通过代理 Pod 连接。
        </p>

        <NForm label-placement="left" label-width="100" style="margin-top: 16px">
          <NFormItem label="代理方式">
            <NRadioGroup v-model:value="proxyMode">
              <NRadio value="existing" :disabled="proxyOptions.length === 0">
                选择已有代理
              </NRadio>
              <NRadio value="create">创建新代理</NRadio>
            </NRadioGroup>
          </NFormItem>

          <template v-if="proxyMode === 'existing'">
            <NFormItem label="选择代理">
              <NSelect
                v-model:value="selectedProxyName"
                :options="proxyOptions"
                placeholder="请选择一个已有代理"
                :disabled="proxyOptions.length === 0"
              />
            </NFormItem>
            <p v-if="proxyOptions.length === 0" class="no-proxy-hint">
              当前命名空间无可用代理，请创建新代理
            </p>
          </template>

          <template v-if="proxyMode === 'create'">
            <NFormItem label="代理名称">
              <NInput v-model:value="newProxyForm.name" placeholder="输入代理名称" />
            </NFormItem>
            <NFormItem label="目标端口">
              <NInputNumber
                v-model:value="newProxyForm.targetPort"
                :min="1"
                :max="65535"
                placeholder="目标端口"
              />
            </NFormItem>
            <NFormItem label="容器镜像">
              <NInput
                v-model:value="newProxyForm.image"
                placeholder="留空使用默认镜像 alpine/socat"
              />
              <template #feedback>
                <span class="image-hint">
                  内网环境可配置私有仓库镜像，如: registry.example.com/alpine/socat
                </span>
              </template>
            </NFormItem>
          </template>
        </NForm>
      </div>
    </NModal>
  </NSpace>
</template>

<style scoped>
.proxy-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.proxy-item {
  padding: 8px 12px;
  background: var(--n-color-embedded);
  border-radius: 4px;
}

.proxy-name {
  font-weight: 500;
}

.proxy-target {
  color: var(--n-text-color-3);
  font-size: 12px;
}

.proxy-modal-content {
  padding: 8px 0;
}

.target-info {
  margin: 0 0 8px;
}

.target-host {
  margin: 0 0 8px;
}

.target-host code {
  background: var(--n-color-embedded);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 13px;
}

.hint {
  color: var(--n-text-color-3);
  font-size: 13px;
  margin: 0;
}

.no-proxy-hint {
  color: var(--n-text-color-3);
  font-size: 12px;
  margin: 4px 0 0;
}

.image-hint {
  color: var(--n-text-color-3);
  font-size: 12px;
}
</style>
