<script setup lang="ts">
import { 
  NCard, 
  NSpace, 
  NButton, 
  NIcon, 
  NDataTable,
  NInput,
  useMessage,
  useDialog,
} from 'naive-ui'
import { AddOutline, TrashOutline, RefreshOutline } from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import { useRouter } from 'vue-router'
import { ref, h } from 'vue'
import type { DataTableColumns } from 'naive-ui'

const store = useMySQLStore()
const router = useRouter()
const message = useMessage()
const dialog = useDialog()

const newDbName = ref('')

const columns: DataTableColumns<{ name: string }> = [
  {
    title: '数据库名称',
    key: 'name',
    render(row) {
      return h(
        'a',
        {
          style: 'color: var(--zx-accent-cyan); cursor: pointer; font-size: 12px;',
          onClick: () => router.push(`/mysql/${row.name}`),
        },
        row.name
      )
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 80,
    render(row) {
      return h(
        NButton,
        {
          size: 'tiny',
          type: 'error',
          quaternary: true,
          onClick: () => handleDrop(row.name),
        },
        { icon: () => h(NIcon, { size: 14 }, { default: () => h(TrashOutline) }) }
      )
    },
  },
]

async function handleCreate() {
  if (!newDbName.value.trim()) {
    message.warning('请输入数据库名称')
    return
  }
  
  try {
    await store.createDatabase({ name: newDbName.value.trim() })
    message.success(`数据库 "${newDbName.value}" 创建成功`)
    newDbName.value = ''
  } catch (e) {
    message.error((e as Error).message)
  }
}

function handleDrop(name: string) {
  dialog.warning({
    title: '删除数据库',
    content: `确定要删除数据库 "${name}" 吗？此操作不可撤销。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await store.dropDatabase(name)
        message.success(`数据库 "${name}" 已删除`)
      } catch (e) {
        message.error((e as Error).message)
      }
    },
  })
}

function handleRefresh() {
  store.fetchDatabases()
  message.success('已刷新')
}
</script>

<template>
  <div class="database-list">
    <NCard class="glass-card">
      <template #header>
        <NSpace align="center" justify="space-between">
          <span class="title-font" style="font-size: 14px">数据库列表</span>
          <NButton size="tiny" @click="handleRefresh">
            <template #icon>
              <NIcon size="14"><RefreshOutline /></NIcon>
            </template>
            刷新
          </NButton>
        </NSpace>
      </template>
      
      <!-- Create Database -->
      <NSpace class="create-section" align="center" :size="8">
        <NInput 
          v-model:value="newDbName" 
          placeholder="新数据库名称"
          size="small"
          @keyup.enter="handleCreate"
        />
        <NButton type="primary" size="small" @click="handleCreate">
          <template #icon>
            <NIcon size="14"><AddOutline /></NIcon>
          </template>
          创建
        </NButton>
      </NSpace>
      
      <!-- Database Table -->
      <NDataTable
        :columns="columns"
        :data="store.databases"
        :loading="store.loading"
        :bordered="false"
        size="small"
        striped
      />
    </NCard>
  </div>
</template>

<style scoped>
.database-list {
  padding: 16px;
}

.create-section {
  margin-bottom: 12px;
}

.create-section :deep(.n-input) {
  width: 240px;
}
</style>
