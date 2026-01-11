<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import {
  NCard,
  NGrid,
  NGridItem,
  NButton,
  NIcon,
  NSpace,
  NEmpty,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NSelect,
  NSwitch,
  NTag,
  NPopconfirm,
  NSpin,
  useMessage,
} from 'naive-ui'
import {
  AddOutline,
  ServerOutline,
  KeyOutline,
  CreateOutline,
  TrashOutline,
  FlashOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  SearchOutline,
  CloudUploadOutline,
} from '@vicons/ionicons5'
import { useConnectionsStore } from '@/stores/connections'
import ServiceDiscovery from '@/components/ServiceDiscovery.vue'
import PortForwardStatus from '@/components/PortForwardStatus.vue'
import { connectionApi, clusterApi, type Connection, type Cluster } from '@/api'

const store = useConnectionsStore()
const message = useMessage()

// Modal state
const showModal = ref(false)
const showDiscovery = ref(false)
const isEditing = ref(false)
const formRef = ref()
const testing = ref(false)
const testResult = ref<{ success: boolean; error?: string } | null>(null)

// K8s & Cluster state
const connectionMethod = ref<'direct' | 'k8s'>('direct')
const clusters = ref<Cluster[]>([])
const loadingClusters = ref(false)
const kubeconfigContent = ref('')
const selectedK8sContext = ref('')

// Form model
const formModel = ref<Connection>({
  name: '',
  type: 'mysql',
  host: '127.0.0.1',
  port: 3306,
  username: '',
  password: '',
  database_name: '',
  is_default: false,
  source: 'local',
})

const clusterOptions = computed(() => {
  const options = clusters.value.map(c => ({
    label: c.name,
    value: c.id
  }))
  options.push({ label: '+ 新建集群 (上传 Kubeconfig)', value: -1 })
  return options
})

async function fetchClusters() {
  loadingClusters.value = true
  try {
    const data = await clusterApi.getAll()
    clusters.value = data || []
  } catch (e) {
    console.error('Failed to fetch clusters:', e)
  } finally {
    loadingClusters.value = false
  }
}

function handleKubeconfigUpload(options: { file: any }) {
  const file = options.file.file
  if (!file) return

  const reader = new FileReader()
  reader.onload = (e) => {
    kubeconfigContent.value = e.target?.result as string
    message.success('Kubeconfig 已读取')
  }
  reader.readAsText(file)
}

const typeOptions = [
  { label: 'MySQL', value: 'mysql' },
  { label: 'Redis', value: 'redis' },
  { label: 'MongoDB', value: 'mongodb' },
  { label: 'MinIO', value: 'minio' },
]

const defaultPorts: Record<string, number> = {
  mysql: 3306,
  redis: 6379,
  mongodb: 27017,
  minio: 9000,
}

// Grouped connections
const groupedConnections = computed(() => {
  return {
    mysql: store.mysqlConnections,
    redis: store.redisConnections,
    mongodb: store.mongodbConnections,
    minio: store.minioConnections,
  }
})

function getTypeIcon(type: string) {
  switch (type) {
    case 'mysql':
    case 'mongodb':
    case 'minio':
      return ServerOutline
    case 'redis':
      return KeyOutline
    default:
      return ServerOutline
  }
}

function getTypeColor(type: string) {
  switch (type) {
    case 'mysql':
      return '#00758F'
    case 'redis':
      return '#DC382D'
    case 'mongodb':
      return '#4DB33D'
    case 'minio':
      return '#C72C48'
    default:
      return '#00FFFF'
  }
}

function getTypeLabel(type: string) {
  switch (type) {
    case 'mysql':
      return 'MySQL'
    case 'redis':
      return 'Redis'
    case 'mongodb':
      return 'MongoDB'
    case 'minio':
      return 'MinIO'
    default:
      return type
  }
}

function openAddModal(type?: string) {
  isEditing.value = false
  connectionMethod.value = 'direct'
  kubeconfigContent.value = ''
  formModel.value = {
    name: '',
    type: (type as Connection['type']) || 'mysql',
    host: '127.0.0.1',
    port: defaultPorts[type || 'mysql'],
    username: '',
    password: '',
    database_name: '',
    is_default: false,
    source: 'local',
  }
  testResult.value = null
  showModal.value = true
  fetchClusters()
}

function openEditModal(conn: Connection) {
  isEditing.value = true
  connectionMethod.value = conn.source === 'k8s' ? 'k8s' : 'direct'
  kubeconfigContent.value = ''
  formModel.value = { ...conn }
  testResult.value = null
  showModal.value = true
  fetchClusters()
}

