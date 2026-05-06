use serde::Deserialize;
use serde_json::json;

use crate::error::{AppError, Result};
use crate::models::{AiConfig, DocumentType, ParsedRequirements, SelfReviewIssue, Severity};

#[derive(Clone)]
pub struct AiClient {
    config: AiConfig,
    http: reqwest::Client,
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_default();
        Self { config, http }
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
        let parsed: ParsedRequirements = serde_json::from_str(json_str)
            .map_err(|e| AppError::Ai(format!("AI 返回 JSON 解析失败: {}", e)))?;
        validate_requirements(&parsed)
            .map_err(|e| AppError::Ai(format!("AI 返回内容校验失败: {}", e)))?;
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
        match self.config.provider {
            crate::models::AiProvider::Ollama => self.chat_ollama(prompt).await,
            crate::models::AiProvider::Claude => self.chat_claude(prompt).await,
            crate::models::AiProvider::DeepSeek => self.chat_deepseek(prompt).await,
        }
    }

    async fn chat_ollama(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.config.base_url);
        let data = self
            .chat_post_json(
                &url,
                vec![],
                json!({
                    "model": self.config.model,
                    "prompt": prompt,
                    "stream": false,
                }),
            )
            .await?;
        let resp: OllamaResponse = serde_json::from_value(data)
            .map_err(|e| AppError::Ai(format!("Ollama 响应解析失败: {}", e)))?;
        Ok(resp.response)
    }

    async fn chat_post_json(
        &self,
        url: &str,
        headers: Vec<(&str, String)>,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let mut req = self.http.post(url).json(&body);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let res = req.send().await?;
        let res = res.error_for_status()
            .map_err(|e| AppError::Ai(format!("AI 请求失败: {}", e)))?;
        Ok(res.json().await?)
    }

    async fn chat_claude(&self, prompt: &str) -> Result<String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Ai("Claude API Key 未配置".to_string()))?;
        self.chat_api_compatible(
            "https://api.anthropic.com/v1/messages",
            vec![
                ("x-api-key", api_key.clone()),
                ("anthropic-version", "2023-06-01".to_string()),
            ],
            json!({
                "model": self.config.model,
                "max_tokens": 4096,
                "messages": [{"role": "user", "content": prompt}],
            }),
            |data| {
                data.get("content")?
                    .as_array()?
                    .first()?
                    .get("text")?
                    .as_str()
            },
        )
        .await
    }

    async fn chat_deepseek(&self, prompt: &str) -> Result<String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Ai("DeepSeek API Key 未配置".to_string()))?;
        self.chat_api_compatible(
            &format!("{}/chat/completions", self.config.base_url),
            vec![("Authorization", format!("Bearer {}", api_key))],
            json!({
                "model": self.config.model,
                "messages": [{"role": "user", "content": prompt}],
            }),
            |data| {
                data.get("choices")?
                    .as_array()?
                    .first()?
                    .get("message")?
                    .get("content")?
                    .as_str()
            },
        )
        .await
    }

    async fn chat_api_compatible(
        &self,
        url: &str,
        mut headers: Vec<(&str, String)>,
        body: serde_json::Value,
        extract: impl FnOnce(&serde_json::Value) -> Option<&str>,
    ) -> Result<String> {
        headers.push(("content-type", "application/json".to_string()));
        let data = self.chat_post_json(url, headers, body).await?;
        extract(&data)
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::Ai("AI 响应中未找到内容".to_string()))
    }

    /// T5: 检测文档中的自相矛盾（AI）
    pub async fn check_contradictions(&self, text: &str) -> Result<Vec<SelfReviewIssue>> {
        let prompt = build_contradiction_prompt(text);
        let response = self.chat(&prompt).await?;
        let json_str = extract_json_array(&response)?;
        let items: Vec<AiReviewItem> = serde_json::from_str(json_str)
            .map_err(|e| AppError::Ai(format!("矛盾检测 JSON 解析失败: {}", e)))?;
        Ok(items.into_iter().map(|it| it.to_issue("contradiction")).collect())
    }

    /// T5: 检测上下文逻辑断裂（AI）
    pub async fn check_context_logic(&self, text: &str) -> Result<Vec<SelfReviewIssue>> {
        let prompt = build_context_logic_prompt(text);
        let response = self.chat(&prompt).await?;
        let json_str = extract_json_array(&response)?;
        let items: Vec<AiReviewItem> = serde_json::from_str(json_str)
            .map_err(|e| AppError::Ai(format!("逻辑检测 JSON 解析失败: {}", e)))?;
        Ok(items.into_iter().map(|it| it.to_issue("logic")).collect())
    }
}

/// AI 返回的审查项中间格式
#[derive(Debug, serde::Deserialize)]
struct AiReviewItem {
    message: String,
    original: String,
    suggestion: String,
    severity: String,
}

