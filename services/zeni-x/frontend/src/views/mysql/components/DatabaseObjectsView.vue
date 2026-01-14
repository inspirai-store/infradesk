<template>
  <div class="database-objects-view">
    <NTabs v-model:value="activeTab" type="line">
      <!-- Views Tab -->
      <NTabPane name="views" tab="Views">
        <div class="tab-content">
          <div class="toolbar">
            <NButton size="small" @click="loadViews">Refresh</NButton>
          </div>
          <NDataTable
            :columns="viewColumns"
            :data="views"
            :loading="loadingViews"
            :row-key="(row: ViewInfo) => row.name"
            size="small"
            striped
          />
        </div>
      </NTabPane>

      <!-- Procedures Tab -->
      <NTabPane name="procedures" tab="Procedures">
        <div class="tab-content">
          <div class="toolbar">
            <NButton size="small" @click="loadProcedures">Refresh</NButton>
          </div>
          <NDataTable
            :columns="procedureColumns"
            :data="procedures"
            :loading="loadingProcedures"
            :row-key="(row: ProcedureInfo) => row.name"
            size="small"
            striped
          />
        </div>
      </NTabPane>

      <!-- Triggers Tab -->
      <NTabPane name="triggers" tab="Triggers">
        <div class="tab-content">
          <div class="toolbar">
            <NButton size="small" @click="loadTriggers">Refresh</NButton>
          </div>
          <NDataTable
            :columns="triggerColumns"
            :data="triggers"
            :loading="loadingTriggers"
            :row-key="(row: TriggerInfo) => row.name"
            size="small"
            striped
          />
        </div>
      </NTabPane>

      <!-- Processes Tab -->
      <NTabPane name="processes" tab="Processes">
        <div class="tab-content">
          <div class="toolbar">
            <NButton size="small" @click="loadProcesses">Refresh</NButton>
          </div>
          <NDataTable
            :columns="processColumns"
            :data="processes"
            :loading="loadingProcesses"
            :row-key="(row: ProcessInfo) => row.id"
            size="small"
            striped
          />
        </div>
      </NTabPane>

      <!-- Variables Tab -->
      <NTabPane name="variables" tab="Variables">
        <div class="tab-content">
          <div class="toolbar">
            <NInput
              v-model:value="variableFilter"
              placeholder="Filter variables..."
              size="small"
              style="width: 200px"
              clearable
              @update:value="debouncedLoadVariables"
            />
            <NButton size="small" @click="loadVariables">Refresh</NButton>
          </div>
          <NDataTable
            :columns="variableColumns"
            :data="variables"
            :loading="loadingVariables"
            :row-key="(row: ServerVariable) => row.name"
            size="small"
            striped
            :max-height="400"
          />
        </div>
      </NTabPane>
    </NTabs>

    <!-- View Definition Modal -->
    <NModal v-model:show="showViewDefinition" preset="card" title="View Definition" style="width: 600px">
      <NCode :code="currentDefinition" language="sql" />
    </NModal>

    <!-- Procedure Definition Modal -->
    <NModal v-model:show="showProcedureDefinition" preset="card" title="Procedure Definition" style="width: 600px">
      <NCode :code="currentDefinition" language="sql" />
    </NModal>

    <!-- Trigger Definition Modal -->
    <NModal v-model:show="showTriggerDefinition" preset="card" title="Trigger Definition" style="width: 600px">
      <NCode :code="currentDefinition" language="sql" />
    </NModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h, watch } from 'vue'
