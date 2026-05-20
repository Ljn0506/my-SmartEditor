# Smart-Editor 设计规格说明书 v3.0

| 项目 | 内容 |
|-----|------|
| 文档版本 | v3.0 |
| 创建日期 | 2026-04-30 |
| 最后更新 | 2026-04-30 |
| 文档状态 | 已定稿 |
| 目标用户 | 企业信息安全售前/商务团队 |
| 核心场景 | 需求分析 → 智能生成 → 人工确认 → 自动校对 → 知识沉淀 |

---

## 1. 项目概述

### 1.1 项目背景

信息安全行业的售前和商务团队在日常工作中需要频繁编写投标文件、技术方案、商务提案等文档。这些工作存在以下痛点：

- **重复劳动多**：相似项目需要重复编写大量相同内容
- **知识分散**：历史经验和优秀案例分散在 NAS 服务器各处（约 80GB、上万文件），难以快速检索
- **需求理解耗时**：客户需求文档（招标文件、需求规格书）篇幅长，人工提炼要点效率低
- **格式要求严**：投标文档对格式规范要求严格，偏离表需逐条核对，人工检查效率低且易遗漏废标项
- **时间压力大**：投标截止日期紧迫，需要快速产出高质量文档

### 1.2 项目目标

开发一款 Tauri 桌面端智能文档助手，实现：

1. **需求智能解析**：上传客户需求文件后自动提取技术要点和商务条款
2. **智能卡片生成**：基于解析结果和本地知识库，按章节生成卡片化内容；技术方案全自动，商务条款人机协作
3. **多文件一致性**：同时生成技术方案、商务响应等多份文档时，自动保持参数、承诺、人名一致
4. **一键规范检查**：文档组装完成后，自动检测偏离项、废标风险、格式规范、内容一致性（无需上传文件）
5. **知识持续沉淀**：新文档脱敏后可加入知识库，形成良性循环

### 1.3 目标用户

| 用户角色 | 核心需求 | 使用场景 |
|---------|---------|---------|
| 售前工程师 | 快速编写技术方案 | 投标技术方案编制 |
| 商务人员 | 规范商务文档、检查偏离项 | 投标文件商务部分编制 |
| 项目经理 | 统一文档标准、团队协作 | 项目文档管理和审核 |

---

## 2. 系统架构

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Tauri 桌面应用（统一窗口）                        │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                     React + TypeScript 前端界面                    │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐  │  │
│  │  │ 需求上传     │ │ 智能生成     │ │ 校对面板    │ │ 设置面板  │  │  │
│  │  │ • 拖放上传   │ │ • 章节卡片   │ │ • 本地文件  │ │ • AI配置  │  │  │
│  │  │ • 要点提取   │ │ • 一键确认   │ │   直接校对  │ │ • NAS路径 │  │  │
│  │  │ • 触发生成   │ │ • 逐张审核   │ │ • 预览定位  │ │ • SMB配置 │  │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘  │  │
│  │  ┌───────────────────────────────────────────────────────────┐  │  │
│  │  │     创作模式：TipTap 富文本编辑器                          │  │  │
│  │  │     （需求上传/智能生成/编辑组装时使用）                    │  │  │
│  │  │                                                             │  │  │
│  │  │     校对模式：文档预览区（只读）                            │  │  │
│  │  │     （校对Tab选择本地文件时使用）                           │  │  │
│  │  └───────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                     Tauri Rust 后端（内嵌）                        │  │
│  │  • NAS 文件扫描 & 解析（docx / pdf / xlsx / txt）                 │  │
│  │  • Meilisearch 全文索引管理                                       │  │
│  │  • SQLite 元数据 & 知识库数据库                                   │  │
│  │  • AI 客户端（本地 Ollama / 云端 API 自适应切换）                 │  │
│  │  • 文档脱敏引擎                                                   │  │
│  │  • 剪贴板桥接（向 Word/WPS 插入内容）                             │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │
              ┌─────────────────┼─────────────────┐
              ▼                 ▼                 ▼
         ┌─────────┐      ┌─────────┐      ┌───────────┐
         │   NAS   │      │ Ollama  │      │ 云端 API  │
         │ 共享盘  │      │(本地LLM)│      │(可选降级) │
         │ 80GB+   │      │         │      │           │
         └─────────┘      └─────────┘      └───────────┘
