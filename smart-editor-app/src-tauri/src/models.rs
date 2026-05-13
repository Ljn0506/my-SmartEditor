use serde::{Deserialize, Serialize};

macro_rules! define_enum {
    ($name:ident { $($variant:ident = $str:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
        pub enum $name {
            $(
                #[serde(rename = $str)]
                $variant,
            )+
        }

        impl $name {
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $str,)+
                }
            }

            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    $($str => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl rusqlite::types::ToSql for $name {
            fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
                Ok(self.as_str().into())
            }
        }

        impl rusqlite::types::FromSql for $name {
            fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
                let s = value.as_str()?;
                Self::from_str(s).ok_or_else(|| {
                    rusqlite::types::FromSqlError::Other(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid {}: {}", stringify!($name), s),
                    )))
                })
            }
        }
    };
}

define_enum!(DocAttr {
    BidResponse = "投标应答",
    TechnicalProposal = "技术方案",
    ImplementationPlan = "实施方案",
    ContractAgreement = "合同协议",
    ReportMaterial = "汇报材料",
});

define_enum!(BusinessDomain {
    NetworkSecurity = "网络安全",
    ApplicationSecurity = "应用安全",
    DataSecurity = "数据安全",
    SecurityOperation = "安全运营",
    SecurityManagement = "安全管理",
});

define_enum!(SecurityLayer {
    Detection = "检测层",
    Defense = "防御层",
    Analysis = "分析层",
    Governance = "治理层",
});

define_enum!(ContentModule {
    TechnicalProposal = "技术方案",
    BusinessTerms = "商务条款",
    ImplementationPlan = "实施计划",
    QualificationProof = "资质证明",
    DeviationExplanation = "偏离说明",
    CaseIntroduction = "案例介绍",
    BidResponse = "投标应答",
});

define_enum!(ProjectPhase {
    Proposal = "方案阶段",
    Bidding = "投标阶段",
    Contract = "合同阶段",
});

define_enum!(AiProvider {
    Ollama = "Ollama",
    Claude = "Claude",
    DeepSeek = "DeepSeek",
});

define_enum!(Severity {
    Error = "Error",
    Warning = "Warning",
    Info = "Info",
});

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    #[serde(rename = "High")]
    High,
    #[serde(rename = "Medium")]
    Medium,
    #[serde(rename = "Low")]
    Low,
}

#[allow(dead_code)]
impl RiskLevel {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "High" => Some(Self::High),
            "Medium" => Some(Self::Medium),
            "Low" => Some(Self::Low),
            _ => None,
        }
    }
}

#[allow(dead_code)]
impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[allow(dead_code)]
impl rusqlite::types::ToSql for RiskLevel {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.as_str().into())
    }
}

#[allow(dead_code)]
impl rusqlite::types::FromSql for RiskLevel {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let s = value.as_str()?;
        Self::from_str(s).ok_or_else(|| {
            rusqlite::types::FromSqlError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid RiskLevel: {}", s),
            )))
        })
    }
}

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
    pub doc_attr: Option<DocAttr>,
    pub business_domain: Option<BusinessDomain>,
    pub security_layer: Option<SecurityLayer>,
    pub content_module: Option<ContentModule>,
    pub project_phase: Option<ProjectPhase>,
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

/// 模板查询过滤条件
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateFilter {
    pub doc_attr: Option<DocAttr>,
    pub business_domain: Option<BusinessDomain>,
    pub content_module: Option<ContentModule>,
    pub project_phase: Option<ProjectPhase>,
    pub keyword: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
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
    pub doc_attr: Option<DocAttr>,
    pub business_domain: Option<BusinessDomain>,
    pub content_module: Option<ContentModule>,
    pub project_phase: Option<ProjectPhase>,
    pub tags: Vec<String>,
    pub relevance: f64,
    pub source_file: Option<String>,
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

/// 段落信息（T6：前后端统一 paragraph_index 口径）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub index: usize,
    pub text: String,
    pub char_offset: usize,
}

/// 结构化解析结果（T6）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocumentStructured {
    pub text: String,
    pub paragraphs: Vec<Paragraph>,
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
    pub paragraph_index: Option<usize>,
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

/// 投标文件自查问题项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReviewIssue {
    pub id: i64,
    pub category: String,
    pub sub_category: String,
    pub message: String,
    pub severity: Severity,
    pub position: Option<usize>,
    pub paragraph_index: Option<usize>,
    pub original: Option<String>,
    pub suggestion: Option<String>,
    pub auto_fixable: bool,
}

/// 投标文件自查报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReviewReport {
    pub issues: Vec<SelfReviewIssue>,
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
    pub provider: AiProvider,
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

/// 一键修复模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FixMode {
    Copy,
    Overwrite,
}

/// 段落修改明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphChange {
    pub paragraph_index: usize,
    pub paragraph_text: String,
    pub original: String,
    pub modified: String,
    pub issue_category: String,
}

/// 一键修复结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    pub mode: FixMode,
    pub output_path: String,
    pub backup_path: Option<String>,
    pub changes: Vec<ParagraphChange>,
}


// ========== Phase 2: 智能生成卡片模型 ==========

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CardStatus {
    Draft,
    Confirmed,
    Rejected,
}

impl CardStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "confirmed" => Some(Self::Confirmed),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

/// 章节大纲（生成前）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardOutline {
    pub id: String,
    pub chapter: String,
    pub title: String,
    pub document_target: String,
}

/// 参数占位符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamPlaceholder {
    pub key: String,
    pub label: String,
    pub default_value: Option<String>,
}

/// 章节卡片（生成后）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub chapter: String,
    pub title: String,
    pub content: String,
    pub source_refs: Vec<String>,
    pub document_target: String,
    pub status: CardStatus,
    pub generated_by: String,
    pub related_cards: Vec<String>,
    pub param_placeholders: Vec<ParamPlaceholder>,
    pub risk_flags: Vec<String>,
}

/// 一致性检查问题项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub parameter: String,
    pub expected_value: String,
    pub actual_value: String,
    pub location: String,
    pub severity: Severity,
}

/// 一致性检查报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyReport {
    pub total_checked: usize,
    pub issues: Vec<ConsistencyIssue>,
}

// ========== Phase 3: 全局参数表 + 跨文档一致性 ==========

/// 全局参数表（跨文档一致性核心）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalParams {
    pub project_name: String,
    pub client_name: String,
    pub contract_amount: Option<String>,
    pub delivery_days: Option<i32>,
    pub warranty_years: Option<i32>,
    pub response_time: Option<String>,
    pub project_manager: Option<String>,
    pub qps: Option<i32>,
    pub concurrent_users: Option<i32>,
}

