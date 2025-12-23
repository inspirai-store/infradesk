<script setup lang="ts">
import { computed, h } from 'vue'
import { useRouter } from 'vue-router'
import { 
  NTree, 
  NInput, 
  NSpace, 
  NButton, 
  NIcon, 
  NSpin,
  NEmpty,
  useMessage,
  useDialog,
} from 'naive-ui'
import { 
  AddOutline, 
  RefreshOutline,
  ServerOutline,
  GridOutline,
} from '@vicons/ionicons5'
import type { TreeOption } from 'naive-ui'
import { useMySQLStore } from '@/stores/mysql'
import { ref, watch } from 'vue'

const router = useRouter()
const store = useMySQLStore()
const message = useMessage()
const dialog = useDialog()

const searchValue = ref('')
const expandedKeys = ref<string[]>([])

const treeData = computed<TreeOption[]>(() => {
  const filtered = store.databases.filter(db => 
    db.name.toLowerCase().includes(searchValue.value.toLowerCase())
  )
  
  return filtered.map(db => ({
    key: db.name,
    label: db.name,
    prefix: () => h(NIcon, { color: '#00758F', size: 14 }, { default: () => h(ServerOutline) }),
    children: store.currentDatabase === db.name ? 
      store.tables.map(table => ({
        key: `${db.name}/${table.name}`,
        label: table.name,
        prefix: () => h(NIcon, { color: '#8B5CF6', size: 14 }, { default: () => h(GridOutline) }),
        isLeaf: true,
      })) : [],
  }))
})

function handleSelect(keys: string[]) {
  if (keys.length === 0) return
  
  const key = keys[0]
  if (key.includes('/')) {
    const [database, table] = key.split('/')
    router.push(`/mysql/${database}/${table}`)
  } else {
    router.push(`/mysql/${key}`)
  }
}

async function handleExpand(keys: string[]) {
  expandedKeys.value = keys
  
  for (const key of keys) {
    if (!key.includes('/') && key !== store.currentDatabase) {
      await store.fetchTables(key)
    }
  }
}

function handleRefresh() {
  store.fetchDatabases()
  message.success('已刷新')
}

function handleCreateDatabase() {
  dialog.create({
    title: '创建数据库',
    content: () => h('div', { style: 'padding: 12px 0' }, [
      h(NInput, {
        placeholder: '输入数据库名称',
        size: 'small',
        onUpdateValue: (val: string) => {
          (dialog as unknown as { inputValue: string }).inputValue = val
        },
      }),
    ]),
    positiveText: '创建',
    negativeText: '取消',
    onPositiveClick: async () => {
      const name = (dialog as unknown as { inputValue: string }).inputValue
      if (!name) {
        message.error('请输入数据库名称')
        return false
      }
      try {
        await store.createDatabase(name)
        message.success(`数据库 "${name}" 创建成功`)
      } catch (e) {
        message.error((e as Error).message)
        return false
      }
    },
  })
}

watch(() => store.currentDatabase, (newDb) => {
  if (newDb && !expandedKeys.value.includes(newDb)) {
    expandedKeys.value = [...expandedKeys.value, newDb]
  }
})
</script>

<template>
  <div class="database-tree">
    <!-- Search & Actions -->
    <div class="tree-actions">
      <NInput 
        v-model:value="searchValue" 
        placeholder="搜索数据库..." 
        clearable
        size="tiny"
      />
      <NSpace :size="2">
        <NButton size="tiny" quaternary @click="handleRefresh">
          <template #icon>
            <NIcon size="14"><RefreshOutline /></NIcon>
          </template>
        </NButton>
        <NButton size="tiny" quaternary @click="handleCreateDatabase">
          <template #icon>
            <NIcon size="14"><AddOutline /></NIcon>
          </template>
        </NButton>
      </NSpace>
    </div>
    
    <!-- Tree -->
    <NSpin :show="store.loading">
      <NTree
        v-if="treeData.length > 0"
        :data="treeData"
        :expanded-keys="expandedKeys"
        selectable
        block-line
        @update:selected-keys="handleSelect"
        @update:expanded-keys="handleExpand"
      />
      <NEmpty v-else description="暂无数据库" size="small" />
    </NSpin>
  </div>
</template>

<style scoped>
.database-tree {
  height: calc(100% - 56px);
  overflow: auto;
  padding: 8px;
}

.tree-actions {
  display: flex;
  gap: 6px;
  margin-bottom: 8px;
}

.tree-actions :deep(.n-input) {
  flex: 1;
}
</style>
