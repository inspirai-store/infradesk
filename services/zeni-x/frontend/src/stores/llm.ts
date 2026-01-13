import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getApiAdapter } from '@/api/adapter'
import type { LLMConfigResponse, CreateLLMConfigRequest, UpdateLLMConfigRequest } from '@/api/types'

export type LLMProvider = 'openai' | 'anthropic' | 'custom'

// Frontend-facing config interface (for compatibility)
export interface LLMConfig {
  id?: number
  name?: string
  provider: LLMProvider
  apiKey: string  // Only populated when actively making LLM calls
  baseURL?: string
  model: string
  maxTokens?: number
  temperature?: number
  isDefault?: boolean
}

export const useLLMStore = defineStore('llm', () => {
  // All available LLM configs
  const configs = ref<LLMConfigResponse[]>([])

  // Currently selected/default config ID
  const selectedConfigId = ref<number | null>(null)

  // Loading state
  const loading = ref(false)

  // Error state
  const error = ref<string | null>(null)

  // Cached API key for the current config (not persisted)
  const cachedApiKey = ref<string | null>(null)

  // Get the currently selected config
  const currentConfig = computed<LLMConfigResponse | null>(() => {
    if (!selectedConfigId.value) {
      // Return default config if no specific selection
      return configs.value.find(c => c.is_default) || configs.value[0] || null
    }
    return configs.value.find(c => c.id === selectedConfigId.value) || null
  })

  // Legacy compatibility: config ref that maps to currentConfig
  const config = computed<LLMConfig>(() => {
    const current = currentConfig.value
    if (!current) {
      return {
        provider: 'openai' as LLMProvider,
        apiKey: '',
        baseURL: '',
        model: 'gpt-4o-mini',
        maxTokens: 2000,
        temperature: 0.7
      }
    }
    return {
      id: current.id,
      name: current.name,
      provider: current.provider as LLMProvider,
      apiKey: cachedApiKey.value || '',
      baseURL: current.base_url,
      model: current.model,
      maxTokens: current.max_tokens,
      temperature: current.temperature,
      isDefault: current.is_default
    }
  })

  // Check if a config is available and has an API key
  const isConfigured = computed(() => {
    const current = currentConfig.value
    return current !== null && current.has_api_key
  })

  // Load all configs from backend
  async function loadConfigs(): Promise<void> {
    const api = getApiAdapter()
    loading.value = true
    error.value = null
    try {
      configs.value = await api.llmConfig.getAll()
      // If we have a default, select it
      const defaultConfig = configs.value.find(c => c.is_default)
      if (defaultConfig && !selectedConfigId.value) {
        selectedConfigId.value = defaultConfig.id
      }
    } catch (e) {
      console.error('Failed to load LLM configs:', e)
      error.value = e instanceof Error ? e.message : 'Failed to load configs'
    } finally {
      loading.value = false
    }
  }

  // Create a new config
  async function createConfig(data: CreateLLMConfigRequest): Promise<LLMConfigResponse | null> {
    const api = getApiAdapter()
    loading.value = true
    error.value = null
    try {
      const newConfig = await api.llmConfig.create(data)
      configs.value.push(newConfig)
      // If this is the first config or marked as default, select it
      if (newConfig.is_default || configs.value.length === 1) {
        selectedConfigId.value = newConfig.id
      }
      return newConfig
    } catch (e) {
      console.error('Failed to create LLM config:', e)
      error.value = e instanceof Error ? e.message : 'Failed to create config'
      return null
    } finally {
      loading.value = false
    }
  }

  // Update an existing config
  async function updateConfig(id: number, data: UpdateLLMConfigRequest): Promise<LLMConfigResponse | null> {
    const api = getApiAdapter()
    loading.value = true
    error.value = null
    try {
      const updated = await api.llmConfig.update(id, data)
      const index = configs.value.findIndex(c => c.id === id)
      if (index !== -1) {
        configs.value[index] = updated
      }
      // Clear cached API key if we updated it
      if (data.api_key && id === selectedConfigId.value) {
        cachedApiKey.value = null
      }
      return updated
    } catch (e) {
      console.error('Failed to update LLM config:', e)
      error.value = e instanceof Error ? e.message : 'Failed to update config'
      return null
    } finally {
      loading.value = false
    }
  }

  // Delete a config
  async function deleteConfig(id: number): Promise<boolean> {
    const api = getApiAdapter()
    loading.value = true
    error.value = null
    try {
      await api.llmConfig.delete(id)
      configs.value = configs.value.filter(c => c.id !== id)
      // If we deleted the selected config, select another
      if (selectedConfigId.value === id) {
        selectedConfigId.value = null
        cachedApiKey.value = null
        const defaultConfig = configs.value.find(c => c.is_default)
        if (defaultConfig) {
          selectedConfigId.value = defaultConfig.id
        } else if (configs.value.length > 0) {
          selectedConfigId.value = configs.value[0].id
        }
      }
      return true
    } catch (e) {
      console.error('Failed to delete LLM config:', e)
      error.value = e instanceof Error ? e.message : 'Failed to delete config'
      return false
    } finally {
      loading.value = false
    }
  }

  // Set a config as default
  async function setDefault(id: number): Promise<boolean> {
    const api = getApiAdapter()
    loading.value = true
    error.value = null
    try {
      const updated = await api.llmConfig.setDefault(id)
      // Update local state
      configs.value = configs.value.map(c => ({
        ...c,
        is_default: c.id === id
      }))
      const index = configs.value.findIndex(c => c.id === id)
      if (index !== -1) {
        configs.value[index] = updated
      }
      return true
    } catch (e) {
      console.error('Failed to set default LLM config:', e)
      error.value = e instanceof Error ? e.message : 'Failed to set default'
      return false
    } finally {
      loading.value = false
    }
  }

  // Select a config
  function selectConfig(id: number): void {
    selectedConfigId.value = id
    cachedApiKey.value = null
  }

  // Legacy compatibility: saveConfig
  async function saveConfig(newConfig: Partial<LLMConfig>): Promise<void> {
    if (currentConfig.value) {
      // Update existing config
      await updateConfig(currentConfig.value.id, {
        name: newConfig.name,
        provider: newConfig.provider,
        api_key: newConfig.apiKey,
        base_url: newConfig.baseURL,
        model: newConfig.model,
        max_tokens: newConfig.maxTokens,
        temperature: newConfig.temperature
      })
    } else {
      // Create new config
      await createConfig({
        name: newConfig.name || 'Default',
        provider: newConfig.provider || 'openai',
        api_key: newConfig.apiKey,
        base_url: newConfig.baseURL,
        model: newConfig.model || 'gpt-4o-mini',
        max_tokens: newConfig.maxTokens,
        temperature: newConfig.temperature,
        is_default: true
      })
    }
  }

  // Legacy compatibility: clearConfig
  async function clearConfig(): Promise<void> {
    if (currentConfig.value) {
      await deleteConfig(currentConfig.value.id)
    }
  }

  // Get API key for making LLM calls
  async function getApiKey(): Promise<string | null> {
    if (cachedApiKey.value) {
      return cachedApiKey.value
    }

    const current = currentConfig.value
    if (!current) {
      return null
    }

    const api = getApiAdapter()
    try {
      const apiKey = await api.llmConfig.getApiKey(current.id)
      cachedApiKey.value = apiKey
      return apiKey
    } catch (e) {
      console.error('Failed to get API key:', e)
      return null
    }
  }

  // Get request headers for LLM API calls
  async function getHeaders(): Promise<Record<string, string>> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json'
    }

    const apiKey = await getApiKey()
    if (!apiKey) {
      return headers
    }

    const provider = currentConfig.value?.provider || 'openai'
    switch (provider) {
      case 'openai':
        headers['Authorization'] = `Bearer ${apiKey}`
        break
      case 'anthropic':
        headers['x-api-key'] = apiKey
        headers['anthropic-version'] = '2023-06-01'
        break
      case 'custom':
        headers['Authorization'] = `Bearer ${apiKey}`
        break
    }

    return headers
  }

  // Get API base URL
  function getBaseURL(): string {
    const current = currentConfig.value
    if (current?.base_url) {
      return current.base_url
    }

    const provider = current?.provider || 'openai'
    switch (provider) {
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

  // Initialize: load configs on store creation
  loadConfigs()

  return {
    // State
    configs,
    selectedConfigId,
    loading,
    error,

    // Computed
    config,
    currentConfig,
    isConfigured,

    // Actions
    loadConfigs,
    createConfig,
    updateConfig,
    deleteConfig,
    setDefault,
    selectConfig,

    // Legacy compatibility
    saveConfig,
    clearConfig,
    getHeaders,
    getBaseURL,

    // New API key method
    getApiKey
  }
})