```

### 2.2 技术选型

| 层级 | 组件 | 选型 | 理由 |
|-----|------|------|------|
| **桌面框架** | 主框架 | Tauri (Rust + WebView) | 包体极小（< 10MB），原生跨平台（Win/Mac），内置文件系统权限 |
| **前端** | UI 框架 | React + TypeScript | 组件生态成熟，团队易维护 |
| **前端** | 富文本编辑器 | Tiptap (ProseMirror) | 专业级富文本，AI 指令生态成熟，易扩展 |
| **前端** | 样式 | Tailwind CSS | 快速构建统一风格界面 |
| **前端** | 状态管理 | Zustand（推荐）或 React Context | 全局状态共享（需求清单、卡片状态） |
| **后端** | Rust 后端 | Tauri Command + Tokio | 与前端深度集成，无需独立服务进程 |
| **检索** | 全文搜索引擎 | Meilisearch（内嵌进程） | 启动快、中文分词优秀、支持过滤/排序/高亮 |
| **数据** | 元数据存储 | SQLite | 零配置、单文件、跨平台 |
| **文档解析** | docx | `docx-rs` / `mammoth` | Rust/JS 原生解析，无需 Python 依赖 |
| **文档解析** | pdf | `pdf-extract` / `lopdf` | 提取纯文本用于索引和 AI 分析 |
| **文档解析** | xlsx | `calamine` / `xlsx-rs` | 提取表格文本，保留行列结构 |
| **AI** | 本地 LLM | Ollama + qwen2.5:14b / deepseek-r1:14b | 数据不出本地，隐私合规 |
| **AI** | 云端降级 | Claude API / DeepSeek API | 本地机器配置不足时自动/手动切换 |
| **插入 Word** | 桥接方案 | 富文本剪贴板 | 剪贴板兼容性最好 |

---

## 3. 核心工作流（v3.0 更新）

```
┌─────────────┐     ┌─────────────┐     ┌─────────────────────────────┐
│ 1.需求上传   │ ──► │ 2.要点提取   │ ──► │ 3.智能生成（章节卡片）       │
│   & 解析    │     │   (AI)      │     │   • 技术方案：全自动         │
└─────────────┘     └─────────────┘     │   • 商务响应：人机协作       │
                                        └─────────────┬───────────────┘
                                                      │
                                                      ▼
                                        ┌─────────────────────────────┐
                                        │ 4.卡片确认 & 组装            │
                                        │   • 技术卡片一键批量确认     │
                                        │   • 商务卡片逐张人工审核     │
                                        │   • 多文件一致性检查         │
                                        └─────────────┬───────────────┘
                                                      │
                                                      ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────────────────────┐
│ 7.知识沉淀   │ ◄── │ 6.导出完善   │ ◄── │ 5.Tiptap 编辑器编辑完善     │
│  (脱敏入库)  │     │ (.docx)      │     │   • 插入卡片               │
└─────────────┘     └─────────────┘     │   • 调整格式               │
                                        │   • 补充内容               │
                                        └─────────────────────────────┘
                                                      │
                                                      ▼
                                        ┌─────────────────────────────┐
                                        │ 8.统一校对（本地文件直接检查）│
                                        │   • 选择本地 .docx/.doc     │
                                        │   • 预览区定位              │
                                        │   • 回到 Word 修改          │
                                        └─────────────────────────────┘
```

**关键变更说明（相比 v2.x）**：
- 模板库（LibraryTab）不再作为独立前端模块，转为后台知识库数据源
- 智能推送（PushTab）升级为"智能生成"，承担核心内容产出职责
- 校对面板不导入编辑器，直接选择**本地 .docx/.doc 文件**执行检查，右侧为**只读预览区**
- 新增"多文件一致性检查"环节，确保同时生成的技术/商务文档参数对齐
- 全局参数表约束：AI 不生成参数，所有参数从知识库历史素材中提取

---

## 4. 功能模块详细设计

### 4.1 需求上传面板（UploadTab）

**定位**：工作流入口，解析招标文件并触发后续生成。

**功能点**：

| 功能 | 说明 | 优先级 |
|-----|------|-------|
| 文件拖放上传 | 支持 .docx / .doc / .pdf / .xlsx / .txt | P0 |
| 文档类型自动识别 | 技术文档 / 商务文档 / 混合文档 | P0 |
| AI 要点提取 | 自动提取技术要点、商务条款、资质要求、评分标准、承诺条款 | P0 |
| 要点编辑 | 用户可勾选/取消、编辑文字、添加自定义要点 | P0 |
| 触发智能生成 | 确认要点后，一键进入"智能生成"Tab 开始生成卡片 | P0 |
| 需求清单全局同步 | 提取的要点列表存入前端全局状态，供校对面板自动读取 | P0 |

**数据流**：
1. 用户拖入文件 → Rust 后端解析提取纯文本
2. 前端将文本发送至 AI 客户端 → 返回结构化 JSON（`ParsedRequirements`）
3. 前端渲染可编辑勾选列表
4. 用户确认后，需求清单写入全局状态（Zustand / Context）
5. 自动切换至"智能生成"Tab

### 4.2 智能生成面板（GenTab，替代原 PushTab + LibraryTab）

**定位**：核心工作区。基于需求要点和知识库，按章节生成卡片化内容。

**生成策略（基于知识库改写原则）**：

> **核心约束**：
> 1. **AI 不得凭空生成内容**。技术方案的每一个段落必须基于本地知识库中已有的历史素材进行改写、重组、适配。
> 2. **AI 不得编造参数**。所有具体数值（QPS、并发数、节点数、金额、工期等）必须从知识库历史素材中提取，或以占位符留空。
> 3. **知识库是唯一的素材来源**。如果知识库中没有与当前招标需求匹配的内容，系统必须提示"缺少素材"，而不是让 AI 硬编。

| 文档类型 | 内容来源 | AI 作用 | 确认方式 |
|---------|---------|---------|---------|
| 技术方案 | **本地知识库历史素材**（必填） | 根据招标需求匹配素材 → 改写客户名/场景 → 重组段落结构 → 填充参数占位符 | 一键批量确认（参数标黄待填） |
| 商务响应 | **本地知识库历史素材** + 需求文件 | 匹配商务条款模板 → 改写响应内容 → 标注偏离项 | 逐张人工审核（敏感条款高亮） |
| 实施计划 | **本地知识库历史素材** + 全局参数表 | 重组交付步骤 → 替换工期/节点参数 | 人工填充关键节点 |
| 多份同时 | 共享知识库素材池 | 分别匹配各自所需的素材 | 分别确认 |

**冷启动检查**：
- 生成前，系统先评估知识库中与当前需求相关的素材覆盖率
- 覆盖率 < 50% 时，提示"知识库素材不足，建议先补充以下类别的历史文档：xxx"
- 生成结果中标注每张卡片的内容来源（"本段落基于《xxx项目技术方案》改写"）

**参数填充流程**：
```
AI 生成章节草稿
    │
    ▼
