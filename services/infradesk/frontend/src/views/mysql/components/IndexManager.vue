<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import {
  NSpace,
  NButton,
  NIcon,
  NDataTable,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NSwitch,
  NTag,
  useMessage,
  useDialog,
} from 'naive-ui'
import { AddOutline, TrashOutline, RefreshOutline } from '@vicons/ionicons5'
import { mysqlApi } from '@/api'
import type { IndexInfo, CreateIndexRequest } from '@/api/types'
import type { DataTableColumns } from 'naive-ui'

const props = defineProps<{
  database: string
  table: string
}>()

const message = useMessage()
const dialog = useDialog()

const indexes = ref<IndexInfo[]>([])
const loading = ref(false)

// Create index modal state
const showCreateModal = ref(false)
const newIndex = ref<CreateIndexRequest>({
  name: '',
  columns: [],
  unique: false,
  index_type: 'BTREE',
})

// Available columns for index (fetched from table schema)
const availableColumns = ref<string[]>([])

const indexTypeOptions = [
  { label: 'BTREE', value: 'BTREE' },
  { label: 'HASH', value: 'HASH' },
]

const columns: DataTableColumns<IndexInfo> = [
  {
    title: '索引名',
    key: 'name',
    width: 150,
    render(row) {
      if (row.is_primary) {
        return h(NTag, { type: 'warning', size: 'small' }, { default: () => row.name })
      }
      return row.name
    },
  },
  {
    title: '列',
    key: 'columns',
    render(row) {
      return row.columns.join(', ')
    },
  },
  {
    title: '唯一',
    key: 'unique',
    width: 80,
    render(row) {
      return row.unique ? h(NTag, { type: 'success', size: 'small' }, { default: () => '是' }) : '-'
    },
  },
  {
    title: '类型',
    key: 'index_type',
    width: 80,
  },
  {
    title: '操作',
    key: 'actions',
    width: 80,
    render(row) {
      if (row.is_primary) {
        return '-'
      }
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

async function fetchIndexes() {
  loading.value = true
  try {
    indexes.value = await mysqlApi.listIndexes(props.database, props.table)
  } catch (e) {
    message.error((e as Error).message)
  } finally {
    loading.value = false
  }
}

async function fetchColumns() {
  try {
    const schema = await mysqlApi.getTableSchema(props.database, props.table) as { columns: { name: string }[] }
    availableColumns.value = schema.columns.map(c => c.name)
  } catch (e) {
    console.error('Failed to fetch columns:', e)
  }
}

function handleCreate() {
  resetCreateForm()
  showCreateModal.value = true
}

async function submitCreate() {
  if (!newIndex.value.name.trim()) {
    message.warning('Please enter index name')
    return
  }

  if (newIndex.value.columns.length === 0) {
    message.warning('Please select at least one column')
    return
  }

  try {
    await mysqlApi.createIndex(props.database, props.table, newIndex.value)
    message.success(`Index "${newIndex.value.name}" created successfully`)
    showCreateModal.value = false
    fetchIndexes()
  } catch (e) {
    message.error((e as Error).message)
  }
}

function handleDrop(indexName: string) {
  dialog.warning({
    title: 'Drop Index',
    content: `Are you sure you want to drop index "${indexName}"? This action cannot be undone.`,
    positiveText: 'Drop',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      try {
        await mysqlApi.dropIndex(props.database, props.table, indexName)
        message.success(`Index "${indexName}" dropped`)
        fetchIndexes()
      } catch (e) {
        message.error((e as Error).message)
      }
    },
  })
}

function resetCreateForm() {
  newIndex.value = {
    name: '',
    columns: [],
    unique: false,
    index_type: 'BTREE',
  }
}

// Render function import for h()
import { h } from 'vue'

onMounted(() => {
  fetchIndexes()
  fetchColumns()
})

watch(() => [props.database, props.table], () => {
  fetchIndexes()
  fetchColumns()
})
</script>

<template>
  <div class="index-manager">
    <NSpace justify="space-between" align="center" class="toolbar">
      <span class="title-font" style="font-size: 13px">索引列表</span>
      <NSpace :size="6">
        <NButton size="tiny" @click="fetchIndexes">
          <template #icon>
            <NIcon size="14"><RefreshOutline /></NIcon>
          </template>
          刷新
        </NButton>
        <NButton size="tiny" type="primary" @click="handleCreate">
          <template #icon>
            <NIcon size="14"><AddOutline /></NIcon>
          </template>
          创建索引
        </NButton>
      </NSpace>
    </NSpace>

    <NDataTable
      :columns="columns"
      :data="indexes"
      :loading="loading"
      :bordered="false"
      size="small"
      striped
      style="margin-top: 12px"
    />

    <!-- Create Index Modal -->
    <NModal
      v-model:show="showCreateModal"
      title="创建索引"
      preset="card"
      style="width: 500px"
    >
      <NForm size="small" label-placement="left" label-width="80px">
        <NFormItem label="索引名">
          <NInput v-model:value="newIndex.name" placeholder="输入索引名" />
        </NFormItem>
        <NFormItem label="列">
          <NSelect
            v-model:value="newIndex.columns"
            :options="availableColumns.map(c => ({ label: c, value: c }))"
            multiple
            placeholder="选择列"
          />
        </NFormItem>
        <NFormItem label="唯一索引">
          <NSwitch v-model:value="newIndex.unique" />
        </NFormItem>
        <NFormItem label="索引类型">
          <NSelect v-model:value="newIndex.index_type" :options="indexTypeOptions" />
        </NFormItem>
      </NForm>

      <template #footer>
        <NSpace justify="end" :size="8">
          <NButton size="small" @click="showCreateModal = false">取消</NButton>
          <NButton size="small" type="primary" @click="submitCreate">创建</NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>

<style scoped>
.index-manager {
  padding: 12px;
}

.toolbar {
  padding-bottom: 8px;
  border-bottom: 1px solid var(--n-border-color);
}
</style>
