<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { NLayout, NLayoutSider, NLayoutContent, NSpace, NIcon, NTag, NEmpty, NButton } from 'naive-ui'
import { ServerOutline, CheckmarkCircleOutline, CloseCircleOutline, AddOutline } from '@vicons/ionicons5'
import { useRouter } from 'vue-router'
import { useMySQLStore } from '@/stores/mysql'
import { useConnectionsStore } from '@/stores/connections'
import DatabaseTree from './components/DatabaseTree.vue'
import ConnectionSelector from '@/components/ConnectionSelector.vue'

const router = useRouter()
const store = useMySQLStore()
const connStore = useConnectionsStore()

function handleConnectionChange() {
  // Refresh data when connection changes
  if (connStore.hasActiveConnection('mysql')) {
    store.fetchServerInfo()
    store.fetchDatabases()
  }
}

onMounted(() => {
  connStore.initFromStorage()
  connStore.fetchConnections().then(() => {
    if (connStore.hasActiveConnection('mysql')) {
      store.fetchServerInfo()
      store.fetchDatabases()
    }
  })
})

// Watch for connection changes
watch(() => connStore.getActiveConnectionId('mysql'), (newId, oldId) => {
  if (newId !== oldId && newId) {
    store.fetchServerInfo()
    store.fetchDatabases()
  }
})

function goToConnections() {
  router.push('/connections')
}
</script>

<template>
  <NLayout has-sider class="mysql-layout">
    <!-- Database Tree Sidebar -->
    <NLayoutSider
      bordered
      :width="260"
      class="tree-sider"
    >
      <div class="sider-header">
        <NSpace align="center" justify="space-between">
          <NSpace align="center" :size="6">
            <NIcon size="16" color="#00758F">
              <ServerOutline />
            </NIcon>
            <span class="title-font">MySQL</span>
          </NSpace>
          <NTag 
            v-if="connStore.hasActiveConnection('mysql')"
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
        </NSpace>
        
        <!-- Connection Selector -->
        <div class="connection-row">
          <ConnectionSelector type="mysql" @change="handleConnectionChange" />
        </div>
        
        <div v-if="store.serverInfo?.version && connStore.hasActiveConnection('mysql')" class="server-version">
          v{{ store.serverInfo.version }}
        </div>
      </div>
      
      <!-- Show tree only if connection is selected -->
      <template v-if="connStore.hasActiveConnection('mysql')">
        <DatabaseTree />
      </template>
      <template v-else>
        <NEmpty description="请先选择连接" size="small" class="no-connection">
          <template #extra>
            <NButton size="small" type="primary" @click="goToConnections">
              <template #icon>
                <NIcon><AddOutline /></NIcon>
              </template>
              添加连接
            </NButton>
          </template>
        </NEmpty>
      </template>
    </NLayoutSider>
    
    <!-- Main Content -->
    <NLayoutContent class="main-content">
      <template v-if="connStore.hasActiveConnection('mysql')">
        <router-view />
      </template>
      <template v-else>
        <div class="no-connection-content">
          <NEmpty description="请在左侧选择或创建 MySQL 连接" size="large">
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
.mysql-layout {
  height: 100%;
}

.tree-sider {
  background: var(--zx-bg-secondary) !important;
}

.sider-header {
  padding: 10px 12px;
  border-bottom: 1px solid var(--zx-border);
}

.sider-header .title-font {
  font-size: 13px;
}

.connection-row {
  margin-top: 8px;
}

.server-version {
  font-size: 10px;
  color: var(--zx-text-secondary);
  margin-top: 4px;
}

.main-content {
  background: var(--zx-bg-primary);
  overflow: auto;
}

.no-connection {
  padding: 40px 20px;
}

.no-connection-content {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}
</style>