正则扫描内容中的数值/参数位置
    │
    ▼
从知识库检索相关历史素材 ──→ 提取历史参数值
    │                              │
    ▼                              ▼
内容中的参数替换为占位符 [PARAM:qps]   参数来源标注
    │
    ▼
前端展示：描述文字正常显示，占位符处显示"待填参数"卡片
    │
    ▼
用户点击占位符 → 选择"使用历史值 xxx"或"手动输入新值"
    │
    ▼
所有参数填满后，卡片状态变为"可确认"
```

**卡片结构**：

```typescript
interface ChapterCard {
  id: string;                    // 唯一标识
  chapter: string;               // 章节编号，如 "3.1"
  title: string;                 // 章节标题
  content: string;               // 生成的正文（Markdown）
  sourceRefs: string[];          // 引用的知识库素材来源
  documentTarget: string;        // 目标文档（技术方案 / 商务响应 / 实施计划）
  status: "draft" | "confirmed" | "rejected";
  generatedBy: "ai" | "manual" | "template";
  relatedCards: string[];        // 关联卡片（用于一致性检查）
  riskFlags?: string[];          // 风险提示（如"报价敏感"、"承诺风险"）
  paramPlaceholders?: ParamPlaceholder[]; // 参数占位符列表（AI 不生成参数）
}

