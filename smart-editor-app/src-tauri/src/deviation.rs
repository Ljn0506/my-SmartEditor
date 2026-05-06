use once_cell::sync::Lazy;
use regex::Regex;

use crate::models::{
    DeviationCheckResult, DeviationReport, DeviationStatus, RequirementItem, ResponseMatch,
};

const MANDATORY_KEYWORDS: &[&str] = &[
    "★", "必须", "须", "应", "不得", "禁止", "强制", "关键", "务必", "一定",
];

/// 强制词 stop-list：含 "须"/"应" 单字的常见非强制词
const MANDATORY_STOP_LIST: &[&str] = &["应用", "须知", "适应", "反应"];

const STOP_WORDS: &[&str] = &[
    "的", "了", "和", "是", "在", "有", "被", "将", "为", "与", "及", "或", "等", "所述", "该",
    "此", "上述", "以下", "以上",
];

static RE_SECTION_NUM: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(\d+(?:\.\d+)*)\s*[、.．]?\s*").unwrap());
static RE_NON_KEYWORD: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^一-龥a-zA-Z0-9]+").unwrap());
static RE_VALIDITY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"有效期\s*(?:不少于?|至少)?\s*(\d+)\s*天").unwrap());
static RE_NUMERIC_VALUE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d+(?:\.\d+)?)\s*(个|项|人|天|年|月|周|小时|分钟|秒|万元|元|亿元|套|台|次|页|份|家|条|款|章|节|点|类|种|组|批|单|笔|件|辆|艘|架|座|处|间|亩|公顷|平方米|立方米|吨|千克|公斤|克|升|毫升|米|千米|公里|厘米|毫米|Mbps|Gbps|GB|TB|MB|KB|PB|B|%)").unwrap()
});

/// 从招标文件文本提取强制要求项
pub fn extract_requirements(req_text: &str) -> Vec<RequirementItem> {
    let mut items = Vec::new();
    let mut current_section = String::new();
    let mut id = 1i64;

    for line in req_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 提取章节编号
        if let Some(cap) = RE_SECTION_NUM.captures(trimmed) {
            current_section = cap.get(1).map_or("", |m| m.as_str()).to_string();
            continue;
        }

        // 判断是否为强制要求
        let has_mandatory_kw = MANDATORY_KEYWORDS.iter().any(|kw| trimmed.contains(kw));
        let has_stop_word = MANDATORY_STOP_LIST.iter().any(|sw| trimmed.contains(sw));
        let is_mandatory = has_mandatory_kw && !has_stop_word;
        if !is_mandatory {
            continue;
        }

        let keywords = extract_keywords(trimmed);
        items.push(RequirementItem {
            id,
            section: current_section.clone(),
            text: trimmed.to_string(),
            category: crate::category::infer_category(trimmed),
            mandatory: true,
            keywords,
        });
        id += 1;
    }

    items
}

/// 在投标文本中定位最匹配的应答段落
pub fn find_response(req: &RequirementItem, paragraphs: &[&str]) -> Option<ResponseMatch> {
    let mut best: Option<ResponseMatch> = None;

    for (idx, para) in paragraphs.iter().enumerate() {
        let score = calc_relevance(&req.keywords, para);
        if score > 0.0 {
            if let Some(ref current) = best {
                if score > current.relevance_score {
                    best = Some(ResponseMatch {
                        paragraph_index: idx,
                        text: para.to_string(),
                        relevance_score: score,
                    });
                }
            } else {
                best = Some(ResponseMatch {
                    paragraph_index: idx,
                    text: para.to_string(),
                    relevance_score: score,
                });
            }
        }
    }

    best
}

#[derive(Debug, Clone)]
struct NumericValue {
    value: f64,
    unit: String,
    raw: String,
}

