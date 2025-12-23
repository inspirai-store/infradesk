<script setup lang="ts">
import { onMounted } from 'vue'
import { NLayout, NLayoutSider, NLayoutContent, NSpace, NIcon, NTag } from 'naive-ui'
import { ServerOutline, CheckmarkCircleOutline, CloseCircleOutline } from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import DatabaseTree from './components/DatabaseTree.vue'

const store = useMySQLStore()

onMounted(() => {
  store.fetchServerInfo()
  store.fetchDatabases()
})
</script>

<template>
  <NLayout has-sider class="mysql-layout">
    <!-- Database Tree Sidebar -->
    <NLayoutSider
      bordered
      :width="240"
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
        <div v-if="store.serverInfo?.version" class="server-version">
          v{{ store.serverInfo.version }}
        </div>
      </div>
      
      <DatabaseTree />
    </NLayoutSider>
    
    <!-- Main Content -->
    <NLayoutContent class="main-content">
      <router-view />
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

.server-version {
  font-size: 10px;
  color: var(--zx-text-secondary);
  margin-top: 2px;
}

.main-content {
  background: var(--zx-bg-primary);
  overflow: auto;
}
</style>
