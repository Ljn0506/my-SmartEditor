# Claude API 配置指南

## 简介

Claude 是 Anthropic 出品的大语言模型，在英文长文本理解和结构化输出方面表现优异。适合处理技术方案、合同条款等复杂文档。

## 前提条件

1. 注册 Anthropic 账号：[https://console.anthropic.com](https://console.anthropic.com)
2. 获取 API Key：`Settings → API Keys → Create Key`
3. 账户需有可用余额（新用户有少量免费额度）

## 配置参数

| 参数 | 值 | 说明 |
|------|-----|------|
| Provider | `Claude` | 选择"Claude API" |
| Base URL | `https://api.anthropic.com` | 官方地址，通常无需修改 |
| API Key | `sk-ant-...` | 从 Anthropic Console 获取 |
| Model | 见下表 | 根据预算和需求选择 |

## 推荐模型

| 模型名 | 上下文长度 | 适用场景 |
|--------|------------|----------|
| `claude-3-5-sonnet-20241022` | 200K | 性价比首选，文档处理推荐 |
| `claude-3-opus-20240229` | 200K | 最强推理能力，复杂分析 |
| `claude-3-haiku-20240307` | 200K | 速度最快，简单任务 |

## 配置示例

```json
{
  "provider": "Claude",
  "base_url": "https://api.anthropic.com",
  "api_key": "sk-ant-api03-your-key-here",
  "model": "claude-3-5-sonnet-20241022",
  "timeout_secs": 120
}
```

## 注意事项

- **Base URL 固定**：Claude 使用官方固定地址 `https://api.anthropic.com/v1/messages`，`base_url` 只填域名即可
- **Header 特殊**：Claude 使用 `x-api-key` 而非标准 `Authorization: Bearer`
- **计费方式**：按输入 + 输出 token 计费，长文档费用较高
- **国内访问**：国内网络可能需要代理才能访问 `api.anthropic.com`
