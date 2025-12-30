<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NSpace,
  NButton,
  NIcon,
  NInput,
  NCard,
  NText,
  NAlert,
  NTag,
  NCollapse,
  NCollapseItem,
  NTooltip,
  useMessage,
} from 'naive-ui'
import {
  SparklesOutline,
  ChevronForwardOutline,
  CheckmarkCircleOutline,
  WarningOutline,
  BulbOutline,
  SettingsOutline,
} from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'
import { useLLMStore } from '@/stores/llm'
import { getLLMService } from '@/services/llm'
import LLMConfigPanel from './LLMConfigPanel.vue'

interface Props {
  show: boolean
  database: string
}

interface Emits {
  (e: 'update:show', value: boolean): void
  (e: 'applySQL', sql: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const mysqlStore = useMySQLStore()
const llmStore = useLLMStore()
const message = useMessage()

// 状态
const activeTab = ref<'generate' | 'optimize' | 'diagnose'>('generate')
const naturalLanguageInput = ref('')
const currentSQL = ref('')
const isProcessing = ref(false)
const suggestions = ref<Suggestion[]>([])
const diagnosis = ref<Diagnosis | null>(null)
const showLLMConfig = ref(false)

// 诊断错误的输入值（用于 v-model）
const diagnosisErrorInput = computed({
  get: () => diagnosis.value?.error || '',
  set: (val) => {
    if (!diagnosis.value) {
      diagnosis.value = { error: val, explanation: '', suggestions: [] }
    } else {
      diagnosis.value.error = val
    }
  }
})

interface Suggestion {
  id: string
  type: 'optimization' | 'warning' | 'info'
  title: string
  description: string
  sql?: string
  impact: 'high' | 'medium' | 'low'
}

interface Diagnosis {
  error: string
  explanation: string
  suggestions: string[]
  fixedSQL?: string
}

// 获取当前数据库的表列表（用于上下文）
const tableContext = computed(() => {
  const tables = mysqlStore.tables.map(t => ({
    name: t.name,
    columns: [] // 实际应用中可以从 tableSchema 获取
  }))
  return {
    database: props.database,
    tables
  }
})

// 生成 SQL
async function generateSQL() {
  if (!naturalLanguageInput.value.trim()) {
    message.warning('请输入查询描述')
    return
  }

  isProcessing.value = true
  try {
    if (llmStore.isConfigured) {
      // 使用真实 LLM API
      const service = getLLMService()
      const result = await service.generateSQL({
        prompt: naturalLanguageInput.value,
        databaseSchema: tableContext.value
      })
      currentSQL.value = result.sql
      message.success('SQL 已生成')
    } else {
      // 使用模拟规则
      message.info('未配置 LLM API，使用模拟生成（功能有限）')
      const result = await mockAIGenerateSQL(naturalLanguageInput.value, tableContext.value)
      currentSQL.value = result.sql
      message.success('SQL 已生成（模拟）')
    }
  } catch (e) {
    message.error((e as Error).message)
  } finally {
    isProcessing.value = false
  }
}

// 优化当前 SQL
async function optimizeSQL() {
  if (!currentSQL.value.trim()) {
    message.warning('请先生成或输入 SQL')
    return
  }

  isProcessing.value = true
  try {
    if (llmStore.isConfigured) {
      // 使用真实 LLM API
      const service = getLLMService()
      const result = await service.optimizeSQL({
        sql: currentSQL.value,
        databaseSchema: tableContext.value
      })
      // 为每个建议添加 id
      suggestions.value = result.suggestions.map((s, index) => ({
        ...s,
        id: `llm-${index}-${Date.now()}`
      }))
      if (result.optimizedSQL) {
        currentSQL.value = result.optimizedSQL
      }
      message.success(`找到 ${result.suggestions.length} 条优化建议`)
    } else {
      // 使用模拟规则
      message.info('未配置 LLM API，使用模拟分析（功能有限）')
      const result = await mockAIOptimizeSQL(currentSQL.value, tableContext.value)
      suggestions.value = result.suggestions
      if (result.optimizedSQL) {
        currentSQL.value = result.optimizedSQL
      }
      message.success(`找到 ${result.suggestions.length} 条优化建议（模拟）`)
    }
  } catch (e) {
    message.error((e as Error).message)
  } finally {
    isProcessing.value = false
  }
}

// 诊断 SQL 错误
async function diagnoseSQLError(error: string) {
  if (!currentSQL.value.trim()) {
    message.warning('请先输入 SQL')
    return
  }

  isProcessing.value = true
  try {
    if (llmStore.isConfigured) {
      // 使用真实 LLM API
      const service = getLLMService()
      diagnosis.value = await service.diagnoseSQLError({
        sql: currentSQL.value,
        error
      })
      message.success('诊断完成')
    } else {
      // 使用模拟规则
      message.info('未配置 LLM API，使用模拟诊断（功能有限）')
      diagnosis.value = await mockAIDiagnoseSQL(currentSQL.value, error)
      message.success('诊断完成（模拟）')
    }
  } catch (e) {
    message.error((e as Error).message)
  } finally {
    isProcessing.value = false
  }
}

// 应用 SQL
function applySQL() {
  if (!currentSQL.value.trim()) {
    message.warning('没有可应用的 SQL')
    return
  }
  emit('applySQL', currentSQL.value)
  emit('update:show', false)
}

// 获取建议类型颜色
function getSuggestionTypeColor(type: Suggestion['type']) {
  switch (type) {
    case 'optimization': return 'success'
    case 'warning': return 'warning'
    case 'info': return 'info'
  }
}

// 获取建议类型图标
function getSuggestionTypeIcon(type: Suggestion['type']) {
  switch (type) {
    case 'optimization': return BulbOutline
    case 'warning': return WarningOutline
    case 'info': return CheckmarkCircleOutline
  }
}

// 模拟 AI API 响应（实际应用中应调用真实 AI 服务）
async function mockAIGenerateSQL(prompt: string, context: any): Promise<{ sql: string }> {
  // 模拟延迟
  await new Promise(resolve => setTimeout(resolve, 1500))

  // 简单的规则匹配（实际应用中应使用真实 AI API）
  const lowerPrompt = prompt.toLowerCase()

  if (lowerPrompt.includes('所有') && lowerPrompt.includes('用户')) {
    return { sql: 'SELECT * FROM users;' }
  } else if (lowerPrompt.includes('用户') && lowerPrompt.includes('数量')) {
    return { sql: 'SELECT COUNT(*) as user_count FROM users;' }
  } else if (lowerPrompt.includes('最近') && lowerPrompt.includes('订单')) {
    return { sql: 'SELECT * FROM orders ORDER BY created_at DESC LIMIT 10;' }
  } else if (lowerPrompt.includes('创建') && lowerPrompt.includes('用户')) {
    return { sql: 'INSERT INTO users (username, email, created_at) VALUES (?, ?, NOW());' }
  } else if (lowerPrompt.includes('更新') && lowerPrompt.includes('用户')) {
    return { sql: 'UPDATE users SET updated_at = NOW() WHERE id = ?;' }
  } else if (lowerPrompt.includes('删除') && lowerPrompt.includes('用户')) {
    return { sql: 'DELETE FROM users WHERE id = ?;' }
  }

  // 默认生成一个通用查询
  return {
    sql: `-- 请根据您的需求修改以下 SQL\nSELECT * FROM ${context.tables[0]?.name || 'table_name'} LIMIT 10;`
  }
}

// 模拟 AI 优化建议
async function mockAIOptimizeSQL(sql: string, _context: any): Promise<{
  suggestions: Suggestion[]
  optimizedSQL?: string
}> {
  await new Promise(resolve => setTimeout(resolve, 1000))

  const suggestions: Suggestion[] = []
  let optimizedSQL = sql

  // 检查 SELECT *
  if (/SELECT\s+\*\s+FROM/i.test(sql)) {
    suggestions.push({
      id: '1',
      type: 'optimization',
      title: '避免使用 SELECT *',
      description: '只查询需要的列可以减少网络传输和内存使用',
      impact: 'high'
    })
  }

  // 检查是否有 LIMIT
  if (!/LIMIT\s+\d+/i.test(sql) && /SELECT/i.test(sql)) {
    suggestions.push({
      id: '2',
      type: 'optimization',
      title: '建议添加 LIMIT 子句',
      description: '对于大表查询，添加 LIMIT 可以避免返回过多数据',
      impact: 'medium'
    })
    optimizedSQL = sql.trim().endsWith(';') ? sql.slice(0, -1) + ' LIMIT 100;' : sql + ' LIMIT 100;'
  }

  // 检查是否有 WHERE 条件
  if (/DELETE\s+FROM/i.test(sql) && !/WHERE/i.test(sql)) {
    suggestions.push({
      id: '3',
      type: 'warning',
      title: 'DELETE 缺少 WHERE 条件',
      description: '这会删除表中的所有数据，请确认是否为预期操作',
      impact: 'high'
    })
  }

  // 检查是否有索引提示
  if (/SELECT/i.test(sql) && /WHERE\s+.+\s*LIKE\s*['"].*%.*%['"]/i.test(sql)) {
    suggestions.push({
      id: '4',
      type: 'warning',
      title: '前缀模糊查询可能无法使用索引',
      description: 'LIKE "%value%" 无法使用普通索引，考虑使用全文索引或其他方案',
      impact: 'medium'
    })
  }

  return { suggestions, optimizedSQL: optimizedSQL !== sql ? optimizedSQL : undefined }
}

// 模拟 AI 诊断
async function mockAIDiagnoseSQL(_sql: string, error: string): Promise<Diagnosis> {
  await new Promise(resolve => setTimeout(resolve, 1000))

  const lowerError = error.toLowerCase()

  if (lowerError.includes('syntax')) {
    return {
      error,
      explanation: 'SQL 语法错误，可能是关键字拼写错误、缺少分隔符或括号不匹配',
      suggestions: [
        '检查关键字拼写是否正确',
        '确保每条语句以分号结尾',
        '检查括号是否成对匹配',
        '检查字符串是否使用正确的引号'
      ]
    }
  } else if (lowerError.includes('table') && lowerError.includes("doesn't exist")) {
    return {
      error,
      explanation: '引用的表不存在',
      suggestions: [
        '检查表名拼写是否正确',
        '确认当前数据库是否包含该表',
        '检查数据库连接是否正确'
      ]
    }
  } else if (lowerError.includes('column') && lowerError.includes('not found')) {
    const match = error.match(/column ['"](.+?)['"]/i)
    const columnName = match ? match[1] : '该列'
    return {
      error,
      explanation: `列 "${columnName}" 在表中不存在`,
      suggestions: [
        '检查列名拼写是否正确',
        '确认列是否存在于指定的表中',
        '使用 SHOW COLUMNS 查看表结构'
      ]
    }
  }

  return {
    error,
    explanation: '无法确定具体的错误类型',
    suggestions: [
      '检查 SQL 语法是否正确',
      '确认表名和列名是否存在',
      '查看完整的错误信息'
    ]
  }
}
</script>

<template>
  <div class="ai-assistant-container">
    <NCard
      v-if="show"
      class="glass-card"
      :style="{ maxHeight: '600px', overflow: 'auto' }"
    >
      <template #header>
        <NSpace align="center" justify="space-between" style="width: 100%">
          <NSpace align="center">
            <NIcon size="18" color="var(--zx-accent-cyan)">
              <SparklesOutline />
            </NIcon>
            <span class="title-font neon-text">AI 助手</span>
            <NTag v-if="!llmStore.isConfigured" type="warning" size="tiny" round>
              未配置
            </NTag>
          </NSpace>
          <NSpace :size="8">
            <NTooltip placement="bottom">
              <template #trigger>
                <NButton size="tiny" quaternary circle @click="showLLMConfig = true">
                  <template #icon><NIcon size="14"><SettingsOutline /></NIcon></template>
                </NButton>
              </template>
              LLM 配置
            </NTooltip>
            <NButton size="tiny" quaternary @click="emit('update:show', false)">
              ✕
            </NButton>
          </NSpace>
        </NSpace>
      </template>

      <!-- 功能标签页 -->
      <NSpace vertical :size="16">
        <!-- Tab 切换 -->
        <NSpace :size="8">
          <NButton
            :type="activeTab === 'generate' ? 'primary' : 'default'"
            size="small"
            @click="activeTab = 'generate'"
          >
            <template #icon><NIcon size="14"><SparklesOutline /></NIcon></template>
            生成 SQL
          </NButton>
          <NButton
            :type="activeTab === 'optimize' ? 'primary' : 'default'"
            size="small"
            @click="activeTab = 'optimize'"
          >
            <template #icon><NIcon size="14"><BulbOutline /></NIcon></template>
            优化建议
          </NButton>
          <NButton
            :type="activeTab === 'diagnose' ? 'primary' : 'default'"
            size="small"
            @click="activeTab = 'diagnose'"
          >
            <template #icon><NIcon size="14"><WarningOutline /></NIcon></template>
            错误诊断
          </NButton>
        </NSpace>

        <!-- 生成 SQL -->
        <template v-if="activeTab === 'generate'">
          <NSpace vertical :size="12">
            <NInput
              v-model:value="naturalLanguageInput"
              type="textarea"
              placeholder="用自然语言描述您想要的查询，例如：'查询最近7天的订单数据' 或 '获取所有管理员用户'"
              :autosize="{ minRows: 3, maxRows: 6 }"
            />
            <NSpace :size="8">
              <NButton
                type="primary"
                size="small"
                :loading="isProcessing"
                @click="generateSQL"
              >
                <template #icon><NIcon size="14"><SparklesOutline /></NIcon></template>
                生成 SQL
              </NButton>
            </NSpace>

            <!-- 生成的 SQL -->
            <NCard v-if="currentSQL" size="small" :bordered="true">
              <template #header>
                <NText strong style="font-size: 12px">生成的 SQL</NText>
              </template>
              <pre class="sql-display">{{ currentSQL }}</pre>
              <template #footer>
                <NSpace justify="end">
                  <NButton size="tiny" @click="applySQL">
                    <template #icon><NIcon size="12"><ChevronForwardOutline /></NIcon></template>
                    应用到编辑器
                  </NButton>
                </NSpace>
              </template>
            </NCard>
          </NSpace>
        </template>

        <!-- 优化建议 -->
        <template v-if="activeTab === 'optimize'">
          <NSpace vertical :size="12">
            <NInput
              v-model:value="currentSQL"
              type="textarea"
              placeholder="输入或粘贴需要优化的 SQL..."
              :autosize="{ minRows: 4, maxRows: 8 }"
            />
            <NButton
              type="primary"
              size="small"
              :loading="isProcessing"
              @click="optimizeSQL"
            >
              <template #icon><NIcon size="14"><BulbOutline /></NIcon></template>
              分析优化
            </NButton>

            <!-- 优化建议列表 -->
            <template v-if="suggestions.length > 0">
              <NCollapse :default-expanded-names="suggestions.map(s => s.id)">
                <NCollapseItem
                  v-for="suggestion in suggestions"
                  :key="suggestion.id"
                  :name="suggestion.id"
                >
                  <template #header>
                    <NSpace align="center" :size="8">
                      <NIcon :component="getSuggestionTypeIcon(suggestion.type)" :color="`var(--zx-${suggestion.type})`" />
                      <NText strong style="font-size: 13px">{{ suggestion.title }}</NText>
                      <NTag :type="getSuggestionTypeColor(suggestion.type)" size="tiny">
                        {{ suggestion.impact }}
                      </NTag>
                    </NSpace>
                  </template>
                  <NText style="font-size: 12px">{{ suggestion.description }}</NText>
                </NCollapseItem>
              </NCollapse>
            </template>
          </NSpace>
        </template>

        <!-- 错误诊断 -->
        <template v-if="activeTab === 'diagnose'">
          <NSpace vertical :size="12">
            <NInput
              v-model:value="currentSQL"
              type="textarea"
              placeholder="粘贴有错误的 SQL..."
              :autosize="{ minRows: 4, maxRows: 8 }"
            />
            <NInput
              v-model:value="diagnosisErrorInput"
              type="textarea"
              placeholder="粘贴错误信息..."
              :autosize="{ minRows: 2, maxRows: 4 }"
            />
            <NButton
              type="primary"
              size="small"
              :loading="isProcessing"
              @click="diagnoseSQLError(diagnosisErrorInput)"
            >
              <template #icon><NIcon size="14"><WarningOutline /></NIcon></template>
              诊断错误
            </NButton>

            <!-- 诊断结果 -->
            <template v-if="diagnosis">
              <NAlert type="info" :bordered="false">
                <template #header>
                  <NText strong style="font-size: 12px">分析结果</NText>
                </template>
                {{ diagnosis.explanation }}
              </NAlert>

              <NCard size="small" title="建议修复方案">
                <ul class="suggestions-list">
                  <li v-for="(suggestion, index) in diagnosis.suggestions" :key="index">
                    {{ suggestion }}
                  </li>
                </ul>
              </NCard>

              <NButton v-if="diagnosis.fixedSQL" size="small" @click="currentSQL = diagnosis.fixedSQL">
                应用修复后的 SQL
              </NButton>
            </template>
          </NSpace>
        </template>
      </NSpace>
    </NCard>

    <!-- LLM 配置面板 -->
    <LLMConfigPanel v-model:show="showLLMConfig" />
  </div>
</template>

<style scoped>
.ai-assistant-container :deep(.n-card) {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--zx-border);
}

.sql-display {
  background: rgba(0, 0, 0, 0.3);
  padding: 12px;
  border-radius: 4px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: var(--zx-text-1);
  white-space: pre-wrap;
  word-break: break-all;
  overflow-x: auto;
}

.suggestions-list {
  margin: 0;
  padding-left: 20px;
  font-size: 12px;
}

.suggestions-list li {
  margin-bottom: 4px;
  color: var(--zx-text-2);
}
</style>
