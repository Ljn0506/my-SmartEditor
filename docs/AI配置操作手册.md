# AI 配置操作手册

## 概述

智能文档编辑器支持四种 AI Provider，分别适用于不同场景：

- **Ollama**：本地运行，数据不出境，适合高隐私要求场景
- **Claude**：Anthropic 出品，英文长文本理解能力强
- **DeepSeek**：深度求索，中文出色且价格低廉，国内直连
- **Kimi**：月之暗面，支持 **200 万字超长上下文**，超长文档首选

---

## 快速选择表

| 场景 | 推荐 Provider | 理由 |
|------|--------------|------|
| 数据不能出境 | Ollama | 本地运行，零外发 |
| 超长招标文件（>10万字）| Kimi | 200万字上下文窗口 |
| 预算敏感 + 中文文档 | DeepSeek | 约 ¥1/百万 token |
| 复杂技术方案分析 | Claude | 推理能力最强 |
| 无网络环境 | Ollama | 完全离线可用 |
| 国内网络直连 | DeepSeek / Kimi | 无需代理 |

---

## 通用配置入口

设置 → AI 配置 → 选择 Provider → 填写对应参数

通用参数说明：

| 参数 | 必填 | 说明 |
|------|------|------|
| Provider | 是 | Ollama / Claude / DeepSeek / Kimi |
| Base URL | 是 | API 根地址，见各 Provider 章节 |
| API Key | 视 Provider | Ollama 可留空，其余必填 |
| Model | 是 | 模型名称，见各 Provider 推荐表 |
| 超时时间 | 否 | 默认 120 秒，Ollama 建议 300 秒 |

---

## Ollama（本地部署）

### 简介

Ollama 是本地运行的 LLM 服务，无需联网即可使用。所有模型文件存放在本地机器上，文档内容不会上传到任何第三方服务器。

### 前提条件

