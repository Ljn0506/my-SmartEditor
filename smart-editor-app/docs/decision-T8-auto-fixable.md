# T8 决策：auto_fixable 字段与「一键修复」处置

| 项 | 内容 |
|----|------|
| 决策日期 | 2026-05-03 |
| 决策人 | planner_alice |
| 关联 spec | `2026-04-30-smart-editor-design-v3.md` 第 4.4 / 5.4 / 10 节 |
| 影响范围 | 校对面板前端 + SelfReviewIssue 字段语义 + Phase 1 任务清单 |

## 1. 冲突陈述

spec 内三处对「自动修复」的定义不一致：

1. **4.4 表格**（line 373-381）：所有 7 个检查维度的「自动修复」列**全部是否**，并明确「仅提示，不改文件」
2. **4.4 末尾说明**（line 383）："校对模式**不修改本地文件**，所有修改需用户回到 Word/WPS 中手动完成"
3. **5.4 SelfReviewIssue 结构**（line 509）：保留 `auto_fixable: bool` 字段
4. **Phase 1 task #5**（line 873）："前端：一键修复 + 导出报告"

**字面冲突**：4.4 和 4.4 末尾说明明确不改文件，但 task #5 说要「一键修复」。

## 2. 决策

**砍掉「一键修复」语义中的「写文件」动作；保留 `auto_fixable` 字段但语义重定义为「是否提供可粘贴的修复建议」。**

### 具体规定

| 项 | 处置 |
|----|------|
| Phase 1 task #5「一键修复」 | **删除** |
| Phase 1 task #5「导出报告」 | **保留** |
| 替换项 | 新增「复制建议到剪贴板」按钮（仅当 `auto_fixable=true && suggestion!=None`） |
| `auto_fixable` 字段 | **保留**，但语义改为「该 issue 提供机器可读的 suggestion，UI 可显示『复制建议』按钮」 |
| `suggestion` 字段 | **保留**，作为「复制建议」的内容来源 |
| 校对面板写文件 | **禁止**——前端不得在校对模式下调用任何修改本地文件的命令 |

### auto_fixable 字段在各 sub_category 下的取值规范

| sub_category | auto_fixable | 理由 |
|--------------|--------------|------|
| contradiction | `false` | AI 检测，无确定性修复方案 |
| logic | `false` | AI 检测，无确定性修复方案 |
| repetition | `false` | 重复段落需人工判断保留哪一份 |
| sensitive | `true` | 可提供「将客户名替换为 [客户公司]」的脱敏建议字符串 |
| punctuation | `true` | 可提供规范化后的标点字符串 |
| placeholder | `false` | 占位符要填什么是业务问题，无法机器决定 |

> 现有 `applyPunctuationFixes` 在校对模式下**禁用**——无论 punctuation 的 auto_fixable 是 true 还是 false，都不写文件。该函数可在「智能生成」模式（Tiptap 编辑中）下保留使用。

## 3. 对前端的影响

### 3.1 旧设计 CheckTab（App.tsx:402-561）

- `applyPunctuationFixes` 调用：**移除**（校对模式不写文件）
- 跳转用 `editor.state.doc.descendants`：**移除**（改用预览区 paragraph_index 滚动）
- `editor.getText()`：**移除**（数据来自 `parse_document` 返回的 text）

### 3.2 新 CheckResultsPanel（已建）

- 在 issue 卡片底部按 `auto_fixable` 显示按钮：
  - `auto_fixable=true`：显示「复制建议」按钮 → `navigator.clipboard.writeText(suggestion)`
  - `auto_fixable=false`：仅显示提示文本「请前往 Word 手动修改」
- 「导出报告」按钮位置不变，落点 `export_self_review_report_markdown`（如已有则复用）

## 4. 对后端的影响

### 4.1 self_review.rs

- 各子检查在写 `SelfReviewIssue` 时按上方表格设置 `auto_fixable`
- `suggestion` 字段在 `auto_fixable=true` 时必须填充非空字符串

### 4.2 export 命令

新增（如已有 `export_deviation_report_markdown` 可参考）：

```rust
#[tauri::command]
fn export_self_review_report_markdown(report: SelfReviewReport) -> Result<String, String>
```

输出 Markdown 字符串，前端通过文件保存对话框落盘。

## 5. 对测试的影响（T9）

新增/修改 cargo test 用例：

| ID | 测试 |
|----|------|
| B6 | sensitive issue 的 `auto_fixable` 必须为 true 且 suggestion 非空 |
| B7 | placeholder issue 的 `auto_fixable` 必须为 false |
| B8 | repetition / contradiction / logic issue 的 `auto_fixable` 必须为 false |
| B9 | punctuation issue 在 SelfReviewReport 中（T7 完成后）`auto_fixable` 必须为 true 且 suggestion 非空 |

新增/修改 vitest 用例：

| ID | 测试 |
|----|------|
| F6 | `auto_fixable=true` 的 issue 卡片渲染「复制建议」按钮 |
| F7 | `auto_fixable=false` 的 issue 卡片不渲染「复制建议」按钮，仅显示提示 |
| F8 | 校对模式下不存在调用 `applyPunctuationFixes` 的代码路径 |

## 6. spec 修订建议（次要）

`2026-04-30-smart-editor-design-v3.md` 应更新：

1. line 873 修改为：`- [ ] 前端：导出报告 + 单条复制建议（auto_fixable=true 时）`
2. line 509 后追加注释：`// auto_fixable 仅指「是否有可粘贴的 suggestion」；校对模式不直接修改本地文件`
3. line 380-381 表格备注列改为：`仅提示 + 复制建议` / `仅提示`

> spec 修订由 planner 完成，不阻塞 coder 开工。

## 7. 给 Xin 的执行指令

T1-T7 全部按本决策实施。如发现本决策内部矛盾，停手回我 [REPLY]，不要自行延伸解释。
