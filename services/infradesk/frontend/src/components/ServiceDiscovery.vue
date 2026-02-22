<script setup lang="ts">
import { ref, computed, h } from 'vue'
import {
  NModal,
  NButton,
  NSpace,
  NEmpty,
  NSpin,
  NIcon,
  NTag,
  NDataTable,
  NUpload,
  NSelect,
  useMessage,
  useDialog,
  type DataTableColumns,
  type UploadFileInfo,
} from 'naive-ui'
import {
  CloudDownloadOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  ServerOutline,
  CloudUploadOutline,
  StopCircleOutline,
} from '@vicons/ionicons5'
import { k8sApi, type DiscoveredService } from '@/api'

// Props
interface Props {
  show: boolean
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  'update:show': [value: boolean]
  imported: [count: number]
}>()

const message = useMessage()
const dialog = useDialog()

// State
const discovering = ref(false)
const importing = ref(false)
const services = ref<DiscoveredService[]>([])
const selectedServices = ref<Set<string>>(new Set())
const kubeconfigContent = ref<string>('')
const kubeconfigUploaded = ref(false)
const abortController = ref<AbortController | null>(null)
const loadingClusters = ref(false)
const clusters = ref<string[]>([])
const selectedCluster = ref<string>('')

// Computed
const showModal = computed({
  get: () => props.show,
  set: (val) => emit('update:show', val),
})

const hasServices = computed(() => services.value.length > 0)

const selectedCount = computed(() => selectedServices.value.size)

const canImport = computed(() => selectedCount.value > 0 && !importing.value)

// Methods
async function handleKubeconfigUpload(options: { file: UploadFileInfo }) {
  const file = options.file.file
  if (!file) return

  const reader = new FileReader()
  reader.onload = async (e) => {
    kubeconfigContent.value = e.target?.result as string
    kubeconfigUploaded.value = true
    message.success('Kubeconfig 已上传')
    
    // 自动加载集群列表
    await loadClusters()
  }
  reader.onerror = () => {
    message.error('读取文件失败')
  }
  reader.readAsText(file)
}

async function loadClusters() {
  if (!kubeconfigContent.value) return

  loadingClusters.value = true
  try {
    const data = await k8sApi.listClusters(kubeconfigContent.value)
    clusters.value = data.clusters || []

    if (clusters.value.length > 0) {
      // 默认选择第一个集群
      selectedCluster.value = clusters.value[0]
      message.success(`检测到 ${clusters.value.length} 个集群`)
    } else {
      message.warning('未找到任何集群配置')
    }
  } catch (e: any) {
    message.error(`加载集群列表失败: ${(e as Error).message}`)
    clusters.value = []
  } finally {
    loadingClusters.value = false
  }
}

function clearKubeconfig() {
  kubeconfigContent.value = ''
  kubeconfigUploaded.value = false
  clusters.value = []
  selectedCluster.value = ''
  message.info('已清除 kubeconfig')
}

async function handleDiscover() {
  discovering.value = true
  services.value = []
  selectedServices.value.clear()
  
  // Create abort controller for cancellation
  abortController.value = new AbortController()
  
  try {
    const data = await k8sApi.discover(
      kubeconfigContent.value || undefined,
      selectedCluster.value || undefined,
      abortController.value.signal
    )
    services.value = data || []

    if (services.value.length === 0) {
      message.warning('未发现任何中间件服务')
    } else {
      message.success(`发现 ${services.value.length} 个中间件服务`)
      // 默认选中所有有凭据的服务
      services.value.forEach(svc => {
        if (svc.has_credentials) {
          selectedServices.value.add(getServiceKey(svc))
        }
      })
    }
  } catch (e: any) {
    const error = e as Error
    // Check if cancelled
    if (error.message.includes('abort') || error.message.includes('cancel')) {
      message.info('扫描已取消')
    } else if (error.message.includes('not available') || error.message.includes('503')) {
      message.error('K8s 服务发现不可用：请上传 kubeconfig 或确保应用运行在 K8s 集群中')
    } else {
      message.error(`发现服务失败: ${error.message}`)
    }
  } finally {
    discovering.value = false
    abortController.value = null
  }
}

