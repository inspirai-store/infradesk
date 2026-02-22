<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import * as monaco from 'monaco-editor'
import { getSQLCompletionProvider } from './AutoCompleteProvider'
import { mysqlApi } from '@/api'
import { formatSQL } from '@/services/sqlFormatter'
import type { Table } from '@/types/query'

interface Props {
  modelValue: string
  database?: string
  language?: string
  readOnly?: boolean
  minHeight?: string
  maxHeight?: string
}

interface Emits {
  (e: 'update:modelValue', value: string): void
  (e: 'execute'): void
  (e: 'ready'): void
  (e: 'format'): void
}

const props = withDefaults(defineProps<Props>(), {
  language: 'mysql',
  readOnly: false,
  minHeight: '160px',
  maxHeight: '400px'
})

const emit = defineEmits<Emits>()

const containerRef = ref<HTMLElement>()
let editor: monaco.editor.IStandaloneCodeEditor | null = null
const completionProvider = getSQLCompletionProvider()

// 加载 Schema
async function loadSchema(database: string) {
  if (!database) return

  try {
    const data = await mysqlApi.getDatabaseSchema(database) as { tables_detail?: any[] }
    const tablesDetail = data.tables_detail || []

    // 转换为 Table 类型
    const tables: Table[] = tablesDetail.map((t: any) => ({
      name: t.name,
      engine: t.engine,
      rowCount: 0,
      comment: t.comment,
      columns: t.columns || []
    }))

    // 更新补全提供者的缓存
    completionProvider.updateSchema(database, tables)
  } catch (error) {
    console.error('Failed to load schema:', error)
  }
}

// 初始化编辑器
onMounted(async () => {
  if (!containerRef.value) return

  // 注册 MySQL 语言
  monaco.languages.register({ id: 'mysql' })

  // 配置 MySQL 语法高亮
  monaco.languages.setMonarchTokensProvider('mysql', {
    keywords: [
      'SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE', 'CREATE', 'DROP', 'ALTER',
      'TABLE', 'DATABASE', 'INDEX', 'VIEW', 'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER',
      'ON', 'AS', 'ORDER', 'BY', 'GROUP', 'HAVING', 'LIMIT', 'OFFSET', 'UNION', 'DISTINCT',
      'AND', 'OR', 'NOT', 'IN', 'LIKE', 'BETWEEN', 'IS', 'NULL', 'TRUE', 'FALSE',
      'PRIMARY', 'KEY', 'FOREIGN', 'REFERENCES', 'UNIQUE', 'CONSTRAINT', 'DEFAULT',
      'AUTO_INCREMENT', 'ENGINE', 'CHARSET', 'COLLATE', 'IF', 'EXISTS', 'CASCADE',
      'RESTRICT', 'SET', 'SHOW', 'DESCRIBE', 'EXPLAIN', 'USE', 'GRANT', 'REVOKE'
    ],
    operators: [
      '=', '>', '<', '!', '~', '?', '&', '|', '^', '+', '-', '*', '/', '%'
    ],
    builtinFunctions: [
      'COUNT', 'SUM', 'AVG', 'MIN', 'MAX', 'CONCAT', 'SUBSTRING', 'LENGTH',
      'TRIM', 'UPPER', 'LOWER', 'NOW', 'DATE', 'TIME', 'YEAR', 'MONTH', 'DAY',
      'COALESCE', 'IFNULL', 'CAST', 'CONVERT', 'ROUND', 'FLOOR', 'CEIL', 'ABS'
    ],
    builtinVariables: ['TRUE', 'FALSE', 'NULL'],
    tokenizer: {
      root: [
        [/[^a-zA-Z0-9_]/, ''],
        [/@keywords/, { token: 'keyword', next: '@root' }],
        [/@builtinFunctions/, { token: 'type.identifier', next: '@root' }],
        [/@builtinVariables/, { token: 'constant', next: '@root' }],
        [/@operators/, 'operator'],
        [/\d+/, 'number'],
        [/[;(),.]/, 'delimiter'],
        [/"([^"\\]|\\.)*$/, 'string.invalid'],
        [/"/, 'string', '@string'],
        [/'([^'\\]|\\.)*$/, 'string.invalid'],
        [/'/, 'string', '@string']
      ],
      string: [
        [/[^\\']+/, 'string'],
        [/''/, 'string.escape'],
        [/'/, 'string', '@pop']
      ]
    }
  })

  // 注册自动补全提供者
  monaco.languages.registerCompletionItemProvider('mysql', completionProvider)

  // 创建编辑器实例
  editor = monaco.editor.create(containerRef.value, {
    value: props.modelValue,
    language: props.language,
    theme: 'vs-dark',
    readOnly: props.readOnly,
    minimap: { enabled: false },
    fontSize: 12,
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    wordWrap: 'on',
    automaticLayout: true,
    tabSize: 2,
    padding: { top: 12, bottom: 12 },
    suggest: {
      showKeywords: true,
      showSnippets: true
    },
    quickSuggestions: {
      other: true,
      comments: false,
      strings: false
    },
    parameterHints: {
      enabled: true
    }
  })

  // 监听内容变化
  editor.onDidChangeModelContent(() => {
    const value = editor?.getValue() || ''
    emit('update:modelValue', value)
  })

  // 快捷键：Ctrl+Enter 执行
  editor.addAction({
    id: 'execute-query',
    label: 'Execute Query',
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter],
    run: () => {
      emit('execute')
    }
  })

  // 快捷键：Ctrl+Shift+F 格式化
  editor.addAction({
    id: 'format-sql',
    label: 'Format SQL',
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyF],
    run: () => {
      formatCurrentSQL()
    }
  })

  // 加载 Schema
  if (props.database) {
    await loadSchema(props.database)
  }

  emit('ready')
})

