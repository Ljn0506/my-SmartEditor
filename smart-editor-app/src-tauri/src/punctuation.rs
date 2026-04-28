use once_cell::sync::Lazy;
use regex::Regex;

static RE_CONSECUTIVE_CN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([，。！？；：、]{2,})").unwrap());
static RE_CN_ELLIPSIS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([\u{4e00}-\u{9fff}])\.{3,}([\u{4e00}-\u{9fff}])").unwrap());
static RE_FW_LETTER: Lazy<Regex> = Lazy::new(|| Regex::new(r"[Ａ-Ｚａ-ｚ]+").unwrap());
static RE_FW_NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"[０-９]+").unwrap());

/// 标点符号问题项
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PunctuationIssue {
    pub id: i64,
    pub message: String,
    pub severity: String, // "error" | "warning"
    pub original: String,
    pub suggestion: String,
    pub position: usize, // 字符位置（非字节位置）
}

/// 主入口：对文本执行全部标点规范检查
pub fn check_punctuation(text: &str) -> Vec<PunctuationIssue> {
    let mut issues = Vec::new();
    let mut next_id = 1i64;

    next_id = check_mixed_punctuation(text, &mut issues, next_id);
    next_id = check_consecutive_punctuation(text, &mut issues, next_id);
    next_id = check_quote_pairs(text, &mut issues, next_id);
    check_fullwidth_letters_numbers(text, &mut issues, next_id);

    issues
}

// --- 规则 1：中英文标点混用 ---

fn check_mixed_punctuation(text: &str, issues: &mut Vec<PunctuationIssue>, mut id: i64) -> i64 {
    // 英文标点 → 中文标点映射
    let mappings: &[(char, char, &str)] = &[
        (',', '，', "应使用中文逗号"),
        ('.', '。', "应使用中文句号"),
        ('!', '！', "应使用中文感叹号"),
        ('?', '？', "应使用中文问号"),
        (';', '；', "应使用中文分号"),
        (':', '：', "应使用中文冒号"),
    ];

    let chars: Vec<char> = text.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if let Some((_, chn, msg)) = mappings.iter().find(|&&(e, _, _)| e == ch).copied() {
            let prev_is_cjk = i > 0 && is_cjk(chars[i - 1]);
            let next_is_cjk = i + 1 < chars.len() && is_cjk(chars[i + 1]);
            if prev_is_cjk || next_is_cjk {
                issues.push(PunctuationIssue {
                    id,
                    message: format!("{}{}", msg, chn),
                    severity: "warning".to_string(),
                    original: ch.to_string(),
                    suggestion: chn.to_string(),
                    position: i,
                });
                id += 1;
            }
        }

        // 英文括号
        if ch == '(' || ch == ')' {
            let prev_is_cjk = i > 0 && is_cjk(chars[i - 1]);
            let next_is_cjk = i + 1 < chars.len() && is_cjk(chars[i + 1]);
            if prev_is_cjk || next_is_cjk {
                let sugg = if ch == '(' { '（' } else { '）' };
                issues.push(PunctuationIssue {
                    id,
                    message: "应使用中文括号".to_string(),
                    severity: "warning".to_string(),
                    original: ch.to_string(),
                    suggestion: sugg.to_string(),
                    position: i,
                });
                id += 1;
            }
        }
    }

    id
}

fn is_cjk(ch: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&ch)
        || ('\u{3000}'..='\u{303f}').contains(&ch)
        || ('\u{ff00}'..='\u{ffef}').contains(&ch)
}

// --- 规则 2：连续标点 ---

fn check_consecutive_punctuation(
    text: &str,
    issues: &mut Vec<PunctuationIssue>,
    mut id: i64,
) -> i64 {
    // 连续 2 个及以上相同中文标点（排除合法省略号 ……）
    for cap in RE_CONSECUTIVE_CN.captures_iter(text) {
        let m = cap.get(0).unwrap();
        let start = text[..m.start()].chars().count();
        let orig = m.as_str();
        // 如果只有一个字符重复，如 "，，"；如果是 "……" 则跳过
        if orig == "……" {
            continue;
        }
        // 检查是否全部为同一字符
        let first = orig.chars().next().unwrap();
        if orig.chars().all(|c| c == first) {
            issues.push(PunctuationIssue {
                id,
                message: format!("连续出现多个{}", first),
                severity: "error".to_string(),
                original: orig.to_string(),
                suggestion: first.to_string(),
                position: start,
            });
            id += 1;
        }
    }

    // 英文连续句号 "..."（3个及以上）在中文语境中
    for cap in RE_CN_ELLIPSIS.captures_iter(text) {
        let m = cap.get(0).unwrap();
        let start = text[..m.start()].chars().count();
        let orig = m.as_str().to_string();
        let sugg = orig.replace("...", "……").replace("....", "……");
        issues.push(PunctuationIssue {
            id,
            message: "中文中应使用省略号……".to_string(),
            severity: "warning".to_string(),
            original: orig,
            suggestion: sugg,
            position: start,
        });
        id += 1;
    }

    id
}

// --- 规则 3：引号配对 ---

