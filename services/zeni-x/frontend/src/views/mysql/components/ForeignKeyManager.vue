<script setup lang="ts">
import { ref, onMounted, watch, h } from 'vue'
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
  NTag,
  useMessage,
  useDialog,
} from 'naive-ui'
import { AddOutline, TrashOutline, RefreshOutline } from '@vicons/ionicons5'
import { mysqlApi } from '@/api'
import type { ForeignKeyInfo, CreateForeignKeyRequest } from '@/api/types'
import type { DataTableColumns } from 'naive-ui'

const props = defineProps<{
  database: string
  table: string
}>()

const message = useMessage()
const dialog = useDialog()

const foreignKeys = ref<ForeignKeyInfo[]>([])
const loading = ref(false)

// Create FK modal state
const showCreateModal = ref(false)
const newFK = ref<CreateForeignKeyRequest>({
  name: '',
  columns: [],
  ref_table: '',
  ref_columns: [],
  on_delete: 'RESTRICT',
  on_update: 'RESTRICT',
})

// Available columns and tables
const availableColumns = ref<string[]>([])
const availableTables = ref<string[]>([])
const refTableColumns = ref<string[]>([])

const actionOptions = [
  { label: 'RESTRICT', value: 'RESTRICT' },
  { label: 'CASCADE', value: 'CASCADE' },
  { label: 'SET NULL', value: 'SET NULL' },
  { label: 'NO ACTION', value: 'NO ACTION' },
]

