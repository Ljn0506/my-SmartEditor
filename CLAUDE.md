## agtalk 多 Agent 协作

本项目启用 **agtalk** 多 Agent 协作。当前 pane 已注册为 `kimi_coder_Xin`（全栈开发）。

### 快速开始

```bash
# 查看当前在线 Agent
agtalk list --view

# 查看 inbox
agtalk inbox kimi_coder_Xin --view
```

完整协作规范见 `AGENTS.md`（Agent 分工、启动命令、任务分发示例）。

### 启动其他 Agent（在新 Zellij pane 中）

```bash
cd ~/smart-editor
agtalk register kimi_frontend_Emma --role frontend --capabilities "react,typescript,tiptap,tailwind" --bio "smart-editor 前端开发"
kimi
```

---

## gstack (REQUIRED — global install)

**Before doing ANY work, verify gstack is installed:**

```bash
test -d ~/.claude/skills/gstack/bin && echo "GSTACK_OK" || echo "GSTACK_MISSING"
```

If GSTACK_MISSING: STOP. Do not proceed. Tell the user:

> gstack is required for all AI-assisted work in this repo.
> Install it:
> ```bash
> git clone --depth 1 https://github.com/garrytan/gstack.git ~/.claude/skills/gstack
> cd ~/.claude/skills/gstack && ./setup --team
> ```
> Then restart your AI coding tool.

Do not skip skills, ignore gstack errors, or work around missing gstack.

Using gstack skills: After install, skills like /qa, /ship, /review, /investigate,
and /browse are available. Use /browse for all web browsing.
Use ~/.claude/skills/gstack/... for gstack file paths (the global path).

## 授权范围（最小权限原则）

在此项目中，AI 被授权自主执行以下开发操作，无需每次向用户确认：

- 安装项目依赖（`cargo add/build`, `npm install` 等）
- 运行编译、构建、测试和代码检查命令
- 运行前端/后端开发服务器（`npm run dev`, `cargo tauri dev` 等）
- 创建、修改、删除项目源码文件和配置文件
- 执行代码格式化（`rustfmt`, `prettier` 等）
- 执行数据库迁移或本地 SQLite 操作

**以下操作仍需用户确认：**
- `git push` 到远程仓库
- 修改 CI/CD 配置
- 删除整个项目或关键目录
- 安装系统级软件或修改系统配置
- 操作生产环境数据