interface ParamPlaceholder {
  key: string;                   // 参数名，如 "qps"
  label: string;                 // 显示名称，如 "系统 QPS"
  sourceValue?: string;          // 从知识库提取的原始值
  filledValue?: string;          // 用户确认/修改后的值
  status: "pending" | "filled_from_kb" | "filled_manual" | "mismatch";
  kbSource?: string;             // 来源知识库文件
}
```

**UI 布局**：

```
┌─────────────────────────────────────────────────────────────┐
│ 智能生成                                                     │
├─────────────────────────────────────────────────────────────┤
│  📄 待生成文档                                               │
│  ☑ 技术方案.docx     [生成进度: 8/8 章节]                   │
│  ☑ 商务响应.docx     [生成进度: 5/5 章节] ⚠️ 待审核         │
│  ☐ 实施计划.docx（可选）                                     │
├─────────────────────────────────────────────────────────────┤
│  📋 章节卡片（技术方案）                                      │
│  ├─ 3.1 系统架构 [知识库改写] ✓  [预览] [编辑] [换素材]     │
│  │   来源：《某银行防火墙部署方案》《等保三级通用方案》       │
│  ├─ 3.2 安全设计 [知识库改写] ✓  [预览] [编辑] [换素材]     │
│  │   来源：《某政府安全设计方案》                             │
│  ├─ 3.3 实施方案 [素材不足] ⚠️                               │
│  │   知识库无匹配素材，请手动补充或跳过                       │
│  └─ ...                                                      │
│  [一键确认全部无待定卡片]                                     │
├─────────────────────────────────────────────────────────────┤
│  📋 章节卡片（商务响应） ⚠️ 需人工审核                         │
│  ├─ 2.1 报价清单 [知识库改写] ⚠️  [预览] [编辑] [确认] [拒绝]│
│  ├─ 2.2 服务承诺 [知识库改写] ⚠️  [预览] [编辑] [确认] [拒绝]│
│  └─ ...                                                      │
├─────────────────────────────────────────────────────────────┤
│  [一致性检查] [进入编辑器]                                    │
└─────────────────────────────────────────────────────────────┘
```

**参数占位符 UI**：

```
┌─────────────────────────────────────────────────────────────┐
│ 3.1 系统架构 [知识库改写]                                    │
│  来源：《某银行防火墙部署方案》                               │
│                                                              │
│  本系统采用分布式微服务架构，通过负载均衡实现高可用...        │
│  系统峰值处理能力达到 [PARAM:qps] QPS，部署 [PARAM:node_count]│
│  台服务器，存储容量 [PARAM:storage] TB...                     │
│                                                              │
│  📋 待填参数：                                                │
│  ├─ QPS: [使用历史值 10,000 ▼] [手动输入]                   │
│  ├─ 节点数: [使用历史值 8 ▼] [手动输入]                      │
│  └─ 存储容量: [知识库无匹配] [手动输入] ⚠️                    │
│                                                              │
│  [预览] [编辑] [换素材] [参数填满后确认]                     │
└─────────────────────────────────────────────────────────────┘
```

**关键交互**：
- 点击"预览" → 右侧编辑器显示该卡片内容（只读或轻量编辑）
- 点击"换素材" → AI 基于不同的知识库历史素材重新改写本章节
- 参数占位符必须全部填满（从知识库选择或手动输入）后，卡片才可确认
- 技术卡片"一键确认"仅对**无待定参数且素材充足**的卡片生效
- 商务卡片必须逐张确认，带风险标签的卡片强制要求人工查看
- 素材不足的卡片显示"[素材不足]"，提示用户补充知识库或手动编写

### 4.3 全局参数表（多文件一致性核心）

在生成阶段维护一套全局 Key-Value 参数，所有卡片生成时引用：

```typescript
interface GlobalParams {
  projectName: string;           // 项目名称
  clientName: string;            // 客户名称
  contractAmount?: string;       // 合同金额
  deliveryDays: number;          // 交付工期（天）
  warrantyYears: number;         // 维保年限
  responseTime: string;          // 响应时间承诺
  projectManager: string;        // 项目经理
  qps?: number;                  // 性能指标
  concurrentUsers?: number;      // 并发用户数
  // ... 其他关键参数
}
```

**参数来源与填充规则**：

1. **知识库优先**：生成卡片时，系统先从知识库检索与当前章节最相关的历史素材，提取其中的参数值作为默认值
2. **全局参数表兜底**：若知识库无匹配参数，使用全局参数表中的值（如工期、维保年限等用户预设值）
3. **强制留空**：若以上均无匹配，AI 必须在内容中使用占位符 `[PARAM:xxx]`，**不得编造数值**
4. **用户终审**：所有参数值（包括从知识库提取的）在卡片确认前必须由用户人工确认或修改

**一致性检查时机**：
1. 生成时：参数从知识库提取后，自动注入全局参数表，确保跨文档一致
2. 确认后：组装前走 `check_cross_document_consistency()` 命令
3. 校对时：最终文档再次校验

### 4.4 校对面板（ReviewTab，v3.0 重大更新）

**定位**：对**本地已完成的投标文档**进行统一检查。不导入编辑器，直接选择本地 `.docx`/`.doc`/`.pdf` 文件执行校对。

**触发方式**：手动触发（不实时）。

**工作流程**：
```
在 Word/WPS 中完成投标文档
           │
           ▼
打开智能助手 → 校对 Tab
           │
           ▼
选择本地 .docx/.doc 文件 → 解析为纯文本预览
           │
           ▼
点击 [检查投标文件]
           │
           ▼
左侧显示分类检查结果
           │
           ▼
点击问题项 → 右侧预览区滚动定位 + 高亮段落
           │
           ▼
回到 Word 中修改对应位置
           │
           ▼
