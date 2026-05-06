# 校对功能设计文档

## 1. 需求概要

**目标**：让用户在「校对」Tab 中，一键检查当前编辑器中的投标文件，无需上传任何文件。

**核心约束**：
- 手动触发（不实时）
- 偏离检查保留，招标文件需求从「需求上传」Tab 自动同步
- 点击问题项可定位到编辑器具体段落并高亮
- 废标风险置顶红色高亮

## 2. 检查维度

| 分类 | 检查项 | 自动修复 | 实现策略 |
|------|--------|----------|----------|
| **偏离风险** | 未响应、正/轻微/重大偏离 | 否 | AI（需求列表 vs 编辑器文本语义匹配） |
| **废标风险** | 资质缺失、承诺缺失、有效期问题 | 否 | AI + 关键词规则 |
| **内容一致性** | 自相矛盾、上下文逻辑断裂 | 否 | AI（跨段落语义分析） |
| **内容质量** | 重复段落 | 否 | 本地算法（simhash / 编辑距离） |
| **内容质量** | 敏感信息未脱敏（客户姓名、IP、内部人名） | 否 | 本地正则 + 关键词库 |
| **格式问题** | 标点符号（中文标点规范） | 是 | 本地正则 |
| **格式问题** | 空白占位符（`[待补充]`、`XXX`、`________`） | 是 | 本地正则 |

## 3. 数据流

```
[需求上传 Tab]
    ↓ 解析招标文件
[招标需求清单] ──→ [前端全局状态 / 后端缓存]
                          ↓
                    [校对 Tab 自动读取]
                          ↓
[编辑器文本] + [招标需求清单] ──→ [检查按钮触发]
                          ↓
              ┌───────────┼───────────┐
              ↓           ↓           ↓
        [偏离检查]   [自查检查]   [格式检查]
              ↓           ↓           ↓
        [AI 后端]    [AI/本地]    [本地后端]
              └───────────┴───────────┘
                          ↓
              [分类结果面板] ←─ 点击定位 ─→ [编辑器高亮]
```

## 4. 后端接口设计（Rust Commands）

### 4.1 偏离检查（复用已有，改数据源）

原命令 `check_deviation_files(bidPath, reqPath)` 改为：

```rust
#[tauri::command]
async fn check_deviation(
    state: tauri::State<'_, AppState>,
    req_items: Vec<RequirementItem>,
    bid_text: String,
) -> Result<DeviationReport, String>
```

**改动点**：不再接受文件路径，直接接受解析好的需求列表 + 编辑器纯文本。

### 4.2 废标风险检查（已有，保留）

```rust
#[tauri::command]
async fn check_fatal_risks_text(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<Vec<FatalRisk>, String>
```

### 4.3 新增：投标文件自查

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReviewIssue {
    pub id: i64,
    pub category: String,       // "consistency" | "quality" | "format"
    pub sub_category: String,   // "contradiction" | "repetition" | "sensitive" | "logic" | "punctuation" | "placeholder"
    pub message: String,
    pub severity: Severity,     // Error | Warning | Info
    pub position: Option<usize>,
    pub paragraph_index: Option<usize>,
    pub original: Option<String>,
    pub suggestion: Option<String>,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReviewReport {
    pub issues: Vec<SelfReviewIssue>,
}

#[tauri::command]
async fn check_self_review(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<SelfReviewReport, String>
```

**内部拆分**：
- `check_self_review` 调用多个子检查函数，聚合结果
- `check_contradictions(text) -> Vec<SelfReviewIssue>` — AI 检测自相矛盾
- `check_context_logic(text) -> Vec<SelfReviewIssue>` — AI 检测上下文逻辑
- `check_repetition(text) -> Vec<SelfReviewIssue>` — 本地算法检测重复段落
- `check_sensitive_info(text) -> Vec<SelfReviewIssue>` — 本地正则检测敏感信息
- `check_placeholders(text) -> Vec<SelfReviewIssue>` — 本地正则检测占位符
- `check_punctuation(text) -> Vec<SelfReviewIssue>` — 已有，复用

### 4.4 招标需求同步（前端状态）

推荐方案 A（前端全局状态）：
- UploadTab 解析后写入 React Context / Zustand
- CheckTab 直接读取，无需后端通信

## 5. 前端组件结构

```
CheckTab
├── CheckHeader
│   ├── [检查投标文件] 按钮
│   └── 需求清单来源提示（已同步 / 未同步）
├── CheckResultsPanel
│   ├── DeviationSection (偏离风险)
│   ├── FatalRiskSection (废标风险) — 置顶，红色
│   ├── ConsistencySection (内容一致性：矛盾 + 逻辑)
│   ├── QualitySection (内容质量：重复 + 敏感信息)
│   └── FormatSection (格式问题：标点 + 占位符)
│       └── 带 [应用修复] 按钮
└── CheckActionsFooter
    ├── [一键修复格式问题]
    └── [导出报告]
```

**定位与高亮**：
- 点击问题项 -> 编辑器滚动到段落 + 高亮
- 通过 `editor.chain().focus().setTextSelection({ from, to })` 或自定义 decoration 实现

## 6. 各检查项算法/策略

### 6.1 重复段落（本地）
- 段落两两比较，编辑距离 / 长度比 > 0.8 判定重复
- 或 simhash 汉明距离 < 阈值

### 6.2 敏感信息（本地）
正则规则库：
- 客户姓名：`(?:客户|甲方|招标人|用户)[：:]\s*(\S{2,10})` + 常见客户名黑名单
- IP 地址：`\b(?:\d{1,3}\.){3}\d{1,3}\b`
- 内部人名：中文姓名模式 + 上下文
- 内部域名：`(?:\.internal|\.corp|\.local)\b`

### 6.3 自相矛盾（AI）
- 文档分块，prompt AI 找出语义矛盾对
- 例："7x24 支持" vs "工作日 9-18 点"

### 6.4 上下文逻辑（AI）
- prompt AI 检查段落间承接关系

### 6.5 标点/占位符（本地，已有）
- 复用现有 `punctuation.rs`

## 7. 任务拆分与依赖

| 任务 | 依赖 | 负责方 |
|------|------|--------|
| 1. 前端：重构 CheckTab（分类面板、定位、高亮） | 无 | coder |
| 2. 后端：修改偏离检查数据源（接受 reqItems + text） | 无 | coder |
| 3. 后端：新增 `check_self_review` 及子检查 | 任务 2 | coder |
| 4. 前端：招标需求全局状态同步 | 无 | coder |
| 5. 前端：一键修复 + 导出报告 | 任务 1 | coder |
| 6. 审查：代码审查 + 测试 | 所有任务完成后 | planner |
