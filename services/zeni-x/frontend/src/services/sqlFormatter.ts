/**
 * SQL 格式化服务
 * 内置实现，不依赖外部库
 */

export interface FormatOptions {
  keywordCase?: 'upper' | 'lower'
  indentStyle?: 'standard' | 'compact'
  tabWidth?: number
  language?: 'mysql' | 'postgresql' | 'mssql'
}

/**
 * 格式化 SQL 语句
 */
export function formatSQL(sql: string, options: FormatOptions = {}): string {
  const {
    keywordCase = 'upper',
    indentStyle = 'standard',
    tabWidth = 2
  } = options

  // 移除多余的空白
  let formatted = sql.trim().replace(/\s+/g, ' ')

  // 在关键字前后添加换行
  formatted = insertNewlinesAroundKeywords(formatted, indentStyle)

  // 处理逗号
  formatted = formatCommas(formatted)

  // 处理括号
  formatted = formatParentheses(formatted)

  // 调整关键字大小写
  formatted = adjustKeywordCase(formatted, keywordCase)

  // 添加缩进
  formatted = applyIndentation(formatted, tabWidth)

  return formatted.trim()
}

/**
 * 在关键字周围插入换行
 */
function insertNewlinesAroundKeywords(sql: string, style: 'standard' | 'compact'): string {
  const majorKeywords = [
    'SELECT', 'FROM', 'WHERE', 'GROUP BY', 'HAVING', 'ORDER BY',
    'LIMIT', 'OFFSET', 'UNION', 'INSERT INTO', 'UPDATE', 'DELETE',
    'CREATE TABLE', 'ALTER TABLE', 'DROP TABLE', 'JOIN', 'LEFT JOIN',
    'RIGHT JOIN', 'INNER JOIN', 'OUTER JOIN', 'ON', 'AND', 'OR'
  ]

  const minorKeywords = [
    'AS', 'DISTINCT', 'PRIMARY KEY', 'FOREIGN KEY', 'REFERENCES',
    'NOT NULL', 'DEFAULT', 'AUTO_INCREMENT', 'UNIQUE', 'CONSTRAINT'
  ]

  let result = sql

  // 处理主要关键字（前后换行）
  for (const keyword of majorKeywords) {
    const regex = new RegExp(`\\b${keyword}\\b`, 'gi')
    result = result.replace(regex, `\n${keyword}\n`)
  }

  // 处理次要关键字（根据样式决定是否换行）
  if (style === 'standard') {
    for (const keyword of minorKeywords) {
      const regex = new RegExp(`\\b${keyword}\\b`, 'gi')
      result = result.replace(regex, `\n${keyword}\n`)
    }
  }

  // 清理多余的换行
  result = result.replace(/\n\s*\n\s*/g, '\n')

  return result
}

/**
 * 格式化逗号
 */
function formatCommas(sql: string): string {
  // 逗号后添加换行
  return sql.replace(/,\s*/g, ',\n')
}

/**
 * 格式化括号
 */
function formatParentheses(sql: string): string {
  // 在括号前后保持适当的空白
  return sql
    .replace(/\(\s*/g, '(\n  ')
    .replace(/\s*\)/g, '\n)')
}

/**
 * 调整关键字大小写
 */
function adjustKeywordCase(sql: string, keywordCase: 'upper' | 'lower'): string {
  const keywords = [
    'SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE', 'CREATE',
    'DROP', 'ALTER', 'TABLE', 'DATABASE', 'INDEX', 'VIEW', 'JOIN',
    'LEFT', 'RIGHT', 'INNER', 'OUTER', 'ON', 'AS', 'ORDER', 'BY',
    'GROUP', 'HAVING', 'LIMIT', 'OFFSET', 'UNION', 'DISTINCT', 'AND',
    'OR', 'NOT', 'IN', 'LIKE', 'BETWEEN', 'IS', 'NULL', 'PRIMARY',
    'KEY', 'FOREIGN', 'REFERENCES', 'UNIQUE', 'CONSTRAINT', 'DEFAULT',
    'AUTO_INCREMENT', 'ENGINE', 'CHARSET', 'COLLATE', 'IF', 'EXISTS',
    'CASCADE', 'RESTRICT', 'SET', 'SHOW', 'DESCRIBE', 'EXPLAIN',
    'USE', 'GRANT', 'REVOKE'
  ]

  let result = sql

  for (const keyword of keywords) {
    const regex = new RegExp(`\\b${keyword}\\b`, 'gi')
    result = result.replace(regex, keywordCase === 'upper' ? keyword : keyword.toLowerCase())
  }

  return result
}

/**
 * 应用缩进
 */
function applyIndentation(sql: string, tabWidth: number): string {
  const lines = sql.split('\n')
  const result: string[] = []
  let indentLevel = 0
  const indent = ' '.repeat(tabWidth)

  for (let line of lines) {
    line = line.trim()

    if (!line) {
      continue
    }

    // 减少缩进（遇到闭合括号或特定关键字）
    if (line.startsWith(')') || /^(AND|OR|ON)$/i.test(line)) {
      indentLevel = Math.max(0, indentLevel - 1)
    }

    // 应用当前缩进
    result.push(indent.repeat(indentLevel) + line)

    // 增加缩进（遇到开放括号或特定关键字）
    if (line.endsWith('(') || /^(SELECT|FROM|WHERE|GROUP BY|ORDER BY|HAVING|JOIN|LEFT JOIN|RIGHT JOIN|INNER JOIN|OUTER JOIN)$/i.test(line)) {
      indentLevel++
    }

    // 闭合括号后减少缩进
    if (line.startsWith(')')) {
      indentLevel = Math.max(0, indentLevel - 1)
    }
  }

  return result.join('\n')
}

/**
 * 简单格式化（紧凑格式）
 */
export function formatSQLCompact(sql: string): string {
  return formatSQL(sql, { indentStyle: 'compact', tabWidth: 2 })
}

/**
 * 标准格式化
 */
export function formatSQLStandard(sql: string): string {
  return formatSQL(sql, { indentStyle: 'standard', tabWidth: 2 })
}

/**
 * 验证 SQL 语法（基础检查）
 */
export function validateSQLBasic(sql: string): { valid: boolean; errors: string[] } {
  const errors: string[] = []

  // 检查括号匹配
  let openParens = 0
  for (const char of sql) {
    if (char === '(') openParens++
    if (char === ')') openParens--
    if (openParens < 0) {
      errors.push('括号不匹配：多余的右括号')
      break
    }
  }
  if (openParens > 0) {
    errors.push('括号不匹配：缺少右括号')
  }

  // 检查基本的 SQL 语句结构
  const upperSQL = sql.toUpperCase().trim()
  const hasMainKeyword = /^(SELECT|INSERT|UPDATE|DELETE|CREATE|DROP|ALTER|SHOW|DESCRIBE|EXPLAIN|USE|GRANT|REVOKE)/.test(upperSQL)
  if (!hasMainKeyword) {
    errors.push('缺少主要的 SQL 关键字（SELECT、INSERT、UPDATE 等）')
  }

  return {
    valid: errors.length === 0,
    errors
  }
}
