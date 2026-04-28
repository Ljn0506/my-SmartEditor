use regex::Regex;

use crate::error::Result;

/// 脱敏规则引擎
pub struct Desensitizer {
    rules: Vec<DesensitizeRule>,
}

struct DesensitizeRule {
    name: &'static str,
    pattern: Regex,
    replacement: &'static str,
}

impl Desensitizer {
    pub fn new() -> Self {
        let rules = vec![
            DesensitizeRule {
                name: "chinese_bank",
                pattern: Regex::new(r"中国(工商银行|农业银行|建设银行|银行|邮政储蓄银行|招商银行|交通银行|中信银行|光大银行|民生银行|平安银行|浦发银行|兴业银行|华夏银行|广发银行|渤海银行|浙商银行)").unwrap(),
                replacement: "客户A（国有大型银行）",
            },
            DesensitizeRule {
                name: "amount_yuan",
                pattern: Regex::new(r"\d{1,3}(,\d{3})*\s*(?:万|元|万元)(?:\s*(?:人民币|RMB|元))?").unwrap(),
                replacement: "[项目预算]",
            },
            DesensitizeRule {
                name: "date_chinese",
                pattern: Regex::new(r"\d{4}\s*年\s*\d{1,2}\s*月\s*\d{1,2}\s*日").unwrap(),
                replacement: "[项目启动时间]",
            },
            DesensitizeRule {
                name: "date_iso",
                pattern: Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap(),
                replacement: "[项目启动时间]",
            },
            DesensitizeRule {
                name: "internal_system",
                pattern: Regex::new(r"[\w\u4e00-\u9fa5]+核心系统[\w\u4e00-\u9fa5]*").unwrap(),
                replacement: "客户核心交易系统",
            },
            DesensitizeRule {
                name: "person_name",
                pattern: Regex::new(r"[\u4e00-\u9fa5]{2,4}经理").unwrap(),
                replacement: "[客户方项目经理]",
            },
            DesensitizeRule {
                name: "company_name_suffix",
                pattern: Regex::new(r"[\u4e00-\u9fa5]{2,10}(科技有限公司|股份有限公司|集团有限公司|有限公司|有限责任公司)").unwrap(),
                replacement: "[客户公司名称]",
            },
        ];
        Self { rules }
    }

    pub fn desensitize(&self, text: &str) -> Result<String> {
        let mut result = text.to_string();
        for rule in &self.rules {
            result = rule
                .pattern
                .replace_all(&result, rule.replacement)
                .to_string();
        }
        Ok(result)
    }

    /// 返回脱敏前后的对比列表
    pub fn analyze(&self, text: &str) -> Vec<DesensitizeHit> {
        let mut hits = Vec::new();
        for rule in &self.rules {
            for mat in rule.pattern.find_iter(text) {
                hits.push(DesensitizeHit {
                    rule_name: rule.name.to_string(),
                    original: mat.as_str().to_string(),
                    replacement: rule.replacement.to_string(),
                    start: mat.start(),
                    end: mat.end(),
                });
            }
        }
        hits.sort_by_key(|h| h.start);
        hits
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DesensitizeHit {
    pub rule_name: String,
    pub original: String,
    pub replacement: String,
    pub start: usize,
    pub end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desensitize_amount_yuan() {
        let engine = Desensitizer::new();
        let result = engine.desensitize("项目预算100万元。").unwrap();
        assert_eq!(result, "项目预算[项目预算]。");

        let result2 = engine.desensitize("费用为500元。").unwrap();
        assert_eq!(result2, "费用为[项目预算]。");

        let result3 = engine.desensitize("金额1,000,000元。").unwrap();
        assert_eq!(result3, "金额[项目预算]。");

        let result4 = engine.desensitize("价格100万 人民币。").unwrap();
        assert_eq!(result4, "价格[项目预算]。");
    }

    #[test]
    fn test_desensitize_date() {
        let engine = Desensitizer::new();
        let result = engine.desensitize("日期：2024年1月15日").unwrap();
        assert_eq!(result, "日期：[项目启动时间]");

        let result2 = engine.desensitize("日期：2024-01-15").unwrap();
        assert_eq!(result2, "日期：[项目启动时间]");
    }

    #[test]
    fn test_desensitize_bank() {
        let engine = Desensitizer::new();
        let result = engine.desensitize("开户行：中国工商银行").unwrap();
        assert_eq!(result, "开户行：客户A（国有大型银行）");
    }

    #[test]
    fn test_desensitize_person_name() {
        let engine = Desensitizer::new();
        let result = engine.desensitize("联系人：张三经理").unwrap();
        assert_eq!(result, "联系人：[客户方项目经理]");
    }

    #[test]
    fn test_desensitize_company() {
        let engine = Desensitizer::new();
        let result = engine.desensitize("承建方：北京某某科技有限公司").unwrap();
        assert_eq!(result, "承建方：[客户公司名称]");
    }

    #[test]
    fn test_desensitize_combined() {
        let engine = Desensitizer::new();
        let input = "张三经理代表北京科技有限公司，于2024年3月1日在中国建设银行开户，预算100万元。";
        let result = engine.desensitize(input).unwrap();
        assert!(result.contains("[客户方项目经理]"));
        assert!(result.contains("[客户公司名称]"));
        assert!(result.contains("[项目启动时间]"));
        assert!(result.contains("客户A（国有大型银行）"));
        assert!(result.contains("[项目预算]"));
    }

    #[test]
    fn test_analyze_hits() {
        let engine = Desensitizer::new();
        let hits = engine.analyze("金额500元。李四经理。");
        assert_eq!(hits.len(), 2);
        let amount_hit = hits.iter().find(|h| h.rule_name == "amount_yuan").unwrap();
        assert_eq!(amount_hit.original, "500元");
        let person_hit = hits.iter().find(|h| h.rule_name == "person_name").unwrap();
        assert_eq!(person_hit.original, "李四经理");
    }

    #[test]
    fn test_analyze_no_hits() {
        let engine = Desensitizer::new();
        let hits = engine.analyze("这是一段没有任何敏感信息的普通文本。");
        assert!(hits.is_empty());
    }
}
