use regex::Regex;

use crate::models::{SelfReviewIssue, SelfReviewReport, Severity};

/// 预计算段落位置映射，避免 O(N²) 扫描
struct ParagraphMap {
    byte_ranges: Vec<(usize, usize, usize)>, // (start, end, paragraph_index)
    char_ranges: Vec<(usize, usize, usize)>,
}

impl ParagraphMap {
    fn new(text: &str) -> Self {
        let mut byte_ranges = Vec::new();
        let mut char_ranges = Vec::new();
        let mut byte_pos = 0usize;
        let mut char_pos = 0usize;
        let mut idx = 0usize;

        for line in text.lines() {
            let line_byte_len = line.len() + 1; // +1 for '\n'
            let line_char_len = line.chars().count() + 1;
            if !line.trim().is_empty() {
                byte_ranges.push((byte_pos, byte_pos + line.len(), idx));
                char_ranges.push((char_pos, char_pos + line.chars().count(), idx));
                idx += 1;
            }
            byte_pos += line_byte_len;
            char_pos += line_char_len;
        }

        Self { byte_ranges, char_ranges }
    }

    fn find_by_byte(&self, pos: usize) -> Option<usize> {
        self.byte_ranges
            .iter()
            .find(|(s, e, _)| *s <= pos && pos < *e)
            .map(|(_, _, idx)| *idx)
    }

    fn find_by_char(&self, pos: usize) -> Option<usize> {
        self.char_ranges
            .iter()
            .find(|(s, e, _)| *s <= pos && pos < *e)
            .map(|(_, _, idx)| *idx)
    }
}

/// 主入口：对投标文本执行本地自查（重复、敏感信息、占位符）
/// AI 检查（矛盾、逻辑）由独立 async 命令处理
pub fn check_self_review(text: &str) -> SelfReviewReport {
    let mut issues = Vec::new();
    let mut next_id = 1i64;
    let para_map = ParagraphMap::new(text);

    next_id = check_repetition(text, &mut issues, next_id);
    next_id = check_sensitive_info(text, &para_map, &mut issues, next_id);
    next_id = check_placeholders(text, &para_map, &mut issues, next_id);
    let _ = check_punctuation_as_issues(text, &para_map, &mut issues, next_id);

    SelfReviewReport { issues }
}

// ── 重复段落检测 ──

fn check_repetition(text: &str, issues: &mut Vec<SelfReviewIssue>, mut next_id: i64) -> i64 {
    let paragraphs: Vec<&str> = text
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && s.len() > 20)
        .collect();

    for i in 0..paragraphs.len() {
        for j in (i + 1)..paragraphs.len() {
            let a = paragraphs[i];
            let b = paragraphs[j];
            let similarity = calc_similarity(a, b);
            if similarity >= 0.75 {
                issues.push(SelfReviewIssue {
                    id: next_id,
                    category: "quality".to_string(),
                    sub_category: "repetition".to_string(),
                    message: format!(
                        "发现重复段落（相似度 {:.0}%），建议合并或删除冗余内容",
                        similarity * 100.0
                    ),
                    severity: Severity::Warning,
                    position: None,
                    paragraph_index: Some(j),
                    original: Some(b.to_string()),
                    suggestion: Some("删除冗余段落或改写为差异化表述".to_string()),
                    auto_fixable: false,
                });
                next_id += 1;
                break; // 每个段落只报一次重复
            }
        }
    }
    next_id
}

/// 简化版相似度：公共子串长度 / max(len_a, len_b)
fn calc_similarity(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let len_a = a_chars.len();
    let len_b = b_chars.len();
    if len_a == 0 || len_b == 0 {
        return 0.0;
    }

    // 最长公共子序列（LCS）简化版 — 只计算最长公共子串
    let mut max_len = 0usize;
    let max_i = std::cmp::min(len_a, 200);
    let max_j = std::cmp::min(len_b, 200);
    for i in 0..max_i {
        for j in 0..max_j {
            if a_chars[i] == b_chars[j] {
                let mut l = 0usize;
                while i + l < max_i && j + l < max_j && a_chars[i + l] == b_chars[j + l] {
                    l += 1;
                }
                if l > max_len {
                    max_len = l;
                }
            }
        }
    }

    max_len as f64 / std::cmp::max(len_a, len_b) as f64
}

// ── 敏感信息检测 ──

