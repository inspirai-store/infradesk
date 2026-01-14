<script setup lang="ts">
import { onMounted, ref, h, computed } from 'vue'
import {
  NDataTable, NButton, NSpace, NInput, NTag, NIcon, useMessage, NModal,
  NFormItem, NSelect, NPopconfirm, NEmpty, NSpin,
  type DataTableColumns
} from 'naive-ui'
import type { RowKey } from 'naive-ui/es/data-table/src/interface'
import {
  AddOutline, RefreshOutline, PersonOutline, KeyOutline, TrashOutline,
  ShieldCheckmarkOutline, RemoveCircleOutline
} from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import { mysqlApi } from '@/api'
import type { UserInfo, UserGrantsResponse, UserGrantInfo } from '@/api'

const store = useMySQLStore()
const message = useMessage()

// Create user dialog
const showCreateDialog = ref(false)
const createForm = ref({
  username: '',
  host: '%',
  password: '',
})

// Change password dialog
const showPasswordDialog = ref(false)
const passwordForm = ref({
  username: '',
  host: '',
  new_password: '',
})

// Grant privileges dialog
const showGrantDialog = ref(false)
const grantForm = ref({
  username: '',
  host: '',
  database: '*',
  privileges: [] as string[],
})

// Revoke privileges dialog
const showRevokeDialog = ref(false)
const revokeForm = ref({
  username: '',
  host: '',
  database: '*',
  privileges: [] as string[],
})

// User grants cache
const userGrantsCache = ref<Record<string, UserGrantsResponse>>({})
const loadingGrants = ref<Record<string, boolean>>({})

// Available privileges
const privilegeOptions = [
  { label: 'ALL PRIVILEGES', value: 'ALL PRIVILEGES' },
  { label: 'SELECT', value: 'SELECT' },
  { label: 'INSERT', value: 'INSERT' },
  { label: 'UPDATE', value: 'UPDATE' },
  { label: 'DELETE', value: 'DELETE' },
  { label: 'CREATE', value: 'CREATE' },
  { label: 'DROP', value: 'DROP' },
  { label: 'ALTER', value: 'ALTER' },
  { label: 'INDEX', value: 'INDEX' },
  { label: 'REFERENCES', value: 'REFERENCES' },
  { label: 'CREATE VIEW', value: 'CREATE VIEW' },
  { label: 'SHOW VIEW', value: 'SHOW VIEW' },
  { label: 'TRIGGER', value: 'TRIGGER' },
  { label: 'EXECUTE', value: 'EXECUTE' },
  { label: 'CREATE ROUTINE', value: 'CREATE ROUTINE' },
  { label: 'ALTER ROUTINE', value: 'ALTER ROUTINE' },
]

// Database options for grants
const databaseOptions = computed(() => {
  const options = [{ label: '所有数据库 (*)', value: '*' }]
  for (const db of store.databases) {
    options.push({ label: db.name, value: db.name })
  }
  return options
})

// User identifier helper
function getUserKey(user: UserInfo): string {
  return `${user.user}@${user.host}`
}

