<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  NSelect,
  NSpace,
  NIcon,
  NButton,
  NTag,
  NTooltip,
} from 'naive-ui'
import {
  AddOutline,
  SettingsOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
} from '@vicons/ionicons5'
import { useConnectionsStore } from '@/stores/connections'

const props = defineProps<{
  type: 'mysql' | 'redis' | 'mongodb' | 'minio'
}>()

const emit = defineEmits<{
  (e: 'change', id: number | null): void
}>()

const router = useRouter()
const store = useConnectionsStore()

const connections = computed(() => store.getConnectionsByType(props.type))

const activeId = computed({
  get: () => store.getActiveConnectionId(props.type),
  set: (val) => {
    store.setActiveConnection(props.type, val)
    store.saveToStorage()
    emit('change', val)
  }
})

const activeConnection = computed(() => store.getActiveConnection(props.type))

const options = computed(() => {
  return connections.value.map(c => ({
    label: c.name,
    value: c.id,
    host: `${c.host}:${c.port}`,
    isDefault: c.is_default,
  }))
})

function renderLabel(option: { label: string; value: number; host: string; isDefault?: boolean }) {
  return option.label
}

function goToConnections() {
  router.push('/connections')
}
</script>

<template>
  <div class="connection-selector">
    <NSpace align="center" :size="8" :wrap="false">
      <!-- Connection Dropdown -->
      <NSelect
        v-model:value="activeId"
        :options="options"
        :render-label="renderLabel"
        placeholder="选择连接"
        size="small"
        style="width: 160px"
        :consistent-menu-width="false"
      >
        <template #empty>
          <div class="empty-connections">
            <p>暂无连接配置</p>
            <NButton size="tiny" type="primary" @click="goToConnections">
              <template #icon>
                <NIcon><AddOutline /></NIcon>
              </template>
              添加连接
            </NButton>
          </div>
        </template>
      </NSelect>

      <!-- Connection Status -->
      <NTooltip v-if="activeConnection">
        <template #trigger>
          <NTag :type="activeConnection ? 'success' : 'default'" size="tiny" round>
            <template #icon>
              <NIcon :size="10">
                <CheckmarkCircleOutline v-if="activeConnection" />
                <CloseCircleOutline v-else />
              </NIcon>
            </template>
            {{ activeConnection.host }}:{{ activeConnection.port }}
          </NTag>
        </template>
        <div class="connection-tooltip">
          <div><strong>{{ activeConnection.name }}</strong></div>
          <div>{{ activeConnection.host }}:{{ activeConnection.port }}</div>
          <div v-if="activeConnection.username">用户: {{ activeConnection.username }}</div>
        </div>
      </NTooltip>

      <!-- Settings Button -->
      <NTooltip>
        <template #trigger>
          <NButton size="tiny" quaternary circle @click="goToConnections">
            <template #icon>
              <NIcon><SettingsOutline /></NIcon>
            </template>
          </NButton>
        </template>
        管理连接
      </NTooltip>
    </NSpace>
  </div>
</template>

<style scoped>
.connection-selector {
  display: inline-flex;
  align-items: center;
}

.empty-connections {
  padding: 12px;
  text-align: center;
}

.empty-connections p {
  margin-bottom: 8px;
  color: var(--zx-text-secondary);
  font-size: 12px;
}

.connection-tooltip {
  font-size: 11px;
  line-height: 1.4;
}

.connection-tooltip div {
  margin-bottom: 2px;
}
</style>