function onTypeChange(type: string) {
  formModel.value.port = defaultPorts[type] || 3306
  if (connectionMethod.value === 'k8s') {
    formModel.value.k8s_service_port = defaultPorts[type] || 3306
  }
}

async function handleTest() {
  testing.value = true
  testResult.value = null
  try {
    let result
    if (connectionMethod.value === 'k8s') {
      // K8s 临时测试
      result = await connectionApi.testK8s({
        type: formModel.value.type,
        username: formModel.value.username,
        password: formModel.value.password,
        database_name: formModel.value.database_name,
        kubeconfig: kubeconfigContent.value || undefined,
        context: selectedK8sContext.value || undefined,
        k8s_namespace: formModel.value.k8s_namespace || '',
        k8s_service_name: formModel.value.k8s_service_name || '',
        k8s_service_port: formModel.value.k8s_service_port || 0,
        cluster_id: formModel.value.cluster_id || undefined,  // Pass cluster_id for kubeconfig lookup
      })
    } else {
      // 直连测试
      result = await store.testConnection(formModel.value)
    }
    
    testResult.value = result
    if (result.success) {
      message.success('连接成功！')
    } else {
      message.error(result.error || '连接失败')
    }
  } catch (e: any) {
    message.error(e.message || '测试连接出错')
  } finally {
    testing.value = false
  }
}

async function handleSubmit() {
  try {
    const data = { ...formModel.value }
    data.source = connectionMethod.value === 'k8s' ? 'k8s' : 'local'

    if (connectionMethod.value === 'k8s') {
      // 如果是新建集群，先创建集群
      if (data.cluster_id === -1) {
        if (!kubeconfigContent.value) {
          message.error('请上传 Kubeconfig')
          return
        }
        const cluster = await clusterApi.create({
          name: `Cluster-${Date.now()}`, // 简单生成个名字，或者让用户输
          kubeconfig: kubeconfigContent.value,
          context: selectedK8sContext.value,
        })
        data.cluster_id = cluster.id
      }
      
      // K8s 模式下 Host 默认为 localhost，Port 为 0 (等待转发)
      if (!isEditing.value) {
        data.host = 'localhost'
        data.port = 0
      }
    }

    if (isEditing.value && data.id) {
      await store.updateConnection(data.id, data)
      message.success('连接已更新')
    } else {
      await store.createConnection(data)
      message.success('连接已创建')
    }
    showModal.value = false
  } catch (e) {
    message.error((e as Error).message)
  }
}

async function handleDelete(conn: Connection) {
  if (!conn.id) return
  try {
    await store.deleteConnection(conn.id)
    message.success('连接已删除')
  } catch (e) {
    message.error((e as Error).message)
  }
}

function isActive(conn: Connection) {
  return store.getActiveConnectionId(conn.type) === conn.id
}

function setActive(conn: Connection) {
  if (conn.id) {
    store.setActiveConnection(conn.type, conn.id)
    store.saveToStorage()
    message.success(`已切换到 ${conn.name}`)
  }
}

function handleImported(_count: number) {
  // 重新加载连接列表
  store.fetchConnections()
}

onMounted(() => {
  store.initFromStorage()
  store.fetchConnections()
})
</script>