const columns: DataTableColumns<UserInfo> = [
  {
    type: 'expand',
    renderExpand: (rowData) => {
      const key = getUserKey(rowData)
      const grants = userGrantsCache.value[key]
      const loading = loadingGrants.value[key]

      if (loading) {
        return h('div', { class: 'grants-loading' }, [
          h(NSpin, { size: 'small' }),
          h('span', { style: 'margin-left: 8px' }, '加载权限信息...')
        ])
      }

      if (!grants || grants.grants.length === 0) {
        return h(NEmpty, { description: '暂无权限信息', size: 'small' })
      }

      return h('div', { class: 'grants-panel' }, [
        h('h4', { class: 'grants-title' }, '用户权限'),
        h('div', { class: 'grants-list' }, grants.grants.map((grant: UserGrantInfo) =>
          h('div', { class: 'grant-item' }, [
            h('code', { class: 'grant-statement' }, grant.grant_statement)
          ])
        ))
      ])
    }
  },
  {
    title: '用户名',
    key: 'user',
    render: (row) => h('div', { class: 'flex items-center gap-2' }, [
      h(NIcon, { size: 16, color: '#00758F' }, { default: () => h(PersonOutline) }),
      h('span', row.user)
    ])
  },
  {
    title: '主机',
    key: 'host',
    render: (row) => h(NTag, { type: row.host === '%' ? 'info' : 'default' }, { default: () => row.host })
  },
  {
    title: '标识',
    key: 'identifier',
    render: (row) => h('code', {}, `\`${row.user}\`@\`${row.host}\``)
  },
  {
    title: '操作',
    key: 'actions',
    width: 280,
    render: (row) => h(NSpace, { size: 4 }, {
      default: () => [
        h(NButton, {
          size: 'tiny',
          quaternary: true,
          onClick: () => openPasswordDialog(row)
        }, {
          icon: () => h(NIcon, { size: 14 }, { default: () => h(KeyOutline) }),
          default: () => '改密码'
        }),
        h(NButton, {
          size: 'tiny',
          quaternary: true,
          type: 'success',
          onClick: () => openGrantDialog(row)
        }, {
          icon: () => h(NIcon, { size: 14 }, { default: () => h(ShieldCheckmarkOutline) }),
          default: () => '授权'
        }),
        h(NButton, {
          size: 'tiny',
          quaternary: true,
          type: 'warning',
          onClick: () => openRevokeDialog(row)
        }, {
          icon: () => h(NIcon, { size: 14 }, { default: () => h(RemoveCircleOutline) }),
          default: () => '撤销'
        }),
        h(NPopconfirm, {
          onPositiveClick: () => handleDeleteUser(row)
        }, {
          trigger: () => h(NButton, {
            size: 'tiny',
            quaternary: true,
            type: 'error'
          }, {
            icon: () => h(NIcon, { size: 14 }, { default: () => h(TrashOutline) }),
            default: () => '删除'
          }),
          default: () => `确定要删除用户 "${row.user}"@"${row.host}" 吗？`
        })
      ]
    })
  }
]

onMounted(() => {
  loadUsers()
})

async function loadUsers() {
  try {
    await store.fetchUsers()
  } catch (e) {
    message.error((e as Error).message)
  }
}

// Handle row expand to load grants
async function handleExpandedRowKeys(keys: RowKey[]) {
  for (const key of keys) {
    const keyStr = String(key)
    if (!userGrantsCache.value[keyStr] && !loadingGrants.value[keyStr]) {
      const [username, host] = keyStr.split('@')
      await loadUserGrants(username, host)
    }
  }
}

async function loadUserGrants(username: string, host: string) {
  const key = `${username}@${host}`
  loadingGrants.value[key] = true

  try {
    const grants = await mysqlApi.showGrants(username, host)
    userGrantsCache.value[key] = grants
  } catch (e) {
    message.error(`加载权限失败: ${(e as Error).message}`)
  } finally {
    loadingGrants.value[key] = false
  }
}

function openCreateDialog() {
  createForm.value = {
    username: '',
    host: '%',
    password: '',
  }
  showCreateDialog.value = true
}

async function handleCreateUser() {
  if (!createForm.value.username) {
    message.error('请输入用户名')
    return
  }

  if (!createForm.value.password) {
    message.error('请输入密码')
    return
  }

  try {
    await store.createUser(createForm.value)
    message.success(`用户 "${createForm.value.username}" 创建成功`)
    showCreateDialog.value = false
  } catch (e) {
    message.error((e as Error).message)
  }
}

// Password dialog
function openPasswordDialog(user: UserInfo) {
  passwordForm.value = {
    username: user.user,
    host: user.host,
    new_password: '',
  }
  showPasswordDialog.value = true
}

async function handleChangePassword() {
  if (!passwordForm.value.new_password) {
    message.error('请输入新密码')
    return
  }

  try {
    await mysqlApi.alterUserPassword(passwordForm.value)
    message.success('密码修改成功')
    showPasswordDialog.value = false
  } catch (e) {
    message.error((e as Error).message)
  }
}

// Delete user
async function handleDeleteUser(user: UserInfo) {
  try {
    await mysqlApi.dropUser({ username: user.user, host: user.host })
    message.success(`用户 "${user.user}"@"${user.host}" 已删除`)
    // Clear grants cache
    delete userGrantsCache.value[getUserKey(user)]
    await loadUsers()
  } catch (e) {
    message.error((e as Error).message)
  }
}

// Grant dialog
function openGrantDialog(user: UserInfo) {
  grantForm.value = {
    username: user.user,
    host: user.host,
    database: '*',
    privileges: ['SELECT'],
  }
  showGrantDialog.value = true
}

