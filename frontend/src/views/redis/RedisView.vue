<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { NLayout, NLayoutContent, NSpace, NIcon, NTag, NStatistic, NGrid, NGridItem, NEmpty, NButton } from 'naive-ui'
import { KeyOutline, CheckmarkCircleOutline, CloseCircleOutline, AddOutline } from '@vicons/ionicons5'
import { useRouter } from 'vue-router'
import { useRedisStore } from '@/stores/redis'
import { useConnectionsStore } from '@/stores/connections'
import ConnectionSelector from '@/components/ConnectionSelector.vue'

const router = useRouter()
const store = useRedisStore()
const connStore = useConnectionsStore()

function handleConnectionChange() {
  // Refresh data when connection changes
  if (connStore.hasActiveConnection('redis')) {
    store.fetchServerInfo()
    store.fetchKeys()
  }
}

onMounted(() => {
  connStore.initFromStorage()
  connStore.fetchConnections().then(() => {
    if (connStore.hasActiveConnection('redis')) {
      store.fetchServerInfo()
      store.fetchKeys()
    }
  })
})

// Watch for connection changes
watch(() => connStore.getActiveConnectionId('redis'), (newId, oldId) => {
  if (newId !== oldId && newId) {
    store.fetchServerInfo()
    store.fetchKeys()
  }
})

function goToConnections() {
  router.push('/connections')
}
</script>

<template>
  <NLayout class="redis-layout">
    <!-- Header Stats -->
    <div class="redis-header">
      <NSpace align="center" justify="space-between">
        <NSpace align="center" :size="8">
          <NIcon size="18" color="#DC382D">
            <KeyOutline />
          </NIcon>
          <span class="title-font" style="font-size: 15px">Redis</span>
          <NTag 
            v-if="connStore.hasActiveConnection('redis')"
            :type="store.serverInfo?.connected ? 'success' : 'error'" 
            size="tiny"
            round
          >
            <template #icon>
              <NIcon size="10">
                <CheckmarkCircleOutline v-if="store.serverInfo?.connected" />
                <CloseCircleOutline v-else />
              </NIcon>
            </template>
            {{ store.serverInfo?.connected ? '在线' : '离线' }}
          </NTag>
          
          <!-- Connection Selector -->
          <ConnectionSelector type="redis" @change="handleConnectionChange" />
        </NSpace>
        
        <NGrid v-if="store.serverInfo && connStore.hasActiveConnection('redis')" :cols="4" :x-gap="16">
          <NGridItem>
            <NStatistic label="版本" :value="store.serverInfo.version || 'N/A'" />
          </NGridItem>
          <NGridItem>
            <NStatistic label="Key 总数" :value="store.serverInfo.total_keys || 0" />
          </NGridItem>
          <NGridItem>
            <NStatistic label="内存" :value="store.serverInfo.used_memory || 'N/A'" />
          </NGridItem>
          <NGridItem>
            <NStatistic label="客户端" :value="store.serverInfo.connected_clients || 0" />
          </NGridItem>
        </NGrid>
      </NSpace>
    </div>
    
    <!-- Main Content -->
    <NLayoutContent class="main-content">
      <template v-if="connStore.hasActiveConnection('redis')">
        <router-view />
      </template>
      <template v-else>
        <div class="no-connection-content">
          <NEmpty description="请选择或创建 Redis 连接" size="large">
            <template #extra>
              <NButton type="primary" @click="goToConnections">
                <template #icon>
                  <NIcon><AddOutline /></NIcon>
                </template>
                管理连接
              </NButton>
            </template>
          </NEmpty>
        </div>
      </template>
    </NLayoutContent>
  </NLayout>
</template>

<style scoped>
.redis-layout {
  height: 100%;
}

.redis-header {
  padding: 12px 16px;
  background: var(--zx-bg-secondary);
  border-bottom: 1px solid var(--zx-border);
}

.main-content {
  background: var(--zx-bg-primary);
  overflow: auto;
  height: calc(100% - 60px);
}

.no-connection-content {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}
</style>
