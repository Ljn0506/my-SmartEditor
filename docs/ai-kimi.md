# Kimi (Moonshot) 配置指南

## 简介

Kimi 是月之暗面（Moonshot AI）出品的大语言模型，以超长上下文窗口（最高 200 万字）著称。非常适合处理招投标文件这类超长文档，一次上传整份标书即可分析。

## 前提条件

1. 注册 Moonshot 开放平台：[https://platform.moonshot.cn](https://platform.moonshot.cn)
2. 获取 API Key：`用户中心 → API Key 管理 → 新建`
3. 充值账户（支持支付宝/微信）

## 配置参数

| 参数 | 值 | 说明 |
|------|-----|------|
| Provider | `Kimi` | 选择"Kimi (Moonshot)" |
| Base URL | `https://api.moonshot.cn` | 官方地址 |
| API Key | `sk-...` | 从 Moonshot 平台获取 |
| Model | 见下表 | 根据上下文长度需求选择 |

## 推荐模型

| 模型名 | 上下文长度 | 适用场景 |
|--------|------------|----------|
| `moonshot-v1-8k` | 8K | 简短文档、快速测试 |
| `moonshot-v1-32k` | 32K | 常规招投标文件 |
| `moonshot-v1-128k` | 128K | 大型合同、技术方案 |
| `moonshot-v1-1m` | 1M (100万) | 超长文档、批量标书 |

## 配置示例

```json
{
  "provider": "Kimi",
  "base_url": "https://api.moonshot.cn",
  "api_key": "sk-your-key-here",
  "model": "moonshot-v1-128k",
  "timeout_secs": 120
}
```

## 注意事项

- **API 格式**：OpenAI 兼容（`/v1/chat/completions`），Header 为 `Authorization: Bearer <key>`
- **超长上下文优势**：Kimi 最大支持 200 万字上下文，是处理长篇招标文件的最佳选择
- **计费方式**：按输入 + 输出 token 计费，上下文越长费用越高
- **并发限制**：默认每秒 3 次请求，如需更高并发请联系平台
- **国内直连**：`api.moonshot.cn` 国内可直接访问，无需代理
- **模型选择建议**：一般招投标文件在 5-20 万字之间，选 `moonshot-v1-32k` 或 `moonshot-v1-128k` 即可