重新选择文件 → 再次检查
```

**检查维度**：

| 分类 | 检查项 | 自动修复 | 实现策略 |
|------|--------|----------|----------|
| **偏离风险** | 未响应、正/轻微/重大偏离 | 否 | AI（需求列表 vs 解析文本语义匹配） |
| **废标风险** | 资质缺失、承诺缺失、有效期问题 | 否 | AI + 关键词规则 |
| **内容一致性** | 自相矛盾、上下文逻辑断裂 | 否 | AI（跨段落语义分析） |
| **内容质量** | 重复段落 | 否 | 本地算法（simhash / 编辑距离） |
| **内容质量** | 敏感信息未脱敏（客户姓名、IP、内部人名） | 否 | 本地正则 + 关键词库 |
| **格式问题** | 标点符号（中文标点规范） | 否（仅提示，不改文件） | 本地正则 |
| **格式问题** | 空白占位符（`[待补充]`、`XXX`、`________`） | 否（仅提示，不改文件） | 本地正则 |

> **注意**：校对模式**不修改本地文件**，只提供检查结果和定位。所有修改需用户回到 Word/WPS 中手动完成。

**数据来源**：
- **偏离检查**：自动读取「需求上传」面板已解析的需求清单（前端全局状态）
- **自查检查**：读取解析后的投标文档纯文本
- **无已解析需求时**：显示提示"请先前往「需求上传」解析招标文件"

**界面布局（校对模式）**：

```
┌─────────────────────────────────────────────────────────────┐
│ [📎 需求上传] [✨ 智能生成] [🔍 校对] [⚙️ 设置]              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────┐  ┌─────────────────────────────┐ │
│  │  校对结果面板         │  │  文档预览区（只读）          │ │
│  │                     │  │                             │ │
│  │  [选择文件]          │  │  3.1 系统架构               │ │
│  │  demo_bid.docx       │  │  ─────────────              │ │
│  │  [检查投标文件]      │  │  本系统采用分布式架构...    │ │
│  │                     │  │                             │ │
│  │  🔴 偏离风险 (12)    │  │  3.2 安全设计               │ │
│  │    ├─ 未响应：第3.2条│  │  ─────────────              │ │
│  │       [定位] [片段]  │  │  通过多层次安全防护...      │ │
│  │    └─ 正偏离：第5.1条│  │                             │ │
│  │       [定位] [片段]  │  │  [高亮段落]                 │ │
│  │                     │  │  部署 [PARAM:node_count]... │ │
│  │  🔴 废标风险 (2)     │  │                             │ │
│  │    ├─ 资质缺失       │  │                             │ │
│  │       [定位]         │  │                             │ │
│  │                     │  │                             │ │
│  │  ⚠️ 内容一致性 (3)   │  │                             │ │
│  │  📝 内容质量 (5)     │  │                             │ │
│  │  🔧 格式问题 (8)     │  │                             │ │
│  │                     │  │                             │ │
│  │  [导出报告]          │  │                             │ │
│  └──────────────────────┘  └─────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**定位与高亮交互**：
- 点击任意问题项的 **[定位]** → 右侧预览区自动滚动到对应段落 + **黄底高亮**
- 点击 **[片段]** → 展开显示原文片段（无需滚动即可查看上下文）
- 格式问题仅**提示**，不自动修改文件（用户需回 Word 修改）
- 废标风险**置顶红色高亮**

**解析与定位精度**：
- `.docx` → `docx-rs` 解析为段落列表，保留段落索引
- `.doc` → 提示"建议转换为 .docx 以获得最佳定位体验"，或用基础文本提取
- `.pdf` → `pdf-extract` 提取文本，段落边界可能不准确，定位精度下降
- 定位精度为**段落级**（非字符级），因为解析后的纯文本与 Word 原始排版存在差异

### 4.5 Tiptap 编辑器

**定位**：内容展示和编辑核心区。

**功能点**：
- 接收并渲染卡片内容（Markdown → 富文本）
- 支持段落级高亮（校对定位时使用）
- 支持卡片拖拽插入
- 支持标准富文本编辑（增删改、调格式）
- 底部操作栏：复制到剪贴板、字符统计

---

## 5. 后端接口设计（Rust Commands）

### 5.1 需求解析（已有，保留）

```rust
#[tauri::command]
fn parse_requirement_file(file_path: String) -> Result<ParsedDocument, String>

#[tauri::command]
async fn parse_and_extract(
    state: tauri::State<'_, AppState>,
    file_path: String,
) -> Result<ParsedRequirements, String>
```

### 5.2 偏离检查（本地文件模式）

```rust
// 前端先调用 parse_document(file_path) 获取文本并显示预览
// 点击检查时，将解析后的文本传给偏离检查命令
#[tauri::command]
async fn check_deviation(
    state: tauri::State<'_, AppState>,
    req_items: Vec<RequirementItem>,
    bid_text: String,
) -> Result<DeviationReport, String>
```

**调用流程**：
1. 用户选择本地 `.docx`/`.doc` 文件
2. 前端调用 `parse_document(file_path)` → 获取纯文本 + 段落结构
3. 纯文本在右侧预览区渲染
4. 用户点击"检查投标文件"
5. 前端将 `bid_text` + `req_items`（全局状态中的需求清单）传给 `check_deviation`
6. 返回偏离报告（含段落索引，用于预览区定位）

### 5.3 废标风险检查（已有，保留）

```rust
#[tauri::command]
async fn check_fatal_risks_text(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<Vec<FatalRisk>, String>
```

