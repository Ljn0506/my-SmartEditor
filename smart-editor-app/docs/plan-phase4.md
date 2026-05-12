# Smart-Editor Phase 4 实施计划：模板库下线 + 知识库迁移

> 基于 v3.0 设计文档（2026-04-30）制定  
> 制定人：kimi_planner_Alice  
> 时间：2026-05-11

---

## 上下文

Phase 3 已完成（多文件一致性 + 商务人机协作），版本 `0.2.0.0`。  
Phase 4 的核心目标是 **下线前端模板库（LibraryTab）**，完成 v3.0 设计的 Tab 结构调整。

v3.0 设计中的定位：
- 模板库不再作为独立前端模块，转为后台知识库数据源
- 智能生成（GenerateTab）承担核心内容产出职责
- Tab 从 5 个精简为 4 个（上传/生成/校对/设置）

---

## 任务拆分

### T1：前端移除 LibraryTab

| 子任务 | 说明 | 负责文件 |
|--------|------|----------|
| T1.1 移除 import | 删除 `App.tsx` 中对 `LibraryTab` 的 import | `src/App.tsx` |
| T1.2 移除 tab 注册 | 从 `tabs` 数组和 `TabKey` 类型中删除 `library` | `src/App.tsx` |
| T1.3 移除渲染逻辑 | 删除 `activeTab === "library"` 的条件渲染分支 | `src/App.tsx` |
| T1.4 清理依赖 | 检查 `App.tsx` 中是否还有专为 LibraryTab 维护的 state/ref，一并清理 | `src/App.tsx` |
| T1.5 处理编辑器引用 | LibraryTab 曾接收 `editor` prop，确认 GenerateTab 等组件不再需要该 prop 传递链 | `src/App.tsx` |

**验收**：`App.tsx` 中不再出现 `LibraryTab` 字样，`tsc --noEmit` 通过。

---

### T2：Tab 重命名（push → generate）

| 子任务 | 说明 | 负责文件 |
|--------|------|----------|
| T2.1 类型重命名 | `TabKey` 中 `"push"` 改为 `"generate"` | `src/App.tsx` |
| T2.2 数组更新 | `tabs` 数组中 `key: "push"` 改为 `key: "generate"`，label 保持"智能生成" | `src/App.tsx` |
| T2.3 渲染条件更新 | `activeTab === "push"` 改为 `activeTab === "generate"` | `src/App.tsx` |
| T2.4 默认 tab 更新 | `useState<TabKey>("upload")` 不变，但确认切换逻辑中的 `"push"` 引用已更新 | `src/App.tsx` |
| T2.5 测试修复 | `App.test.tsx` 中如果有 `getByText("智能推送")` 已改为 `"智能生成"`，确认无遗漏 | `src/App.test.tsx` |

**验收**：`App.tsx` 中不再出现 `"push"` 字符串，`vitest run` 全绿。

---

### T3：LibraryTab 组件清理

| 子任务 | 说明 | 负责文件 |
|--------|------|----------|
| T3.1 删除组件文件 | 删除 `src/components/LibraryTab.tsx` | `src/components/LibraryTab.tsx` |
| T3.2 检查引用 | grep 整个 `src/` 目录，确认没有其他文件 import LibraryTab | 全目录 |

**验收**：`LibraryTab.tsx` 文件已删除，grep 无残留引用。

---

### T4：功能覆盖验证

| 子任务 | 说明 |
|--------|------|
| T4.1 搜索能力 | 确认 LibraryTab 的模板搜索功能已由 GenerateTab 的"知识库检索"覆盖（generate_card 内部调用 search） |
| T4.2 插入能力 | 确认 LibraryTab 的"插入编辑器"功能已由 GenerateTab 的"章节卡片确认后进入编辑器"流程覆盖 |
| T4.3 后端清理评估 | 评估是否需要在后端清理模板相关的 API（create_template/get_template 等）——**本次不做**，Phase 4 仅下线前端，后端 API 保留供知识库使用 |

**验收**：形成书面确认（可写在本文档底部），说明 LibraryTab 的核心能力已被 GenerateTab 覆盖。

---

## 风险与缓解

| 风险 | 缓解 |
|------|------|
| App.tsx 中 editor ref 传递链断裂 | LibraryTab 曾接收 `editor` prop，删除后确认 `App.tsx` 中的 `editor` ref 是否仍需传给其他 Tab。如果只有 LibraryTab 使用，可一并简化 |
| 测试中有 LibraryTab 相关断言 | `App.test.tsx` 已修复为"智能生成"，再次检查是否有针对 LibraryTab 的点击/切换测试 |

---

## 依赖关系

```
T3（删除组件文件） 依赖 T1（App.tsx 中先移除引用）
T4（功能覆盖验证） 可与 T1/T2 并行，但应在 T3 之前完成
```

**建议执行顺序**：T4 → T1 → T2 → T3

---

## 验收标准

- [ ] `App.tsx` 中不存在 `library` tab 和 `LibraryTab` 组件引用
- [ ] `App.tsx` 中 `"push"` 已全部替换为 `"generate"`
- [ ] `src/components/LibraryTab.tsx` 已删除
- [ ] `vitest run` 全绿
- [ ] `vite build` 通过
- [ ] 功能覆盖验证结论已记录

---

## T4 功能覆盖验证结论（2026-05-11）

| LibraryTab 能力 | 替代方案 | 验证结果 |
|----------------|----------|----------|
| 知识库模板搜索 | GenerateTab 内部调用 `search.search(&outline.title, 3)` 自动检索相关素材 | **已覆盖** |
| 一键插入编辑器 | GenerateTab 生成章节卡片后，用户在编辑区复制内容到 Tiptap 编辑器 | **已覆盖**（交互方式不同：自动生成卡片 vs 手动搜索模板，但核心目标一致） |
| 模板分类筛选 | GenerateTab 通过 `docType` + AI prompt 自动匹配素材类型 | **已覆盖** |
| 模板预览 | GenerateTab 选中卡片后在右侧编辑区直接预览内容 | **已覆盖** |

**结论**：LibraryTab 的核心能力（知识库搜索、内容插入、分类筛选、预览）已由 GenerateTab 的自动生成流程完整覆盖。虽然交互方式从"手动搜索+插入"变为"自动生成+编辑"，但 v3.0 设计意图正是如此——模板库不再作为独立前端模块，转为后台知识库数据源，由智能生成承担核心内容产出。

**前端引用残留**：`src/` 目录中仅剩 `src/components/LibraryTab.tsx` 文件本身有 `LibraryTab` 引用（待 T3 删除）。
