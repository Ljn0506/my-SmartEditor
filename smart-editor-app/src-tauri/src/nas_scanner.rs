use std::path::Path;

use walkdir::WalkDir;

use crate::error::Result;
use crate::models::{DocumentType, Template};
use crate::parser::parse_document;

const SUPPORTED_EXTS: &[&str] = &["docx", "pdf", "xlsx", "xls", "txt", "md"];

/// 扫描目录，返回所有支持的文件路径
pub fn scan_directory(dir_path: &str) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(dir_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if let Some(ext) = entry.path().extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if SUPPORTED_EXTS.contains(&ext.as_str()) {
                files.push(entry.path().to_string_lossy().to_string());
            }
        }
    }
    Ok(files)
}

/// 根据文件名自动识别文档类型（技术 / 商务）
pub fn detect_document_type(file_name: &str) -> DocumentType {
    let lower = file_name.to_lowercase();
    let business_keywords = [
        "符合性",
        "评审",
        "商务",
        "承诺函",
        "偏离表",
        "资质",
        "报价",
        "合同",
        "协议",
        "商务条款",
        "付款",
        "保证金",
        "投标函",
        "符合",
    ];
    let tech_keywords = [
        "技术方案",
        "技术需求",
        "实施方案",
        "设计",
        "架构",
        "等保",
        "渗透",
        "安全",
        "漏洞",
        "防火墙",
        "入侵检测",
        "数据安全",
        "密码",
        "加密",
        "态势感知",
        "soc",
        "waf",
        "态势",
    ];

    let mut biz_score = 0i32;
    let mut tech_score = 0i32;

    for kw in &business_keywords {
        if lower.contains(kw) {
            biz_score += 1;
        }
    }
    for kw in &tech_keywords {
        if lower.contains(kw) {
            tech_score += 1;
        }
    }

    if biz_score >= tech_score {
        DocumentType::Business
    } else {
        DocumentType::Technical
    }
}

/// 将单个文件解析为 Template（未入库）
pub fn file_to_template(file_path: &str) -> Result<Template> {
    let text = parse_document(file_path)?;
    let path = Path::new(file_path);
    let file_name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "未命名".to_string());

    let _doc_type = detect_document_type(&file_name);
    let (doc_attr, business_domain, content_module) = infer_categories(&file_name, &text);

    let tags = infer_tags(&file_name, &text);

    Ok(Template {
        id: None,
        title: file_name,
        content: text,
        content_html: None,
        doc_attr,
        business_domain,
        security_layer: None,
        content_module,
        project_phase: Some("投标阶段".to_string()),
        tags,
        source_file: Some(file_path.to_string()),
        source_para_range: None,
        created_at: None,
        updated_at: None,
        use_count: 0,
        rating: 0,
    })
}

fn infer_categories(
    file_name: &str,
    _text: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let lower = file_name.to_lowercase();

    let doc_attr = if lower.contains("投标") || lower.contains("应答") {
        Some("投标应答".to_string())
    } else if lower.contains("技术方案") || lower.contains("设计方案") {
        Some("技术方案".to_string())
    } else if lower.contains("实施") {
        Some("实施方案".to_string())
    } else if lower.contains("合同") || lower.contains("协议") {
        Some("合同协议".to_string())
    } else {
        Some("汇报材料".to_string())
    };

    let business_domain = if lower.contains("等保") || lower.contains("等级保护") {
        Some("网络安全".to_string())
    } else if lower.contains("渗透") || lower.contains("漏洞") || lower.contains("代码审计")
    {
        Some("应用安全".to_string())
    } else if lower.contains("数据") || lower.contains("数据库") || lower.contains("隐私") {
        Some("数据安全".to_string())
    } else if lower.contains("soc") || lower.contains("运营") || lower.contains("态势") {
        Some("安全运营".to_string())
    } else if lower.contains("管理") || lower.contains("制度") || lower.contains("体系") {
        Some("安全管理".to_string())
    } else {
        Some("网络安全".to_string())
    };

    let content_module = if lower.contains("偏离") || lower.contains("差异") {
        Some("偏离说明".to_string())
    } else if lower.contains("资质") || lower.contains("证书") || lower.contains("业绩") {
        Some("资质证明".to_string())
    } else if lower.contains("案例") || lower.contains("项目经历") {
        Some("案例介绍".to_string())
    } else if lower.contains("实施") || lower.contains("计划") || lower.contains("进度") {
        Some("实施计划".to_string())
    } else if lower.contains("商务") || lower.contains("报价") || lower.contains("合同") {
        Some("商务条款".to_string())
    } else {
        Some("技术方案".to_string())
    };

    (doc_attr, business_domain, content_module)
}