### 5.4 新增：投标文件自查

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

**内部子检查函数**：
- `check_contradictions(text)` — AI 检测自相矛盾
- `check_context_logic(text)` — AI 检测上下文逻辑断裂
- `check_repetition(text)` — 本地算法检测重复段落（simhash / 编辑距离）
- `check_sensitive_info(text)` — 本地正则检测敏感信息
- `check_placeholders(text)` — 本地正则检测空白占位符
- `check_punctuation(text)` — 已有，复用

### 5.5 新增：跨文档一致性检查

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub field: String,          // 参数名，如 "responseTime"
    pub expected: String,       // 全局参数表中的值
    pub found: String,          // 实际在文档中找到的值
    pub location: String,       // 所在文档和章节
}

#[tauri::command]
async fn check_cross_document_consistency(
    state: tauri::State<'_, AppState>,
    global_params: GlobalParams,
    documents: Vec<DocumentSlice>,  // 各文档的章节切片
) -> Result<Vec<ConsistencyIssue>, String>
```

### 5.6 新增：章节卡片生成（AI 不生成参数）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCardRequest {
    pub requirements: ParsedRequirements,
    pub document_target: String,      // "technical" | "business" | "implementation"
    pub chapter_outline: Vec<String>, // 章节大纲
    pub global_params: GlobalParams,
    pub search_results: Vec<SearchResult>, // 知识库检索结果
}

#[tauri::command]
async fn generate_chapter_cards(
    state: tauri::State<'_, AppState>,
    request: GenerateCardRequest,
) -> Result<Vec<ChapterCard>, String>
```

**AI Prompt 设计（技术方案章节）**：

```
你是一位资深信息安全售前工程师。你的任务是基于公司历史项目素材，为当前招标需求改写技术方案段落。

## 核心约束（必须遵守）
1. **严禁凭空生成内容**。你只能基于"参考资料"中的历史素材进行改写、重组、适配。
2. 如果参考资料中完全没有与当前章节相关的内容，输出"[素材不足：本章节缺少知识库支撑]"，不得硬编。
3. **严禁编造任何具体参数**（QPS、并发数、节点数、存储容量、带宽、金额、工期等）。
4. 遇到需要填写参数的地方，使用占位符格式：[PARAM:参数名]。所有参数必须从参考资料中提取。

## 改写规则
- 替换客户名称和项目场景（使用当前招标需求的客户名）
- 保留技术架构、实现思路、产品选型等核心内容
- 调整段落结构以匹配当前章节的主题
- 参数使用占位符，如："系统峰值处理能力达到 [PARAM:qps] QPS"

## 章节主题
{chapter_title}

## 需求要点（当前招标项目）
{requirements}

## 参考资料（来自本地知识库的历史素材，唯一内容来源）
{search_results}

## 输出要求
1. 使用 Markdown 格式，标题层级清晰
2. 在内容末尾标注来源："本段落基于《xxx》改写"
3. 参数处使用 [PARAM:xxx] 占位符
4. 每个章节至少 200 字（素材不足时除外）

请直接输出文档内容。
```

**参数提取与填充（Rust 后端）**：

```rust
/// 从 AI 生成的内容中提取参数占位符
pub fn extract_param_placeholders(content: &str) -> Vec<ParamPlaceholder> {
    let re = regex::Regex::new(r"\[PARAM:([a-zA-Z_]+)\]").unwrap();
    re.captures_iter(content)
        .map(|cap| ParamPlaceholder {
            key: cap[1].to_string(),
            label: map_param_key_to_label(&cap[1]),
            source_value: None,
            filled_value: None,
            status: ParamStatus::Pending,
            kb_source: None,
        })
        .collect()
}

/// 从知识库素材中提取参数值
pub fn extract_params_from_kb(
    kb_content: &str,
    param_keys: &[String],
) -> HashMap<String, String> {
    // 基于正则规则从知识库文本中提取参数
    // 例如：从 "系统支持 10000 QPS 并发" 中提取 qps=10000
}
```

### 5.7 知识库检索（已有，保留）

```rust
#[tauri::command]
async fn search_knowledge_base(
    state: tauri::State<'_, AppState>,
    query: String,
    filters: Option<SearchFilters>,
) -> Result<Vec<SearchResult>, String>
```

---

## 6. 前端状态设计（Zustand 推荐）

```typescript
interface AppStore {
  // 需求解析结果（全局共享）
  parsedRequirements: ParsedRequirements | null;
  setParsedRequirements: (req: ParsedRequirements) => void;

  // 章节卡片
  chapterCards: ChapterCard[];
  setChapterCards: (cards: ChapterCard[]) => void;
  updateCardStatus: (id: string, status: ChapterCard['status']) => void;
  updateCardContent: (id: string, content: string) => void;

  // 全局参数表
  globalParams: GlobalParams | null;
  setGlobalParams: (params: GlobalParams) => void;

  // 校对结果（缓存）
  reviewReport: SelfReviewReport | null;
  setReviewReport: (report: SelfReviewReport) => void;
  clearReviewReport: () => void;
}
```

