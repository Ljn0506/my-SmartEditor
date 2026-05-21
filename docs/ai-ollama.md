# Ollama 配置指南

## 简介

Ollama 是本地运行的 LLM 服务，无需联网即可使用，适合对数据隐私要求高的场景。所有模型文件都存放在本地机器上，文档内容不会上传到第三方服务器。

## 前提条件

1. 安装 Ollama：[https://ollama.com/download](https://ollama.com/download)
2. 拉取所需模型：`ollama pull <模型名>`

## 配置参数

| 参数 | 值 | 说明 |
|------|-----|------|
| Provider | `Ollama` | 选择"本地 Ollama" |
| Base URL | `http://localhost:11434` | Ollama 默认端口 |
| API Key | 留空 | Ollama 本地模式不需要 Key |
| Model | 见下表 | 根据任务选择模型 |

## 推荐模型

| 模型名 | 适用场景 | 拉取命令 |
|--------|----------|----------|
| `qwen2.5:14b` | 中文文档处理首选 | `ollama pull qwen2.5:14b` |
| `llama3.1:8b` | 通用英文任务 | `ollama pull llama3.1:8b` |
| `deepseek-r1:14b` | 推理密集型任务 | `ollama pull deepseek-r1:14b` |

## 配置示例

```json
{
  "provider": "Ollama",
  "base_url": "http://localhost:11434",
  "api_key": null,
  "model": "qwen2.5:14b",
  "timeout_secs": 300
}
```

## 注意事项

- **显存要求**：14B 模型需要约 8GB 显存/内存，32B 需要约 16GB
- **超时设置**：本地模型推理较慢，建议 timeout 设为 300 秒以上
- **并发限制**：Ollama 默认单线程，大文件处理时请耐心等待
- **网络隔离**：`base_url` 使用 `localhost` 时，`validate_ai_url` 会校验并阻止（当前版本允许本地地址，但建议留意安全设置）