function handleCancelDiscover() {
  if (abortController.value) {
    abortController.value.abort()
    abortController.value = null
  }
}

async function handleImport() {
  if (!canImport.value) return
  
  const toImport = services.value.filter(svc => 
    selectedServices.value.has(getServiceKey(svc))
  )
  
  importing.value = true
  try {
    // 第一次尝试导入（不强制覆盖）
    const result = await k8sApi.importConnections(
      toImport,
      false,
      kubeconfigContent.value,
      selectedCluster.value,
      selectedCluster.value // 使用集群名称作为标识
    )
    
    // 统计结果
    const skippedServices = result.results.filter(r => r.skipped)
    const hasSkipped = skippedServices.length > 0
    
    // 如果有跳过的服务，询问是否强制覆盖
    if (hasSkipped) {
      importing.value = false // 在对话框显示前停止 loading
      
      const skippedNames = skippedServices.map(r => r.name).join('\n')
      
      // 使用 Naive UI 的 Dialog 进行二次确认
      dialog.warning({
        title: '发现重复连接',
        content: `以下 ${skippedServices.length} 个连接已存在：\n\n${skippedNames}\n\n是否要强制覆盖这些已有连接？`,
        positiveText: '强制覆盖',
        negativeText: '取消',
        onPositiveClick: async () => {
          // 用户确认强制覆盖
          importing.value = true
          try {
            const overrideResult = await k8sApi.importConnections(
              toImport,
              true,
              kubeconfigContent.value,
              selectedCluster.value,
              selectedCluster.value
            )
            
            // 显示覆盖结果
            const successMsg = []
            if (overrideResult.success > overrideResult.updated) {
              successMsg.push(`新建 ${overrideResult.success - overrideResult.updated} 个`)
            }
            if (overrideResult.updated > 0) {
              successMsg.push(`覆盖 ${overrideResult.updated} 个`)
            }
            
            if (successMsg.length > 0) {
              message.success(`成功导入连接：${successMsg.join('，')}`)
              emit('imported', overrideResult.success)
            }
            
            if (overrideResult.failed > 0) {
              const failedNames = overrideResult.results
                .filter(r => !r.success && !r.skipped)
                .map(r => `${r.name}: ${r.error}`)
                .join(', ')
              message.error(`${overrideResult.failed} 个连接导入失败: ${failedNames}`)
            }
            
            // 清空选择并关闭弹窗
            if (overrideResult.success > 0) {
              services.value = []
              selectedServices.value.clear()
              showModal.value = false
            }
          } catch (e) {
            message.error(`强制覆盖失败: ${(e as Error).message}`)
          } finally {
            importing.value = false
          }
        },
        onNegativeClick: () => {
          // 用户取消覆盖
          const newCreated = result.success - (result.updated || 0)
          if (newCreated > 0) {
            message.success(`成功导入 ${newCreated} 个新连接，跳过 ${skippedServices.length} 个重复连接`)
            emit('imported', newCreated)
            // 清空选择并关闭弹窗
            services.value = []
            selectedServices.value.clear()
            showModal.value = false
          } else {
            message.info(`跳过 ${skippedServices.length} 个重复连接`)
          }
        }
      })
      return // 提前返回，不执行后面的 finally
    } else {
      // 没有跳过的服务，直接显示结果
      if (result.success > 0) {
        message.success(`成功导入 ${result.success} 个连接`)
        emit('imported', result.success)
      }
      
      // 只显示真正失败的连接（排除被跳过的）
      const reallyFailed = result.results.filter(r => !r.success && !r.skipped)
      if (reallyFailed.length > 0) {
        const failedNames = reallyFailed
          .map(r => `${r.name}: ${r.error}`)
          .join(', ')
        message.error(`${reallyFailed.length} 个连接导入失败: ${failedNames}`)
      }
      
      if (result.success > 0) {
        // 清空选择并关闭弹窗
        services.value = []
        selectedServices.value.clear()
        showModal.value = false
      }
    }
  } catch (e) {
    message.error(`导入失败: ${(e as Error).message}`)
  } finally {
    importing.value = false
  }
}

