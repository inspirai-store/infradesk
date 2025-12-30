import { defineStore } from 'pinia'
import { ref } from 'vue'

export type LLMProvider = 'openai' | 'anthropic' | 'custom'

export interface LLMConfig {
  provider: LLMProvider
  apiKey: string
  baseURL?: string
  model: string
  maxTokens?: number
  temperature?: number
}

export const useLLMStore = defineStore('llm', () => {
  const config = ref<LLMConfig>({
    provider: 'openai',
    apiKey: '',
    baseURL: '',
    model: 'gpt-4o-mini',
    maxTokens: 2000,
    temperature: 0.7
  })

  const isConfigured = ref(false)

  // 从 localStorage 加载配置
  function loadConfig() {
    const saved = localStorage.getItem('llm-config')
    if (saved) {
      try {
        const parsed = JSON.parse(saved)
        config.value = { ...config.value, ...parsed }
        isConfigured.value = !!config.value.apiKey
      } catch (e) {
        console.error('Failed to load LLM config:', e)
      }
    }
  }

  // 保存配置到 localStorage
  function saveConfig(newConfig: Partial<LLMConfig>) {
    config.value = { ...config.value, ...newConfig }
    localStorage.setItem('llm-config', JSON.stringify(config.value))
    isConfigured.value = !!config.value.apiKey
  }

  // 清除配置
  function clearConfig() {
    config.value = {
      provider: 'openai',
      apiKey: '',
      baseURL: '',
      model: 'gpt-4o-mini',
      maxTokens: 2000,
      temperature: 0.7
    }
    localStorage.removeItem('llm-config')
    isConfigured.value = false
  }

  // 获取请求头
  function getHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json'
    }

    switch (config.value.provider) {
      case 'openai':
        headers['Authorization'] = `Bearer ${config.value.apiKey}`
        break
      case 'anthropic':
        headers['x-api-key'] = config.value.apiKey
        headers['anthropic-version'] = '2023-06-01'
        break
      case 'custom':
        if (config.value.baseURL?.includes('openai')) {
          headers['Authorization'] = `Bearer ${config.value.apiKey}`
        } else {
          headers['Authorization'] = `Bearer ${config.value.apiKey}`
        }
        break
    }

    return headers
  }

  // 获取 API 端点
  function getBaseURL(): string {
    if (config.value.baseURL) {
      return config.value.baseURL
    }

    switch (config.value.provider) {
      case 'openai':
        return 'https://api.openai.com/v1'
      case 'anthropic':
        return 'https://api.anthropic.com/v1'
      case 'custom':
        return 'https://api.openai.com/v1'
      default:
        return 'https://api.openai.com/v1'
    }
  }

  // 初始化时加载配置
  loadConfig()

  return {
    config,
    isConfigured,
    saveConfig,
    clearConfig,
    getHeaders,
    getBaseURL
  }
})