// 格式化当前 SQL
function formatCurrentSQL() {
  if (!editor) return

  // 获取选中的 SQL 或全部内容
  const selection = editor.getSelection()
  let sqlToFormat = ''

  if (selection && !selection.isEmpty()) {
    sqlToFormat = editor.getModel()?.getValueInRange(selection) || ''
  } else {
    sqlToFormat = editor.getValue() || ''
  }

  if (!sqlToFormat.trim()) {
    return
  }

  try {
    const formatted = formatSQL(sqlToFormat, {
      keywordCase: 'upper',
      indentStyle: 'standard',
      tabWidth: 2
    })

    // 如果有选区，只替换选中的内容；否则替换全部
    if (selection && !selection.isEmpty()) {
      editor.executeEdits('format', [
        {
          range: selection,
          text: formatted
        }
      ])
    } else {
      const position = editor.getPosition()
      editor.setValue(formatted)
      if (position) {
        editor.setPosition(position)
      }
    }

    emit('format')
  } catch (error) {
    console.error('Format error:', error)
  }
}

// 监听数据库变化，重新加载 Schema
watch(() => props.database, async (newDatabase) => {
  if (newDatabase && editor) {
    await loadSchema(newDatabase)
  }
})

// 监听外部值变化
watch(() => props.modelValue, (newValue) => {
  if (editor && newValue !== editor.getValue()) {
    const position = editor.getPosition()
    editor.setValue(newValue)
    if (position) {
      editor.setPosition(position)
    }
  }
})

// 监听只读状态变化
watch(() => props.readOnly, (newValue) => {
  if (editor) {
    editor.updateOptions({ readOnly: newValue })
  }
})

onBeforeUnmount(() => {
  if (editor) {
    editor.dispose()
    editor = null
  }
})

// 暴露方法
defineExpose({
  focus: () => {
    if (editor) {
      editor.focus()
    }
  },
  getValue: () => {
    return editor?.getValue() || ''
  },
  setValue: (value: string) => {
    if (editor) {
      editor.setValue(value)
    }
  },
  getSelectedText: () => {
    if (editor) {
      const selection = editor.getSelection()
      if (selection) {
        return editor.getModel()?.getValueInRange(selection) || ''
      }
    }
    return ''
  },
  insertText: (text: string) => {
    if (editor) {
      const position = editor.getPosition()
      if (position) {
        editor.executeEdits('insertText', [
          {
            range: new monaco.Range(position.lineNumber, position.column, position.lineNumber, position.column),
            text
          }
        ])
        editor.focus()
      }
    }
  },
  format: () => {
    formatCurrentSQL()
  }
})
</script>

<template>
  <div
    ref="containerRef"
    class="monaco-sql-editor"
    :style="{
      minHeight: props.minHeight,
      maxHeight: props.maxHeight
    }"
  />
</template>

<style scoped>
.monaco-sql-editor {
  width: 100%;
  border: 1px solid var(--zx-border);
  border-radius: 6px;
  overflow: hidden;
}

.monaco-sql-editor:focus-within {
  border-color: var(--zx-accent-cyan);
  box-shadow: 0 0 0 2px rgba(0, 255, 255, 0.2);
}
</style>
