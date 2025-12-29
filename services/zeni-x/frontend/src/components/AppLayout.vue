<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { 
  NLayout, 
  NLayoutSider, 
  NLayoutContent, 
  NMenu, 
  NIcon,
  NSpace,
  NDivider,
} from 'naive-ui'
import { 
  ServerOutline, 
  KeyOutline, 
  HomeOutline,
  TerminalOutline,
  LinkOutline,
} from '@vicons/ionicons5'
import type { MenuOption } from 'naive-ui'
import { h } from 'vue'

const route = useRoute()
const router = useRouter()

const activeKey = computed(() => {
  const path = route.path
  if (path.startsWith('/connections')) return 'connections'
  if (path.startsWith('/mysql')) return 'mysql'
  if (path.startsWith('/redis')) return 'redis'
  return 'home'
})

const menuOptions: MenuOption[] = [
  {
    label: '控制台',
    key: 'home',
    icon: () => h(NIcon, null, { default: () => h(HomeOutline) }),
  },
  {
    label: '连接管理',
    key: 'connections',
    icon: () => h(NIcon, null, { default: () => h(LinkOutline) }),
  },
  {
    type: 'divider',
    key: 'd1',
  },
  {
    label: 'MySQL 管理',
    key: 'mysql',
    icon: () => h(NIcon, null, { default: () => h(ServerOutline) }),
  },
  {
    label: 'Redis 管理',
    key: 'redis',
    icon: () => h(NIcon, null, { default: () => h(KeyOutline) }),
  },
  {
    type: 'divider',
    key: 'd2',
  },
  {
    label: 'SQL 查询',
    key: 'query',
    icon: () => h(NIcon, null, { default: () => h(TerminalOutline) }),
  },
]

function handleMenuSelect(key: string) {
  switch (key) {
    case 'home':
      router.push('/')
      break
    case 'connections':
      router.push('/connections')
      break
    case 'mysql':
      router.push('/mysql')
      break
    case 'redis':
      router.push('/redis')
      break
    case 'query':
      router.push('/mysql/query')
      break
  }
}
</script>

<template>
  <NLayout class="layout grid-bg" has-sider>
    <!-- Sidebar -->
    <NLayoutSider
      bordered
      collapse-mode="width"
      :collapsed-width="56"
      :width="180"
      show-trigger
      class="sidebar"
    >
      <!-- Logo -->
      <div class="logo">
        <NSpace align="center" justify="center">
          <span class="logo-text title-font neon-text">ZENI-X</span>
        </NSpace>
      </div>
      
      <NDivider style="margin: 0" />
      
      <!-- Navigation -->
      <NMenu
        :value="activeKey"
        :options="menuOptions"
        :collapsed-width="56"
        :collapsed-icon-size="18"
        @update:value="handleMenuSelect"
      />
      
      <!-- Footer -->
      <div class="sidebar-footer">
        <span class="version">v0.1.0</span>
      </div>
    </NLayoutSider>
    
    <!-- Main Content -->
    <NLayoutContent class="main-content">
      <router-view />
    </NLayoutContent>
  </NLayout>
</template>

<style scoped>
.layout {
  height: 100vh;
}

.sidebar {
  background: var(--zx-bg-secondary) !important;
}

.logo {
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.logo-text {
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 2px;
}

.main-content {
  background: var(--zx-bg-primary);
  overflow: auto;
}

.sidebar-footer {
  position: absolute;
  bottom: 10px;
  left: 0;
  right: 0;
  text-align: center;
}

.version {
  color: var(--zx-text-secondary);
  font-size: 10px;
}
</style>
