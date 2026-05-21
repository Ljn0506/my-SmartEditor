# DeepSeek API 配置指南

## 简介

DeepSeek 是深度求索出品的大语言模型，中文能力出色且价格低廉。API 兼容 OpenAI 格式，易于接入。适合预算有限但需要处理中文招投标文档的场景。

## 前提条件

1. 注册 DeepSeek 开放平台：[https://platform.deepseek.com](https://platform.deepseek.com)
2. 获取 API Key：`API Keys → 创建 API Key`
3. 充值账户（支持支付宝，价格极低）

## 配置参数

| 参数 | 值 | 说明 |
|------|-----|------|
| Provider | `DeepSeek` | 选择"DeepSeek API" |
| Base URL | `https://api.deepseek.com` | 官方地址 |
| API Key | `sk-...` | 从 DeepSeek 平台获取 |
| Model | 见下表 | 根据任务选择 |

## 推荐模型

| 模型名 | 特点 | 价格 |
|--------|------|------|
| `deepseek-chat` | 通用对话模型，文档处理首选 | 约 ¥1/百万 token |
| `deepseek-reasoner` | 带推理过程，复杂分析用 | 约 ¥4/百万 token |

## 配置示例

```json
{
  "provider": "DeepSeek",
  "base_url": "https://api.deepseek.com",
  "api_key": "sk-your-key-here",
  "model": "deepseek-chat",
  "timeout_secs": 120
}
```

## 注意事项

- **API 格式**：OpenAI 兼容（`/chat/completions`），Header 为 `Authorization: Bearer <key>`
- **并发限制**：默认每秒 30 次请求，日常使用足够
- **上下文长度**：默认支持 64K，部分场景可申请 128K
- **联网搜索**：模型本身不带联网搜索，知识截止有日期限制
- **国内直连**：`api.deepseek.com` 国内可直接访问，无需代理
