# T9 验收测试门 — 校对功能

| 项 | 内容 |
|----|------|
| 启动日期 | 2026-05-03 |
| Tester | planner_alice (兼任) |
| 关联 spec | `2026-04-30-smart-editor-design-v3.md` 4.4 / 5.4 + decision-T8 |
| 适用范围 | Phase 1 校对功能（含 T1-T7 全部 coder 交付） |

## 验收原则

> **"声称完成 ≠ 测试通过"**。Xin 任何 [DONE] 都必须经此门，未通过则 PR 状态打回 in_progress。

---

## 1. 后端 cargo test 清单（B 系列）

### B1 ⚠ 依赖 T6 — `parse_document(.docx)` 返回结构化段落

**期望签名**：
```rust
pub struct ParsedDocument {
    pub text: String,
    pub paragraphs: Vec<Paragraph>,
}
pub struct Paragraph {
    pub index: usize,
    pub text: String,
    pub char_offset: usize,
}
```

**测试断言**：
- 给定一个 fixture `.docx`（`tests/fixtures/sample.docx`，3 段非空文本）
- 调用后 `paragraphs.len() >= 3`
- `paragraphs[i].char_offset` 严格递增（i.e., `paragraphs[i+1].char_offset > paragraphs[i].char_offset`）
- 每个 `paragraph.text` 非空且 trim 后非空
- `text` 字符串包含所有 `paragraphs[i].text`

**测试位置**：`src-tauri/src/parser.rs` 的 `tests` 模块（已有）

**当前状态**：⏸ T6 未完成时 `#[ignore = "depends on T6"]`

---

### B2 ⚠ 依赖 T4 — `parse_document(.doc)` 返回明确 sentinel 错误

**期望行为**：当文件扩展名为 `.doc` 时返回 `Err`，错误消息中包含 `"unsupported_format"` 或 `"请转 .docx"` 等明确标志。**不能**只返回通用的 `"不支持的文件格式: doc"`。

**测试断言**：
```rust
let result = parse_document("nonexistent.doc");
assert!(result.is_err());
let err = format!("{}", result.unwrap_err());
assert!(
    err.contains("doc 格式不支持") || err.contains("请转 .docx"),
    "Expected actionable error message for .doc, got: {}", err
);
```

**测试位置**：`src-tauri/src/parser.rs` 的 `tests` 模块

**当前状态**：⚠ 当前返回 "不支持的文件格式: doc" — 不够明确，需 T4 改进

---

### B3 ✅ 可立即实施 — `check_deviation_items` 边界处理

**期望行为**：传入空 `req_items` 不 panic，返回 `total=0`。

**测试位置**：`src-tauri/src/deviation.rs` 的 `tests` 模块

**已实施**：✅ 见 `test_check_deviation_items_empty` 

---

### B4 ⚠ 依赖 T5 — `check_self_review` 6 类全命中

**期望行为**：当输入文本同时含 placeholder + sensitive + repetition + punctuation + contradiction + context_logic 6 类问题，`SelfReviewReport.issues` 中应同时出现 6 个 `sub_category`。

**测试要点**：
- 准备一个故意构造的 fixture 文本，每类至少触发一次
- contradiction / context_logic 部分可注入 mock AI（依 T5 设计）
- 断言 6 个 sub_category 都被报出

**当前状态**：⏸ T5 未完成时 `#[ignore = "depends on T5"]`

---

### B5 ⚠ 依赖 T6 — `paragraph_index` 与 `parse_document.paragraphs` 一致

**期望行为**：`SelfReviewIssue.paragraph_index` 必须是 `parse_document` 返回的 `paragraphs` 数组下标，而不是行号或其他口径。

**测试断言**：
- 解析 fixture .docx → 得到 `paragraphs[]`
- 调用 `check_self_review(parsed.text)` → 得到 issues
- 对每个有 `paragraph_index` 的 issue，验证 `paragraphs[issue.paragraph_index]` 存在且其 text 与 issue 的 `original` 字段在同一段落

**当前状态**：⏸ T6 未完成时 `#[ignore = "depends on T6"]`

---

### B6 ✅ 可立即实施 — sensitive issue auto_fixable=true（per T8）

**期望行为**：sensitive 类 issue 的 `auto_fixable` 必须为 `true` 且 `suggestion` 非空。

**当前代码**（self_review.rs:132）：`auto_fixable: false` ❌ 与 T8 决策冲突

**已实施**：✅ 见 `test_sensitive_auto_fixable_per_t8` （**当前会 FAIL**，驱动 Xin 在 T7/T8 实施时修正）

---

### B7 ✅ 可立即实施 — placeholder issue auto_fixable=false（per T8）

**期望行为**：placeholder 类 issue 的 `auto_fixable` 必须为 `false`。

**当前代码**（self_review.rs:177）：`auto_fixable: true` ❌ 与 T8 决策冲突

**已实施**：✅ 见 `test_placeholder_auto_fixable_per_t8` （**当前会 FAIL**，驱动 Xin 修正）

---

### B8 ✅ 可立即实施 — repetition / contradiction / logic auto_fixable=false

**期望行为**：repetition / contradiction / logic 类 issue 的 `auto_fixable` 必须为 `false`。

**已实施**：✅ 见 `test_repetition_auto_fixable_false` (passes currently); contradiction/logic ⏸ 等 T5

---

### B9 ⚠ 依赖 T7 — punctuation 并入 SelfReviewReport

**期望行为**：`check_self_review(text)` 在 T7 完成后应调用 `check_punctuation` 并把结果以 `category="format", sub_category="punctuation", auto_fixable=true` 形式塞进 issues。