1. 安装 Ollama：[https://ollama.com/download](https://ollama.com/download)
2. 拉取所需模型：`ollama pull <模型名>`

### 配置参数

| 参数 | 值 |
|------|-----|
| Provider | `Ollama` |
| Base URL | `http://localhost:11434` |
| API Key | 留空 |
| Model | `qwen2.5:14b`（推荐）|

### 推荐模型

| 模型名 | 适用场景 | 拉取命令 |
|--------|----------|----------|
| `qwen2.5:14b` | 中文文档处理首选 | `ollama pull qwen2.5:14b` |
| `llama3.1:8b` | 通用英文任务 | `ollama pull llama3.1:8b` |
| `deepseek-r1:14b` | 推理密集型任务 | `ollama pull deepseek-r1:14b` |

### 配置示例

```json
{
  "provider": "Ollama",
  "base_url": "http://localhost:11434",
  "api_key": null,
  "model": "qwen2.5:14b",
  "timeout_secs": 300
}
```

### 注意事项

- **显存要求**：14B 模型约需 8GB 显存/内存，32B 约需 16GB
- **超时设置**：本地推理较慢，建议 timeout 设为 300 秒以上
- **并发限制**：Ollama 默认单线程，大文件处理请耐心等待

---

## Claude（Anthropic）

### 简介

Claude 是 Anthropic 出品的大语言模型，在英文长文本理解和结构化输出方面表现优异。适合处理技术方案、合同条款等复杂文档。

### 前提条件

1. 注册 Anthropic 账号：[https://console.anthropic.com](https://console.anthropic.com)
2. 获取 API Key：`Settings → API Keys → Create Key`
3. 账户需有可用余额

### 配置参数

| 参数 | 值 |
|------|-----|
| Provider | `Claude` |
| Base URL | `https://api.anthropic.com` |
| API Key | `sk-ant-...` |
| Model | `claude-3-5-sonnet-20241022`（推荐）|

### 推荐模型

| 模型名 | 上下文长度 | 特点 |
|--------|------------|------|
| `claude-3-5-sonnet-20241022` | 200K | 性价比首选 |
| `claude-3-opus-20240229` | 200K | 最强推理能力 |
| `claude-3-haiku-20240307` | 200K | 速度最快 |

### 配置示例

```json
{
  "provider": "Claude",
  "base_url": "https://api.anthropic.com",
  "api_key": "sk-ant-api03-your-key-here",
  "model": "claude-3-5-sonnet-20241022",
  "timeout_secs": 120
}
```

### 注意事项

- **Base URL 固定**：只填域名 `https://api.anthropic.com`，程序会自动拼接 `/v1/messages`
- **Header 特殊**：使用 `x-api-key` 而非标准 `Authorization: Bearer`
- **国内访问**：可能需要代理才能访问 `api.anthropic.com`

---

## DeepSeek（深度求索）

### 简介

DeepSeek 中文能力出色且价格低廉。API 兼容 OpenAI 格式，国内可直接访问，无需代理。

### 前提条件

1. 注册开放平台：[https://platform.deepseek.com](https://platform.deepseek.com)
2. 获取 API Key：`API Keys → 创建 API Key`
3. 充值账户（支持支付宝）

### 配置参数

| 参数 | 值 |
|------|-----|
| Provider | `DeepSeek` |
| Base URL | `https://api.deepseek.com` |
| API Key | `sk-...` |
| Model | `deepseek-chat`（推荐）|

### 推荐模型

| 模型名 | 特点 | 参考价格 |
|--------|------|----------|
| `deepseek-chat` | 通用对话，文档处理首选 | 约 ¥1/百万 token |
| `deepseek-reasoner` | 带推理过程，复杂分析 | 约 ¥4/百万 token |

### 配置示例

```json
{
  "provider": "DeepSeek",
  "base_url": "https://api.deepseek.com",
  "api_key": "sk-your-key-here",
  "model": "deepseek-chat",
  "timeout_secs": 120
}
```

### 注意事项

- **API 格式**：OpenAI 兼容（`/chat/completions`），Header 为 `Authorization: Bearer <key>`
- **并发限制**：默认每秒 30 次请求
- **上下文长度**：默认 64K，部分场景可申请 128K

---

## Kimi（月之暗面 / Moonshot）

### 简介

Kimi 以 **200 万字超长上下文窗口** 著称，是处理超长招投标文件的最佳选择。一次上传整份标书即可完整分析，无需分段。

### 前提条件

1. 注册 Moonshot 开放平台：[https://platform.moonshot.cn](https://platform.moonshot.cn)
2. 获取 API Key：`用户中心 → API Key 管理 → 新建`
3. 充值账户（支持支付宝/微信）

### 配置参数

| 参数 | 值 |
|------|-----|
| Provider | `Kimi` |
| Base URL | `https://api.moonshot.cn` |
| API Key | `sk-...` |
| Model | `moonshot-v1-128k`（推荐）|

### 推荐模型

| 模型名 | 上下文长度 | 适用场景 |
|--------|------------|----------|
| `moonshot-v1-8k` | 8K | 简短文档、快速测试 |
| `moonshot-v1-32k` | 32K | 常规招投标文件 |
| `moonshot-v1-128k` | 128K | 大型合同、技术方案 |
| `moonshot-v1-1m` | 1M (100万) | 超长文档、批量标书 |

### 配置示例

```json
{
  "provider": "Kimi",
  "base_url": "https://api.moonshot.cn",
  "api_key": "sk-your-key-here",
  "model": "moonshot-v1-128k",
  "timeout_secs": 120
}
```

### 注意事项

- **超长上下文优势**：最大支持 200 万字，远超其他 Provider
- **模型选择建议**：一般招投标文件在 5-20 万字之间，选 `32k` 或 `128k` 即可
- **并发限制**：默认每秒 3 次请求
- **国内直连**：`api.moonshot.cn` 国内可直接访问

---

## 常见问题

### Q: 可以同时配置多个 Provider 吗？
A: 目前只支持单 Provider 配置，切换需重新设置。

### Q: 为什么 Ollama 不需要 API Key？
A: Ollama 运行在本地，不涉及外部服务认证。

### Q: 文档处理失败怎么办？
A: 依次检查：①网络连通性 ②API Key 有效性 ③模型名称是否正确 ④超时时间是否足够。

### Q: 哪种方案最省钱？
A: 有本地机器选 Ollama（零成本）；云端方案 DeepSeek 最便宜（约 ¥1/百万 token）。