---

## 7. 数据模型

### 7.1 SQLite 表结构

保留 `templates` 表作为知识库数据源，但前端不再以"模板库"形式展示：

```sql
-- 知识库素材表（原 templates 表）
CREATE TABLE knowledge_assets (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    content_html TEXT,
    doc_attr TEXT,
    business_domain TEXT,
    security_layer TEXT,
    content_module TEXT,
    project_phase TEXT,
    tags TEXT,
    source_file TEXT,
    source_para_range TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    use_count INTEGER DEFAULT 0,
    rating INTEGER DEFAULT 0
);

-- AI 配置表（已有，保留）
CREATE TABLE ai_config (
    id INTEGER PRIMARY KEY,
    provider TEXT,
    base_url TEXT,
    api_key TEXT,
    model TEXT,
    updated_at TIMESTAMP
);
```

---

## 8. 知识库 NAS 目录架构（保留 v1.0 规范）

详见 `2026-04-27-knowledge-base-framework.md`，本版本无变更。

**核心原则**：
- 四级分类，三级目录
- 文件名即元数据
- 不改内容，只改位置

```
/Volumes/NAS/知识库/
├── 00-通用素材/
├── 01-方案阶段/
├── 02-投标阶段/
└── 03-合同阶段/
```

---

## 9. 非功能需求

### 9.1 性能要求

| 指标 | 要求 | 说明 |
|-----|------|------|
| 应用启动时间 | ≤ 3 秒 | 从双击到界面可用 |
| 需求文件解析 | ≤ 5 秒 | 50MB 以内文档完成解析和要点提取 |
| 章节卡片生成 | ≤ 15 秒/张 | AI 生成单章内容 |
| 文档检查速度 | ≤ 5 秒/万字 | 校对检查完成时间 |
| 知识库检索 | ≤ 2 秒 | 返回搜索结果（80GB 知识库） |

### 9.2 安全性要求

- **默认本地优先**：所有数据处理优先在本地完成，NAS 文件不离开内网
- **AI 隐私模式**：默认调用本地 Ollama，数据不上传云端
- **脱敏保障**：原始客户资料在本地处理，脱敏后才进入知识库索引
- **无痕处理**：上传的需求文件解析后仅保留提取的要点文本，不存储原始文件

### 9.4 NAS/SMB 接入配置（v3.0 新增）

**定位**：支持局域网内 SMB 共享盘作为知识库数据源，应用内配置连接信息，自动挂载/连接，按需下载，本地索引。

**连接方式**：

| 模式 | 说明 | 适用场景 |
|-----|------|---------|
| **SMB 自动挂载** | 应用内配置服务器地址、共享名、用户名、密码，启动时自动触发 OS 挂载 | 推荐，首次配置后无感使用 |
| **本地挂载路径** | NAS 已由 OS 挂载到本地路径（如 `/Volumes/NAS/` 或 `Z:\`），应用直接访问 | 已有 IT 部门统一挂载的环境 |

**配置项（设置面板）**：

```
┌─────────────────────────────────────────┐
│ NAS 知识库配置                           │
├─────────────────────────────────────────┤
│ 连接方式： ● SMB自动挂载  ○ 本地路径     │
│                                         │
│ SMB 服务器地址： 192.168.1.100           │
│ 共享名称：      知识库                    │
│ 用户名：        smarteditor              │
│ 密码：          ********                 │
│ 挂载路径：      /Volumes/NAS-知识库      │
│                                         │
│ [测试连接] [保存配置]                    │
│                                         │
│ 本地缓存路径： ~/.smart-editor/cache/    │
│ 索引状态：      已索引 12,456 个文件     │
│ [重新索引] [清理缓存]                    │
└─────────────────────────────────────────┘
```

**按需下载策略**：

```
首次配置 NAS
    │
    ▼
扫描 NAS 目录结构 ──→ 仅下载元数据（文件名、路径、大小、修改时间、四维分类推断）
    │                      │
    ▼                      ▼
本地 SQLite 元数据库      本地 Meilisearch 索引（文件名 + 路径 + 推断的分类标签）
    │
    ▼
用户搜索/命中知识库文件
    │
    ▼
按需下载文件内容到本地缓存（~/.smart-editor/cache/）
    │
    ▼
