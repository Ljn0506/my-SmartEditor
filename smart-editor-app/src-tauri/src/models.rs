use serde::{Deserialize, Serialize};

/// 文档类型：技术文档 vs 商务文档
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Technical,
    Business,
}

/// 模板卡片（对应 SQLite templates 表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub content_html: Option<String>,
    /// 文档属性：投标应答、技术方案、实施方案、合同协议、汇报材料
    pub doc_attr: Option<String>,
    /// 业务领域：网络安全、应用安全、数据安全、安全运营、安全管理
    pub business_domain: Option<String>,
    /// 安全层级：检测层、防御层、分析层、治理层
    pub security_layer: Option<String>,
    /// 内容模块：技术方案、商务条款、实施计划、资质证明、偏离说明、案例介绍
    pub content_module: Option<String>,
    /// 项目阶段：方案阶段、投标阶段、合同阶段
    pub project_phase: Option<String>,
    /// 标签列表
    pub tags: Vec<String>,
    /// 来源 NAS 文件路径
    pub source_file: Option<String>,
    /// 来源段落范围
    pub source_para_range: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub use_count: i64,
    pub rating: i64,
}

/// 技术需求要点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: i64,
    pub text: String,
    /// certain / uncertain
    pub certainty: String,
    pub note: Option<String>,
    /// 用户是否勾选
    pub selected: bool,
}

/// 商务需求要点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRequirement {
    pub id: i64,
    pub category: String,
    pub text: String,
    pub certainty: String,
}

/// 符合性审查项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceItem {
    pub id: i64,
    pub item: String,
    pub required: bool,
    pub note: Option<String>,
}

/// 评分标准
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringCriteria {
    pub id: i64,
    pub item: String,
    pub weight: String,
    pub scoring_standard: String,
}

/// 承诺条款
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub id: i64,
    pub text: String,
    pub risk_level: String,
}

/// 需求解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRequirements {
    pub document_type: DocumentType,
    pub requirements: Vec<Requirement>,
    pub business_requirements: Vec<BusinessRequirement>,
    pub compliance_items: Vec<ComplianceItem>,
    pub scoring_criteria: Vec<ScoringCriteria>,
    pub commitments: Vec<Commitment>,
}

/// 搜索结果项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub content: String,
    pub doc_attr: Option<String>,
    pub business_domain: Option<String>,
    pub content_module: Option<String>,
    pub project_phase: Option<String>,
    pub tags: Vec<String>,
    pub relevance: f64,
}

/// 解析后的文档内容（预留：文档导入解析功能使用）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub file_path: String,
    pub file_name: String,
    pub document_type: DocumentType,
    pub text: String,
}

/// 偏离检查项（预留：投标偏离检查功能使用）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationItem {
    pub id: i64,
    pub section: String,
    pub requirement: String,
    pub response: String,
    pub deviation_type: String,
    pub risk_level: String,
}

/// 招标文件中的强制要求项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementItem {
    pub id: i64,
    pub section: String,
    pub text: String,
    pub category: String,
    pub mandatory: bool,
    pub keywords: Vec<String>,
}

/// 投标文件中的应答段落匹配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMatch {
    pub paragraph_index: usize,
    pub text: String,
    pub relevance_score: f64,
}

/// 偏离状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviationStatus {
    None,
    Positive,
    Minor,
    Major,
}

/// 偏离检查单项结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationCheckResult {
    pub id: i64,
    pub section: String,
    pub requirement_text: String,
    pub response_text: Option<String>,
    pub status: DeviationStatus,
    pub risk_level: String,
    pub explanation: String,
    pub suggestion: String,
}

/// 偏离检查汇总报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationReport {
    pub total: usize,
    pub none_count: usize,
    pub positive_count: usize,
    pub minor_count: usize,
    pub major_count: usize,
    pub fatal_risk_count: usize,
    pub items: Vec<DeviationCheckResult>,
}

/// 废标风险项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatalRisk {
    pub category: String,
    pub description: String,
    pub risk_level: String,
    pub suggestion: String,
}

/// 校对检查结果（预留：文档校对功能使用）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub category: String,
    pub items: Vec<CheckItem>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckItem {
    pub id: i64,
    pub message: String,
    pub severity: String,
    pub location: Option<String>,
}

/// AI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String, // "ollama" | "claude" | "deepseek"
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
}

/// 生成草稿请求（预留：AI 草稿生成功能使用）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRequest {
    pub requirements: Vec<Requirement>,
    pub business_requirements: Vec<BusinessRequirement>,
    pub references: Vec<String>,
    pub document_type: DocumentType,
}