**测试断言**：
- 输入故意带中文标点错误的文本
- 报告中出现至少一个 sub_category="punctuation" 的 issue
- 该 issue 的 `auto_fixable=true` 且 suggestion 非空

**当前状态**：⏸ T7 未完成时 `#[ignore = "depends on T7"]`

---

## 2. 前端 vitest 清单（F 系列）

### F1 ⚠ 依赖 T4 — .doc 文件选择拦截

**测试位置**：`src/App.test.tsx` 或 `src/components/CheckResultsPanel.test.tsx`

**测试要点**：
- mock `dialog.open` 返回 `xxx.doc`
- 触发选择 → 断言显示拦截提示，**不**调用 `invoke('parse_document')`

---

### F2 ⚠ 依赖 T6 — .docx 预览段落数 = paragraphs.length

**测试要点**：
- mock `invoke('parse_document')` 返回 `{ text: "...", paragraphs: [{...}, {...}, {...}] }`
- 触发选择 → 预览区渲染段落数 === 3

---

### F3 ⚠ 依赖 T2/T3 — 点击 issue 跳转 + 高亮

**测试要点**：
- 渲染 CheckResultsPanel 带 mock issues
- 触发 onClick → 验证 onJumpToParagraph 被调用 + 黄底高亮 class 应用到对应段落

---

### F4 ✅ 可立即实施 — requirements 空时按钮 disabled

**测试要点**：
- 渲染 CheckResultsPanel，RequirementsContext 中 reqItems = []
- 「检查投标文件」按钮带 disabled 属性
- 显示「请先前往需求上传解析招标文件」提示

---

### F5 ⚠ 依赖 T1-T3 — Tab 切换状态保留

**测试要点**：
- 切到「校对」执行检查 → 切到「需求上传」→ 切回「校对」
- 检查结果仍展示，bid 文件路径仍记忆

---

### F6 ⚠ 依赖 T7+T8 — auto_fixable=true 渲染「复制建议」

**测试要点**：
- mock issue with `auto_fixable=true, suggestion="..."`
- 卡片渲染「复制建议」按钮
- 点击 → `navigator.clipboard.writeText(suggestion)` 被调用

---

### F7 ⚠ 依赖 T7+T8 — auto_fixable=false 不渲染按钮

**测试要点**：
- mock issue with `auto_fixable=false`
- 卡片**不**渲染「复制建议」按钮，仅显示「请前往 Word 手动修改」

---

### F8 ✅ 可立即实施（结构性） — 校对模式不调用 applyPunctuationFixes

**测试要点**：
- 静态扫描或运行时断言：CheckResultsPanel 渲染下不存在 `applyPunctuationFixes` 调用路径

**实施方式**：grep 测试 + import 检查

---

## 3. 集成手测脚本（E 系列）

> 由 tester 手动执行，非自动化（需要真实 .docx 文件）。

### E1 完整 happy path

```text
1. cargo tauri dev 启动
2. 上传招标 .docx → 「需求上传」Tab → 解析成功 → 看到需求列表
3. 切到「校对」Tab
4. 选投标 .docx
5. 点「检查投标文件」
6. 验证：
   - 偏离结果（红色致命标识）展示
   - SelfReviewReport 5 类（repetition/sensitive/placeholder/punctuation/contradiction）展示
   - 标点检查结果（如有）展示
7. 点击任意 issue → 预览区滚动到对应段落 + 黄底高亮
8. 点「复制建议」按钮 → 剪贴板正确写入
9. 点「导出报告」→ Markdown 文件落盘
```

### E2 cargo test 全绿

```bash
cd smart-editor-app/src-tauri
cargo test --all
# 期望：所有非 ignored 测试通过
cargo test --all -- --include-ignored
# 期望：T6/T7 完成后所有 ignored 测试也通过
```

### E3 npm test 全绿

```bash
cd smart-editor-app
# 注：当前 package.json 缺 "test" script，需补：
# "test": "vitest run",
# "test:watch": "vitest"
npx vitest run
# 期望：所有 vitest 用例通过
```

### E4 cargo build --release 无错误

```bash
cd smart-editor-app/src-tauri
cargo build --release 2>&1 | tee /tmp/release-build.log
# 期望：exit 0；warning 计入次轮整改清单但不阻塞验收
```

---

## 4. 失败处置流程

1. 任何 B/F/E 不通过 → tester 写 `[REPLY]` 给 Xin，列具体失败 case + log 摘录
2. Xin 必须 `git stash` 当前自我评估，重做对应 T 任务
3. 修复后重新跑全套 → tester 二次验证 → 通过后才允许 `agtalk done` 当条任务

> **未通过 = 任务未完成**。spec 内任何「我觉得 OK」的自评不作数。

## 5. 当前可见 fail（已知红灯）

下面 3 个测试**当前会 FAIL**，因为 self_review.rs 的 auto_fixable 字段与 T8 决策不一致。这是**故意的**——红灯驱动 Xin 在 T7 实施时修复：

| 测试 | 文件 | 当前状态 | 修复 |
|------|------|---------|------|
| `test_sensitive_auto_fixable_per_t8` | self_review.rs | ❌ RED | T7 实施时把 sensitive 的 `auto_fixable: false` 改成 `true`，并补 suggestion |
| `test_placeholder_auto_fixable_per_t8` | self_review.rs | ❌ RED | T7 实施时把 placeholder 的 `auto_fixable: true` 改成 `false` |
| `test_repetition_auto_fixable_false` | self_review.rs | ✅ GREEN | 当前已对，作为正向回归 |

> 后续 Xin 完成 T6/T7 后，把对应 `#[ignore]` 标记移除并跑全套——这是 tester gate 的硬通过条件。
