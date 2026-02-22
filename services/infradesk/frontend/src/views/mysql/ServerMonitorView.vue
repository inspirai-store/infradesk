<script setup lang="ts">
import { ref, computed, h, onMounted } from 'vue'
import {
  NTabs,
  NTabPane,
  NDataTable,
  NButton,
  NInput,
  NPopconfirm,
  NCard,
  NStatistic,
  NGrid,
  NGridItem,
  useMessage,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { mysqlApi } from '@/api'
import type { ProcessInfo, ServerVariable } from '@/api'
import { useDebounceFn } from '@vueuse/core'

const message = useMessage()

// Tab state
const activeTab = ref('processes')

// Data states
const processes = ref<ProcessInfo[]>([])
const variables = ref<ServerVariable[]>([])

// Loading states
const loadingProcesses = ref(false)
const loadingVariables = ref(false)

// Filter
const variableFilter = ref('')

// Process columns
const processColumns = computed<DataTableColumns<ProcessInfo>>(() => [
  { title: 'ID', key: 'id', width: 80 },
  { title: 'User', key: 'user', width: 100 },
  { title: 'Host', key: 'host', ellipsis: { tooltip: true }, width: 150 },
  { title: 'DB', key: 'db', width: 100 },
  { title: 'Command', key: 'command', width: 100 },
  { title: 'Time (s)', key: 'time', width: 80 },
  { title: 'State', key: 'state', ellipsis: { tooltip: true } },
  { title: 'Info', key: 'info', ellipsis: { tooltip: true } },
  {
    title: 'Actions',
    key: 'actions',
    width: 80,
    fixed: 'right',
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
  { title: 'Variable Name', key: 'name', ellipsis: { tooltip: true }, width: 300 },
  { title: 'Value', key: 'value', ellipsis: { tooltip: true } },
])

// Stats computed
const processStats = computed(() => ({
  total: processes.value.length,
  sleeping: processes.value.filter(p => p.command === 'Sleep').length,
  active: processes.value.filter(p => p.command !== 'Sleep').length,
}))

// Load functions
async function loadProcesses() {
  loadingProcesses.value = true
  try {
    processes.value = await mysqlApi.getProcessList()
  } catch (err: unknown) {
    message.error((err as Error).message || 'Failed to load processes')
  } finally {
    loadingProcesses.value = false
  }
}

async function loadVariables() {
  loadingVariables.value = true
  try {
    variables.value = await mysqlApi.getServerVariables(variableFilter.value || undefined)
  } catch (err: unknown) {
    message.error((err as Error).message || 'Failed to load variables')
  } finally {
    loadingVariables.value = false
  }
}

const debouncedLoadVariables = useDebounceFn(loadVariables, 300)

// Kill process
async function killProcess(id: number) {
  try {
    await mysqlApi.killProcess(id)
    message.success(`Process ${id} killed`)
    loadProcesses()
  } catch (err: unknown) {
    message.error((err as Error).message || 'Failed to kill process')
  }
}

onMounted(() => {
  loadProcesses()
})
</script>

<template>
  <div class="server-monitor-view">
    <div class="view-header">
      <h2>Server Monitor</h2>
      <p class="subtitle">Monitor MySQL server processes and variables</p>
    </div>

    <NTabs v-model:value="activeTab" type="line">
      <!-- Processes Tab -->
      <NTabPane name="processes" tab="Processes">
        <div class="tab-content">
          <!-- Stats -->
          <NGrid :cols="3" :x-gap="16" style="margin-bottom: 16px">
            <NGridItem>
              <NCard size="small">
                <NStatistic label="Total Connections" :value="processStats.total" />
              </NCard>
            </NGridItem>
            <NGridItem>
              <NCard size="small">
                <NStatistic label="Active" :value="processStats.active" />
              </NCard>
            </NGridItem>
            <NGridItem>
              <NCard size="small">
                <NStatistic label="Sleeping" :value="processStats.sleeping" />
              </NCard>
            </NGridItem>
          </NGrid>

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
            :scroll-x="1000"
            :max-height="500"
          />
        </div>
      </NTabPane>

      <!-- Variables Tab -->
      <NTabPane name="variables" tab="Variables">
        <div class="tab-content">
          <div class="toolbar">
            <NInput
              v-model:value="variableFilter"
              placeholder="Filter variables (e.g., innodb, max)"
              size="small"
              style="width: 300px"
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
            :max-height="500"
          />
        </div>
      </NTabPane>
    </NTabs>
  </div>
</template>

<style scoped>
.server-monitor-view {
  padding: 16px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.view-header {
  margin-bottom: 16px;
}

.view-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 500;
}

.view-header .subtitle {
  margin: 4px 0 0 0;
  font-size: 13px;
  color: #999;
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