function getServiceKey(service: DiscoveredService): string {
  return `${service.namespace}/${service.name}`
}


function toggleAll() {
  if (selectedServices.value.size === services.value.length) {
    selectedServices.value.clear()
  } else {
    services.value.forEach(svc => {
      selectedServices.value.add(getServiceKey(svc))
    })
  }
}

function getTypeColor(type: string) {
  const colors: Record<string, string> = {
    mysql: '#00758F',
    postgresql: '#336791',
    redis: '#DC382D',
    mongodb: '#4DB33D',
    minio: '#C72C48',
  }
  return colors[type] || '#00FFFF'
}

// Table columns
const columns: DataTableColumns<DiscoveredService> = [
  {
    type: 'selection',
  },
  {
    title: '服务名',
    key: 'name',
    render: (row) => row.name,
  },
  {
    title: '类型',
    key: 'type',
    render: (row) => {
      return h(NTag, {
        type: 'info',
        size: 'small',
        style: { backgroundColor: getTypeColor(row.type) + '20', color: getTypeColor(row.type) }
      }, { default: () => row.type.toUpperCase() })
    },
  },
  {
    title: '命名空间',
    key: 'namespace',
    render: (row) => row.namespace,
  },
  {
    title: '地址',
    key: 'host',
    render: (row) => `${row.host}:${row.port}`,
    ellipsis: {
      tooltip: true,
    },
  },
  {
    title: '用户名',
    key: 'username',
    render: (row) => row.username || '-',
  },
  {
    title: '凭据状态',
    key: 'has_credentials',
    render: (row) => {
      return h(NIcon, {
        size: 18,
        color: row.has_credentials ? '#22C55E' : '#EF4444',
      }, {
        default: () => h(row.has_credentials ? CheckmarkCircleOutline : CloseCircleOutline)
      })
    },
  },
]

// Don't auto discover - wait for user to click the button
</script>