<template>
  <div class="connections-page">
    <!-- Header -->
    <div class="page-header">
      <NSpace justify="space-between" align="center">
        <div>
          <h1 class="title-font neon-text">连接管理</h1>
          <p class="subtitle">管理数据库连接配置</p>
        </div>
        <NSpace>
          <NButton type="info" @click="showDiscovery = true">
            <template #icon>
              <NIcon><SearchOutline /></NIcon>
            </template>
            自动发现
          </NButton>
          <NButton type="default" @click="$router.push('/port-forward')">
            <template #icon>
              <NIcon><FlashOutline /></NIcon>
            </template>
            端口转发
          </NButton>
          <NButton type="primary" @click="openAddModal()">
            <template #icon>
              <NIcon><AddOutline /></NIcon>
            </template>
            新建连接
          </NButton>
        </NSpace>
      </NSpace>
    </div>

    <NSpin :show="store.loading">
      <!-- Connection Groups -->
      <div v-for="(connections, type) in groupedConnections" :key="type" class="connection-group">
        <div class="group-header">
          <NSpace align="center" :size="8">
            <NIcon :size="18" :color="getTypeColor(type as string)">
              <component :is="getTypeIcon(type as string)" />
            </NIcon>
            <span class="group-title">{{ getTypeLabel(type as string) }}</span>
            <NTag size="tiny" round>{{ connections.length }}</NTag>
          </NSpace>
          <NButton size="tiny" quaternary @click="openAddModal(type as string)">
            <template #icon>
              <NIcon><AddOutline /></NIcon>
            </template>
            添加
          </NButton>
        </div>

        <NGrid v-if="connections.length > 0" :cols="3" :x-gap="12" :y-gap="12">
          <NGridItem v-for="conn in connections" :key="conn.id">
            <NCard 
              class="connection-card glass-card" 
              :class="{ active: isActive(conn) }"
              size="small"
            >
              <NSpace vertical :size="8">
                <NSpace justify="space-between" align="center">
                  <span class="conn-name">{{ conn.name }}</span>
                  <NSpace :size="4">
                    <NTag v-if="isActive(conn)" type="success" size="tiny" round>当前</NTag>
                    <NTag v-if="conn.is_default" type="info" size="tiny" round>默认</NTag>
                  </NSpace>
                </NSpace>
                
                <!-- 端口转发状态 -->
                <PortForwardStatus
                  v-if="conn.id && conn.forward_id"
                  :connection-id="conn.id"
                  :auto-refresh="true"
                  :refresh-interval="10000"
                />
                
                <div class="conn-info">
                  <template v-if="conn.forward_local_port">
                    localhost:{{ conn.forward_local_port }} → {{ conn.host }}:{{ conn.port }}
                  </template>
                  <template v-else>
                    <span>{{ conn.host }}:{{ conn.port }}</span>
                    <span v-if="conn.username"> · {{ conn.username }}</span>
                  </template>
                </div>

                <NSpace :size="8">
                  <NButton 
                    v-if="!isActive(conn)"
                    size="tiny" 
                    type="primary" 
                    ghost
                    @click="setActive(conn)"
                  >
                    <template #icon>
                      <NIcon><CheckmarkCircleOutline /></NIcon>
                    </template>
                    使用
                  </NButton>
                  <NButton size="tiny" quaternary @click="openEditModal(conn)">
                    <template #icon>
                      <NIcon><CreateOutline /></NIcon>
                    </template>
                    编辑
                  </NButton>
                  <NPopconfirm @positive-click="handleDelete(conn)">
                    <template #trigger>
                      <NButton size="tiny" quaternary type="error">
                        <template #icon>
                          <NIcon><TrashOutline /></NIcon>
                        </template>
                        删除
                      </NButton>
                    </template>
                    确定删除连接 "{{ conn.name }}" 吗？
                  </NPopconfirm>
                </NSpace>
              </NSpace>
            </NCard>
          </NGridItem>
        </NGrid>

        <NEmpty v-else description="暂无连接配置" size="small" class="empty-state">
          <template #extra>
            <NButton size="small" @click="openAddModal(type as string)">
              <template #icon>
                <NIcon><AddOutline /></NIcon>
              </template>
              添加 {{ getTypeLabel(type as string) }} 连接
            </NButton>
          </template>
        </NEmpty>
      </div>
    </NSpin>

    <!-- Add/Edit Modal -->
    <NModal 
      v-model:show="showModal" 
      preset="card"
      :title="isEditing ? '编辑连接' : '新建连接'"
      style="width: 480px"
      :mask-closable="false"
    >
      <NForm
        ref="formRef"
        :model="formModel"
        label-placement="left"
        label-width="80"
      >
        <NFormItem label="连接名称" path="name">
          <NInput v-model:value="formModel.name" placeholder="例如：生产环境 MySQL" />
        </NFormItem>

        <NFormItem label="类型" path="type">
          <NSelect 
            v-model:value="formModel.type" 
            :options="typeOptions" 
            :disabled="isEditing"
            @update:value="onTypeChange"
          />
        </NFormItem>

        <NFormItem label="连接方式">
          <NSelect
            v-model:value="connectionMethod"
            :options="[
              { label: '直连 (Host/Port)', value: 'direct' },
              { label: 'K8s Service (PortForward)', value: 'k8s' }
            ]"
          />
        </NFormItem>

        <!-- 直连字段 -->
        <template v-if="connectionMethod === 'direct'">
          <NFormItem label="主机地址" path="host">
            <NInput v-model:value="formModel.host" placeholder="127.0.0.1 或 IP 地址" />
          </NFormItem>

          <NFormItem label="端口" path="port">
            <NInputNumber v-model:value="formModel.port" :min="1" :max="65535" style="width: 100%" />
          </NFormItem>
        </template>

        <!-- K8s 字段 -->
        <template v-else>
          <NFormItem label="集群" path="cluster_id">
            <NSelect
              v-model:value="formModel.cluster_id"
              :options="clusterOptions"
              :loading="loadingClusters"
              placeholder="选择已保存的集群或新建"
            />
          </NFormItem>

          <NFormItem v-if="formModel.cluster_id === -1" label="Kubeconfig">
            <NUpload
              :max="1"
              :show-file-list="false"
              :custom-request="handleKubeconfigUpload"
            >
              <NButton secondary block>
                <template #icon><NIcon><CloudUploadOutline /></NIcon></template>
                {{ kubeconfigContent ? '已上传' : '点击上传 Kubeconfig' }}
              </NButton>
            </NUpload>
          </NFormItem>

          <NFormItem label="命名空间" path="k8s_namespace">
            <NInput v-model:value="formModel.k8s_namespace" placeholder="例如: default" />
          </NFormItem>

          <NFormItem label="服务名称" path="k8s_service_name">
            <NInput v-model:value="formModel.k8s_service_name" placeholder="例如: mysql-service" />
          </NFormItem>

          <NFormItem label="服务端口" path="k8s_service_port">
            <NInputNumber v-model:value="formModel.k8s_service_port" :min="1" :max="65535" style="width: 100%" />
          </NFormItem>

          <NFormItem label="本地端口" path="forward_local_port">
            <NInputNumber
              v-model:value="formModel.forward_local_port"
              :min="0"
              :max="65535"
              style="width: 100%"
              placeholder="留空自动分配"
            />
            <template #feedback>
              <span style="color: var(--zx-text-secondary); font-size: 11px;">
                端口转发的本地端口，留空或设为 0 则自动分配可用端口
              </span>
            </template>
          </NFormItem>
        </template>

        <NFormItem label="用户名" path="username">
          <NInput v-model:value="formModel.username" placeholder="数据库用户名" />
        </NFormItem>

        <NFormItem label="密码" path="password">
          <NInput
            v-model:value="formModel.password"
            placeholder="数据库密码"
          />
        </NFormItem>

        <NFormItem v-if="formModel.type === 'mysql' || formModel.type === 'mongodb'" label="数据库" path="database_name">
          <NInput v-model:value="formModel.database_name" placeholder="默认数据库（可选）" />
        </NFormItem>

        <NFormItem v-if="formModel.type === 'redis'" label="DB 索引" path="database_name">
          <NInput v-model:value="formModel.database_name" placeholder="默认 0" />
        </NFormItem>

        <NFormItem label="设为默认" path="is_default">
          <NSwitch v-model:value="formModel.is_default" />
        </NFormItem>

        <!-- Test Result -->
        <div v-if="testResult" class="test-result" :class="testResult.success ? 'success' : 'error'">
          <NIcon :size="16">
            <CheckmarkCircleOutline v-if="testResult.success" />
            <CloseCircleOutline v-else />
          </NIcon>
          <span>{{ testResult.success ? '连接成功' : testResult.error }}</span>
        </div>
      </NForm>

      <template #footer>
        <NSpace justify="space-between">
          <NButton :loading="testing" @click="handleTest">
            <template #icon>
              <NIcon><FlashOutline /></NIcon>
            </template>
            测试连接
          </NButton>
          <NSpace>
            <NButton @click="showModal = false">取消</NButton>
            <NButton type="primary" @click="handleSubmit">
              {{ isEditing ? '保存' : '创建' }}
            </NButton>
          </NSpace>
        </NSpace>
      </template>
    </NModal>

    <!-- Service Discovery Modal -->
    <ServiceDiscovery 
      v-model:show="showDiscovery" 
      @imported="handleImported"
    />
  </div>
</template>

<style scoped>
.connections-page {
  padding: 20px;
  min-height: 100%;
}

.page-header {
  margin-bottom: 24px;
}

.page-header h1 {
  font-size: 22px;
  margin-bottom: 4px;
}

.subtitle {
  color: var(--zx-text-secondary);
  font-size: 12px;
}

.connection-group {
  margin-bottom: 24px;
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--zx-border);
}

.group-title {
  font-size: 14px;
  font-weight: 600;
}

.connection-card {
  transition: all 0.2s ease;
}

.connection-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 255, 255, 0.12);
}

.connection-card.active {
  border-color: var(--zx-accent-cyan);
  box-shadow: 0 0 12px rgba(0, 255, 255, 0.2);
}

.conn-name {
  font-weight: 600;
  font-size: 13px;
}

.conn-info {
  font-size: 11px;
  color: var(--zx-text-secondary);
}

.empty-state {
  padding: 24px;
  background: var(--zx-bg-secondary);
  border-radius: 8px;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 12px;
  margin-bottom: 12px;
}

.test-result.success {
  background: rgba(34, 197, 94, 0.1);
  color: #22C55E;
}

.test-result.error {
  background: rgba(239, 68, 68, 0.1);
  color: #EF4444;
}
</style>

