# Phase 1 重建实施计划（v3 校对面板）

> 基于 Alice msg `90a30a11` + `bd54fa4e` 制定
> 制定人：kimi_coder_Xin
> 时间：2026-05-07

---

## 总体策略

**单线程串行，前置任务验收通过后再继续。**
T6（parse_document 结构化）是所有后续工作的根基，必须先完成并验收。

**预估总耗时：1 整天**

---

## 任务拆分

### 🔷 T6：parse_document 结构化返回（前置，2h）

**目标**：让 `parse_document` 返回统一段落索引，作为所有检查命令的根基。

#### 6.1 数据模型（models.rs）
```rust
pub struct Paragraph {
    pub index: usize,
    pub text: String,
    pub char_offset: usize,
}

pub struct ParsedDocument {
    pub text: String,
    pub paragraphs: Vec<Paragraph>,
}
```

#### 6.2 parser.rs 改造
- 修改 `parse_document(file_path)` 返回 `Result<ParsedDocument, String>`
- `paragraphs` 按 `docx-rs` 段落为单元
- `char_offset` 严格递增（段落起始字符位置）
- `.doc` → `Err("unsupported_format")`
- `.docx` → 保证 `paragraphs.len() > 0`

#### 6.3 单元测试（B1/B2）
- B1: `parse_document(".docx")` 返回 paragraphs 数量 > 0，char_offset 严格递增
- B2: `parse_document(".doc")` 返回明确的 `Err("unsupported_format")`

#### 验收门
- `cargo test` 新增测试通过
- `cargo clippy` 零 warning
- 向 Alice 发送 [REPLY] T6 完成，等待验收后再继续

---

### 🔷 后端：检查命令完整实现（4h）

**依赖**：T6 验收通过

#### 2.1 models.rs 补充类型
```rust
pub struct SelfReviewIssue {
    pub id: i64,
    pub category: String,
    pub sub_category: String,
    pub message: String,
    pub severity: Severity,
    pub position: Option<usize>,
    pub paragraph_index: Option<usize>,
    pub original: Option<String>,
    pub suggestion: Option<String>,
    pub auto_fixable: bool,
}

pub struct SelfReviewReport {
    pub issues: Vec<SelfReviewIssue>,
}

pub enum Severity { Error, Warning, Info }
```

#### 2.2 check_deviation 改造
- 签名：`async fn check_deviation(state, req_items, bid_text) -> Result<DeviationReport, String>`
- `paragraph_index` 与 `parse_document.paragraphs` 索引一致

#### 2.3 check_self_review 完整实现
内部子检查全部落地：

| 子检查 | 类型 | 说明 |
|--------|------|------|
| `check_contradictions(text)` | AI | 检测自相矛盾（分块 prompt） |
| `check_context_logic(text)` | AI | 检测上下文逻辑断裂 |
| `check_repetition(text)` | 本地 | simhash/编辑距离 ≥75% 相似度 |
| `check_sensitive_info(text)` | 本地 | 正则 + 关键词库 |
| `check_placeholders(text)` | 本地 | `[待补充]`、`XXX`、`________` |
| `check_punctuation(text)` | 本地 | 复用已有模块，结果统一归入 issues |

- `SelfReviewIssue.category`：`"consistency"` | `"quality"` | `"format"`
- `SelfReviewIssue.sub_category`：`"contradiction"` | `"logic"` | `"repetition"` | `"sensitive"` | `"punctuation"` | `"placeholder"`
- `check_punctuation` 内部调用后，issue 的 `category="format"`, `sub_category="punctuation"`

#### 2.4 check_fatal_risks_text 保留
已有命令不动，确认签名兼容新数据流。

#### 2.5 apply_self_review_fixes（新增）
- 签名：`async fn apply_self_review_fixes(file_path, issues) -> Result<String, String>`
- 仅处理 `auto_fixable=true` 的 issues（标点 + 占位符）
- 使用 `docx-rs` 读取 → 定位段落 → 修改文本 → 写入 `{原文件名}_fixed.docx`
- **绝不覆盖原始文件**
- 返回新文件路径

#### 2.6 main.rs 注册
注册所有新/改造命令：
- `parse_document`
- `check_deviation`
- `check_fatal_risks_text`
- `check_self_review`
- `apply_self_review_fixes`

#### 单元测试（B3-B5）
- B3: `check_deviation_items` 输入空 req_items 不 panic，返回 total=0
- B4: `check_self_review` 同时含 6 类 issue 全部命中（AI 子检查可用 mock）
- B5: `paragraph_index` 与 `parse_document.paragraphs` 索引一致

#### 验收门
- `cargo check` 零错误
- `cargo test` 全绿
- `cargo clippy` 零 warning

---

### 🔷 前端：ReviewTab 完整实现（4h）

**依赖**：后端检查命令全部可用

