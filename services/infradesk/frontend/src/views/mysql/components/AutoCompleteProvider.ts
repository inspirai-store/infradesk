/**
 * SQL 自动补全提供者
 * 为 Monaco Editor 提供 SQL 智能补全功能
 */

import * as monaco from 'monaco-editor'
import type { Table, SQLFunction } from '@/types/query'

// 补全上下文分析结果
interface CompletionContextAnalysis {
  // 当前单词
  currentWord: string
  // 当前单词起始位置
  wordStart: number
  // 是否在表名后面（如 users.）
  isAfterTableDot: boolean
  // 提取的表名
  tableName?: string
  // 是否在 FROM 子句中
  isInFromClause: boolean
  // 是否在函数调用中
  isInFunctionCall: boolean
  // 行内容
  lineUntilPos: string
}

export class SQLCompletionProvider implements monaco.languages.CompletionItemProvider {
  private schemaCache: Map<string, Table[]> = new Map()
  private functions: SQLFunction[] = []
  private keywords: string[] = []

  constructor() {
    // 动态导入函数和关键字
    this.loadBuiltins()
  }

  private async loadBuiltins() {
    const { BUILTIN_FUNCTIONS, SQL_KEYWORDS } = await import('@/types/query')
    this.functions = BUILTIN_FUNCTIONS
    this.keywords = SQL_KEYWORDS
  }

  /**
   * 更新 Schema 缓存
   */
  updateSchema(database: string, tables: Table[]): void {
    this.schemaCache.set(database, tables)
  }

  /**
   * 清除 Schema 缓存
   */
  clearSchema(database?: string): void {
    if (database) {
      this.schemaCache.delete(database)
    } else {
      this.schemaCache.clear()
    }
  }

  /**
   * Monaco Editor 自动补全接口实现
   */
  provideCompletionItems(
    model: monaco.editor.ITextModel,
    position: monaco.Position
  ): monaco.languages.CompletionList {
    const line = model.getLineContent(position.lineNumber)
    const lineUntilPos = line.substring(0, position.column - 1)

    // 分析上下文
    const context = this.analyzeContext(lineUntilPos)

    // 计算替换范围
    const wordStart = position.column - 1 - context.currentWord.length
    const range: monaco.IRange = {
      startLineNumber: position.lineNumber,
      startColumn: wordStart + 1,
      endLineNumber: position.lineNumber,
      endColumn: position.column
    }

    // 根据上下文返回不同的补全建议
    if (context.isAfterTableDot && context.tableName) {
      return this.provideColumnCompletions(context.tableName, range)
    } else if (context.isInFromClause || this.looksLikeTableReference(lineUntilPos)) {
      return this.provideTableCompletions(range)
    } else if (context.isInFunctionCall) {
      return this.provideFunctionCompletions(range)
    } else {
      return this.provideKeywordAndTopLevelCompletions(context.currentWord, range)
    }
  }

  /**
   * 分析当前上下文
   */
  private analyzeContext(line: string): CompletionContextAnalysis {
    const trimmed = line.trim()
    const currentWordMatch = trimmed.match(/(\w+)$/)
    const currentWord = currentWordMatch ? currentWordMatch[1] : ''
    const wordStart = currentWordMatch ? line.lastIndexOf(currentWord) : 0

    // 检查是否在表名后面（如 users.）
    const tableDotMatch = trimmed.match(/(\w+)\.$/)
    const isAfterTableDot = !!tableDotMatch
    const tableName = tableDotMatch ? tableDotMatch[1] : undefined

    // 检查是否在 FROM 子句中
    const isInFromClause = /\bFROM\s+/i.test(trimmed.toUpperCase())

    // 检查是否在函数调用中（括号未闭合）
    const openParens = (trimmed.match(/\(/g) || []).length
    const closeParens = (trimmed.match(/\)/g) || []).length
    const isInFunctionCall = openParens > closeParens

    return {
      currentWord,
      wordStart,
      isAfterTableDot,
      tableName,
      isInFromClause,
      isInFunctionCall,
      lineUntilPos: trimmed
    }
  }