fn check_quote_pairs(text: &str, issues: &mut Vec<PunctuationIssue>, mut id: i64) -> i64 {
    // ASCII 直引号 " 和 ' 的开闭字符相同，无法在不解析语义的情况下判定配对，
    // 这里只检查可区分开闭的中文/角标引号。
    let pairs: &[(char, char, &str)] = &[
        ('\u{201C}', '\u{201D}', "双引号"),
        ('\u{2018}', '\u{2019}', "单引号"),
        ('「', '」', "直角引号"),
        ('【', '】', "方头括号"),
        ('『', '』', "双直角引号"),
        ('《', '》', "书名号"),
        ('（', '）', "圆括号"),
    ];

    for (open, close, name) in pairs {
        let open_count = text.chars().filter(|&c| c == *open).count();
        let close_count = text.chars().filter(|&c| c == *close).count();
        if open_count != close_count {
            let pos = text.chars().count(); // 放到末尾作为全文问题
            issues.push(PunctuationIssue {
                id,
                message: format!("{}未配对：左{}个，右{}个", name, open_count, close_count),
                severity: "error".to_string(),
                original: format!("{}{}", open, close),
                suggestion: format!("确保{}成对出现", name),
                position: pos,
            });
            id += 1;
        }
    }

    id
}

// --- 规则 4：全角字母/数字 ---

fn check_fullwidth_letters_numbers(
    text: &str,
    issues: &mut Vec<PunctuationIssue>,
    mut id: i64,
) -> i64 {
    // 全角字母 Ａ-Ｚａ-ｚ
    for m in RE_FW_LETTER.find_iter(text) {
        let start = text[..m.start()].chars().count();
        let orig = m.as_str();
        let sugg: String = orig
            .chars()
            .map(|c| {
                if ('Ａ'..='Ｚ').contains(&c) {
                    ((c as u32 - 'Ａ' as u32) + 'A' as u32) as u8 as char
                } else if ('ａ'..='ｚ').contains(&c) {
                    ((c as u32 - 'ａ' as u32) + 'a' as u32) as u8 as char
                } else {
                    c
                }
            })
            .collect();
        issues.push(PunctuationIssue {
            id,
            message: "全角字母建议改为半角".to_string(),
            severity: "warning".to_string(),
            original: orig.to_string(),
            suggestion: sugg,
            position: start,
        });
        id += 1;
    }

    // 全角数字 ０-９
    for m in RE_FW_NUMBER.find_iter(text) {
        let start = text[..m.start()].chars().count();
        let orig = m.as_str();
        let sugg: String = orig
            .chars()
            .map(|c| {
                if ('０'..='９').contains(&c) {
                    ((c as u32 - '０' as u32) + '0' as u32) as u8 as char
                } else {
                    c
                }
            })
            .collect();
        issues.push(PunctuationIssue {
            id,
            message: "全角数字建议改为半角".to_string(),
            severity: "warning".to_string(),
            original: orig.to_string(),
            suggestion: sugg,
            position: start,
        });
        id += 1;
    }

    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixed_comma() {
        let text = "你好,世界";
        let issues = check_punctuation(text);
        let comma = issues.iter().find(|i| i.original == ",");
        assert!(comma.is_some(), "应检测到英文逗号混用");
        assert_eq!(comma.unwrap().suggestion, "，");
        assert_eq!(comma.unwrap().position, 2); // "你"(0) "好"(1) ","(2)
    }

    #[test]
    fn test_mixed_period() {
        let text = "这是一个句子.后面还有";
        let issues = check_punctuation(text);
        let period = issues.iter().find(|i| i.message.contains("句号"));
        assert!(period.is_some(), "应检测到英文句号混用");
    }

    #[test]
    fn test_consecutive_punctuation() {
        let text = "你好，，世界";
        let issues = check_punctuation(text);
        let dup = issues.iter().find(|i| i.message.contains("连续"));
        assert!(dup.is_some(), "应检测到连续逗号");
        assert_eq!(dup.unwrap().suggestion, "，");
    }

    #[test]
    fn test_ellipsis_allowed() {
        let text = "他说……然后走了";
        let issues = check_punctuation(text);
        assert!(
            !issues.iter().any(|i| i.original == "……"),
            "省略号不应被误报"
        );
    }

    #[test]
    fn test_quote_unpaired() {
        let text = "他说「你好";
        let issues = check_punctuation(text);
        let quote = issues.iter().find(|i| i.message.contains("直角引号"));
        assert!(quote.is_some(), "应检测到引号未配对");
    }

    #[test]
    fn test_fullwidth_letter() {
        let text = "使用ＡＢＣ测试";
        let issues = check_punctuation(text);
        let fw = issues.iter().find(|i| i.message.contains("全角字母"));
        assert!(fw.is_some(), "应检测到全角字母");
        assert_eq!(fw.unwrap().suggestion, "ABC");
    }

    #[test]
    fn test_fullwidth_number() {
        let text = "数字１２３";
        let issues = check_punctuation(text);
        let fw = issues.iter().find(|i| i.message.contains("全角数字"));
        assert!(fw.is_some(), "应检测到全角数字");
        assert_eq!(fw.unwrap().suggestion, "123");
    }

    #[test]
    fn test_no_issue_clean_text() {
        let text = "你好，世界。这是一个测试！";
        let issues = check_punctuation(text);
        assert!(issues.is_empty(), "规范文本不应有警告");
    }

    #[test]
    fn test_position_is_char_index() {
        let text = "哈啰,world";
        let issues = check_punctuation(text);
        assert!(!issues.is_empty());
        for issue in &issues {
            // position 应小于字符总数
            assert!(issue.position <= text.chars().count());
        }
    }
}