解析文件内容 → 全文入 Meilisearch → AI 生成时作为参考素材
```

**关键规则**：
- 不一次性下载全部 80GB，仅下载目录结构和命中文件
- 缓存文件按 LRU 策略清理（保留最近 30 天访问的文件，上限 5GB）
- 文件内容解析后，全文索引存入本地 Meilisearch，原始文件可清理
- NAS 文件修改时间（mtime）变化时，触发本地缓存失效和重新下载

**读写支持**：
- **读**：按需下载到本地缓存 → 解析 → 索引
- **写**：脱敏入库时，将新素材上传回 NAS 对应目录（如 `00-通用素材/待分类/`），由责任人定期整理

**技术实现（Rust）**：

```rust
// 新增 nas_mount.rs 模块
pub struct NasConfig {
    pub mode: NasMode,           // SmbAutoMount | LocalPath
    pub server: Option<String>,  // SMB 服务器地址，如 "192.168.1.100"
    pub share: Option<String>,   // 共享名，如 "知识库"
    pub username: Option<String>,
    pub password: Option<String>,
    pub mount_point: String,     // 本地挂载点，如 "/Volumes/NAS-知识库"
    pub cache_dir: String,       // 本地缓存目录
}

pub enum NasMode {
    SmbAutoMount,
    LocalPath,
}

/// 测试 SMB 连接是否可达
pub fn test_smb_connection(config: &NasConfig) -> Result<bool>

/// 触发 OS 挂载（macOS: mount_smbfs, Windows: net use）
pub fn mount_smb_share(config: &NasConfig) -> Result<()>

/// 扫描 NAS 目录，返回文件元数据列表（不下载内容）
pub fn scan_nas_metadata(mount_point: &str) -> Result<Vec<FileMetadata>>

/// 按需下载文件到本地缓存
pub fn download_to_cache(remote_path: &str, cache_dir: &str) -> Result<String> // 返回本地缓存路径

/// 上传脱敏后的素材回 NAS
pub fn upload_to_nas(local_path: &str, remote_dir: &str) -> Result<()>
```

**权限要求**：
- macOS: 应用需要 `com.apple.security.files.user-selected.read-write`（用户选择文件夹读写）或全磁盘访问权限（若自动挂载到 `/Volumes/`）
- Windows: 应用需要以当前用户权限执行 `net use`
- Tauri v2: 若使用 `fs:allow-read` 访问挂载点，需在 `capabilities/default.json` 中添加对应路径权限（当前最小权限配置需扩展）

### 9.5 AI 自适应策略

| 场景 | 策略 |
|-----|------|
| 本地 Ollama 可用且响应正常 | 优先使用本地模型 |
| 本地 Ollama 未安装或模型未下载 | 提示用户安装，或询问是否切换到云端 |
| 本地 Ollama 响应超时（> 60 秒） | 自动降级到云端 API，并提示用户 |
| 生成超长文档（> 4000 token 输出） | 本地模型可能效果差，建议切换云端 |

---

## 10. 实施计划

### Phase 1：校对功能（进行中）
- [ ] 前端：重构 CheckTab（分类面板、定位、高亮、一键修复）
- [ ] 后端：修改偏离检查数据源（接受 reqItems + text）
- [ ] 后端：新增 `check_self_review` 及子检查函数
- [ ] 前端：招标需求全局状态同步
- [ ] 前端：一键修复 + 导出报告

### Phase 2：智能生成 MVP
- [ ] 前端：新建 GenTab（章节卡片列表、预览、确认交互）
- [ ] 后端：新增 `generate_chapter_cards` 命令
- [ ] 后端：实现全局参数表注入
- [ ] 前端：Zustand 全局状态接入
- [ ] 前端：Tiptap 卡片插入支持

### Phase 3：多文件一致性 + 商务人机协作
- [ ] 后端：新增 `check_cross_document_consistency`
- [ ] 前端：多文件并行生成 UI
- [ ] 前端：商务卡片强制审核流程
- [ ] 前端：风险标签高亮

### Phase 4：模板库下线 + 知识库迁移
- [ ] 前端：移除 LibraryTab
- [ ] 后端：确认 templates 表数据完整迁移到 knowledge_assets
- [ ] 前端：确保所有原模板库功能由智能生成覆盖

---

## 11. 附录

### 11.1 变更记录

| 版本 | 日期 | 变更内容 | 作者 |
|-----|------|---------|------|
| v3.0 | 2026-04-30 | 重大重构：引入智能生成工作流，校对面板去文件上传，模板库转为后台知识库 | planner_alice |
| v2.1 | 2026-04-21 | 初始版本，定义系统架构、功能模块、数据流 | - |
| v1.0 | 2026-04-27 | 知识库 NAS 目录规范 | - |

### 11.2 配套文档

- `2026-04-27-knowledge-base-framework.md` — NAS 知识库目录规范（仍有效）
- `docs/plan-check-feature.md` — 校对功能详细设计（Phase 1）
- `docs/plan-workflow-redesign.md` — 工作流重塑规划（v3.0 决策记录）