/// 判定偏离状态
pub fn judge_deviation(req: &RequirementItem, resp: &ResponseMatch) -> DeviationCheckResult {
    let resp_lower = resp.text.to_lowercase();

    // 1. 关键词规则
    let keyword_status = if resp_lower.contains("不满足")
        || resp_lower.contains("偏离")
        || resp_lower.contains("低于")
        || resp_lower.contains("小于")
        || resp_lower.contains("不足")
        || resp_lower.contains("不支持")
    {
        DeviationStatus::Major
    } else if resp_lower.contains("部分")
        || resp_lower.contains("略")
        || resp_lower.contains("基本")
        || resp_lower.contains("一定程度上")
    {
        DeviationStatus::Minor
    } else if resp_lower.contains("优于")
        || resp_lower.contains("高于")
        || resp_lower.contains("大于")
        || resp_lower.contains("超过")
    {
        DeviationStatus::Positive
    } else if resp_lower.contains("满足")
        || resp_lower.contains("响应")
        || resp_lower.contains("符合")
        || resp_lower.contains("完全")
        || resp_lower.contains("承诺")
    {
        DeviationStatus::None
    } else {
        if resp.relevance_score >= 0.5 {
            DeviationStatus::Minor
        } else {
            DeviationStatus::Major
        }
    };

    // 2. 数值比较增强（硬数据优先）
    let req_vals = extract_numeric_values(&req.text);
    let resp_vals = extract_numeric_values(&resp.text);
    let (status, numeric_note) =
        if let Some((numeric_status, reason)) = match_numeric(&req_vals, &resp_vals) {
            // 数值冲突时，数值优先
            (numeric_status, Some(reason))
        } else {
            (keyword_status, None)
        };

    let (risk_level, mut explanation, suggestion) = match status {
        DeviationStatus::None => (
            "无".to_string(),
            "完全满足招标要求".to_string(),
            "直接通过".to_string(),
        ),
        DeviationStatus::Positive => (
            "低".to_string(),
            "技术指标优于招标要求".to_string(),
            "需确认客户是否接受正偏离".to_string(),
        ),
        DeviationStatus::Minor => (
            "中".to_string(),
            "非关键条款存在轻微差异".to_string(),
            "评估影响，准备说明材料".to_string(),
        ),
        DeviationStatus::Major => (
            "高".to_string(),
            "关键条款不满足招标要求".to_string(),
            "可能废标，需重点关注并准备替代方案".to_string(),
        ),
    };

    if let Some(note) = numeric_note {
        explanation = format!("{} [{}]", explanation, note);
    }

    DeviationCheckResult {
        id: req.id,
        section: req.section.clone(),
        requirement_text: req.text.clone(),
        response_text: Some(resp.text.clone()),
        status,
        risk_level,
        explanation,
        suggestion,
        paragraph_index: Some(resp.paragraph_index),
    }
}

/// 主入口：对两份文本执行偏离检查
pub fn check_deviation(bid_text: &str, req_text: &str) -> DeviationReport {
    let requirements = extract_requirements(req_text);
    check_deviation_items(&requirements, bid_text)
}

/// 接受已解析的需求列表 + 投标文本，执行偏离检查
/// 用于「校对」Tab：招标文件已在「需求上传」Tab 解析好，直接传入需求项
pub fn check_deviation_items(req_items: &[RequirementItem], bid_text: &str) -> DeviationReport {
    let paragraphs: Vec<&str> = bid_text
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let mut items = Vec::new();
    let mut none_count = 0usize;
    let mut positive_count = 0usize;
    let mut minor_count = 0usize;
    let mut major_count = 0usize;
    let mut fatal_risk_count = 0usize;

    for req in req_items {
        let result = if let Some(resp) = find_response(req, &paragraphs) {
            judge_deviation(req, &resp)
        } else {
            DeviationCheckResult {
                id: req.id,
                section: req.section.clone(),
                requirement_text: req.text.clone(),
                response_text: None,
                status: DeviationStatus::Major,
                risk_level: "致命".to_string(),
                explanation: "投标文件中未找到对应应答内容".to_string(),
                suggestion: "补充对应应答段落，或明确说明原因".to_string(),
                paragraph_index: None,
            }
        };

        match result.status {
            DeviationStatus::None => none_count += 1,
            DeviationStatus::Positive => positive_count += 1,
            DeviationStatus::Minor => minor_count += 1,
            DeviationStatus::Major => {
                major_count += 1;
                if result.risk_level == "致命" {
                    fatal_risk_count += 1;
                }
            }
        }

        items.push(result);
    }

    DeviationReport {
        total: items.len(),
        none_count,
        positive_count,
        minor_count,
        major_count,
        fatal_risk_count,
        items,
    }
}

