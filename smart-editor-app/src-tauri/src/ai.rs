use serde::Deserialize;
use serde_json::json;

use crate::error::{AppError, Result};
use crate::models::{AiConfig, DocumentType, ParsedRequirements};

pub struct AiClient {
    config: AiConfig,
    http: reqwest::Client,
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    /// 根据文档内容提取需求要点
    pub async fn extract_requirements(
        &self,
        text: &str,
        doc_type: DocumentType,
    ) -> Result<ParsedRequirements> {
        let prompt = match doc_type {
            DocumentType::Technical => build_tech_prompt(text),
            DocumentType::Business => build_business_prompt(text),
        };

        let response = self.chat(&prompt).await?;
        // 尝试从 AI 返回中提取 JSON 部分
        let json_str = extract_json(&response)?;
        let parsed: ParsedRequirements = serde_json::from_str(json_str)?;
        Ok(parsed)
    }

    /// 生成技术方案或商务响应草稿
    pub async fn generate_draft(
        &self,
        requirements: &str,
        references: &[String],
        doc_type: DocumentType,
    ) -> Result<String> {
        let prompt = match doc_type {
            DocumentType::Technical => build_tech_draft_prompt(requirements, references),
            DocumentType::Business => build_business_draft_prompt(requirements, references),
        };
        self.chat(&prompt).await
    }

    async fn chat(&self, prompt: &str) -> Result<String> {
        match self.config.provider.as_str() {
            "ollama" => self.chat_ollama(prompt).await,
            "claude" => self.chat_claude(prompt).await,
            "deepseek" => self.chat_deepseek(prompt).await,
            _ => Err(AppError::Ai(format!(
                "不支持的 AI 提供商: {}",
                self.config.provider
            ))),
        }
    }

    async fn chat_ollama(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.config.base_url);
        let body = json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
        });

        let res = self.http.post(&url).json(&body).send().await?;
        let data: OllamaResponse = res.json().await?;
        Ok(data.response)
    }

    async fn chat_claude(&self, prompt: &str) -> Result<String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Ai("Claude API Key 未配置".to_string()))?;

        let res = self
            .http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&json!({
                "model": self.config.model,
                "max_tokens": 4096,
                "messages": [{"role": "user", "content": prompt}],
            }))
            .send()
            .await?;

        let data: ClaudeResponse = res.json().await?;
        Ok(data
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default())
    }

    async fn chat_deepseek(&self, prompt: &str) -> Result<String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Ai("DeepSeek API Key 未配置".to_string()))?;

        let res = self
            .http
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&json!({
                "model": self.config.model,
                "messages": [{"role": "user", "content": prompt}],
            }))
            .send()
            .await?;

        let data: DeepSeekResponse = res.json().await?;
        Ok(data
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}

fn build_tech_prompt(text: &str) -> String {
    format!(
        r#"你是一位需求分析专家，擅长从招标文件和需求规格说明书中提取技术实现要点。

请分析以下文档内容，提取所有技术实现要点。对每个要点：
- 如果需求明确，直接列出
- 如果需求模糊或需要确认，标注"[需确认]"并给出建议确认的问题

输出严格的 JSON 格式：
{{
  "document_type": "technical",
  "requirements": [
    {{"id": 1, "text": "实现基于角色的访问控制（RBAC）", "certainty": "certain", "selected": true}},
    {{"id": 2, "text": "支持5000+并发用户 [需确认：峰值还是平均？]", "certainty": "uncertain", "selected": false}}
  ],
  "business_requirements": [],
  "compliance_items": [],
  "scoring_criteria": [],
  "commitments": []
}}

文档内容：
{}"#,
        text.chars().take(8000).collect::<String>()
    )
}

fn build_business_prompt(text: &str) -> String {
    format!(
        r#"你是一位商务投标专家，擅长从招标文件、符合性审查缩影和商务评审索引中提取商务条款和评审要求。

请分析以下文档内容，提取所有商务相关要点，分类输出：

输出严格的 JSON 格式：
{{
  "document_type": "business",
  "requirements": [],
  "business_requirements": [
    {{"id": 1, "category": "资质要求", "text": "投标人须具备信息安全等级保护测评机构资质", "certainty": "certain"}}
  ],
  "compliance_items": [
    {{"id": 1, "item": "营业执照", "required": true, "note": "须加盖公章"}}
  ],
  "scoring_criteria": [
    {{"id": 1, "item": "技术方案完整性", "weight": "30分", "scoring_standard": "方案完整、合理、可行得 21-30 分"}}
  ],
  "commitments": [
    {{"id": 1, "text": "承诺所提供产品具有自主知识产权", "risk_level": "high"}}
  ]
}}

文档内容：
{}"#,
        text.chars().take(8000).collect::<String>()
    )
}

fn build_tech_draft_prompt(requirements: &str, references: &[String]) -> String {
    let refs = references.join("\n\n---\n\n");
    format!(
        r#"你是一位资深信息安全售前工程师，擅长编写技术方案文档。

请根据以下信息生成一份完整的技术方案文档：

## 需求要点
{}

## 参考资料
{}

## 输出要求
1. 文档结构：项目概述 → 需求分析 → 方案设计 → 实施计划 → 风险评估
2. 技术描述专业、准确，符合等保/网络安全行业标准
3. 对模糊需求给出合理的假设说明，并在文档中标注"[待确认]"
4. 使用 Markdown 格式，标题层级清晰
5. 内容详实，每个章节至少 200 字

请直接输出文档内容。"#,
        requirements, refs
    )
}

fn build_business_draft_prompt(requirements: &str, references: &[String]) -> String {
    let refs = references.join("\n\n---\n\n");
    format!(
        r#"你是一位资深商务投标专家，擅长编写商务响应文档和偏离表。

请根据以下信息生成一份完整的商务响应文档：

## 商务要点
{}

## 参考资料
{}

## 输出要求
1. 文档结构：公司资质 → 商务条款响应 → 报价说明 → 服务承诺 → 偏离表
2. 对每条商务要求明确标注响应状态：完全响应 / 部分响应 / 偏离（并说明原因）
3. 承诺条款措辞严谨，避免过度承诺和法律风险
4. 偏离表格式规范，包含：序号、招标要求、投标响应、偏离说明
5. 使用 Markdown 格式，表格使用标准 Markdown 表格语法

请直接输出文档内容。"#,
        requirements, refs
    )
}

fn extract_json(text: &str) -> Result<&str> {
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return Ok(&text[start..=end]);
        }
    }
    Err(AppError::Parse("AI 返回中未找到 JSON 内容".to_string()))
}

// --- API Response Types ---

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize, Clone)]
struct ClaudeContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct DeepSeekResponse {
    choices: Vec<DeepSeekChoice>,
}

#[derive(Debug, Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekMessage,
}

#[derive(Debug, Deserialize, Clone)]
struct DeepSeekMessage {
    content: String,
}
