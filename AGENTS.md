# smart-editor — 多 Agent 协作指南（agtalk）

本项目使用 **agtalk** 在 Zellij 多 pane 环境中进行多 Agent 协作开发。

---

## Agent 分工

| Agent | 角色 | 职责 | 工作目录 |
|-------|------|------|----------|
| `kimi_coder_Xin` | coder | 全栈开发实现（前后端功能编码） | `~/smart-editor` |
| `kimi_coder_Bob` | coder | 前端/React 组件与交互开发 | `~/smart-editor` |
| `kimi_backend_Frank` | backend | Rust/Tauri 后端、数据库、AI 接口 | `~/smart-editor` |
| `kimi_reviewer_Alice` | reviewer | 代码审查、重构建议、质量把关 | `~/smart-editor` |
| `kimi_tester_Baekhyun` | tester | 测试用例、端到端验证、QA | `~/smart-editor` |

> **命名规则**: `{tool}_{role}_{name}`，名字从预设列表选取，注册前执行 `agtalk list` 查重。

---

## 快速启动（在新 pane 中）

```bash
# Pane 2: 前端 Coder
cd ~/smart-editor
agtalk register kimi_coder_Bob --role coder --capabilities "react,typescript,tiptap,tailwind,vite" --bio "smart-editor 前端开发，负责 React 组件与编辑器交互"
kimi

# Pane 3: 后端 Developer
cd ~/smart-editor
agtalk register kimi_backend_Frank --role backend --capabilities "rust,tauri,sqlite,async" --bio "smart-editor 后端开发，负责 Tauri 命令与数据处理"
kimi

# Pane 4: Reviewer
cd ~/smart-editor
agtalk register kimi_reviewer_Alice --role reviewer --capabilities "rust,code-review,refactor,architecture" --bio "smart-editor 代码审查与重构专家，专注 Tauri/Rust"
kimi

# Pane 5: 测试 Agent
cd ~/smart-editor
agtalk register kimi_tester_Baekhyun --role tester --capabilities "testing,code-review,qa,vitest" --bio "smart-editor 测试与审查，负责用例与质量把关"
kimi
```

---

## 常用协作命令

### 查看在线状态
```bash
agtalk list --view
```

### 任务分发示例
```bash
# 前端任务
agtalk send kimi_coder_Bob '[TASK] 为 TipTap 编辑器添加工具栏悬浮菜单' --priority 3

# 后端任务
agtalk send kimi_backend_Frank '[TASK] 在 db.rs 中添加文档全文索引接口' --priority 3

# 审查任务
agtalk send kimi_reviewer_Alice '[TASK] 审查 Frank 的 db.rs 修改' --priority 4

# 测试任务
agtalk send kimi_tester_Baekhyun '[TASK] 补充 db.rs 的单元测试' --priority 5
```

### 广播通知
```bash
# 广播信息（不占用 inbox）
agtalk notify all '[INFO] 准备重构 search.rs 接口，各 Agent 暂停相关修改'

# 广播任务（所有 Agent 都收到）
agtalk broadcast '[TASK] 紧急：修复 build 报错' --priority 1
```

### 查看 Inbox 与标记完成
```bash
agtalk inbox <your_name> --view   # 查看消息
agtalk done <msg_id>              # 标记完成
```

---

## 项目架构速览

```
smart-editor-app/
├── src/                    # React 前端
│   ├── App.tsx            # 主应用
│   ├── components/        # UI 组件
│   ├── pages/             # 页面
│   └── utils/             # 工具函数
├── src-tauri/src/         # Rust 后端
│   ├── main.rs            # Tauri 入口
│   ├── ai.rs              # AI API 调用
│   ├── db.rs              # SQLite 数据库
│   ├── deviation.rs       # 偏差检测
│   ├── desensitize.rs     # 脱敏处理
│   ├── nas_scanner.rs     # NAS 文件扫描
│   ├── punctuation.rs     # 标点处理
│   ├── search.rs          # Meilisearch 搜索
│   └── config.rs          # 配置管理
└── docs/                  # 项目文档
```

### 技术栈
- **前端**: React 18 + TypeScript + Tailwind CSS + TipTap + Vite
- **后端**: Tauri v2 + Rust + Tokio + rusqlite + Meilisearch
- **测试**: Vitest + React Testing Library

---

## 协作规范

1. **单线程任务**: 每个 Agent 同时只处理 1 条 `[TASK]`，完成后立即 `done`
2. **前端/后端接口对齐**: 修改 Tauri 命令或前端调用时，同步通知对方 Agent
3. **优先 `[INFO]` 用 notify**: 纯信息类通知使用 `agtalk notify`，不占用 inbox
4. **代码审查流水线**: 
   - coder 完成 → 发送给 reviewer `[TASK] 请审查代码`
   - reviewer 通过后 → 发送给 planner `[DONE] 审查通过，可合并`
5. **工作目录一致**: 所有 Agent 的 `workdir` 都应为 `~/smart-editor`
6. **提交权限受控**: coder/backend/tester 等执行角色**禁止自行 `git commit` / `git push` / `git rebase` / `git reset --hard` / squash 等改动 git 历史的操作**。
   - 完成开发或测试后，仅允许通过 `[REPLY]` 汇报结果（含变更范围、门禁结论），等待研发经理（manager 角色）拍板下一步
   - 即使全量门禁绿、即使没 push，本地历史的 squash/reset 也属越权
   - 例外：`git status` / `git diff` / `git log` 等只读查询无需审批
   - 违反此规则时，manager 有权要求回滚并通报；屡犯将影响后续派单