async function handleGrantPrivileges() {
  if (grantForm.value.privileges.length === 0) {
    message.error('请选择要授予的权限')
    return
  }

  try {
    await mysqlApi.grantPrivileges(grantForm.value.database, {
      username: grantForm.value.username,
      host: grantForm.value.host,
      privileges: grantForm.value.privileges,
    })
    message.success('权限授予成功')
    showGrantDialog.value = false
    // Refresh grants cache
    const key = `${grantForm.value.username}@${grantForm.value.host}`
    delete userGrantsCache.value[key]
    await loadUserGrants(grantForm.value.username, grantForm.value.host)
  } catch (e) {
    message.error((e as Error).message)
  }
}

// Revoke dialog
function openRevokeDialog(user: UserInfo) {
  revokeForm.value = {
    username: user.user,
    host: user.host,
    database: '*',
    privileges: ['SELECT'],
  }
  showRevokeDialog.value = true
}

async function handleRevokePrivileges() {
  if (revokeForm.value.privileges.length === 0) {
    message.error('请选择要撤销的权限')
    return
  }

  try {
    await mysqlApi.revokePrivileges({
      username: revokeForm.value.username,
      host: revokeForm.value.host,
      database: revokeForm.value.database,
      privileges: revokeForm.value.privileges,
    })
    message.success('权限撤销成功')
    showRevokeDialog.value = false
    // Refresh grants cache
    const key = `${revokeForm.value.username}@${revokeForm.value.host}`
    delete userGrantsCache.value[key]
    await loadUserGrants(revokeForm.value.username, revokeForm.value.host)
  } catch (e) {
    message.error((e as Error).message)
  }
}
</script>

