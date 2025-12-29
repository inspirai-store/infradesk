<script setup lang="ts">
import { computed, h, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  NTree,
  NInput,
  NSpace,
  NButton,
  NIcon,
  NSpin,
  NEmpty,
  NDropdown,
  NModal,
  useMessage,
  useDialog,
  NCheckbox,
  NCheckboxGroup,
  NSelect,
  NFormItem,
  NFormGroup,
  NSwitch,
} from 'naive-ui'
import {
  AddOutline,
  RefreshOutline,
  ServerOutline,
  GridOutline,
  TrashOutline,
  CopyOutline,
  CreateOutline,
  PersonOutline,
} from '@vicons/ionicons5'
import type { TreeOption } from 'naive-ui'
import { useMySQLStore } from '@/stores/mysql'
import type { CreateDatabaseRequest, AlterDatabaseRequest, GrantPrivilegesRequest } from '@/api'

const router = useRouter()
const store = useMySQLStore()
const message = useMessage()
const dialog = useDialog()

const searchValue = ref('')
const expandedKeys = ref<string[]>([])
const contextMenuNodeId = ref<string | null>(null)
const contextMenuPosition = ref<{ x: number; y: number }>({ x: 0, y: 0 })
const showCreateDialog = ref(false)
const showAlterDialog = ref(false)
const showGrantDialog = ref(false)
const alterDatabaseName = ref('')
const grantDatabaseName = ref('')

// Create database form state
const createForm = ref<CreateDatabaseRequest>({
  name: '',
  if_not_exists: true,
  charset: 'utf8mb4',
  collate: 'utf8mb4_unicode_ci',
})

// Alter database form state
const alterForm = ref<AlterDatabaseRequest>({
  charset: '',
  collate: '',
})

// Grant privileges form state
const grantForm = ref<GrantPrivilegesRequest>({
  username: '',
  user_host: '%',
  password: '',
  privileges: ['SELECT', 'INSERT', 'UPDATE', 'DELETE'],
  grant_option: false,
})

const privilegeOptions = [
  { label: 'SELECT', value: 'SELECT' },
  { label: 'INSERT', value: 'INSERT' },
  { label: 'UPDATE', value: 'UPDATE' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'CREATE', value: 'CREATE' },
  { label: 'DROP', value: 'DROP' },
  { label: 'REFERENCES', value: 'REFERENCES' },
  { label: 'INDEX', value: 'INDEX' },
  { label: 'ALTER', value: 'ALTER' },
  { label: 'CREATE TEMPORARY TABLES', value: 'CREATE TEMPORARY TABLES' },
  { label: 'LOCK TABLES', value: 'LOCK TABLES' },
  { label: 'EXECUTE', value: 'EXECUTE' },
  { label: 'ALL PRIVILEGES', value: 'ALL PRIVILEGES' },
]

const charsetOptions = [
  { label: 'utf8mb4', value: 'utf8mb4' },
  { label: 'utf8', value: 'utf8' },
  { label: 'latin1', value: 'latin1' },
  { label: 'ascii', value: 'ascii' },
  { label: 'binary', value: 'binary' },
]

const collateOptions = [
  { label: 'utf8mb4_unicode_ci', value: 'utf8mb4_unicode_ci' },
  { label: 'utf8mb4_general_ci', value: 'utf8mb4_general_ci' },
  { label: 'utf8mb4_bin', value: 'utf8mb4_bin' },
  { label: 'utf8_unicode_ci', value: 'utf8_unicode_ci' },
  { label: 'utf8_general_ci', value: 'utf8_general_ci' },
  { label: 'utf8_bin', value: 'utf8_bin' },
  { label: 'latin1_swedish_ci', value: 'latin1_swedish_ci' },
  { label: 'latin1_general_ci', value: 'latin1_general_ci' },
]

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

// Node props for context menu
const nodeProps = ({ option }: { option: TreeOption }) => {
  return {
    onContextmenu(e: MouseEvent) {
      e.preventDefault()
      e.stopPropagation()
      contextMenuNodeId.value = option.key as string
      contextMenuPosition.value = { x: e.clientX, y: e.clientY }
    }
  }
}

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