fn check_sensitive_info(
    text: &str,
    para_map: &ParagraphMap,
    issues: &mut Vec<SelfReviewIssue>,
    mut next_id: i64,
) -> i64 {
    let patterns: Vec<(&str, Regex, &str)> = vec![
        (
            "客户名称泄露",
            Regex::new(r"(?:客户|甲方|招标人|用户)[：:]\s*(\S{2,10})").unwrap(),
            "建议用'某客户'或'招标人'代替具体名称",
        ),
        (
            "IP 地址泄露",
            Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
            "删除或替换为示意性 IP",
        ),
        (
            "内部域名泄露",
            Regex::new(r"(?:[\w-]+\.(?:internal|corp|local))\b").unwrap(),
            "删除内部域名信息",
        ),
        (
            "银行账号泄露",
            Regex::new(r"\b(?:\d{4}\s?){3,6}\d{2,4}\b").unwrap(),
            "删除或脱敏银行账号",
        ),
        (
            "手机号泄露",
            Regex::new(r"\b1[3-9]\d{9}\b").unwrap(),
            "删除或脱敏手机号",
        ),
    ];

    for (msg_template, re, suggestion) in &patterns {
        for cap in re.captures_iter(text) {
            let matched = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let pos = cap.get(0).map(|m| m.start()).unwrap_or(0);
            issues.push(SelfReviewIssue {
                id: next_id,
                category: "quality".to_string(),
                sub_category: "sensitive".to_string(),
                message: format!("{}: {}", msg_template, &matched[..std::cmp::min(matched.len(), 30)]),
                severity: Severity::Error,
                position: Some(pos),
                paragraph_index: para_map.find_by_byte(pos),
                original: Some(matched.to_string()),
                suggestion: Some(suggestion.to_string()),
                auto_fixable: true,
            });
            next_id += 1;
        }
    }
    next_id
}

// ── 占位符检测 ──

fn check_placeholders(
    text: &str,
    para_map: &ParagraphMap,
    issues: &mut Vec<SelfReviewIssue>,
    mut next_id: i64,
) -> i64 {
    let patterns: Vec<(&str, Regex, &str, Severity)> = vec![
        (
            "[待补充] 占位符",
            Regex::new(r"\[待补充\]|\[待定\]|\[待完善\]|\[待填写\]|\[待确认\]").unwrap(),
            "请替换为实际内容",
            Severity::Error,
        ),
        (
            "XXX / ____ 占位符",
            Regex::new(r"XXX+|_{3,}|\*{3,}|【.*?】").unwrap(),
            "请替换为实际内容",
            Severity::Error,
        ),
        (
            "日期/金额占位符",
            Regex::new(r"\d{4}年?\s*[-—]\s*月\s*[-—]\s*日|金额[:：]\s*[-—]+").unwrap(),
            "请填写具体日期或金额",
            Severity::Warning,
        ),
    ];

    for (msg_template, re, suggestion, severity) in &patterns {
        for mat in re.find_iter(text) {
            let matched = mat.as_str();
            issues.push(SelfReviewIssue {
                id: next_id,
                category: "format".to_string(),
                sub_category: "placeholder".to_string(),
                message: format!("{}: {}", msg_template, &matched[..std::cmp::min(matched.len(), 20)]),
                severity: *severity,
                position: Some(mat.start()),
                paragraph_index: para_map.find_by_byte(mat.start()),
                original: Some(matched.to_string()),
                suggestion: Some(suggestion.to_string()),
                auto_fixable: false,
            });
            next_id += 1;
        }
    }
    next_id
}

// ── 标点检查集成 ──