<template>
  <div class="users-view">
    <div class="view-header">
      <div class="header-title">
        <h2>用户管理</h2>
        <p class="subtitle">管理 MySQL 用户账号和权限</p>
      </div>

      <div class="header-actions">
        <NInput
          placeholder="搜索用户..."
          clearable
          style="width: 240px"
        />
        <NSpace :size="8">
          <NButton size="small" quaternary @click="loadUsers">
            <template #icon>
              <NIcon size="14"><RefreshOutline /></NIcon>
            </template>
            刷新
          </NButton>
          <NButton size="small" type="primary" @click="openCreateDialog">
            <template #icon>
              <NIcon size="14"><AddOutline /></NIcon>
            </template>
            创建用户
          </NButton>
        </NSpace>
      </div>
    </div>

    <NDataTable
      :columns="columns"
      :data="store.users"
      :loading="store.loading"
      :bordered="false"
      :row-key="(row: UserInfo) => getUserKey(row)"
      size="small"
      striped
      @update:expanded-row-keys="handleExpandedRowKeys"
    />

    <!-- Create User Dialog -->
    <NModal v-model:show="showCreateDialog" preset="card" title="创建新用户" style="width: 440px">
      <NSpace vertical size="large">
        <NFormItem label="用户名" required>
          <NInput
            v-model:value="createForm.username"
            placeholder="输入用户名"
          />
        </NFormItem>

        <NFormItem label="主机">
          <NInput
            v-model:value="createForm.host"
            placeholder="默认 % 表示任意主机"
          />
          <template #feedback>
            % 表示任意主机，localhost 表示本地，特定 IP 地址限制访问
          </template>
        </NFormItem>

        <NFormItem label="密码" required>
          <NInput
            v-model:value="createForm.password"
            type="password"
            show-password-on="click"
            placeholder="输入密码"
          />
        </NFormItem>

        <NFormItem label="SQL 预览">
          <div class="sql-preview">
            <code>
              CREATE USER '{{ createForm.username || 'username' }}'@'{{ createForm.host || '%' }}' IDENTIFIED BY '***';
            </code>
          </div>
        </NFormItem>
      </NSpace>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showCreateDialog = false">取消</NButton>
          <NButton type="primary" :disabled="!createForm.username || !createForm.password" @click="handleCreateUser">创建</NButton>
        </NSpace>
      </template>
    </NModal>

    <!-- Change Password Dialog -->
    <NModal v-model:show="showPasswordDialog" preset="card" title="修改密码" style="width: 400px">
      <NSpace vertical size="large">
        <NFormItem label="用户">
          <NInput :value="`${passwordForm.username}@${passwordForm.host}`" disabled />
        </NFormItem>

        <NFormItem label="新密码" required>
          <NInput
            v-model:value="passwordForm.new_password"
            type="password"
            show-password-on="click"
            placeholder="输入新密码"
          />
        </NFormItem>

        <NFormItem label="SQL 预览">
          <div class="sql-preview">
            <code>
              ALTER USER '{{ passwordForm.username }}'@'{{ passwordForm.host }}' IDENTIFIED BY '***';
            </code>
          </div>
        </NFormItem>
      </NSpace>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showPasswordDialog = false">取消</NButton>
          <NButton type="primary" :disabled="!passwordForm.new_password" @click="handleChangePassword">确定</NButton>
        </NSpace>
      </template>
    </NModal>

    <!-- Grant Privileges Dialog -->
    <NModal v-model:show="showGrantDialog" preset="card" title="授予权限" style="width: 480px">
      <NSpace vertical size="large">
        <NFormItem label="用户">
          <NInput :value="`${grantForm.username}@${grantForm.host}`" disabled />
        </NFormItem>

        <NFormItem label="数据库">
          <NSelect
            v-model:value="grantForm.database"
            :options="databaseOptions"
            placeholder="选择数据库"
          />
          <template #feedback>
            * 表示所有数据库
          </template>
        </NFormItem>

        <NFormItem label="权限" required>
          <NSelect
            v-model:value="grantForm.privileges"
            :options="privilegeOptions"
            multiple
            placeholder="选择权限"
          />
        </NFormItem>

        <NFormItem label="SQL 预览">
          <div class="sql-preview">
            <code>
              GRANT {{ grantForm.privileges.join(', ') || '...' }} ON {{ grantForm.database === '*' ? '*.*' : `\`${grantForm.database}\`.*` }} TO '{{ grantForm.username }}'@'{{ grantForm.host }}';
            </code>
          </div>
        </NFormItem>
      </NSpace>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showGrantDialog = false">取消</NButton>
          <NButton type="primary" :disabled="grantForm.privileges.length === 0" @click="handleGrantPrivileges">授权</NButton>
        </NSpace>
      </template>
    </NModal>

    <!-- Revoke Privileges Dialog -->
    <NModal v-model:show="showRevokeDialog" preset="card" title="撤销权限" style="width: 480px">
      <NSpace vertical size="large">
        <NFormItem label="用户">
          <NInput :value="`${revokeForm.username}@${revokeForm.host}`" disabled />
        </NFormItem>

        <NFormItem label="数据库">
          <NSelect
            v-model:value="revokeForm.database"
            :options="databaseOptions"
            placeholder="选择数据库"
          />
          <template #feedback>
            * 表示所有数据库
          </template>
        </NFormItem>

        <NFormItem label="权限" required>
          <NSelect
            v-model:value="revokeForm.privileges"
            :options="privilegeOptions"
            multiple
            placeholder="选择要撤销的权限"
          />
        </NFormItem>

        <NFormItem label="SQL 预览">
          <div class="sql-preview">
            <code>
              REVOKE {{ revokeForm.privileges.join(', ') || '...' }} ON {{ revokeForm.database === '*' ? '*.*' : `\`${revokeForm.database}\`.*` }} FROM '{{ revokeForm.username }}'@'{{ revokeForm.host }}';
            </code>
          </div>
        </NFormItem>
      </NSpace>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showRevokeDialog = false">取消</NButton>
          <NButton type="warning" :disabled="revokeForm.privileges.length === 0" @click="handleRevokePrivileges">撤销</NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>

<style scoped>
.users-view {
  padding: 16px;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header-title h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 500;
}

.header-title .subtitle {
  margin: 4px 0 0 0;
  font-size: 13px;
  color: #999;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sql-preview {
  padding: 12px;
  background: #f5f5f5;
  border-radius: 4px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 12px;
  overflow-x: auto;
  width: 100%;
}

.sql-preview code {
  color: #d63384;
  word-break: break-all;
}

/* Grants panel styles */
.grants-loading {
  display: flex;
  align-items: center;
  padding: 12px;
  color: #666;
}

.grants-panel {
  padding: 12px 16px;
  background: #fafafa;
  border-radius: 4px;
}

.grants-title {
  margin: 0 0 12px 0;
  font-size: 13px;
  font-weight: 500;
  color: #333;
}

.grants-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.grant-item {
  padding: 8px 12px;
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 4px;
}

.grant-statement {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 12px;
  color: #00758F;
  word-break: break-all;
}
</style>