function openCreateDialog() {
  createForm.value = {
    name: '',
    if_not_exists: true,
    charset: 'utf8mb4',
    collate: 'utf8mb4_unicode_ci',
  }
  showCreateDialog.value = true
}

async function handleCreateDatabase() {
  if (!createForm.value.name) {
    message.error('请输入数据库名称')
    return
  }

  try {
    await store.createDatabase(createForm.value)
    message.success(`数据库 "${createForm.value.name}" 创建成功`)
    showCreateDialog.value = false
  } catch (e) {
    message.error((e as Error).message)
  }
}

function openAlterDialog(dbName: string) {
  alterDatabaseName.value = dbName
  alterForm.value = {
    charset: '',
    collate: '',
  }
  showAlterDialog.value = true
}

async function handleAlterDatabase() {
  if (!alterForm.value.charset && !alterForm.value.collate) {
    message.error('请至少选择要修改的属性')
    return
  }

  try {
    await store.alterDatabase(alterDatabaseName.value, alterForm.value)
    message.success(`数据库 "${alterDatabaseName.value}" 属性已修改`)
    showAlterDialog.value = false
  } catch (e) {
    message.error((e as Error).message)
  }
}

function openGrantDialog(dbName: string) {
  grantDatabaseName.value = dbName
  grantForm.value = {
    username: '',
    user_host: '%',
    password: '',
    privileges: ['SELECT', 'INSERT', 'UPDATE', 'DELETE'],
    grant_option: false,
  }
  showGrantDialog.value = true
}

async function handleGrantPrivileges() {
  if (!grantForm.value.username) {
    message.error('请输入用户名')
    return
  }

  if (grantForm.value.privileges.length === 0) {
    message.error('请至少选择一个权限')
    return
  }

  // 如果是新用户，必须提供密码
  if (grantForm.value.password === '') {
    message.warning('提示：未提供密码，仅对现有用户授权。如果用户不存在，请提供密码。')
  }

  try {
    await store.grantPrivileges(grantDatabaseName.value, grantForm.value)
    message.success(`已授予用户 "${grantForm.value.username}" 对数据库 "${grantDatabaseName.value}" 的权限`)
    showGrantDialog.value = false
  } catch (e) {
    message.error((e as Error).message)
  }
}

const contextMenuOptions = computed(() => {
  const isDatabase = contextMenuNodeId.value && !contextMenuNodeId.value.includes('/')
  return [
    {
      label: '刷新',
      key: 'refresh',
      icon: () => h(NIcon, null, { default: () => h(RefreshOutline) }),
      disabled: !isDatabase,
    },
    {
      label: '复制名称',
      key: 'copy',
      icon: () => h(NIcon, null, { default: () => h(CopyOutline) }),
    },
    {
      label: '修改属性',
      key: 'alter',
      icon: () => h(NIcon, null, { default: () => h(CreateOutline) }),
      disabled: !isDatabase,
    },
    {
      label: '授权用户',
      key: 'grant',
      icon: () => h(NIcon, null, { default: () => h(PersonOutline) }),
      disabled: !isDatabase,
    },
    {
      type: 'divider',
      key: 'd1',
    },
    {
      label: '删除数据库',
      key: 'drop',
      icon: () => h(NIcon, null, { default: () => h(TrashOutline) }),
      disabled: !isDatabase,
    },
  ]
})

function handleContextMenuSelect(key: string) {
  if (!contextMenuNodeId.value) return

  const nodeKey = contextMenuNodeId.value
  const isDatabase = !nodeKey.includes('/')
  const databaseName = isDatabase ? nodeKey : nodeKey.split('/')[0]

  switch (key) {
    case 'refresh':
      store.fetchTables(databaseName)
      message.success(`已刷新 "${databaseName}"`)
      break
    case 'copy':
      navigator.clipboard.writeText(databaseName)
      message.success(`已复制: ${databaseName}`)
      break
    case 'alter':
      openAlterDialog(databaseName)
      break
    case 'grant':
      openGrantDialog(databaseName)
      break
    case 'drop':
      handleDropDatabase(databaseName)
      break
  }

  contextMenuNodeId.value = null
}