#### 3.1 招标需求全局状态（Context 方案）
- 保留 React Context，Zustand 延到 Phase 2
- `RequirementsContext` 存储：`parsedRequirements`, `parsedText`, `fileName`
- UploadTab 解析后写入，ReviewTab 自动读取

#### 3.2 文件选择组件
- 仅支持 `.docx`、`.doc`
- `.doc` 选择后弹窗：「建议转换为 .docx」，可选继续或取消
- 调用 `parse_document(file_path)` 获取 `ParsedDocument`

#### 3.3 只读预览区
- 渲染 `parse_document.paragraphs` 为段落列表
- 每行显示：`[index] text`
- 点击 IssueList 项 → 预览区滚动到对应段落 + **黄底高亮**
- 段落索引与后端 `paragraph_index` 完全对齐

#### 3.4 分类结果面板（CheckResultsPanel 重构）
分组展示：
- 🔴 偏离风险（DeviationSection）
- 🔴 废标风险（FatalRiskSection）— 置顶红色
- ⚠️ 内容一致性（ConsistencySection：矛盾 + 逻辑）
- 📝 内容质量（QualitySection：重复 + 敏感信息）
- 🔧 格式问题（FormatSection：标点 + 占位符）

每项显示：定位按钮 + 片段预览

#### 3.5 一键修复交互
- ReviewTab 底部 [一键修复可修复项] 按钮
- 点击 → 弹窗二次确认：「将生成修复副本，原始文件保持不变。是否继续？」
- 调用 `apply_self_review_fixes`
- 完成后提示：「已生成修复版：xxx_fixed.docx，请核对后使用」
- 刷新预览区显示修复后内容
- 若原始文件是 `.doc`，按钮 disabled + 提示「请转换为 .docx 后使用一键修复」

#### 3.6 检查按钮逻辑
- 无已解析需求时，「检查投标文件」按钮 **disabled** + 提示
- 点击后并发调用：
  - `check_deviation(req_items, bid_text)`
  - `check_fatal_risks_text(text)`
  - `check_self_review(text)`
- 切换 Tab 后切回，结果状态保留（Context）

#### 3.7 导出报告
- 保留 `export_deviation_report_markdown`
- ReviewTab 底部 [导出报告] 按钮

#### 前端测试（F1-F5）
- F1: `.doc` 文件选择 → 拦截提示，不调 `parse_document`
- F2: `.docx` 文件选择 → 预览区渲染段落数 = `parse_document.paragraphs.length`
- F3: 点击 IssueList 项 → `onJumpToParagraph` 触发，预览区滚动 + 黄底高亮
- F4: requirements 为空时「检查投标文件」按钮 disabled + 显示提示
- F5: 切换 Tab 后再切回，结果状态保留

#### 验收门
- `tsc --noEmit` 通过
- `vitest run` 全绿
- `vite build` 通过

---

### 🔷 集成测试与门禁（2h）

#### E1：Happy Path 手测
1. 上传招标 `.docx` → 切到 ReviewTab
2. 选择投标 `.docx` → 预览区显示段落列表
3. 点「检查投标文件」→ 5 类 issue 都展示
4. 点任意问题 → 预览区跳转准确 + 黄底高亮
5. 点「一键修复」→ 二次确认 → 生成 `_fixed.docx`
6. 点「导出报告」→ Markdown 报告下载

#### E2-E4：自动化门禁
- E2: `cargo test` 全绿
- E3: `npm test` 全绿
- E4: `cargo build --release` 无 warning（warning 记录但不阻塞）

---

## 时间预估

| 阶段 | 任务 | 预估时间 |
|------|------|----------|
| T6 | parse_document 结构化 | 2h |
| 后端 | 检查命令 + 一键修复 | 4h |
| 前端 | ReviewTab 完整 UI | 4h |
| 测试 | 集成 + 六门门禁 | 2h |
| **总计** | | **~12h（1 整天）** |

---

## 风险点

1. **AI 子检查（contradictions / context_logic）**：需要 prompt 调优，可能超时
   - 缓解：先跑通 mock 实现，确保接口和数据流正确
2. **docx-rs 段落定位**：一键修复时需精确匹配段落，可能因格式差异定位不准
   - 缓解：按 index 直接定位 paragraph，不依赖文本匹配
3. **前端预览区滚动高亮**：大量段落时性能问题
   - 缓解：虚拟滚动或分页，MVP 先不做优化

---

## 回复 Alice 的 ACK 草稿

```
[REPLY] Phase 1 重建计划已制定，请审阅

收到 msg 90a30a11 + bd54fa4e，已制定实施计划：

1. T6：parse_document 结构化返回（2h，前置，单独验收）
2. 后端：check_deviation / check_self_review（6类子检查）/ apply_self_review_fixes（4h）
3. 前端：ReviewTab 完整 UI（文件选择/只读预览/分类面板/一键修复+二次确认）（4h）
4. 集成测试 + 六门技术门禁（2h）

计划已写入 docs/plan-phase1-rebuild.md。

如无异议，我先开工 T6。有问题随时 reply。
```
