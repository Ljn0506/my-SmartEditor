# Smart-Editor Phase 3 实施计划：多文件一致性 + 商务人机协作

> 基于 v3.0 设计文档（2026-04-30）制定  
> 制定人：kimi_planner_Alice  
> 时间：2026-05-11

---

## 上下文

Phase 1（校对功能）和 Phase 2（智能生成 MVP）已完成，版本 `0.2.0.0`。  
Phase 3 的核心目标是实现 **跨文档参数一致性保证** 和 **商务卡片的强制人工审核流程**。

**设计原则**：保持现有单文档切换模式（技术/商务各自独立查看），不引入常驻的多文件总览 UI。

---

## 任务清单

### 1. 全局参数表（前后端）

| 子任务 | 说明 | 负责文件 |
|--------|------|----------|
| 1.1 数据模型 | 新增 `GlobalParams` struct（项目名、客户名、合同金额、交付工期、维保年限、响应时间、项目经理、QPS、并发用户数等） | `src-tauri/src/models.rs` |
| 1.2 DB 存储 | 新增 `global_params` 表（`key TEXT PRIMARY KEY, value TEXT NOT NULL`），单条 JSON blob 存储 | `src-tauri/src/db.rs` |
| 1.3 DAO 方法 | `save_global_params` / `get_global_params` | `src-tauri/src/db.rs` |
| 1.4 Tauri 命令 | `get_global_params` / `save_global_params` | `src-tauri/src/main.rs` |
| 1.5 前端 UI | 全局参数配置弹窗（表单填写 → 保存到后端） | `src/components/GenerateTab.tsx` |

### 2. 跨文档一致性检查

| 子任务 | 说明 | 负责文件 |
|--------|------|----------|
| 2.1 AI Prompt | 新增 `build_cross_doc_consistency_prompt`：将全局参数表作为基准，传入技术卡片 + 商务卡片，要求 AI 检查跨文档参数矛盾 | `src-tauri/src/ai.rs` |
| 2.2 AI 方法 | `AiClient::check_cross_document_consistency` | `src-tauri/src/ai.rs` |
| 2.3 Tauri 命令 | `check_cross_document_consistency(global_params, technical_cards, business_cards)` | `src-tauri/src/main.rs` |
| 2.4 前端触发 | GenerateTab 底部新增"跨文档一致性检查"按钮（两个目标都有卡片时才可用）→ 弹窗显示结果 | `src/components/GenerateTab.tsx` |

### 3. 风险标签

| 子任务 | 说明 | 负责文件 |
|--------|------|----------|
| 3.1 数据模型 | `Card` 增加 `risk_flags: Vec<String>` | `src-tauri/src/models.rs` |
| 3.2 DB Schema | `cards` 表增加 `risk_flags TEXT NOT NULL DEFAULT '[]'` | `src-tauri/src/db.rs` |
| 3.3 DAO 更新 | `save_cards` / `get_cards` / `update_card` 处理 `risk_flags` | `src-tauri/src/db.rs` |
| 3.4 AI Prompt | 修改 `build_chapter_prompt`：商务类型要求 AI 在输出末尾以 JSON 格式返回 `{"risk_flags": [...]}` | `src-tauri/src/ai.rs` |
| 3.5 AI 提取 | `generate_chapter` 解析正文后提取风险标签 | `src-tauri/src/ai.rs` |
| 3.6 前端渲染 | 卡片列表中渲染风险标签 badge（红色/橙色） | `src/components/GenerateTab.tsx` |

### 4. 商务卡片强制逐张审核

| 子任务 | 说明 | 负责文件 |
|--------|------|----------|
| 4.1 批量确认拦截 | `handleConfirmAll` 中判断 `docTarget === 'business'` 时，弹窗提示"商务卡片涉及敏感条款，必须逐张审核"，拒绝批量确认 | `src/components/GenerateTab.tsx` |
| 4.2 技术卡片保留 | `docTarget === 'technical'` 时，一键确认逻辑不变 | `src/components/GenerateTab.tsx` |

### 5. 全局参数注入生成流程

