<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { 
  NCard, 
  NGrid, 
  NGridItem, 
  NStatistic, 
  NIcon, 
  NSpace,
  NSpin,
  NTag,
} from 'naive-ui'
import { 
  ServerOutline, 
  KeyOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
} from '@vicons/ionicons5'
import { mysqlApi, redisApi } from '@/api'

const router = useRouter()

interface ServiceStatus {
  name: string
  type: 'mysql' | 'redis'
  connected: boolean
  version: string
  host: string
  port: number
  extra?: Record<string, unknown>
}

const services = ref<ServiceStatus[]>([])
const loading = ref(true)

async function fetchStatus() {
  loading.value = true
  const statuses: ServiceStatus[] = []
  
  try {
    const mysqlRes = await mysqlApi.getInfo()
    statuses.push({
      name: 'MySQL',
      type: 'mysql',
      connected: mysqlRes.data.connected,
      version: mysqlRes.data.version || 'N/A',
      host: mysqlRes.data.host,
      port: mysqlRes.data.port,
    })
  } catch {
    statuses.push({
      name: 'MySQL',
      type: 'mysql',
      connected: false,
      version: 'N/A',
      host: '',
      port: 0,
    })
  }
  
  try {
    const redisRes = await redisApi.getInfo()
    statuses.push({
      name: 'Redis',
      type: 'redis',
      connected: redisRes.data.connected,
      version: redisRes.data.version || 'N/A',
      host: redisRes.data.host,
      port: redisRes.data.port,
      extra: {
        used_memory: redisRes.data.used_memory,
        total_keys: redisRes.data.total_keys,
      },
    })
  } catch {
    statuses.push({
      name: 'Redis',
      type: 'redis',
      connected: false,
      version: 'N/A',
      host: '',
      port: 0,
    })
  }
  
  services.value = statuses
  loading.value = false
}

function navigateTo(type: string) {
  router.push(`/${type}`)
}

onMounted(() => {
  fetchStatus()
})
</script>

<template>
  <div class="home-page">
    <!-- Header -->
    <div class="page-header">
      <h1 class="title-font neon-text">控制台</h1>
      <p class="subtitle">管理数据库服务</p>
    </div>
    
    <!-- Service Cards -->
    <NSpin :show="loading">
      <NGrid :cols="2" :x-gap="16" :y-gap="16" class="service-grid">
        <NGridItem v-for="service in services" :key="service.type">
          <NCard 
            class="service-card glass-card animate-fade-in"
            hoverable
            @click="navigateTo(service.type)"
          >
            <template #header>
              <NSpace align="center" :size="8">
                <NIcon size="22" :class="service.type === 'mysql' ? 'icon-mysql' : 'icon-redis'">
                  <ServerOutline v-if="service.type === 'mysql'" />
                  <KeyOutline v-else />
                </NIcon>
                <span class="card-title title-font">{{ service.name }}</span>
              </NSpace>
            </template>
            
            <template #header-extra>
              <NTag :type="service.connected ? 'success' : 'error'" size="small" round>
                <template #icon>
                  <NIcon size="12">
                    <CheckmarkCircleOutline v-if="service.connected" />
                    <CloseCircleOutline v-else />
                  </NIcon>
                </template>
                {{ service.connected ? '已连接' : '未连接' }}
              </NTag>
            </template>
            
            <NGrid :cols="2" :x-gap="12" :y-gap="8">
              <NGridItem>
                <NStatistic label="版本" :value="service.version" />
              </NGridItem>
              <NGridItem>
                <NStatistic 
                  label="地址" 
                  :value="`${service.host}:${service.port}`" 
                />
              </NGridItem>
              <NGridItem v-if="service.extra?.total_keys !== undefined">
                <NStatistic label="Key 总数" :value="service.extra.total_keys as number" />
              </NGridItem>
              <NGridItem v-if="service.extra?.used_memory">
                <NStatistic label="内存" :value="service.extra.used_memory as string" />
              </NGridItem>
            </NGrid>
          </NCard>
        </NGridItem>
      </NGrid>
    </NSpin>
    
    <!-- Quick Actions -->
    <div class="quick-actions">
      <h2 class="section-title">快捷操作</h2>
      <NGrid :cols="4" :x-gap="12" :y-gap="12">
        <NGridItem>
          <NCard 
            class="action-card glass-card" 
            hoverable
            @click="router.push('/mysql/query')"
          >
            <NSpace vertical align="center" :size="6">
              <NIcon size="24" class="neon-text">
                <ServerOutline />
              </NIcon>
              <span>SQL 查询</span>
            </NSpace>
          </NCard>
        </NGridItem>
        <NGridItem>
          <NCard 
            class="action-card glass-card" 
            hoverable
            @click="router.push('/mysql')"
          >
            <NSpace vertical align="center" :size="6">
              <NIcon size="24" class="icon-mysql">
                <ServerOutline />
              </NIcon>
              <span>浏览 MySQL</span>
            </NSpace>
          </NCard>
        </NGridItem>
        <NGridItem>
          <NCard 
            class="action-card glass-card" 
            hoverable
            @click="router.push('/redis')"
          >
            <NSpace vertical align="center" :size="6">
              <NIcon size="24" class="icon-redis">
                <KeyOutline />
              </NIcon>
              <span>浏览 Redis</span>
            </NSpace>
          </NCard>
        </NGridItem>
      </NGrid>
    </div>
  </div>
</template>

<style scoped>
.home-page {
  padding: 20px;
  min-height: 100%;
}

.page-header {
  margin-bottom: 20px;
}

.page-header h1 {
  font-size: 22px;
  margin-bottom: 4px;
}

.subtitle {
  color: var(--zx-text-secondary);
  font-size: 12px;
}

.service-grid {
  margin-bottom: 24px;
}

.service-card {
  cursor: pointer;
  transition: all 0.2s ease;
}

.service-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 24px rgba(0, 255, 255, 0.15);
}

.card-title {
  font-size: 15px;
  font-weight: 600;
}

.icon-mysql {
  color: #00758F;
}

.icon-redis {
  color: #DC382D;
}

.quick-actions {
  margin-top: 24px;
}

.section-title {
  font-size: 14px;
  margin-bottom: 12px;
  color: var(--zx-text-primary);
}

.action-card {
  cursor: pointer;
  text-align: center;
  padding: 10px;
  transition: all 0.2s ease;
}

.action-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 255, 255, 0.12);
}

.action-card span {
  font-size: 12px;
}
</style>