impl DeviationReport {
    /// 导出为 Markdown 格式
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# 偏离检查报告\n\n");
        md.push_str(&format!(
            "- **总项数**: {}\n- **完全响应**: {}\n- **正偏离**: {}\n- **轻微偏离**: {}\n- **重大偏离**: {}\n- **致命风险**: {}\n\n",
            self.total,
            self.none_count,
            self.positive_count,
            self.minor_count,
            self.major_count,
            self.fatal_risk_count
        ));
        md.push_str("| 序号 | 章节 | 要求内容 | 偏离状态 | 风险等级 | 说明 | 建议 |\n");
        md.push_str("|------|------|----------|----------|----------|------|------|\n");
        for item in &self.items {
            let status_str = match item.status {
                crate::models::DeviationStatus::None => "✅ 完全响应",
                crate::models::DeviationStatus::Positive => "🔵 正偏离",
                crate::models::DeviationStatus::Minor => "🟡 轻微偏离",
                crate::models::DeviationStatus::Major => "🔴 重大偏离",
            };
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                item.id,
                item.section.replace('|', "\\|"),
                item.requirement_text.replace('|', "\\|"),
                status_str,
                item.risk_level,
                item.explanation.replace('|', "\\|"),
                item.suggestion.replace('|', "\\|")
            ));
        }
        md
    }

    /// 导出为 HTML 格式
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<h1>偏离检查报告</h1>\n");
        html.push_str("<ul>\n");
        html.push_str(&format!(
            "<li>总项数: {}</li>\n<li>完全响应: {}</li>\n<li>正偏离: {}</li>\n<li>轻微偏离: {}</li>\n<li>重大偏离: {}</li>\n<li>致命风险: {}</li>\n",
            self.total, self.none_count, self.positive_count, self.minor_count, self.major_count, self.fatal_risk_count
        ));
        html.push_str("</ul>\n");
        html.push_str("<table border=\"1\">\n");
        html.push_str("<tr><th>序号</th><th>章节</th><th>要求内容</th><th>偏离状态</th><th>风险等级</th><th>说明</th><th>建议</th></tr>\n");
        for item in &self.items {
            let (status_class, status_text) = match item.status {
                crate::models::DeviationStatus::None => ("status-none", "✅ 完全响应"),
                crate::models::DeviationStatus::Positive => ("status-positive", "🔵 正偏离"),
                crate::models::DeviationStatus::Minor => ("status-minor", "🟡 轻微偏离"),
                crate::models::DeviationStatus::Major => ("status-major", "🔴 重大偏离"),
            };
            html.push_str(&format!(
                "<tr class=\"{}\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                status_class,
                item.id,
                html_escape(&item.section),
                html_escape(&item.requirement_text),
                status_text,
                html_escape(&item.risk_level),
                html_escape(&item.explanation),
                html_escape(&item.suggestion)
            ));
        }
        html.push_str("</table>\n");
        html
    }

    /// 导出为 JSON 字符串
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// --- 内部工具函数 ---

fn extract_keywords(text: &str) -> Vec<String> {
    let mut result = Vec::new();

    for word in RE_NON_KEYWORD.split(text) {
        let word = word.trim().to_lowercase();
        if word.is_empty() || word.len() < 2 || STOP_WORDS.contains(&word.as_str()) {
            continue;
        }

        let char_count = word.chars().count();
        if char_count > 4 {
            // 长词按字拆分，提高匹配粒度
            for ch in word.chars() {
                let s = ch.to_string();
                if !STOP_WORDS.contains(&s.as_str()) {
                    result.push(s);
                }
            }
        } else {
            result.push(word);
        }
    }

    result
}