fn infer_tags(file_name: &str, text: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let lower_name = file_name.to_lowercase();
    let lower_text = text.to_lowercase();

    if lower_name.contains("等保") || lower_text.contains("等级保护") {
        tags.push("等保".to_string());
        if lower_text.contains("三级") {
            tags.push("三级".to_string());
        }
        if lower_text.contains("二级") {
            tags.push("二级".to_string());
        }
    }
    if lower_name.contains("渗透") || lower_text.contains("渗透测试") {
        tags.push("渗透测试".to_string());
    }
    if lower_text.contains("偏离表") || lower_text.contains("偏离项") {
        tags.push("偏离表".to_string());
    }
    if lower_name.contains("承诺函") {
        tags.push("承诺函".to_string());
    }
    if lower_name.contains("评审") || lower_name.contains("索引") {
        tags.push("评审索引".to_string());
    }

    tags
}

/// 批量导入结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub failed_files: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_document_type_business() {
        assert_eq!(
            detect_document_type("商务报价偏离表.docx"),
            DocumentType::Business
        );
        assert_eq!(
            detect_document_type("资质审查文件.pdf"),
            DocumentType::Business
        );
        assert_eq!(
            detect_document_type("合同协议.docx"),
            DocumentType::Business
        );
    }

    #[test]
    fn test_detect_document_type_technical() {
        assert_eq!(
            detect_document_type("等保2.0技术方案.docx"),
            DocumentType::Technical
        );
        assert_eq!(
            detect_document_type("渗透测试实施方案.pdf"),
            DocumentType::Technical
        );
        assert_eq!(
            detect_document_type("WAF防火墙配置.md"),
            DocumentType::Technical
        );
    }

    #[test]
    fn test_detect_document_type_tie_goes_to_business() {
        // 当商务和技术分数相等时，应返回 Business
        assert_eq!(detect_document_type("通用文档.txt"), DocumentType::Business);
    }

    #[test]
    fn test_scan_directory() {
        let tmp_dir = std::env::temp_dir().join("smart_editor_scan_test");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();
        std::fs::write(tmp_dir.join("a.txt"), "hello").unwrap();
        std::fs::write(tmp_dir.join("b.pdf"), "%PDF").unwrap();
        std::fs::write(tmp_dir.join("c.jpg"), "binary").unwrap();
        std::fs::create_dir_all(tmp_dir.join("subdir")).unwrap();
        std::fs::write(tmp_dir.join("subdir/d.docx"), "docx").unwrap();

        let files = scan_directory(tmp_dir.to_str().unwrap()).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("a.txt")));
        assert!(files.iter().any(|f| f.ends_with("b.pdf")));
        assert!(files.iter().any(|f| f.ends_with("d.docx")));
        assert!(!files.iter().any(|f| f.ends_with("c.jpg")));

        std::fs::remove_dir_all(&tmp_dir).unwrap();
    }

    #[test]
    fn test_scan_directory_empty() {
        let tmp_dir = std::env::temp_dir().join("smart_editor_scan_empty");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let files = scan_directory(tmp_dir.to_str().unwrap()).unwrap();
        assert!(files.is_empty());
        std::fs::remove_dir_all(&tmp_dir).unwrap();
    }

    #[test]
    fn test_infer_categories() {
        let (doc_attr, domain, module) = infer_categories("等保2.0投标应答技术方案.docx", "");
        assert_eq!(doc_attr, Some("投标应答".to_string()));
        assert_eq!(domain, Some("网络安全".to_string()));
        assert_eq!(module, Some("技术方案".to_string()));

        let (doc_attr2, domain2, module2) = infer_categories("偏离表-资质证书.xlsx", "");
        assert_eq!(doc_attr2, Some("汇报材料".to_string()));
        assert_eq!(domain2, Some("网络安全".to_string()));
        assert_eq!(module2, Some("偏离说明".to_string()));
    }

    #[test]
    fn test_infer_tags() {
        let tags = infer_tags("等保三级方案.docx", "本方案针对等级保护三级要求设计");
        assert!(tags.contains(&"等保".to_string()));
        assert!(tags.contains(&"三级".to_string()));
        assert!(!tags.contains(&"二级".to_string()));
    }
}