<template>
  <NModal 
    v-model:show="showModal" 
    preset="card"
    title="自动发现 K8s 服务"
    style="width: 1000px; max-width: 90vw"
    :mask-closable="false"
  >
    <div class="discovery-container">
      <!-- Kubeconfig 上传区域 -->
      <div class="kubeconfig-section" :style="{ opacity: discovering ? 0.6 : 1, pointerEvents: discovering ? 'none' : 'auto' }">
        <NSpace v-if="!kubeconfigUploaded" align="center" :size="12">
          <NUpload
            :max="1"
            :show-file-list="false"
            :custom-request="handleKubeconfigUpload"
          >
            <NButton secondary>
              <template #icon>
                <NIcon><CloudUploadOutline /></NIcon>
              </template>
              上传 Kubeconfig
            </NButton>
          </NUpload>
          <span class="hint-text">支持任何文件（如 ~/.kube/config）</span>
        </NSpace>

        <NSpace v-else align="center" justify="space-between">
          <NSpace vertical :size="8" style="flex: 1">
            <NTag type="success" size="small">
              <template #icon>
                <NIcon><CheckmarkCircleOutline /></NIcon>
              </template>
              已上传 Kubeconfig
            </NTag>
            
            <!-- 集群选择下拉框 -->
            <div v-if="clusters.length > 0" class="cluster-select">
              <span class="label-text">选择集群：</span>
              <NSelect
                v-model:value="selectedCluster"
                :options="clusters.map(c => ({ label: c, value: c }))"
                :loading="loadingClusters"
                placeholder="选择一个集群"
                size="small"
                style="width: 300px"
              />
            </div>
            <span v-else-if="loadingClusters" class="hint-text">正在加载集群列表...</span>
          </NSpace>
          
          <NButton size="tiny" quaternary @click="clearKubeconfig">
            清除
          </NButton>
        </NSpace>
      </div>

      <!-- 顶部操作栏 -->
      <div class="actions-bar">
        <NSpace justify="space-between" align="center">
          <div class="info-text">
            <template v-if="hasServices">
              已发现 {{ services.length }} 个中间件服务，
              已选择 {{ selectedCount }} 个
            </template>
            <template v-else>
              点击"扫描集群"按钮开始发现集群中的中间件服务（扫描可能需要一些时间）
            </template>
          </div>
          <NSpace>
            <NButton 
              v-if="discovering"
              @click="handleCancelDiscover"
              type="error"
              secondary
            >
              <template #icon>
                <NIcon><StopCircleOutline /></NIcon>
              </template>
              取消扫描
            </NButton>
            <NButton 
              v-else
              @click="handleDiscover" 
              :loading="discovering"
              secondary
            >
              <template #icon>
                <NIcon><CloudDownloadOutline /></NIcon>
              </template>
              {{ hasServices ? '重新扫描' : '扫描集群' }}
            </NButton>
          </NSpace>
        </NSpace>
      </div>

      <NSpin :show="discovering">
        <!-- 服务列表 -->
        <div v-if="hasServices" class="services-table">
          <NDataTable
            :columns="columns"
            :data="services"
            :row-key="(row: DiscoveredService) => getServiceKey(row)"
            :checked-row-keys="Array.from(selectedServices)"
            @update:checked-row-keys="(keys) => selectedServices = new Set(keys as string[])"
            :pagination="false"
            :max-height="400"
            size="small"
          />
          
          <div class="table-footer">
            <NSpace justify="space-between">
              <NButton 
                size="small" 
                @click="toggleAll"
                quaternary
              >
                {{ selectedServices.size === services.length ? '取消全选' : '全选' }}
              </NButton>
              <span class="hint-text">
                <NIcon><ServerOutline /></NIcon>
                选择要导入的服务（凭据缺失的可在导入后手动配置）
              </span>
            </NSpace>
          </div>
        </div>

        <!-- 空状态 -->
        <NEmpty 
          v-else
          :description="discovering ? '正在扫描集群...' : '暂无发现的服务'"
          class="empty-state"
        >
          <template #icon>
            <NIcon size="48" :component="ServerOutline" />
          </template>
        </NEmpty>
      </NSpin>
    </div>

    <template #footer>
      <NSpace justify="end">
        <NButton @click="showModal = false">取消</NButton>
        <NButton 
          type="primary" 
          :disabled="!canImport"
          :loading="importing"
          @click="handleImport"
        >
          导入选中的服务 ({{ selectedCount }})
        </NButton>
      </NSpace>
    </template>
  </NModal>
</template>

<style scoped>
.discovery-container {
  min-height: 300px;
}

.kubeconfig-section {
  margin-bottom: 16px;
  padding: 12px;
  background: var(--zx-bg-secondary);
  border-radius: 8px;
}

.cluster-select {
  display: flex;
  align-items: center;
  gap: 8px;
}

.label-text {
  font-size: 12px;
  color: var(--zx-text-secondary);
  white-space: nowrap;
}

.actions-bar {
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--zx-border);
}

.info-text {
  font-size: 13px;
  color: var(--zx-text-secondary);
}

.services-table {
  margin-top: 12px;
}

.table-footer {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--zx-border);
}

.hint-text {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--zx-text-tertiary);
}

.empty-state {
  padding: 48px 24px;
}

:deep(.n-data-table) {
  --n-th-font-size: 12px;
  --n-td-font-size: 12px;
}

:deep(.n-data-table-th) {
  font-weight: 600;
}
</style>

