import { useLLMStore } from '@/stores/llm'

interface GenerateSQLParams {
  prompt: string
  databaseSchema?: {
    database: string
    tables: Array<{
      name: string
      columns: Array<{ name: string; type: string }>
    }>
  }
}

interface OptimizeSQLParams {
  sql: string
  databaseSchema?: {
    database: string
    tables: Array<{
      name: string
      columns: Array<{ name: string; type: string }>
    }>
  }
}

interface DiagnoseSQLErrorParams {
  sql: string
  error: string
}

interface OptimizationSuggestion {
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

export interface ModelInfo {
  id: string
  name?: string
  owned_by?: string
}

export class LLMService {
  private store = useLLMStore()

  /**
   * 获取可用的模型列表
   */
  async fetchModels(): Promise<ModelInfo[]> {
    if (!this.store.isConfigured) {
      throw new Error('LLM 未配置，请先在设置中配置 API Key')
    }

    const baseURL = this.store.getBaseURL()
    const headers = this.store.getHeaders()

    let endpoint = ''
    if (this.store.config.provider === 'anthropic') {
      // Anthropic 不提供模型列表 API，返回预定义列表
      return [
        { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet', owned_by: 'anthropic' },
        { id: 'claude-3-5-haiku-20241022', name: 'Claude 3.5 Haiku', owned_by: 'anthropic' },
        { id: 'claude-3-opus-20240229', name: 'Claude 3 Opus', owned_by: 'anthropic' },
        { id: 'claude-3-sonnet-20240229', name: 'Claude 3 Sonnet', owned_by: 'anthropic' },
        { id: 'claude-3-haiku-20240307', name: 'Claude 3 Haiku', owned_by: 'anthropic' }
      ]
    } else {
      // OpenAI-compatible: 使用 /models 端点
      endpoint = `${baseURL}/models`
    }

    const response = await fetch(endpoint, {
      method: 'GET',
      headers
    })

    if (!response.ok) {
      const errorText = await response.text()
      throw new Error(`获取模型列表失败: ${response.status} ${errorText}`)
    }

    const data = await response.json()

    // OpenAI API 返回 { data: [{ id, owned_by, ... }] }
    if (data.data && Array.isArray(data.data)) {
      return data.data.map((m: any) => ({
        id: m.id,
        name: m.id,
        owned_by: m.owned_by
      }))
    }

    return []
  }

  /**
   * 检查模型是否需要使用 max_completion_tokens
   * 新模型（GPT-4o 及以后）需要用 max_completion_tokens
   * 旧模型（GPT-4 Turbo, GPT-3.5）使用 max_tokens
   */
  private needsMaxCompletionTokens(): boolean {
    const model = this.store.config.model || ''
    // 只有旧模型使用 max_tokens，其他所有新模型都用 max_completion_tokens
    const oldModels = /^(gpt-3\.5-turbo|gpt-4-turbo|gpt-4-0314|gpt-4-0613)$/i
    return !oldModels.test(model)
  }

  private async callLLM(systemPrompt: string, userPrompt: string): Promise<string> {
    if (!this.store.isConfigured) {
      throw new Error('LLM 未配置，请先在设置中配置 API Key')
    }

    const baseURL = this.store.getBaseURL()
    const headers = this.store.getHeaders()

    let endpoint = ''
    let body: any = {}

    if (this.store.config.provider === 'anthropic') {
      endpoint = `${baseURL}/messages`
      body = {
        model: this.store.config.model || 'claude-3-5-sonnet-20241022',
        max_tokens: this.store.config.maxTokens || 2000,
        system: systemPrompt,
        messages: [{ role: 'user', content: userPrompt }]
      }
    } else {
      // OpenAI-compatible
      endpoint = `${baseURL}/chat/completions`

      // 根据模型类型决定使用 max_tokens 还是 max_completion_tokens
      const tokenParam = this.needsMaxCompletionTokens() ? 'max_completion_tokens' : 'max_tokens'

      body = {
        model: this.store.config.model || 'gpt-4o-mini',
        [tokenParam]: this.store.config.maxTokens || 2000,
        temperature: this.store.config.temperature || 0.7,
        messages: [
          { role: 'system', content: systemPrompt },
          { role: 'user', content: userPrompt }
        ]
      }
    }

    const response = await fetch(endpoint, {
      method: 'POST',
      headers,
      body: JSON.stringify(body)
    })

    if (!response.ok) {
      const errorText = await response.text()
      throw new Error(`LLM API 调用失败: ${response.status} ${errorText}`)
    }

    const data = await response.json()

    // 提取响应内容
    if (this.store.config.provider === 'anthropic') {
      return data.content?.[0]?.text || data.message?.content?.[0]?.text || ''
    } else {
      return data.choices?.[0]?.message?.content || ''
    }
  }

  /**
   * 生成 SQL 查询
   */
  async generateSQL(params: GenerateSQLParams): Promise<{ sql: string }> {
    let schemaInfo = ''
    if (params.databaseSchema?.tables?.length) {
      const tables = params.databaseSchema.tables.map(t => {
        const cols = t.columns.map(c => `  ${c.name} ${c.type}`).join('\n')
        return `表 ${t.name}:\n${cols}`
      }).join('\n\n')
      schemaInfo = `\n\n可用数据库 Schema:\n${params.databaseSchema.database}\n${tables}`
    }

    const systemPrompt = `你是一个专业的 SQL 助手，负责根据用户的自然语言描述生成 MySQL 查询语句。

规则：
1. 只输出 SQL 语句，不要有任何解释或说明
2. 使用标准 MySQL 语法
3. 对于不确定的列名使用占位符 ?
4. 对于日期时间相关查询，使用 NOW() 或 CURDATE()
5. 生成的 SQL 应该可以直接执行
6. 如果无法理解用户意图，生成一个注释说明需要用户提供更多信息`

    const userPrompt = `请将以下自然语言转换为 MySQL 查询：\n\n${params.prompt}${schemaInfo}`

    const response = await this.callLLM(systemPrompt, userPrompt)

    // 清理响应，提取 SQL
    let sql = response.trim()

    // 移除可能的 markdown 代码块标记
    sql = sql.replace(/```sql\n?/gi, '')
    sql = sql.replace(/```\n?/gi, '')

    // 提取第一个有效的 SQL 语句
    const sqlMatch = sql.match(/(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|SHOW|DESCRIBE|EXPLAIN)[\s\S]+?;/i)
    if (sqlMatch) {
      sql = sqlMatch[0]
    }

    return { sql }
  }

  /**
   * 优化 SQL 查询
   */
  async optimizeSQL(params: OptimizeSQLParams): Promise<{
    suggestions: OptimizationSuggestion[]
    optimizedSQL?: string
  }> {
    const systemPrompt = `你是一个 SQL 性能优化专家。分析用户提供的 SQL 查询，找出潜在的性能问题并提供优化建议。

返回格式必须是有效的 JSON，格式如下：
{
  "suggestions": [
    {
      "type": "optimization" | "warning" | "info",
      "title": "简短标题",
      "description": "详细说明",
      "impact": "high" | "medium" | "low"
    }
  ],
  "optimizedSQL": "优化后的 SQL（如果有）"
}

主要检查项：
1. 是否使用了 SELECT *
2. 是否缺少 WHERE 条件
3. 是否有合适的索引建议
4. JOIN 是否可以优化
5. 是否可以使用 LIMIT
6. 子查询是否可以改为 JOIN
7. 是否有 N+1 查询问题`

    const userPrompt = `请分析并优化以下 SQL 查询：\n\n${params.sql}`

    const response = await this.callLLM(systemPrompt, userPrompt)

    try {
      // 尝试解析 JSON 响应
      const cleaned = response.replace(/```json\n?/gi, '').replace(/```\n?/gi, '')
      const result = JSON.parse(cleaned)

      // 验证返回的数据结构
      if (!result.suggestions || !Array.isArray(result.suggestions)) {
        throw new Error('无效的响应格式')
      }

      return {
        suggestions: result.suggestions.map((s: any) => ({
          type: s.type || 'info',
          title: s.title || '优化建议',
          description: s.description || '',
          impact: s.impact || 'medium'
        })),
        optimizedSQL: result.optimizedSQL
      }
    } catch (e) {
      console.error('Failed to parse LLM response:', e)
      // 返回默认响应
      return {
        suggestions: [{
          type: 'info',
          title: '无法解析优化建议',
          description: 'LLM 返回的格式不正确，请检查 API 配置或重试',
          impact: 'low'
        }]
      }
    }
  }

  /**
   * 诊断 SQL 错误
   */
  async diagnoseSQLError(params: DiagnoseSQLErrorParams): Promise<Diagnosis> {
    const systemPrompt = `你是一个 MySQL 错误诊断专家。分析 SQL 错误信息，提供清晰的解释和修复建议。

返回格式必须是有效的 JSON，格式如下：
{
  "explanation": "错误原因的清晰解释",
  "suggestions": ["修复建议1", "修复建议2", ...],
  "fixedSQL": "修复后的 SQL（如果可以自动修复）"
}

重点关注：
1. 语法错误
2. 表或列不存在
3. 数据类型不匹配
4. 权限问题
5. 约束冲突`

    const userPrompt = `SQL 语句：\n${params.sql}\n\n错误信息：\n${params.error}\n\n请分析这个错误并提供修复建议。`

    const response = await this.callLLM(systemPrompt, userPrompt)

    try {
      const cleaned = response.replace(/```json\n?/gi, '').replace(/```\n?/gi, '')
      const result = JSON.parse(cleaned)

      return {
        error: params.error,
        explanation: result.explanation || '无法确定具体错误原因',
        suggestions: result.suggestions || [],
        fixedSQL: result.fixedSQL
      }
    } catch (e) {
      console.error('Failed to parse LLM response:', e)
      return {
        error: params.error,
        explanation: 'LLM 返回的格式不正确，请检查 API 配置',
        suggestions: ['请检查 SQL 语法', '确认表名和列名正确', '查看完整错误信息']
      }
    }
  }
}

// 创建单例实例
let llmServiceInstance: LLMService | null = null

export function getLLMService(): LLMService {
  if (!llmServiceInstance) {
    llmServiceInstance = new LLMService()
  }
  return llmServiceInstance
}
