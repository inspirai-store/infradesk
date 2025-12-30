<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  NSpace,
  NCard,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NInputNumber,
  NButton,
  NText,
  NAlert,
  useMessage,
  NCollapse,
  NCollapseItem,
  NIcon,
} from 'naive-ui'
import {
  KeyOutline,
  CheckmarkCircleOutline,
  InformationCircleOutline,
  RefreshOutline,
} from '@vicons/ionicons5'
import { useLLMStore, type LLMProvider } from '@/stores/llm'

interface Props {
  show: boolean
}

interface Emits {
  (e: 'update:show', value: boolean): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()

const llmStore = useLLMStore()
const message = useMessage()

// 表单数据
const formData = ref({
  provider: llmStore.config.provider,
  apiKey: llmStore.config.apiKey,
  baseURL: llmStore.config.baseURL || '',
  model: llmStore.config.model,
  maxTokens: llmStore.config.maxTokens || 2000,
  temperature: llmStore.config.temperature || 0.7
})

const isTesting = ref(false)
const isFetchingModels = ref(false)

// 提供商选项
const providerOptions: Array<{ label: string; value: LLMProvider }> = [
  { label: 'OpenAI', value: 'openai' },
  { label: 'Anthropic (Claude)', value: 'anthropic' },
  { label: '自定义 (OpenAI 兼容)', value: 'custom' }
]

// 模型选项
const modelOptions = ref<Array<{ label: string; value: string }>>([])

// 根据提供商更新模型选项
function updateModelOptions() {
  switch (formData.value.provider) {
    case 'openai':
      modelOptions.value = [
        { label: 'GPT-4o (推荐)', value: 'gpt-4o' },
        { label: 'GPT-4o-mini (快速)', value: 'gpt-4o-mini' },
        { label: 'GPT-4 Turbo', value: 'gpt-4-turbo' },
        { label: 'GPT-3.5 Turbo', value: 'gpt-3.5-turbo' }
      ]
      if (!formData.value.model || formData.value.model === 'custom-model') {
        formData.value.model = 'gpt-4o-mini'
      }
      break
    case 'anthropic':
      modelOptions.value = [
        { label: 'Claude 3.5 Sonnet (推荐)', value: 'claude-3-5-sonnet-20241022' },
        { label: 'Claude 3 Opus', value: 'claude-3-opus-20240229' },
        { label: 'Claude 3 Sonnet', value: 'claude-3-sonnet-20240229' },
        { label: 'Claude 3 Haiku', value: 'claude-3-haiku-20240307' }
      ]
      if (!formData.value.model || formData.value.model === 'custom-model') {
        formData.value.model = 'claude-3-5-sonnet-20241022'
      }
      break
    case 'custom':
      modelOptions.value = [
        { label: '自定义模型', value: 'custom-model' }
      ]
      break
  }
}

// 从 API 获取模型列表
async function fetchModels() {
  if (!formData.value.apiKey.trim()) {
    message.warning('请先输入 API Key')
    return
  }

  // 临时保存当前配置用于获取模型
  llmStore.saveConfig({
    provider: formData.value.provider,
    apiKey: formData.value.apiKey,
    baseURL: formData.value.baseURL || undefined,
    model: formData.value.model,
    maxTokens: formData.value.maxTokens,
    temperature: formData.value.temperature
  })

  isFetchingModels.value = true
  try {
    const { getLLMService } = await import('@/services/llm')
    const service = getLLMService()
    const models = await service.fetchModels()

    if (models.length === 0) {
      message.warning('未获取到模型列表')
      return
    }

    // 转换为选项格式，过滤掉非聊天模型
    modelOptions.value = models
      .filter(m => {
        const id = m.id.toLowerCase()
        // 只显示聊天相关的模型
        return id.includes('gpt') || id.includes('chat') || id.includes('claude')
      })
      .map(m => ({
        label: m.name || m.id,
        value: m.id
      }))

    message.success(`已加载 ${modelOptions.value.length} 个模型`)
  } catch (e) {
    message.error(`获取模型列表失败: ${(e as Error).message}`)
  } finally {
    isFetchingModels.value = false
  }
}

// 提供商变更时更新默认值
function onProviderChange() {
  updateModelOptions()
  // 清空 baseURL，使用默认
  formData.value.baseURL = ''
}

// 保存配置
function saveConfig() {
  if (!formData.value.apiKey.trim()) {
    message.warning('请输入 API Key')
    return
  }

  llmStore.saveConfig({
    provider: formData.value.provider,
    apiKey: formData.value.apiKey,
    baseURL: formData.value.baseURL || undefined,
    model: formData.value.model,
    maxTokens: formData.value.maxTokens,
    temperature: formData.value.temperature
  })

  message.success('配置已保存')
  emit('update:show', false)
}

// 清除配置
function clearConfig() {
  llmStore.clearConfig()
  formData.value = {
    provider: 'openai',
    apiKey: '',
    baseURL: '',
    model: 'gpt-4o-mini',
    maxTokens: 2000,
    temperature: 0.7
  }
  message.success('配置已清除')
}

// 测试连接
async function testConnection() {
  if (!formData.value.apiKey.trim()) {
    message.warning('请先输入 API Key')
    return
  }

  isTesting.value = true
  try {
    // 临时保存配置用于测试
    llmStore.saveConfig({
      provider: formData.value.provider,
      apiKey: formData.value.apiKey,
      baseURL: formData.value.baseURL || undefined,
      model: formData.value.model,
      maxTokens: formData.value.maxTokens,
      temperature: formData.value.temperature
    })

    const { getLLMService } = await import('@/services/llm')
    const service = getLLMService()

    // 发送一个简单的测试请求
    await (service as any).callLLM(
      '你是一个测试助手。回复 "OK" 即可。',
      '测试'
    )

    message.success('连接测试成功！')
  } catch (e) {
    message.error(`连接测试失败: ${(e as Error).message}`)
  } finally {
    isTesting.value = false
  }
}

// 打开外部链接
function openExternalLink(url: string) {
  ;(window as any).open(url, '_blank')
}

onMounted(() => {
  updateModelOptions()
})

// 获取帮助文本
function getHelpText() {
  switch (formData.value.provider) {
    case 'openai':
      return {
        apiKeyUrl: 'https://platform.openai.com/api-keys',
        docsUrl: 'https://platform.openai.com/docs/quickstart'
      }
    case 'anthropic':
      return {
        apiKeyUrl: 'https://console.anthropic.com/settings/keys',
        docsUrl: 'https://docs.anthropic.com/claude/reference/getting-started-with-the-api'
      }
    default:
      return {
        apiKeyUrl: '#',
        docsUrl: '#'
      }
  }
}
</script>

<template>
  <div class="llm-config-container">
    <NCard
      v-if="show"
      class="glass-card"
      style="max-width: 600px"
    >
      <template #header>
        <NSpace align="center" justify="space-between">
          <NSpace align="center">
            <NIcon size="18" color="var(--zx-accent-cyan)">
              <KeyOutline />
            </NIcon>
            <span class="title-font neon-text">LLM API 配置</span>
          </NSpace>
          <NButton size="tiny" quaternary @click="emit('update:show', false)">
            ✕
          </NButton>
        </NSpace>
      </template>