  /**
   * 判断是否看起来像是在引用表
   */
  private looksLikeTableReference(line: string): boolean {
    const upper = line.toUpperCase()
    return /\b(FROM|JOIN|UPDATE|INSERT\s+INTO)\s+\w*$/i.test(upper)
  }

  /**
   * 提供列名补全
   */
  private provideColumnCompletions(tableName: string, range: monaco.IRange): monaco.languages.CompletionList {
    const suggestions: monaco.languages.CompletionItem[] = []

    // 从缓存中查找表
    for (const [, tables] of this.schemaCache) {
      const table = tables.find(t => t.name === tableName)
      if (table) {
        for (const column of table.columns) {
          suggestions.push({
            label: column.name,
            kind: monaco.languages.CompletionItemKind.Field,
            detail: `${column.name}${column.nullable ? '' : ' NOT NULL'}${column.key ? ` ${column.key}` : ''}`,
            documentation: column.comment || `${column.name}: ${column.type}`,
            insertText: column.name,
            sortText: `0_${column.name}`,
            range
          })
        }
        break
      }
    }

    return {
      suggestions
    }
  }

  /**
   * 提供表名补全
   */
  private provideTableCompletions(range: monaco.IRange): monaco.languages.CompletionList {
    const suggestions: monaco.languages.CompletionItem[] = []

    for (const [, tables] of this.schemaCache) {
      for (const table of tables) {
        suggestions.push({
          label: table.name,
          kind: monaco.languages.CompletionItemKind.Class,
          detail: `Table (${table.engine})`,
          documentation: table.comment || `Table: ${table.name}`,
          insertText: table.name,
          sortText: `0_${table.name}`,
          range
        })
      }
    }

    return {
      suggestions
    }
  }

  /**
   * 提供关键字和顶层补全
   */
  private provideKeywordAndTopLevelCompletions(currentWord: string, range: monaco.IRange): monaco.languages.CompletionList {
    const suggestions: monaco.languages.CompletionItem[] = []

    // 添加关键字
    for (const keyword of this.keywords) {
      if (keyword.startsWith(currentWord.toUpperCase())) {
        suggestions.push({
          label: keyword,
          kind: monaco.languages.CompletionItemKind.Keyword,
          insertText: keyword,
          sortText: `1_${keyword}`,
          range
        })
      }
    }

    // 添加函数
    for (const func of this.functions) {
      if (func.name.startsWith(currentWord.toUpperCase())) {
        suggestions.push({
          label: func.name,
          kind: monaco.languages.CompletionItemKind.Function,
          detail: func.signature,
          documentation: `${func.description}\n\nExample:\n${func.example}`,
          insertText: func.name,
          sortText: `2_${func.name}`,
          range
        })
      }
    }

    // 添加表名
    const tableSuggestions = this.provideTableCompletions(range)
    suggestions.push(...tableSuggestions.suggestions)

    return {
      suggestions: suggestions.sort((a, b) => a.sortText!.localeCompare(b.sortText!))
    }
  }

  /**
   * 提供函数补全
   */
  private provideFunctionCompletions(range: monaco.IRange): monaco.languages.CompletionList {
    const suggestions: monaco.languages.CompletionItem[] = []

    for (const func of this.functions) {
      suggestions.push({
        label: func.name,
        kind: monaco.languages.CompletionItemKind.Function,
        detail: func.signature,
        documentation: `${func.description}\n\nExample:\n${func.example}`,
        insertText: func.name,
        sortText: func.name,
        range
      })
    }

    return {
      suggestions
    }
  }
}

// 创建单例实例
let providerInstance: SQLCompletionProvider | null = null

export function getSQLCompletionProvider(): SQLCompletionProvider {
  if (!providerInstance) {
    providerInstance = new SQLCompletionProvider()
  }
  return providerInstance
}
