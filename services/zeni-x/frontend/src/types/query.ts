/**
 * 查询相关类型定义
 */

// SQL 函数定义
export interface SQLFunction {
  name: string
  signature: string
  description: string
  example: string
  returnType: string
}

// 列信息
export interface Column {
  name: string
  type: string
  nullable: boolean
  key: string
  default: string | null
  extra: string
  comment: string
}

// 表信息
export interface Table {
  name: string
  columns: Column[]
  engine: string
  rowCount: number
  comment: string
}

// 数据库 Schema
export interface DatabaseSchema {
  database: string
  tables: Table[]
  cachedAt: Date
}

// 自动补全上下文
export interface CompletionContext {
  database: string
  tables: Table[]
  columnsMap: Map<string, Column[]>
  functions: SQLFunction[]
  keywords: string[]
}

// MySQL 内置函数列表
export const BUILTIN_FUNCTIONS: SQLFunction[] = [
  {
    name: 'COUNT',
    signature: 'COUNT(expr)',
    description: '返回匹配的行数',
    example: 'SELECT COUNT(*) FROM users;',
    returnType: 'number'
  },
  {
    name: 'SUM',
    signature: 'SUM(expr)',
    description: '返回表达式的总和',
    example: 'SELECT SUM(amount) FROM orders;',
    returnType: 'number'
  },
  {
    name: 'AVG',
    signature: 'AVG(expr)',
    description: '返回表达式的平均值',
    example: 'SELECT AVG(price) FROM products;',
    returnType: 'number'
  },
  {
    name: 'MIN',
    signature: 'MIN(expr)',
    description: '返回表达式的最小值',
    example: 'SELECT MIN(age) FROM users;',
    returnType: 'mixed'
  },
  {
    name: 'MAX',
    signature: 'MAX(expr)',
    description: '返回表达式的最大值',
    example: 'SELECT MAX(score) FROM results;',
    returnType: 'mixed'
  },
  {
    name: 'CONCAT',
    signature: 'CONCAT(str1, str2, ...)',
    description: '连接字符串',
    example: "SELECT CONCAT(first_name, ' ', last_name) AS full_name FROM users;",
    returnType: 'string'
  },
  {
    name: 'SUBSTRING',
    signature: 'SUBSTRING(str, start, length)',
    description: '提取子字符串',
    example: "SELECT SUBSTRING(name, 1, 3) FROM users;",
    returnType: 'string'
  },
  {
    name: 'LENGTH',
    signature: 'LENGTH(str)',
    description: '返回字符串长度',
    example: 'SELECT LENGTH(name) FROM users;',
    returnType: 'number'
  },
  {
    name: 'TRIM',
    signature: 'TRIM(str)',
    description: '去除字符串首尾空格',
    example: "SELECT TRIM('  hello  ');",
    returnType: 'string'
  },
  {
    name: 'UPPER',
    signature: 'UPPER(str)',
    description: '转换为大写',
    example: "SELECT UPPER(name) FROM users;",
    returnType: 'string'
  },
  {
    name: 'LOWER',
    signature: 'LOWER(str)',
    description: '转换为小写',
    example: "SELECT LOWER(email) FROM users;",
    returnType: 'string'
  },
  {
    name: 'NOW',
    signature: 'NOW()',
    description: '返回当前日期和时间',
    example: 'SELECT NOW();',
    returnType: 'datetime'
  },
  {
    name: 'DATE',
    signature: 'DATE(expr)',
    description: '提取日期部分',
    example: 'SELECT DATE(created_at) FROM orders;',
    returnType: 'date'
  },
  {
    name: 'YEAR',
    signature: 'YEAR(date)',
    description: '返回年份',
    example: 'SELECT YEAR(birthday) FROM users;',
    returnType: 'number'
  },
  {
    name: 'MONTH',
    signature: 'MONTH(date)',
    description: '返回月份',
    example: 'SELECT MONTH(created_at) FROM orders;',
    returnType: 'number'
  },
  {
    name: 'DAY',
    signature: 'DAY(date)',
    description: '返回日期',
    example: 'SELECT DAY(due_date) FROM tasks;',
    returnType: 'number'
  },
  {
    name: 'COALESCE',
    signature: 'COALESCE(expr1, expr2, ...)',
    description: '返回第一个非空值',
    example: 'SELECT COALESCE(nickname, name) FROM users;',
    returnType: 'mixed'
  },
  {
    name: 'IFNULL',
    signature: 'IFNULL(expr, alt)',
    description: '如果表达式为NULL则返回替代值',
    example: 'SELECT IFNULL(phone, "N/A") FROM users;',
    returnType: 'mixed'
  },
  {
    name: 'CAST',
    signature: 'CAST(expr AS type)',
    description: '转换类型',
    example: 'SELECT CAST(age AS SIGNED) FROM users;',
    returnType: 'mixed'
  },
  {
    name: 'ROUND',
    signature: 'ROUND(num, decimals)',
    description: '四舍五入',
    example: 'SELECT ROUND(price, 2) FROM products;',
    returnType: 'number'
  },
  {
    name: 'FLOOR',
    signature: 'FLOOR(num)',
    description: '向下取整',
    example: 'SELECT FLOOR(rating) FROM products;',
    returnType: 'number'
  },
  {
    name: 'CEIL',
    signature: 'CEIL(num)',
    description: '向上取整',
    example: 'SELECT CEIL(ratio) FROM stats;',
    returnType: 'number'
  },
  {
    name: 'ABS',
    signature: 'ABS(num)',
    description: '返回绝对值',
    example: 'SELECT ABS(balance) FROM accounts;',
    returnType: 'number'
  },
  {
    name: 'GROUP_CONCAT',
    signature: 'GROUP_CONCAT(expr)',
    description: '连接分组中的值',
    example: 'SELECT GROUP_CONCAT(name) FROM users GROUP BY department;',
    returnType: 'string'
  }
]

// MySQL 关键字列表
export const SQL_KEYWORDS = [
  'SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE', 'CREATE', 'DROP', 'ALTER',
  'TABLE', 'DATABASE', 'INDEX', 'VIEW', 'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER',
  'ON', 'AS', 'ORDER', 'BY', 'GROUP', 'HAVING', 'LIMIT', 'OFFSET', 'UNION', 'DISTINCT',
  'AND', 'OR', 'NOT', 'IN', 'LIKE', 'BETWEEN', 'IS', 'NULL', 'TRUE', 'FALSE',
  'PRIMARY', 'KEY', 'FOREIGN', 'REFERENCES', 'UNIQUE', 'CONSTRAINT', 'DEFAULT',
  'AUTO_INCREMENT', 'ENGINE', 'CHARSET', 'COLLATE', 'IF', 'EXISTS', 'CASCADE',
  'RESTRICT', 'SET', 'SHOW', 'DESCRIBE', 'EXPLAIN', 'USE', 'GRANT', 'REVOKE',
  'INTO', 'VALUES', 'DESC', 'ASC', 'FULL', 'CROSS', 'NATURAL', 'USE'
]