| 子任务 | 说明 | 负责文件 |
|--------|------|----------|
| 5.1 Prompt 注入 | `build_chapter_prompt` 增加 `## 全局参数表` 段落（如果提供了全局参数），指令："如果全局参数表中已有该参数值，请直接使用；未提供的才使用 [PARAM:xxx]" | `src-tauri/src/ai.rs` |
| 5.2 命令传参 | `generate_card` 增加可选的 `global_params` 参数 | `src-tauri/src/main.rs` |
| 5.3 前端传参 | GenerateTab 生成卡片时，如果有全局参数则一并传入 | `src/components/GenerateTab.tsx` |

### 6. 多目标卡片加载（按需）

| 子任务 | 说明 | 负责文件 |
|--------|------|----------|
| 6.1 切换加载 | `useEffect` 监听 `docTarget` 变化，自动调用 `get_cards(documentTarget)` 加载对应目标卡片 | `src/components/GenerateTab.tsx` |
| 6.2 切换保存 | 切换目标前，先保存当前卡片到后端 | `src/components/GenerateTab.tsx` |
| 6.3 本地缓存 | `technicalCards` / `businessCards` 本地缓存，减少重复加载 | `src/components/GenerateTab.tsx` |

---

## 数据模型变更

```rust
// models.rs 新增
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalParams {
    pub project_name: String,
    pub client_name: String,
    pub contract_amount: Option<String>,
    pub delivery_days: Option<i32>,
    pub warranty_years: Option<i32>,
    pub response_time: Option<String>,
    pub project_manager: Option<String>,
    pub qps: Option<i32>,
    pub concurrent_users: Option<i32>,
}

// Card 扩展
pub struct Card {
    // ... 现有字段 ...
    pub risk_flags: Vec<String>,        // 新增
}
```

```sql
-- db.rs Schema 变更
ALTER TABLE cards ADD COLUMN risk_flags TEXT NOT NULL DEFAULT '[]';

CREATE TABLE IF NOT EXISTS global_params (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

---

## 关键文件清单

| 文件 | 变更类型 |
|------|----------|
| `src-tauri/src/models.rs` | 新增 `GlobalParams`、`risk_flags` |
| `src-tauri/src/db.rs` | Schema 变更、新增 DAO 方法 |
| `src-tauri/src/ai.rs` | Prompt 改造、新 AI 方法 |
| `src-tauri/src/main.rs` | 注册新命令、调整现有命令签名 |
| `src/components/GenerateTab.tsx` | UI 改造、状态扩展、审核拦截 |

---

## 验收标准

- [ ] 技术卡片可一键批量确认，商务卡片一键确认被阻止并提示
- [ ] 商务卡片生成后显示风险标签（报价敏感/承诺风险等）
- [ ] 全局参数填写后，AI 生成时直接使用已知参数值，未提供的才留 `[PARAM:xxx]` 占位符
- [ ] 技术+商务卡片生成后，跨文档一致性检查能发现参数矛盾
- [ ] `cargo test` / `vitest run` / `vite build` 全绿

---

## 风险与缓解

| 风险 | 缓解 |
|------|------|
| AI prompt 改造后生成质量下降 | 保留原有 prompt 作为 fallback，通过 `doc_type` 条件分支控制 |
| 风险标签 JSON 解析失败 | 解析失败时 `risk_flags = vec![]`，不阻塞主流程 |
| DB schema 变更导致旧数据不兼容 | `risk_flags` 列有 DEFAULT '[]'；`global_params` 表是新增的，不影响旧数据 |
| 全局参数未填写时生成逻辑 | `global_params` 传 `null`，prompt 不注入全局参数段落，行为与 Phase 2 一致 |

---

## Phase 4 预览（模板库下线 + 知识库迁移）

> 以下为 Phase 3 完成后的大致方向，实施前需单独制定详细计划。

| 任务 | 说明 |
|------|------|
| 前端移除 LibraryTab | 当前 LibraryTab 仍在 App.tsx 中注册，需移除组件和路由 |
| 数据库迁移 | `templates` 表数据完整迁移到 `knowledge_assets` 表（或确认现有 `templates` 表已兼容） |
| 功能覆盖验证 | 确保原模板库的核心能力（搜索、插入）已由智能生成覆盖 |
| Tab 重命名 | `push` tab key 正式改为 `generate`（当前 App.tsx 中 `push` 已映射到 GenerateTab，但 key 未改） |