const columns: DataTableColumns<ForeignKeyInfo> = [
  {
    title: '外键名',
    key: 'name',
    width: 150,
  },
  {
    title: '列',
    key: 'columns',
    width: 120,
    render(row) {
      return row.columns.join(', ')
    },
  },
  {
    title: '引用表',
    key: 'ref_table',
    width: 120,
  },
  {
    title: '引用列',
    key: 'ref_columns',
    width: 120,
    render(row) {
      return row.ref_columns.join(', ')
    },
  },
  {
    title: 'ON DELETE',
    key: 'on_delete',
    width: 100,
    render(row) {
      const type = row.on_delete === 'CASCADE' ? 'warning' : 'default'
      return h(NTag, { type, size: 'small' }, { default: () => row.on_delete })
    },
  },
  {
    title: 'ON UPDATE',
    key: 'on_update',
    width: 100,
    render(row) {
      const type = row.on_update === 'CASCADE' ? 'warning' : 'default'
      return h(NTag, { type, size: 'small' }, { default: () => row.on_update })
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

async function fetchForeignKeys() {
  loading.value = true
  try {
    foreignKeys.value = await mysqlApi.listForeignKeys(props.database, props.table)
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

async function fetchTables() {
  try {
    const tables = await mysqlApi.listTables(props.database) as { name: string }[]
    availableTables.value = tables.map(t => t.name).filter(t => t !== props.table)
  } catch (e) {
    console.error('Failed to fetch tables:', e)
  }
}

async function fetchRefTableColumns(tableName: string) {
  if (!tableName) {
    refTableColumns.value = []
    return
  }
  try {
    const schema = await mysqlApi.getTableSchema(props.database, tableName) as { columns: { name: string }[] }
    refTableColumns.value = schema.columns.map(c => c.name)
  } catch (e) {
    console.error('Failed to fetch ref table columns:', e)
  }
}

function handleCreate() {
  resetCreateForm()
  showCreateModal.value = true
}

async function submitCreate() {
  if (newFK.value.columns.length === 0) {
    message.warning('Please select source columns')
    return
  }

  if (!newFK.value.ref_table) {
    message.warning('Please select reference table')
    return
  }

  if (newFK.value.ref_columns.length === 0) {
    message.warning('Please select reference columns')
    return
  }

  if (newFK.value.columns.length !== newFK.value.ref_columns.length) {
    message.warning('Source and reference columns must have the same count')
    return
  }

  try {
    await mysqlApi.createForeignKey(props.database, props.table, newFK.value)
    const fkName = newFK.value.name || `fk_${props.table}_${newFK.value.columns.join('_')}`
    message.success(`Foreign key "${fkName}" created successfully`)
    showCreateModal.value = false
    fetchForeignKeys()
  } catch (e) {
    message.error((e as Error).message)
  }
}

function handleDrop(fkName: string) {
  dialog.warning({
    title: 'Drop Foreign Key',
    content: `Are you sure you want to drop foreign key "${fkName}"? This action cannot be undone.`,
    positiveText: 'Drop',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      try {
        await mysqlApi.dropForeignKey(props.database, props.table, fkName)
        message.success(`Foreign key "${fkName}" dropped`)
        fetchForeignKeys()
      } catch (e) {
        message.error((e as Error).message)
      }
    },
  })
}

function resetCreateForm() {
  newFK.value = {
    name: '',
    columns: [],
    ref_table: '',
    ref_columns: [],
    on_delete: 'RESTRICT',
    on_update: 'RESTRICT',
  }
  refTableColumns.value = []
}

// Watch for ref_table changes to fetch its columns
watch(() => newFK.value.ref_table, (newTable) => {
  newFK.value.ref_columns = []
  fetchRefTableColumns(newTable)
})

onMounted(() => {
  fetchForeignKeys()
  fetchColumns()
  fetchTables()
})

watch(() => [props.database, props.table], () => {
  fetchForeignKeys()
  fetchColumns()
  fetchTables()
})
</script>

<template>
  <div class="fk-manager">
    <NSpace justify="space-between" align="center" class="toolbar">
      <span class="title-font" style="font-size: 13px">外键列表</span>
      <NSpace :size="6">
        <NButton size="tiny" @click="fetchForeignKeys">
          <template #icon>
            <NIcon size="14"><RefreshOutline /></NIcon>
          </template>
          刷新
        </NButton>
        <NButton size="tiny" type="primary" @click="handleCreate">
          <template #icon>
            <NIcon size="14"><AddOutline /></NIcon>
          </template>
          创建外键
        </NButton>
      </NSpace>
    </NSpace>

    <NDataTable
      :columns="columns"
      :data="foreignKeys"
      :loading="loading"
      :bordered="false"
      size="small"
      striped
      style="margin-top: 12px"
    />

    <!-- Create Foreign Key Modal -->
    <NModal
      v-model:show="showCreateModal"
      title="创建外键"
      preset="card"
      style="width: 550px"
    >
      <NForm size="small" label-placement="left" label-width="90px">
        <NFormItem label="外键名">
          <NInput v-model:value="newFK.name" placeholder="可选，留空自动生成" />
        </NFormItem>
        <NFormItem label="源列">
          <NSelect
            v-model:value="newFK.columns"
            :options="availableColumns.map(c => ({ label: c, value: c }))"
            multiple
            placeholder="选择源列"
          />
        </NFormItem>
        <NFormItem label="引用表">
          <NSelect
            v-model:value="newFK.ref_table"
            :options="availableTables.map(t => ({ label: t, value: t }))"
            placeholder="选择引用表"
            filterable
          />
        </NFormItem>
        <NFormItem label="引用列">
          <NSelect
            v-model:value="newFK.ref_columns"
            :options="refTableColumns.map(c => ({ label: c, value: c }))"
            multiple
            placeholder="选择引用列"
            :disabled="!newFK.ref_table"
          />
        </NFormItem>
        <NFormItem label="ON DELETE">
          <NSelect v-model:value="newFK.on_delete" :options="actionOptions" />
        </NFormItem>
        <NFormItem label="ON UPDATE">
          <NSelect v-model:value="newFK.on_update" :options="actionOptions" />
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
.fk-manager {
  padding: 12px;
}

.toolbar {
  padding-bottom: 8px;
  border-bottom: 1px solid var(--n-border-color);
}
</style>