      <NSpace vertical :size="16">
        <!-- 配置说明 -->
        <NAlert type="info" :bordered="false">
          <template #header>
            <NSpace align="center">
              <NIcon size="16" :component="InformationCircleOutline" />
              <span>为什么需要配置？</span>
            </NSpace>
          </template>
          <NText style="font-size: 12px">
            AI 助手使用大语言模型（LLM）来生成 SQL、提供优化建议和诊断错误。
            您需要提供自己的 API Key，我们不会存储或传输您的密钥到任何第三方服务器。
          </NText>
        </NAlert>

        <!-- 配置表单 -->
        <NForm label-placement="top" :show-feedback="false">
          <!-- 提供商选择 -->
          <NFormItem label="LLM 提供商">
            <NSelect
              v-model:value="formData.provider"
              :options="providerOptions"
              @update:value="onProviderChange"
            />
          </NFormItem>

          <!-- API Key -->
          <NFormItem label="API Key">
            <NInput
              v-model:value="formData.apiKey"
              type="password"
              show-password-on="click"
              placeholder="sk-..."
            />
            <template #feedback>
              <NSpace :size="8" style="margin-top: 4px">
                <NText depth="3" style="font-size: 11px">
                  获取 API Key:
                </NText>
                <NText
                  depth="2"
                  style="font-size: 11px; cursor: pointer; text-decoration: underline"
                  @click="openExternalLink(getHelpText().apiKeyUrl)"
                >
                  {{ providerOptions.find(p => p.value === formData.provider)?.label }}
                </NText>
              </NSpace>
            </template>
          </NFormItem>