impl AiReviewItem {
    #[allow(clippy::wrong_self_convention)]
    fn to_issue(self, sub_category: &str) -> SelfReviewIssue {
        let severity = match self.severity.as_str() {
            "Error" => Severity::Error,
            "Warning" => Severity::Warning,
            "Info" => Severity::Info,
            other => {
                log::warn!("AI 返回未知 severity: {}, 已降级为 Info", other);
                Severity::Info
            }
        };
        SelfReviewIssue {
            id: 0, // 由调用方重新编号
            category: "consistency".to_string(),
            sub_category: sub_category.to_string(),
            message: self.message,
            severity,
            position: None,
            paragraph_index: None,
            original: Some(self.original),
            suggestion: Some(self.suggestion),
            auto_fixable: false,
        }
    }
}

fn build_contradiction_prompt(text: &str) -> String {
    format!(
        r#"你是一位文档审查专家，擅长发现技术文档和投标文件中的自相矛盾。

请分析以下文档，找出所有自相矛盾之处。例如：
- 前面说"7x24小时支持"，后面说"工作日9-18点"
- 前面说"交付周期30天"，后面说"交付周期60天"
- 技术指标前后不一致

对每个矛盾，输出以下字段：
- message: 矛盾描述
- original: 涉及的文本片段（50字以内）
- suggestion: 修改建议
- severity: "Error"（严重）或 "Warning"（一般）

输出严格的 JSON 数组格式，不要包含任何其他文字：
[
  {{"message": "...", "original": "...", "suggestion": "...", "severity": "Error"}}
]

文档内容（{}字）：
{}"#,
        text.chars().count(),
        text.chars().take(12000).collect::<String>()
    )
}

fn build_context_logic_prompt(text: &str) -> String {
    format!(
        r#"你是一位文档审查专家，擅长发现技术文档和投标文件中的上下文逻辑断裂。

请分析以下文档，找出所有上下文逻辑问题。例如：
- 前文提到"详见第三章"，但后文没有对应内容
- 论述突然中断，没有结论
- 前后段落缺乏过渡或因果关系不成立
- 引用不存在的图表或章节

对每个问题，输出以下字段：
- message: 问题描述
- original: 涉及的文本片段（50字以内）
- suggestion: 修改建议
- severity: "Error"（严重）或 "Warning"（一般）

输出严格的 JSON 数组格式，不要包含任何其他文字：
[
  {{"message": "...", "original": "...", "suggestion": "...", "severity": "Warning"}}
]

文档内容（{}字）：
{}"#,
        text.chars().count(),
        text.chars().take(12000).collect::<String>()
    )
}

fn extract_json_array(text: &str) -> Result<&str> {
    if let Some(start) = text.find('[') {
        let mut depth = 0;
        let mut in_string = false;
        let mut escape = false;
        for (i, ch) in text[start..].char_indices() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' && in_string {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if in_string {
                continue;
            }
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(&text[start..start + i + ch.len_utf8()]);
                    }
                }
                _ => {}
            }
        }
    }
    Err(AppError::Parse("AI 返回中未找到 JSON 数组".to_string()))
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
        let mut depth = 0;
        let mut in_string = false;
        let mut escape = false;
        for (i, ch) in text[start..].char_indices() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' && in_string {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if in_string {
                continue;
            }
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(&text[start..start + i + ch.len_utf8()]);
                    }
                }
                _ => {}
            }
        }
    }
    Err(AppError::Parse("AI 返回中未找到 JSON 内容".to_string()))
}

fn validate_requirements(parsed: &crate::models::ParsedRequirements) -> Result<()> {
    if parsed.requirements.is_empty()
        && parsed.business_requirements.is_empty()
        && parsed.compliance_items.is_empty()
        && parsed.scoring_criteria.is_empty()
        && parsed.commitments.is_empty()
    {
        return Err(AppError::Validation(
            "AI 返回的需求列表为空，请检查文档内容或重试".to_string(),
        ));
    }
    for req in &parsed.requirements {
        if req.id <= 0 {
            return Err(AppError::Validation(format!(
                "需求项 ID 必须为正整数，收到: {}",
                req.id
            )));
        }
        if req.text.trim().is_empty() {
            return Err(AppError::Validation(
                "需求项 text 不能为空".to_string(),
            ));
        }
    }
    for req in &parsed.business_requirements {
        if req.id <= 0 {
            return Err(AppError::Validation(format!(
                "商务需求项 ID 必须为正整数，收到: {}",
                req.id
            )));
        }
    }
    for item in &parsed.compliance_items {
        if item.id <= 0 {
            return Err(AppError::Validation(format!(
                "符合性审查项 ID 必须为正整数，收到: {}",
                item.id
            )));
        }
    }
    for item in &parsed.scoring_criteria {
        if item.id <= 0 {
            return Err(AppError::Validation(format!(
                "评分标准项 ID 必须为正整数，收到: {}",
                item.id
            )));
        }
    }
    for item in &parsed.commitments {
        if item.id <= 0 {
            return Err(AppError::Validation(format!(
                "承诺条款项 ID 必须为正整数，收到: {}",
                item.id
            )));
        }
    }
    Ok(())
}

// --- API Response Types ---

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}