fn calc_relevance(req_keywords: &[String], paragraph: &str) -> f64 {
    if req_keywords.is_empty() {
        return 0.0;
    }
    let para_keywords = extract_keywords(paragraph);
    if para_keywords.is_empty() {
        return 0.0;
    }

    use std::collections::HashSet;
    let req_set: HashSet<_> = req_keywords.iter().collect();
    let para_set: HashSet<_> = para_keywords.iter().collect();

    let intersection = req_set.intersection(&para_set).count();
    let union = req_set.union(&para_set).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
}

/// 废标项检查
pub fn check_fatal_risks(bid_text: &str) -> Vec<crate::models::FatalRisk> {
    use crate::models::FatalRisk;
    let mut risks = Vec::new();
    let lower = bid_text.to_lowercase();

    // 1. 签字盖章检查
    if !lower.contains("签字") && !lower.contains("盖章") && !lower.contains("签署") {
        risks.push(FatalRisk {
            category: "签字盖章".to_string(),
            description: "投标文件中未提及签字或盖章".to_string(),
            risk_level: "致命".to_string(),
            suggestion: "补充法定代表人签字及公司公章".to_string(),
        });
    }

    // 2. 保证金检查
    if lower.contains("保证金")
        && !lower.contains("已缴纳")
        && !lower.contains("已提交")
        && !lower.contains("已支付")
    {
        risks.push(FatalRisk {
            category: "保证金".to_string(),
            description: "提及保证金但未确认已缴纳".to_string(),
            risk_level: "致命".to_string(),
            suggestion: "明确保证金缴纳状态并附凭证".to_string(),
        });
    }

    // 3. 密封检查
    if lower.contains("密封")
        && !lower.contains("已密封")
        && !lower.contains("密封完好")
        && !lower.contains("按要求密封")
    {
        risks.push(FatalRisk {
            category: "密封".to_string(),
            description: "提及密封但未确认密封状态".to_string(),
            risk_level: "致命".to_string(),
            suggestion: "确认投标文件密封完好".to_string(),
        });
    }

    // 4. 有效期检查
    if let Some(cap) = RE_VALIDITY.captures(&lower) {
        if let Some(days_match) = cap.get(1) {
            if let Ok(days) = days_match.as_str().parse::<i64>() {
                if days < 90 {
                    risks.push(FatalRisk {
                        category: "有效期".to_string(),
                        description: format!("投标有效期仅 {} 天，低于通常要求的 90 天", days),
                        risk_level: "致命".to_string(),
                        suggestion: "将有效期延长至至少 90 天".to_string(),
                    });
                }
            }
        }
    }

    // 5. 资质检查
    if lower.contains("资质")
        && (lower.contains("不满足") || lower.contains("不具备") || lower.contains("无资质"))
    {
        risks.push(FatalRisk {
            category: "资质".to_string(),
            description: "投标文件暗示资质不满足".to_string(),
            risk_level: "致命".to_string(),
            suggestion: "核实资质要求并补充证明材料".to_string(),
        });
    }

    risks
}

// --- 数值比较工具 ---

fn extract_numeric_values(text: &str) -> Vec<NumericValue> {
    RE_NUMERIC_VALUE
        .captures_iter(text)
        .filter_map(|cap| {
            let val: f64 = cap.get(1)?.as_str().parse().ok()?;
            let unit = cap.get(2)?.as_str().trim().to_lowercase();
            let raw = cap.get(0)?.as_str().to_string();
            Some(NumericValue {
                value: val,
                unit,
                raw,
            })
        })
        .collect()
}

fn match_numeric(
    req_vals: &[NumericValue],
    resp_vals: &[NumericValue],
) -> Option<(DeviationStatus, String)> {
    for req_val in req_vals {
        for resp_val in resp_vals {
            if is_same_unit(&req_val.unit, &resp_val.unit) {
                if resp_val.value < req_val.value {
                    return Some((
                        DeviationStatus::Major,
                        format!(
                            "数值对比：招标要求 {}，实际应答 {}（低于要求）",
                            req_val.raw, resp_val.raw
                        ),
                    ));
                } else if resp_val.value > req_val.value {
                    return Some((
                        DeviationStatus::Positive,
                        format!(
                            "数值对比：招标要求 {}，实际应答 {}（优于要求）",
                            req_val.raw, resp_val.raw
                        ),
                    ));
                } else {
                    return Some((
                        DeviationStatus::None,
                        format!(
                            "数值对比：招标要求 {}，实际应答 {}（完全匹配）",
                            req_val.raw, resp_val.raw
                        ),
                    ));
                }
            }
        }
    }
    None
}