fn check_punctuation_as_issues(
    text: &str,
    para_map: &ParagraphMap,
    issues: &mut Vec<SelfReviewIssue>,
    mut next_id: i64,
) -> i64 {
    let punct_issues = crate::punctuation::check_punctuation(text);
    for pi in punct_issues {
        issues.push(SelfReviewIssue {
            id: next_id,
            category: "format".to_string(),
            sub_category: "punctuation".to_string(),
            message: pi.message,
            severity: pi.severity,
            position: Some(pi.position),
            paragraph_index: para_map.find_by_char(pi.position),
            original: Some(pi.original),
            suggestion: Some(pi.suggestion),
            auto_fixable: true,
        });
        next_id += 1;
    }
    next_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_detection() {
        let text = "本项目报价为[待补充]万元，交付周期为 XXX 天。";
        let report = check_self_review(text);
        let placeholders: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.sub_category == "placeholder")
            .collect();
        assert!(!placeholders.is_empty(), "应检测到占位符");
        assert!(placeholders.iter().any(|i| i.original.as_ref().unwrap().contains("[待补充]")));
    }

    #[test]
    fn test_sensitive_ip() {
        let text = "服务器部署在 192.168.1.1 和 10.0.0.1。";
        let report = check_self_review(text);
        let sensitive: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.sub_category == "sensitive")
            .collect();
        assert!(!sensitive.is_empty(), "应检测到 IP 地址");
    }

    #[test]
    fn test_repetition() {
        let text = "我们提供7x24小时技术支持服务。\n\n我们提供7x24小时技术支持服务。";
        let report = check_self_review(text);
        let reps: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.sub_category == "repetition")
            .collect();
        assert!(!reps.is_empty(), "应检测到重复段落");
    }

    /// sensitive 必须 auto_fixable=true 且 suggestion 非空
    #[test]
    fn test_sensitive_auto_fixable() {
        let text = "服务器部署在 192.168.1.1。";
        let report = check_self_review(text);
        let sensitive: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.sub_category == "sensitive")
            .collect();
        assert!(!sensitive.is_empty(), "应检测到 sensitive issue");
        for issue in &sensitive {
            assert!(
                issue.auto_fixable,
                "sensitive 类 issue 必须 auto_fixable=true（可提供脱敏建议）"
            );
            assert!(
                issue.suggestion.as_ref().is_some_and(|s| !s.is_empty()),
                "sensitive issue 必须提供非空 suggestion"
            );
        }
    }

    /// placeholder 必须 auto_fixable=false（业务问题，无法机器决定）
    #[test]
    fn test_placeholder_auto_fixable() {
        let text = "本项目报价为[待补充]万元，交付周期为 XXX 天。";
        let report = check_self_review(text);
        let placeholders: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.sub_category == "placeholder")
            .collect();
        assert!(!placeholders.is_empty(), "应检测到 placeholder issue");
        for issue in &placeholders {
            assert!(
                !issue.auto_fixable,
                "placeholder 类 issue 必须 auto_fixable=false（占位符内容是业务问题）"
            );
        }
    }

    /// repetition 必须 auto_fixable=false（需人工判断保留哪份）
    #[test]
    fn test_repetition_auto_fixable_false() {
        let text = "我们提供7x24小时技术支持服务。\n\n我们提供7x24小时技术支持服务。";
        let report = check_self_review(text);
        let reps: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.sub_category == "repetition")
            .collect();
        assert!(!reps.is_empty());
        for issue in &reps {
            assert!(
                !issue.auto_fixable,
                "repetition 类 issue 必须 auto_fixable=false"
            );
        }
    }

    /// 待 contradiction / context_logic AI 子检查实施后启用
    #[test]
    #[ignore = "depends on contradiction / context_logic AI subchecks"]
    fn test_self_review_six_categories_full_hit() {
        // fixture：故意包含全部 6 类问题
        let text = "\
本项目交付周期为 30 天。\n\
本项目交付周期为 60 天。\n\
\n\
我们提供7x24小时技术支持服务。\n\
我们提供7x24小时技术支持服务。\n\
\n\
客户：北京XX公司\n\
内部域名 srv01.corp\n\
联系电话 13800138000\n\
\n\
本项目报价为[待补充]万元。\n\
交付时间：XXXXXXXX\n\
\n\
方案"; // 故意触发 punctuation 末尾缺标点

        let report = check_self_review(text);
        let categories: std::collections::HashSet<_> =
            report.issues.iter().map(|i| i.sub_category.clone()).collect();

        for expected in [
            "repetition",
            "sensitive",
            "placeholder",
            "punctuation",
            "contradiction",
            "logic",
        ] {
            assert!(
                categories.contains(expected),
                "6 类应全部命中，缺失 {}。已命中: {:?}",
                expected,
                categories
            );
        }
    }

    /// 待 punctuation 集成进 check_self_review 后启用
    #[test]
    #[ignore = "depends on punctuation integration into self_review"]
    fn test_punctuation_integrated_into_self_review() {
        // 故意引入中文标点错误（英文逗号紧跟中文字符）
        let text = "本方案,采用主流架构,具有高可用性。";
        let report = check_self_review(text);
        let punct: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.sub_category == "punctuation")
            .collect();
        assert!(
            !punct.is_empty(),
            "SelfReviewReport 应包含 punctuation issue"
        );
        for issue in &punct {
            assert_eq!(
                issue.category, "format",
                "punctuation 必须归类到 format"
            );
            assert!(
                issue.auto_fixable,
                "punctuation 必须 auto_fixable=true"
            );
            assert!(
                issue.suggestion.as_ref().is_some_and(|s| !s.is_empty()),
                "punctuation 必须提供 suggestion（规范化后的标点）"
            );
        }
    }
}
