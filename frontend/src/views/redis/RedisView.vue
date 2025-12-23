<script setup lang="ts">
import { onMounted } from 'vue'
import { NLayout, NLayoutContent, NSpace, NIcon, NTag, NStatistic, NGrid, NGridItem } from 'naive-ui'
import { KeyOutline, CheckmarkCircleOutline, CloseCircleOutline } from '@vicons/ionicons5'
import { useRedisStore } from '@/stores/redis'

const store = useRedisStore()

onMounted(() => {
  store.fetchServerInfo()
  store.fetchKeys()
})
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
        
        <NGrid :cols="4" :x-gap="16" v-if="store.serverInfo">
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
      <router-view />
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
</style>
