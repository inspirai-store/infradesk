# MySQL 表格直接编辑功能 - 实施计划

## 任务列表

- [ ] 1. 扩展 MySQL Store 添加查询限制功能
  - 在 `src/stores/mysql.ts` 中添加 `queryLimit` 状态（默认值 100）
  - 实现 `saveQueryLimit(limit: number)` 方法，验证范围 10-1000 并保存到 localStorage
  - 实现 `loadQueryLimit()` 方法，从 localStorage 加载配置
  - 实现 `applyLimit(sql: string, limit?: number)` 方法，检测 SQL 是否已有 LIMIT，没有则添加
  - 实现 `extractBaseQuery(sql: string)` 方法，移除 SQL 中的 LIMIT 和 OFFSET 子句
  - 编写单元测试验证 SQL 解析逻辑的正确性
  - _需求：1.1, 1.2, 1.4, 5.2, 5.3, 5.4_

- [ ] 2. 扩展 MySQL API 添加更新记录接口
  - 在 `src/api/mysql.ts` 中添加 `updateRecord` 接口函数
  - 接口参数：database, table, primaryKey, primaryValue, updates
  - 生成 `UPDATE table SET column1 = ?, column2 = ? WHERE primary_key = ?` 语句
  - 在 `src/backend/internal/mysql/query.go` 中实现对应的后端处理函数
  - 添加参数化查询防止 SQL 注入
  - 返回执行结果（成功/失败 + 影响行数）
  - _需求：3.3, 3.4, 3.5_

- [ ] 3. 添加获取表主键的 API 接口
  - 在 `src/api/mysql.ts` 中添加 `getTablePrimaryKey` 接口函数
  - 在 `src/backend/internal/mysql/schema.go` 中实现查询主键的方法
  - 使用 `SHOW KEYS FROM table WHERE Key_name = 'PRIMARY'` 查询主键
  - 处理复合主键场景（返回数组或拼接字符串）
  - 添加错误处理（表不存在、无主键等情况）
  - _需求：3.7_

- [ ] 4. 创建 TableDataEditor 可编辑表格组件
  - 创建 `src/views/mysql/components/TableDataEditor.vue` 组件
  - 定义 Props：database, sql, columns, data, primaryKey
  - 定义 Emits：refresh, save
  - 使用 Naive UI 的 NDataTable 组件，自定义单元格渲染
  - 实现单元格点击进入编辑模式的功能
  - 添加编辑状态追踪（editingCell, modifiedCells）
  - 添加已修改单元格的视觉标识（黄色背景）
  - 禁用主键列和不可编辑类型的编辑
  - _需求：2.1, 2.2, 2.5_

- [ ] 5. 实现单元格编辑交互
  - 在 TableDataEditor 组件中添加 NInput/NInputNumber 作为编辑器
  - 根据列的数据类型选择合适的编辑器组件
  - 实现 Enter 键保存、Esc 键取消的键盘事件处理
  - 实现失去焦点时的自动保存逻辑
  - 处理 NULL 值的编辑（提供清空选项）
  - 处理数据类型验证（如数字、日期）
  - _需求：2.3, 2.4_

- [ ] 6. 实现保存修改功能
  - 在 TableDataEditor 组件中实现 `generateUpdates()` 方法
  - 遍历 modifiedCells 生成 UpdateRecord 数组
  - 添加"保存修改"和"放弃修改"操作栏
  - 实现保存时的确认对话框，显示将要执行的 UPDATE 语句
  - 调用 mysqlApi.updateRecord() 执行更新
  - 成功后刷新数据并清空 modifiedCells
  - 失败时显示错误信息并保留修改状态
  - _需求：3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

- [ ] 7. 实现分页查询功能
  - 在 TableDataEditor 组件中添加分页状态（currentPage, pageSize, totalCount）
  - 实现分页 UI（上一页、下一页、页码显示）
  - 实现 `goToPage(page)` 方法，计算 OFFSET = (page - 1) * pageSize
  - 使用 extractBaseQuery() 获取基础 SQL，然后添加 LIMIT 和 OFFSET
  - 执行分页查询并更新表格数据
  - 禁用不可用的分页按钮（第一页禁用"上一页"）
  - 显示当前页码和总页数
  - _需求：4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [ ] 8. 在 QueryEditor 中集成 TableDataEditor
  - 修改 `src/views/mysql/QueryEditor.vue`
  - 在执行查询前使用 `applyLimit()` 处理 SQL
  - 显示查询限制提示信息（如果被自动限制）
  - 将 NDataTable 替换为 TableDataEditor 组件
  - 获取表主键并传递给 TableDataEditor
  - 处理 TableDataEditor 的 refresh 和 save 事件
  - _需求：1.1, 1.5_

- [ ] 9. 添加全局查询限制设置 UI
  - 创建或修改设置页面组件
  - 添加 "MySQL 查询限制" 配置项
  - 使用 NInputNumber 组件，设置 min=10, max=1000
  - 保存时调用 `saveQueryLimit()` 方法
  - 显示当前配置值和说明文字
  - _需求：5.1, 5.2, 5.3_

- [ ] 10. 添加查询历史记录更新操作
  - 修改 `src/stores/history.ts`
  - 在保存历史记录时记录 UPDATE 操作
  - 记录原始值和新值
  - 在 QueryHistoryPanel 中显示 UPDATE 类型的历史记录
  - _需求：3.5, 非功能需求-安全_

- [ ] 11. 编写集成测试
  - 创建 `TableDataEditor.spec.ts` 测试文件
  - 测试编辑单个单元格的完整流程
  - 测试保存多个修改的流程
  - 测试放弃修改的流程
  - 测试分页查询的正确性
  - 测试错误处理（UPDATE 失败、无主键等）
  - _需求：所有功能需求_

- [ ] 12. 优化和修复
  - 修复 TypeScript 类型错误
  - 优化单元格渲染性能（使用虚拟滚动）
  - 添加加载状态和过渡动画
  - 修复边界情况（空数据、单行数据等）
  - 确保所有功能都有适当的错误提示
  - _非功能需求：性能、用户体验_
