<script setup lang="ts">
import { onMounted, ref, h } from 'vue'
import { NDataTable, NButton, NSpace, NInput, NTag, NIcon, useMessage, NModal, NFormItem, type DataTableColumns } from 'naive-ui'
import { AddOutline, RefreshOutline, PersonOutline } from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import type { UserInfo } from '@/api'

const store = useMySQLStore()
const message = useMessage()

const showCreateDialog = ref(false)
const createForm = ref({
  username: '',
  user_host: '%',
  password: '',
})

const columns: DataTableColumns<UserInfo> = [
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

function openCreateDialog() {
  createForm.value = {
    username: '',
    user_host: '%',
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
</script>

<template>
  <div class="users-view">
    <div class="view-header">
      <div class="header-title">
        <h2>用户管理</h2>
        <p class="subtitle">管理 MySQL 用户账号</p>
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
      size="small"
      striped
    />

    <!-- Create User Dialog -->
    <NModal v-model:show="showCreateDialog" preset="card" title="创建新用户" style="width: 440px">
      <NSpace vertical size="large">
        <!-- Username -->
        <NFormItem label="用户名" required>
          <NInput
            v-model:value="createForm.username"
            placeholder="输入用户名"
          />
        </NFormItem>

        <!-- User Host -->
        <NFormItem label="主机">
          <NInput
            v-model:value="createForm.user_host"
            placeholder="默认 % 表示任意主机"
          />
          <template #feedback>
            % 表示任意主机，localhost 表示本地，特定 IP 地址限制访问
          </template>
        </NFormItem>

        <!-- Password -->
        <NFormItem label="密码" required>
          <NInput
            v-model:value="createForm.password"
            type="password"
            show-password-on="click"
            placeholder="输入密码"
          />
        </NFormItem>

        <!-- SQL Preview -->
        <NFormItem label="SQL 预览">
          <div class="sql-preview">
            <code>
              CREATE USER '{{ createForm.username || 'username' }}'@'{{ createForm.user_host || '%' }}' IDENTIFIED BY '***';
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