fn is_same_unit(a: &str, b: &str) -> bool {
    let a = a.trim();
    let b = b.trim();
    a == b || a.contains(b) || b.contains(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_requirements_with_mandatory_keywords() {
        let text = "1. 资质要求\n投标人必须具有独立法人资格。\n注册资本应不低于1000万元。\n须提供近三年的财务报表。\n本项目工期要求不得少于180天。\n技术支持★7×24小时响应。";
        let items = extract_requirements(text);
        assert_eq!(items.len(), 5);
        assert_eq!(items[0].text, "投标人必须具有独立法人资格。");
        assert!(items[0].mandatory);
        assert_eq!(items[1].text, "注册资本应不低于1000万元。");
        assert!(items[1].mandatory);
        assert_eq!(items[2].text, "须提供近三年的财务报表。");
        assert!(items[2].mandatory);
        assert_eq!(items[3].text, "本项目工期要求不得少于180天。");
        assert!(items[3].mandatory);
        assert_eq!(items[4].text, "技术支持★7×24小时响应。");
        assert!(items[4].mandatory);
    }

    #[test]
    fn test_extract_requirements_empty_and_no_keywords() {
        assert!(extract_requirements("").is_empty());
        assert!(extract_requirements("这是一段普通描述，未包含特殊指令。").is_empty());
    }

    #[test]
    fn test_find_response_match() {
        // 使用带标点的文本，确保分词后包含独立关键词
        let req_text = "投标人、须、具备、法人、资格。";
        let req = RequirementItem {
            id: 1,
            section: "1".to_string(),
            text: req_text.to_string(),
            category: "商务".to_string(),
            mandatory: true,
            keywords: extract_keywords(req_text),
        };
        let bid = "我公司成立于2010年。\n我司具有法人、资格。\n技术团队100人。";
        let bid_paras: Vec<&str> = bid
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let resp = find_response(&req, &bid_paras).unwrap();
        assert_eq!(resp.paragraph_index, 1);
        assert!(resp.text.contains("法人"));
        assert!(resp.relevance_score > 0.0);
    }

    #[test]
    fn test_find_response_no_match() {
        let req = RequirementItem {
            id: 1,
            section: "1".to_string(),
            text: "必须具有飞船驾照".to_string(),
            category: "技术".to_string(),
            mandatory: true,
            keywords: vec!["飞船".to_string(), "驾照".to_string()],
        };
        let bid = "我公司成立于2010年。\n我司具有独立法人资格。";
        let bid_paras: Vec<&str> = bid
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        assert!(find_response(&req, &bid_paras).is_none());
    }

    #[test]
    fn test_judge_deviation_none() {
        let req = RequirementItem {
            id: 1,
            section: "1".to_string(),
            text: "必须响应".to_string(),
            category: "商务".to_string(),
            mandatory: true,
            keywords: vec![],
        };
        let resp = ResponseMatch {
            paragraph_index: 0,
            text: "我公司完全满足要求，承诺按时交付。".to_string(),
            relevance_score: 1.0,
        };
        let result = judge_deviation(&req, &resp);
        assert_eq!(result.status, DeviationStatus::None);
        assert_eq!(result.risk_level, "无");
    }

    #[test]
    fn test_judge_deviation_positive() {
        let req = RequirementItem {
            id: 1,
            section: "1".to_string(),
            text: "性能指标".to_string(),
            category: "技术".to_string(),
            mandatory: true,
            keywords: vec![],
        };
        let resp = ResponseMatch {
            paragraph_index: 0,
            text: "我们的方案优于招标要求，处理性能高于标准30%。".to_string(),
            relevance_score: 1.0,
        };
        let result = judge_deviation(&req, &resp);
        assert_eq!(result.status, DeviationStatus::Positive);
        assert_eq!(result.risk_level, "低");
    }

    #[test]
    fn test_judge_deviation_minor() {
        let req = RequirementItem {
            id: 1,
            section: "1".to_string(),
            text: "功能要求".to_string(),
            category: "技术".to_string(),
            mandatory: true,
            keywords: vec![],
        };
        let resp = ResponseMatch {
            paragraph_index: 0,
            text: "该功能部分满足，在一定程度上可支持。".to_string(),
            relevance_score: 1.0,
        };
        let result = judge_deviation(&req, &resp);
        assert_eq!(result.status, DeviationStatus::Minor);
        assert_eq!(result.risk_level, "中");
    }

    #[test]
    fn test_judge_deviation_major() {
        let req = RequirementItem {
            id: 1,
            section: "1".to_string(),
            text: "安全认证".to_string(),
            category: "技术".to_string(),
            mandatory: true,
            keywords: vec![],
        };
        let resp = ResponseMatch {
            paragraph_index: 0,
            text: "该要求我们不满足，技术方案偏离标准，低于行业水平。".to_string(),
            relevance_score: 1.0,
        };
        let result = judge_deviation(&req, &resp);
        assert_eq!(result.status, DeviationStatus::Major);
        assert_eq!(result.risk_level, "高");
    }

    #[test]
    fn test_judge_deviation_ambiguous_high_relevance() {
        let req = RequirementItem {
            id: 1,
            section: "1".to_string(),
            text: "资质要求".to_string(),
            category: "商务".to_string(),
            mandatory: true,
            keywords: vec![],
        };
        let resp = ResponseMatch {
            paragraph_index: 0,
            text: "关于资质我司有相关经验。".to_string(),
            relevance_score: 0.6,
        };
        let result = judge_deviation(&req, &resp);
        assert_eq!(result.status, DeviationStatus::Minor);
    }

    #[test]
    fn test_judge_deviation_ambiguous_low_relevance() {
        let req = RequirementItem {
            id: 1,
            section: "1".to_string(),
            text: "资质要求".to_string(),
            category: "商务".to_string(),
            mandatory: true,
            keywords: vec![],
        };
        let resp = ResponseMatch {
            paragraph_index: 0,
            text: "我们是一家科技公司。".to_string(),
            relevance_score: 0.2,
        };
        let result = judge_deviation(&req, &resp);
        assert_eq!(result.status, DeviationStatus::Major);
    }

    #[test]
    fn test_check_deviation_end_to_end() {
        // 利用顿号/逗号让 extract_keywords 分词，确保 find_response 跨段落子串匹配
        let req_text = "1. 资质要求\n投标人、须、具备、资质。\n2. 资本要求\n注册、资本、应、达标。\n3. 技术要求\n方案、须、支持、全天候、运行。";
        let bid_text = "投标人、须、具备、资质，完全响应。\n注册、资本、已、达标，完全响应。\n方案、须、支持、全天候、运行，完全满足。";
        let report = check_deviation(bid_text, req_text);
        assert_eq!(report.total, 3);
        assert_eq!(report.none_count, 3);
        assert_eq!(report.major_count, 0);
        assert_eq!(report.fatal_risk_count, 0);
        assert_eq!(report.items[0].status, DeviationStatus::None);
        assert_eq!(report.items[1].status, DeviationStatus::None);
        assert_eq!(report.items[2].status, DeviationStatus::None);
    }

    #[test]
    fn test_report_to_markdown() {
        let req_text = "1. 资质要求\n投标人、须、具备、资质。";
        let bid_text = "投标人、须、具备、资质，完全响应。";
        let report = check_deviation(bid_text, req_text);
        let md = report.to_markdown();
        assert!(md.contains("# 偏离检查报告"));
        assert!(md.contains("✅ 完全响应"));
        assert!(md.contains("| 序号 |"));
    }

    #[test]
    fn test_report_to_html() {
        let req_text = "1. 资质要求\n投标人、须、具备、资质。";
        let bid_text = "投标人、须、具备、资质，完全响应。";
        let report = check_deviation(bid_text, req_text);
        let html = report.to_html();
        assert!(html.contains("<h1>偏离检查报告</h1>"));
        assert!(html.contains("status-none"));
        assert!(html.contains("<table"));
    }

    #[test]
    fn test_report_to_json() {
        let req_text = "1. 资质要求\n投标人、须、具备、资质。";
        let bid_text = "投标人、须、具备、资质，完全响应。";
        let report = check_deviation(bid_text, req_text);
        let json = report.to_json();
        assert!(json.contains("total"));
        assert!(json.contains("items"));
    }

    #[test]
    fn test_check_fatal_risks_signature_missing() {
        let bid = "本项目报价100万元，工期180天。";
        let risks = check_fatal_risks(bid);
        let sig = risks.iter().find(|r| r.category == "签字盖章");
        assert!(sig.is_some(), "应检测到缺少签字盖章");
        assert_eq!(sig.unwrap().risk_level, "致命");
    }

    #[test]
    fn test_check_fatal_risks_deposit_unpaid() {
        let bid = "本项目需缴纳保证金10万元。报价100万元。";
        let risks = check_fatal_risks(bid);
        let dep = risks.iter().find(|r| r.category == "保证金");
        assert!(dep.is_some(), "应检测到保证金未确认缴纳");
    }

    #[test]
    fn test_check_fatal_risks_deposit_paid() {
        let bid = "保证金10万元已缴纳。报价100万元。";
        let risks = check_fatal_risks(bid);
        assert!(
            risks.iter().all(|r| r.category != "保证金"),
            "已缴纳不应触发"
        );
    }

    #[test]
    fn test_check_fatal_risks_validity_too_short() {
        let bid = "投标有效期不少于30天。";
        let risks = check_fatal_risks(bid);
        let val = risks.iter().find(|r| r.category == "有效期");
        assert!(val.is_some(), "应检测到有效期过短");
    }

    #[test]
    fn test_check_fatal_risks_validity_ok() {
        let bid = "投标有效期不少于180天。";
        let risks = check_fatal_risks(bid);
        assert!(
            risks.iter().all(|r| r.category != "有效期"),
            "180天不应触发"
        );
    }

    #[test]
    fn test_check_deviation_missing_response() {
        // 投标文本与两项要求均无关，视为全部无应答
        let req_text = "1. 认证要求\n投标人必须具有ISO27001认证。\n2. 质保要求\n应提供三年质保。";
        let bid_text = "我公司专注于网络安全领域。";
        let report = check_deviation(bid_text, req_text);
        assert_eq!(report.total, 2);
        assert_eq!(report.none_count, 0);
        assert_eq!(report.major_count, 2); // 两项均无匹配应答
        assert_eq!(report.fatal_risk_count, 2);
        let iso_item = report
            .items
            .iter()
            .find(|i| i.requirement_text.contains("ISO27001"))
            .unwrap();
        assert_eq!(iso_item.status, DeviationStatus::Major);
        assert_eq!(iso_item.risk_level, "致命");
        assert!(iso_item.response_text.is_none());
    }

    // ── T9-B3：边界处理 ──

    #[test]
    fn test_check_deviation_items_empty() {
        let report = check_deviation_items(&[], "任意投标文本");
        assert_eq!(report.total, 0);
        assert_eq!(report.none_count, 0);
        assert_eq!(report.positive_count, 0);
        assert_eq!(report.minor_count, 0);
        assert_eq!(report.major_count, 0);
        assert_eq!(report.fatal_risk_count, 0);
        assert!(report.items.is_empty());
    }

    #[test]
    fn test_check_deviation_items_empty_bid_text() {
        let req = RequirementItem {
            id: 1,
            section: "1".to_string(),
            text: "投标人必须具有 ISO27001 认证".to_string(),
            category: "商务".to_string(),
            mandatory: true,
            keywords: extract_keywords("投标人必须具有 ISO27001 认证"),
        };
        let report = check_deviation_items(&[req], "");
        assert_eq!(report.total, 1);
        assert_eq!(report.major_count, 1);
        assert_eq!(report.fatal_risk_count, 1);
    }
}