function handleDropDatabase(dbName: string) {
  dialog.warning({
    title: '确认删除',
    content: `确定要删除数据库 "${dbName}" 吗？此操作不可恢复！`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await store.dropDatabase(dbName)
        message.success(`数据库 "${dbName}" 已删除`)
      } catch (e) {
        message.error((e as Error).message)
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
        <NButton size="tiny" quaternary @click="openCreateDialog">
          <template #icon>
            <NIcon size="14"><AddOutline /></NIcon>
          </template>
        </NButton>
        <NButton size="tiny" quaternary @click="router.push('/mysql/users')">
          <template #icon>
            <NIcon size="14"><PersonOutline /></NIcon>
          </template>
        </NButton>
      </NSpace>
    </div>

    <!-- Tree with Context Menu -->
    <NSpin :show="store.loading">
      <NDropdown
        trigger="manual"
        placement="bottom-start"
        :show="contextMenuNodeId !== null"
        :options="contextMenuOptions"
        :x="contextMenuPosition.x"
        :y="contextMenuPosition.y"
        @clickoutside="contextMenuNodeId = null"
        @select="handleContextMenuSelect"
      />
      <NTree
        v-if="treeData.length > 0"
        :data="treeData"
        :expanded-keys="expandedKeys"
        :node-props="nodeProps"
        selectable
        block-line
        @update:selected-keys="handleSelect"
        @update:expanded-keys="handleExpand"
      />
      <NEmpty v-else description="暂无数据库" size="small" />
    </NSpin>

    <!-- Create Database Dialog -->
    <NModal v-model:show="showCreateDialog" preset="card" title="创建数据库" style="width: 480px">
      <NSpace vertical size="large">
        <!-- Database Name -->
        <NFormItem label="数据库名称" required>
          <NInput
            v-model:value="createForm.name"
            placeholder="输入数据库名称"
            @keyup.enter="handleCreateDatabase"
          />
        </NFormItem>

        <!-- IF NOT EXISTS -->
        <NFormItem label="选项">
          <NSwitch v-model:value="createForm.if_not_exists">
            <template #checked>IF NOT EXISTS</template>
            <template #unchecked>直接创建</template>
          </NSwitch>
        </NFormItem>

        <!-- Character Set -->
        <NFormItem label="字符集">
          <NSelect
            v-model:value="createForm.charset"
            :options="charsetOptions"
            placeholder="选择字符集"
          />
        </NFormItem>

        <!-- Collate -->
        <NFormItem label="排序规则">
          <NSelect
            v-model:value="createForm.collate"
            :options="collateOptions"
            placeholder="选择排序规则"
            filterable
            tag
          />
        </NFormItem>

        <!-- SQL Preview -->
        <NFormItem label="SQL 预览">
          <div class="sql-preview">
            <code>
              CREATE DATABASE
              <template v-if="createForm.if_not_exists"> IF NOT EXISTS</template>
              `{{ createForm.name || 'database_name' }}`
              CHARACTER SET {{ createForm.charset || 'utf8mb4' }}
              COLLATE {{ createForm.collate || 'utf8mb4_unicode_ci' }};
            </code>
          </div>
        </NFormItem>
      </NSpace>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showCreateDialog = false">取消</NButton>
          <NButton type="primary" @click="handleCreateDatabase">创建</NButton>
        </NSpace>
      </template>
    </NModal>

    <!-- Alter Database Dialog -->
    <NModal v-model:show="showAlterDialog" preset="card" title="修改数据库属性" style="width: 480px">
      <NSpace vertical size="large">
        <NFormItem label="数据库名称">
          <NInput :value="alterDatabaseName" disabled />
        </NFormItem>

        <!-- Character Set -->
        <NFormItem label="字符集">
          <NSelect
            v-model:value="alterForm.charset"
            :options="charsetOptions"
            placeholder="不修改留空"
            clearable
          />
        </NFormItem>

        <!-- Collate -->
        <NFormItem label="排序规则">
          <NSelect
            v-model:value="alterForm.collate"
            :options="collateOptions"
            placeholder="不修改留空"
            filterable
            tag
            clearable
          />
        </NFormItem>

        <!-- SQL Preview -->
        <NFormItem label="SQL 预览">
          <div class="sql-preview">
            <code>
              ALTER DATABASE `{{ alterDatabaseName }}`
              <template v-if="alterForm.charset"> CHARACTER SET {{ alterForm.charset }}</template>
              <template v-if="alterForm.collate"> COLLATE {{ alterForm.collate }}</template>;
            </code>
          </div>
        </NFormItem>
      </NSpace>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showAlterDialog = false">取消</NButton>
          <NButton type="primary" :disabled="!alterForm.charset && !alterForm.collate" @click="handleAlterDatabase">修改</NButton>
        </NSpace>
      </template>
    </NModal>

    <!-- Grant Privileges Dialog -->
    <NModal v-model:show="showGrantDialog" preset="card" title="授权用户" style="width: 520px">
      <NSpace vertical size="large">
        <NFormItem label="数据库">
          <NInput :value="grantDatabaseName" disabled />
        </NFormItem>

        <!-- Username -->
        <NFormItem label="用户名" required>
          <NInput
            v-model:value="grantForm.username"
            placeholder="输入用户名"
          />
        </NFormItem>

        <!-- User Host -->
        <NFormItem label="主机">
          <NInput
            v-model:value="grantForm.user_host"
            placeholder="默认 % 表示任意主机"
          />
        </NFormItem>

        <!-- Password -->
        <NFormItem label="密码">
          <NInput
            v-model:value="grantForm.password"
            type="password"
            show-password-on="click"
            placeholder="新用户必填，现有用户可选"
          />
          <template #feedback>
            新用户必须提供密码；如果用户已存在，留空即可
          </template>
        </NFormItem>

        <!-- Privileges -->
        <NFormItem label="权限" required>
          <NCheckboxGroup v-model:value="grantForm.privileges">
            <NSpace vertical>
              <NCheckbox
                v-for="priv in privilegeOptions"
                :key="priv.value"
                :value="priv.value"
                :label="priv.label"
              />
            </NSpace>
          </NCheckboxGroup>
        </NFormItem>

        <!-- Grant Option -->
        <NFormItem label="授权选项">
          <NSwitch v-model:value="grantForm.grant_option">
            <template #checked>允许授权</template>
            <template #unchecked>不允许</template>
          </NSwitch>
          <template #feedback>
            允许该用户将获得的权限授予其他用户
          </template>
        </NFormItem>

        <!-- SQL Preview -->
        <NFormItem label="SQL 预览">
          <div class="sql-preview">
            <code>
              <template v-if="grantForm.password">CREATE USER IF NOT EXISTS '{{ grantForm.username }}'@'{{ grantForm.user_host }}' IDENTIFIED BY '***';</template>
              GRANT {{ grantForm.privileges.join(', ') || '(请选择权限)' }}
              ON `{{ grantDatabaseName }}`.*
              TO '{{ grantForm.username }}'@'{{ grantForm.user_host }}'
              <template v-if="grantForm.grant_option"> WITH GRANT OPTION</template>;
            </code>
          </div>
        </NFormItem>
      </NSpace>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showGrantDialog = false">取消</NButton>
          <NButton type="primary" :disabled="!grantForm.username || grantForm.privileges.length === 0" @click="handleGrantPrivileges">授权</NButton>
        </NSpace>
      </template>
    </NModal>
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

.sql-preview {
  padding: 12px;
  background: #f5f5f5;
  border-radius: 4px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 12px;
  overflow-x: auto;
}

.sql-preview code {
  color: #d63384;
  word-break: break-all;
}
</style>