import {
  NTabs,
  NTabPane,
  NDataTable,
  NButton,
  NModal,
  NCode,
  NInput,
  NSpace,
  NPopconfirm,
  useMessage,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { mysqlApi } from '@/api'
import type {
  ViewInfo,
  ProcedureInfo,
  TriggerInfo,
  ProcessInfo,
  ServerVariable,
} from '@/api'
import { useDebounceFn } from '@vueuse/core'

const props = defineProps<{
  database: string
}>()

const message = useMessage()

// Tab state
const activeTab = ref('views')

// Data states
const views = ref<ViewInfo[]>([])
const procedures = ref<ProcedureInfo[]>([])
const triggers = ref<TriggerInfo[]>([])
const processes = ref<ProcessInfo[]>([])
const variables = ref<ServerVariable[]>([])

// Loading states
const loadingViews = ref(false)
const loadingProcedures = ref(false)
const loadingTriggers = ref(false)
const loadingProcesses = ref(false)
const loadingVariables = ref(false)

// Filter
const variableFilter = ref('')

// Definition modals
const showViewDefinition = ref(false)
const showProcedureDefinition = ref(false)
const showTriggerDefinition = ref(false)
const currentDefinition = ref('')

// View columns
const viewColumns = computed<DataTableColumns<ViewInfo>>(() => [
  { title: 'Name', key: 'name', ellipsis: { tooltip: true } },
  { title: 'Definer', key: 'definer', ellipsis: { tooltip: true } },
  { title: 'Security', key: 'security_type' },
  { title: 'Check Option', key: 'check_option' },
  {
    title: 'Updatable',
    key: 'is_updatable',
    render: (row) => row.is_updatable ? 'Yes' : 'No',
  },
  {
    title: 'Actions',
    key: 'actions',
    width: 150,
    render: (row) =>
      h(NSpace, { size: 'small' }, () => [
        h(
          NButton,
          { size: 'tiny', onClick: () => showViewDef(row.name) },
          () => 'View'
        ),
        h(
          NPopconfirm,
          { onPositiveClick: () => dropView(row.name) },
          {
            trigger: () => h(NButton, { size: 'tiny', type: 'error' }, () => 'Drop'),
            default: () => `Drop view ${row.name}?`,
          }
        ),
      ]),
  },
])

// Procedure columns
const procedureColumns = computed<DataTableColumns<ProcedureInfo>>(() => [
  { title: 'Name', key: 'name', ellipsis: { tooltip: true } },
  { title: 'Type', key: 'routine_type' },
  { title: 'Definer', key: 'definer', ellipsis: { tooltip: true } },
  { title: 'Created', key: 'created', ellipsis: { tooltip: true } },
  { title: 'Security', key: 'security_type' },
  {
    title: 'Actions',
    key: 'actions',
    width: 150,
    render: (row) =>
      h(NSpace, { size: 'small' }, () => [
        h(
          NButton,
          { size: 'tiny', onClick: () => showProcedureDef(row.name, row.routine_type) },
          () => 'View'
        ),
        h(
          NPopconfirm,
          { onPositiveClick: () => dropRoutine(row.name, row.routine_type) },
          {
            trigger: () => h(NButton, { size: 'tiny', type: 'error' }, () => 'Drop'),
            default: () => `Drop ${row.routine_type.toLowerCase()} ${row.name}?`,
          }
        ),
      ]),
  },
])

// Trigger columns
const triggerColumns = computed<DataTableColumns<TriggerInfo>>(() => [
  { title: 'Name', key: 'name', ellipsis: { tooltip: true } },
  { title: 'Event', key: 'event' },
  { title: 'Timing', key: 'timing' },
  { title: 'Table', key: 'table_name' },
  { title: 'Definer', key: 'definer', ellipsis: { tooltip: true } },
  {
    title: 'Actions',
    key: 'actions',
    width: 150,
    render: (row) =>
      h(NSpace, { size: 'small' }, () => [
        h(
          NButton,
          { size: 'tiny', onClick: () => showTriggerDef(row.name) },
          () => 'View'
        ),
        h(
          NPopconfirm,
          { onPositiveClick: () => dropTrigger(row.name) },
          {
            trigger: () => h(NButton, { size: 'tiny', type: 'error' }, () => 'Drop'),
            default: () => `Drop trigger ${row.name}?`,
          }
        ),
      ]),
  },
])

// Process columns
const processColumns = computed<DataTableColumns<ProcessInfo>>(() => [
  { title: 'ID', key: 'id', width: 80 },
  { title: 'User', key: 'user' },
  { title: 'Host', key: 'host', ellipsis: { tooltip: true } },
  { title: 'DB', key: 'db' },
  { title: 'Command', key: 'command' },
  { title: 'Time', key: 'time', width: 80 },
  { title: 'State', key: 'state', ellipsis: { tooltip: true } },
  { title: 'Info', key: 'info', ellipsis: { tooltip: true } },
  {
    title: 'Actions',
    key: 'actions',
    width: 80,
    render: (row) =>
      h(
        NPopconfirm,
        { onPositiveClick: () => killProcess(row.id) },
        {
          trigger: () => h(NButton, { size: 'tiny', type: 'error' }, () => 'Kill'),
          default: () => `Kill process ${row.id}?`,
        }
      ),
  },
])

// Variable columns
const variableColumns = computed<DataTableColumns<ServerVariable>>(() => [
  { title: 'Variable', key: 'name', ellipsis: { tooltip: true } },
  { title: 'Value', key: 'value', ellipsis: { tooltip: true } },
])

// Load functions
async function loadViews() {
  if (!props.database) return
  loadingViews.value = true
  try {
    views.value = await mysqlApi.listViews(props.database)
  } catch (err: any) {
    message.error(err.message || 'Failed to load views')
  } finally {
    loadingViews.value = false
  }
}

async function loadProcedures() {
  if (!props.database) return
  loadingProcedures.value = true
  try {
    procedures.value = await mysqlApi.listProcedures(props.database)
  } catch (err: any) {
    message.error(err.message || 'Failed to load procedures')
  } finally {
    loadingProcedures.value = false
  }
}

async function loadTriggers() {
  if (!props.database) return
  loadingTriggers.value = true
  try {
    triggers.value = await mysqlApi.listTriggers(props.database)
  } catch (err: any) {
    message.error(err.message || 'Failed to load triggers')
  } finally {
    loadingTriggers.value = false
  }
}

async function loadProcesses() {
  loadingProcesses.value = true
  try {
    processes.value = await mysqlApi.getProcessList()
  } catch (err: any) {
    message.error(err.message || 'Failed to load processes')
  } finally {
    loadingProcesses.value = false
  }
}

async function loadVariables() {
  loadingVariables.value = true
  try {
    variables.value = await mysqlApi.getServerVariables(variableFilter.value || undefined)
  } catch (err: any) {
    message.error(err.message || 'Failed to load variables')
  } finally {
    loadingVariables.value = false
  }
}

const debouncedLoadVariables = useDebounceFn(loadVariables, 300)

// Show definition functions
async function showViewDef(name: string) {
  try {
    const def = await mysqlApi.getViewDefinition(props.database, name)
    currentDefinition.value = def.definition
    showViewDefinition.value = true
  } catch (err: any) {
    message.error(err.message || 'Failed to load view definition')
  }
}

async function showProcedureDef(name: string, routineType: string) {
  try {
    const def = await mysqlApi.getProcedureDefinition(props.database, name, routineType)
    currentDefinition.value = def.definition
    showProcedureDefinition.value = true
  } catch (err: any) {
    message.error(err.message || 'Failed to load procedure definition')
  }
}

async function showTriggerDef(name: string) {
  try {
    const def = await mysqlApi.getTriggerDefinition(props.database, name)
    currentDefinition.value = def.definition
    showTriggerDefinition.value = true
  } catch (err: any) {
    message.error(err.message || 'Failed to load trigger definition')
  }
}

// Drop functions
async function dropView(name: string) {
  try {
    await mysqlApi.dropView(props.database, name)
    message.success(`View ${name} dropped`)
    loadViews()
  } catch (err: any) {
    message.error(err.message || 'Failed to drop view')
  }
}

async function dropRoutine(name: string, routineType: string) {
  try {
    if (routineType.toUpperCase() === 'FUNCTION') {
      await mysqlApi.dropFunction(props.database, name)
    } else {
      await mysqlApi.dropProcedure(props.database, name)
    }
    message.success(`${routineType} ${name} dropped`)
    loadProcedures()
  } catch (err: any) {
    message.error(err.message || `Failed to drop ${routineType.toLowerCase()}`)
  }
}

async function dropTrigger(name: string) {
  try {
    await mysqlApi.dropTrigger(props.database, name)
    message.success(`Trigger ${name} dropped`)
    loadTriggers()
  } catch (err: any) {
    message.error(err.message || 'Failed to drop trigger')
  }
}

async function killProcess(id: number) {
  try {
    await mysqlApi.killProcess(id)
    message.success(`Process ${id} killed`)
    loadProcesses()
  } catch (err: any) {
    message.error(err.message || 'Failed to kill process')
  }
}

// Watch tab changes to load data
watch(activeTab, (tab) => {
  switch (tab) {
    case 'views':
      if (views.value.length === 0) loadViews()
      break
    case 'procedures':
      if (procedures.value.length === 0) loadProcedures()
      break
    case 'triggers':
      if (triggers.value.length === 0) loadTriggers()
      break
    case 'processes':
      loadProcesses()
      break
    case 'variables':
      if (variables.value.length === 0) loadVariables()
      break
  }
})

// Watch database changes
watch(() => props.database, () => {
  views.value = []
  procedures.value = []
  triggers.value = []
  // Reload current tab
  switch (activeTab.value) {
    case 'views':
      loadViews()
      break
    case 'procedures':
      loadProcedures()
      break
    case 'triggers':
      loadTriggers()
      break
  }
}, { immediate: true })
</script>

<style scoped>
.database-objects-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px 0;
}

.toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
}
</style>