          <!-- 自定义 Base URL（可选） -->
          <NFormItem v-if="formData.provider === 'custom'" label="API 地址">
            <NInput
              v-model:value="formData.baseURL"
              placeholder="https://api.openai.com/v1"
            />
            <template #feedback>
              <NText depth="3" style="font-size: 11px">
                留空使用默认地址。支持 OpenAI 兼容的 API。
              </NText>
            </template>
          </NFormItem>

          <!-- 模型选择 -->
          <NFormItem label="模型">
            <NSpace vertical :size="8" style="width: 100%">
              <NSpace :size="8" style="width: 100%">
                <NSelect
                  v-model:value="formData.model"
                  :options="modelOptions"
                  placeholder="选择或输入模型名称"
                  filterable
                  tag
                  style="flex: 1"
                />
                <NButton
                  size="small"
                  :loading="isFetchingModels"
                  :disabled="!formData.apiKey"
                  @click="fetchModels"
                >
                  <template #icon>
                    <NIcon :component="RefreshOutline" />
                  </template>
                  获取模型
                </NButton>
              </NSpace>
              <NText depth="3" style="font-size: 11px">
                点击"获取模型"从 API 加载可用模型列表，或手动输入模型名称
              </NText>
            </NSpace>
          </NFormItem>

          <!-- 高级设置 -->
          <NCollapse>
            <NCollapseItem title="高级设置" name="advanced">
              <NSpace vertical :size="12">
                <NFormItem label="最大 Tokens">
                  <NInputNumber
                    v-model:value="formData.maxTokens"
                    :min="100"
                    :max="8000"
                    :step="100"
                    style="width: 100%"
                  />
                  <template #feedback>
                    <NText depth="3" style="font-size: 11px">
                      生成响应的最大长度。越大越智能但越慢。
                    </NText>
                  </template>
                </NFormItem>

                <NFormItem label="Temperature">
                  <NInputNumber
                    v-model:value="formData.temperature"
                    :min="0"
                    :max="2"
                    :step="0.1"
                    :precision="1"
                    style="width: 100%"
                  />
                  <template #feedback>
                    <NText depth="3" style="font-size: 11px">
                      控制响应的随机性。0 更确定性，1 更创造性。
                    </NText>
                  </template>
                </NFormItem>
              </NSpace>
            </NCollapseItem>
          </NCollapse>
        </NForm>

        <!-- 操作按钮 -->
        <NSpace justify="space-between">
          <NSpace>
            <NButton
              size="small"
              :loading="isTesting"
              @click="testConnection"
            >
              测试连接
            </NButton>
            <NButton
              v-if="llmStore.isConfigured"
              size="small"
              type="error"
              ghost
              @click="clearConfig"
            >
              清除配置
            </NButton>
          </NSpace>
          <NButton
            type="primary"
            size="small"
            @click="saveConfig"
          >
            <template #icon>
              <NIcon size="14"><CheckmarkCircleOutline /></NIcon>
            </template>
            保存配置
          </NButton>
        </NSpace>

        <!-- 配置状态指示 -->
        <NCard v-if="llmStore.isConfigured" size="small" :bordered="true">
          <NSpace align="center" :size="8">
            <NIcon size="16" color="var(--zx-success)">
              <CheckmarkCircleOutline />
            </NIcon>
            <NText style="font-size: 12px">
              已配置: {{ providerOptions.find(p => p.value === llmStore.config.provider)?.label }} - {{ llmStore.config.model }}
            </NText>
          </NSpace>
        </NCard>
      </NSpace>
    </NCard>
  </div>
</template>

<style scoped>
.llm-config-container :deep(.n-card) {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--zx-border);
}

.llm-config-container :deep(.n-form-item-label) {
  font-size: 12px;
  font-weight: 500;
}

.llm-config-container :deep(.n-collapse-item) {
  font-size: 12px;
}
</style>
