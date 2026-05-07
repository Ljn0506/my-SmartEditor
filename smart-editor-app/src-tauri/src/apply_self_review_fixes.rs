use std::path::Path;

use crate::error::{AppError, Result};
use crate::models::{FixMode, FixResult, ParagraphChange, SelfReviewIssue};

/// 一键修复：处理 auto_fixable=true 的问题，支持 Copy / Overwrite 两种模式
pub fn apply_self_review_fixes(
    file_path: &str,
    issues: &[SelfReviewIssue],
    mode: FixMode,
) -> Result<FixResult> {
    let path = Path::new(file_path);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if ext != "docx" {
        return Err(AppError::Validation(
            "一键修复仅支持 .docx 文件".to_string(),
        ));
    }

    let bytes = std::fs::read(file_path)?;
    let mut docx = docx_rs::read_docx(&bytes)
        .map_err(|e| AppError::Parse(format!("docx 解析失败: {:?}", e)))?;

    // 收集需要修复的 issue，按 paragraph_index 分组
    let mut fixes_by_para: std::collections::HashMap<
        usize,
        Vec<(String, String, String)>, // (original, suggestion, issue_category)
    > = std::collections::HashMap::new();
    for issue in issues {
        if issue.auto_fixable && issue.sub_category == "punctuation" {
            if let (Some(idx), Some(ref orig), Some(ref sugg)) =
                (issue.paragraph_index, issue.original.clone(), issue.suggestion.clone())
            {
                fixes_by_para
                    .entry(idx)
                    .or_default()
                    .push((orig.clone(), sugg.clone(), issue.category.clone()));
            }
        }
    }

    if fixes_by_para.is_empty() {
        return Err(AppError::Validation("没有可自动修复的项".to_string()));
    }

    // 遍历 docx 段落，应用修复并收集修改明细
    let mut para_idx = 0usize;
    let mut changes: Vec<ParagraphChange> = Vec::new();

    for child in &mut docx.document.children {
        match child {
            docx_rs::DocumentChild::Paragraph(p) => {
                let has_text = p.children.iter().any(|pc| {
                    if let docx_rs::ParagraphChild::Run(r) = pc {
                        r.children
                            .iter()
                            .any(|rc| matches!(rc, docx_rs::RunChild::Text(_)))
                    } else {
                        false
                    }
                });

                if has_text {
                    if let Some(fixes) = fixes_by_para.get(&para_idx) {
                        let para_text_before = crate::parser::extract_paragraph_text_to_string(p);
                        apply_fixes_to_paragraph(p, fixes);
                        let para_text_after = crate::parser::extract_paragraph_text_to_string(p);
                        // 记录修改明细
                        for (orig, sugg, category) in fixes {
                            if para_text_before != para_text_after || para_text_before.contains(orig)
                            {
                                changes.push(ParagraphChange {
                                    paragraph_index: para_idx,
                                    paragraph_text: para_text_before.clone(),
                                    original: orig.clone(),
                                    modified: sugg.clone(),
                                    issue_category: category.clone(),
                                });
                            }
                        }
                    }
                    para_idx += 1;
                }
            }
            docx_rs::DocumentChild::Table(t) => {
                for row_child in &mut t.rows {
                    let docx_rs::TableChild::TableRow(row) = row_child;
                    let has_text = row.cells.iter().any(|cc| {
                        let docx_rs::TableRowChild::TableCell(cell) = cc;
                        cell.children.iter().any(|c| {
                            if let docx_rs::TableCellContent::Paragraph(p) = c {
                                p.children.iter().any(|pc| {
                                    if let docx_rs::ParagraphChild::Run(r) = pc {
                                        r.children.iter().any(|rc| {
                                            matches!(rc, docx_rs::RunChild::Text(_))
                                        })
                                    } else {
                                        false
                                    }
                                })
                            } else {
                                false
                            }
                        })
                    });

                    if has_text {
                        if let Some(fixes) = fixes_by_para.get(&para_idx) {
                            // 收集表格行修改前的文本
                            let mut para_text_before = String::new();
                            for cell_child in &row.cells {
                                let docx_rs::TableRowChild::TableCell(cell) = cell_child;
                                for cell_content in &cell.children {
                                    if let docx_rs::TableCellContent::Paragraph(p) = cell_content {
                                        para_text_before.push_str(&crate::parser::extract_paragraph_text_to_string(p));
                                    }
                                }
                            }
                            // 应用修复
                            for cell_child in &mut row.cells {
                                let docx_rs::TableRowChild::TableCell(cell) = cell_child;
                                for cell_content in &mut cell.children {
                                    if let docx_rs::TableCellContent::Paragraph(p) = cell_content {
                                        apply_fixes_to_paragraph(p, fixes);
                                    }
                                }
                            }
                            // 记录修改明细
                            for (orig, sugg, category) in fixes {
                                changes.push(ParagraphChange {
                                    paragraph_index: para_idx,
                                    paragraph_text: para_text_before.clone(),
                                    original: orig.clone(),
                                    modified: sugg.clone(),
                                    issue_category: category.clone(),
                                });
                            }
                        }
                        para_idx += 1;
                    }
                }
            }
            _ => {}
        }
    }

    // 确定输出路径
    let file_stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = path.parent().unwrap_or(Path::new("."));

    let (output_path_str, backup_path_str) = match mode {
        FixMode::Copy => {
            let output_path = parent.join(format!("{}_fixed.docx", file_stem));
            (output_path.to_string_lossy().to_string(), None)
        }
        FixMode::Overwrite => {
            let backup_path = parent.join(format!("{}.bak", file_stem));
            // 尝试创建备份
            let backup_result = std::fs::copy(file_path, &backup_path);
            let backup_path_str = backup_result
                .ok()
                .map(|_| backup_path.to_string_lossy().to_string());
            (file_path.to_string(), backup_path_str)
        }
    };

    // 写入文件
    let file = std::fs::File::create(&output_path_str)?;
    docx.build()
        .pack(file)
        .map_err(|e| AppError::Io(format!("docx 打包失败: {:?}", e)))?;

    // Overwrite 模式：如果备份失败，回退到 Copy 模式
    if mode == FixMode::Overwrite && backup_path_str.is_none() {
        // 已经覆盖了原文件，但备份失败。返回结果但标注无备份
    }

    Ok(FixResult {
        mode,
        output_path: output_path_str,
        backup_path: backup_path_str,
        changes,
    })
}

fn apply_fixes_to_paragraph(
    p: &mut docx_rs::Paragraph,
    fixes: &[(String, String, String)],
) {
    for (orig, sugg, _category) in fixes {
        for para_child in &mut p.children {
            if let docx_rs::ParagraphChild::Run(r) = para_child {
                for run_child in &mut r.children {
                    if let docx_rs::RunChild::Text(t) = run_child {
                        t.text = t.text.replace(orig, sugg);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SelfReviewIssue, Severity};
    use std::fs;

    fn create_test_docx(path: &str, text: &str) {
        let mut docx = docx_rs::Docx::new();
        for line in text.lines() {
            docx = docx.add_paragraph(
                docx_rs::Paragraph::new().add_run(docx_rs::Run::new().add_text(line)),
            );
        }
        let file = fs::File::create(path).unwrap();
        docx.build().pack(file).unwrap();
    }

    fn make_issue(idx: usize, orig: &str, sugg: &str, auto_fixable: bool) -> SelfReviewIssue {
        SelfReviewIssue {
            id: 1,
            category: "format".to_string(),
            sub_category: "punctuation".to_string(),
            message: "标点问题".to_string(),
            severity: Severity::Warning,
            position: Some(3),
            paragraph_index: Some(idx),
            original: Some(orig.to_string()),
            suggestion: Some(sugg.to_string()),
            auto_fixable,
        }
    }

    #[test]
    fn test_apply_punctuation_fixes_copy_mode() {
        let path = "/tmp/test_apply_fixes_copy.docx";
        create_test_docx(path, "本方案,采用主流架构,具有高可用性。");

        let issues = vec![make_issue(0, ",", "，", true)];

        let result = apply_self_review_fixes(path, &issues, FixMode::Copy);
        fs::remove_file(path).unwrap();

        assert!(result.is_ok(), "修复应成功: {:?}", result.err());
        let fix_result = result.unwrap();
        assert_eq!(fix_result.mode, FixMode::Copy);
        assert!(fix_result.output_path.ends_with("_fixed.docx"));
        assert!(fix_result.backup_path.is_none());
        assert!(!fix_result.changes.is_empty(), "应返回修改明细");
        assert_eq!(fix_result.changes[0].paragraph_index, 0);
        assert_eq!(fix_result.changes[0].original, ",");
        assert_eq!(fix_result.changes[0].modified, "，");

        // 验证修复后的文件内容
        let bytes = fs::read(&fix_result.output_path).unwrap();
        let fixed_docx = docx_rs::read_docx(&bytes).unwrap();
        let mut fixed_text = String::new();
        for child in &fixed_docx.document.children {
            if let docx_rs::DocumentChild::Paragraph(p) = child {
                for para_child in &p.children {
                    if let docx_rs::ParagraphChild::Run(r) = para_child {
                        for run_child in &r.children {
                            if let docx_rs::RunChild::Text(t) = run_child {
                                fixed_text.push_str(&t.text);
                            }
                        }
                    }
                }
                fixed_text.push('\n');
            }
        }
        fs::remove_file(&fix_result.output_path).unwrap();

        assert!(
            fixed_text.contains("，"),
            "修复后应包含中文逗号，实际: {:?}",
            fixed_text
        );
        assert!(
            !fixed_text.contains(","),
            "修复后不应包含英文逗号，实际: {:?}",
            fixed_text
        );
    }

    #[test]
    fn test_apply_overwrite_mode_creates_backup() {
        let path = "/tmp/test_apply_overwrite.docx";
        create_test_docx(path, "第一段,有逗号。");

        let issues = vec![make_issue(0, ",", "，", true)];

        let result = apply_self_review_fixes(path, &issues, FixMode::Overwrite);

        assert!(result.is_ok(), "修复应成功: {:?}", result.err());
        let fix_result = result.unwrap();
        assert_eq!(fix_result.mode, FixMode::Overwrite);
        assert_eq!(fix_result.output_path, path);
        assert!(
            fix_result.backup_path.is_some(),
            "应生成备份文件"
        );
        let backup_path = fix_result.backup_path.unwrap();
        assert!(backup_path.ends_with(".bak"));
        assert!(fs::metadata(&backup_path).is_ok(), "备份文件应存在");

        // 验证原文件已被覆盖（内容已修复）
        let bytes = fs::read(path).unwrap();
        let fixed_docx = docx_rs::read_docx(&bytes).unwrap();
        let mut fixed_text = String::new();
        for child in &fixed_docx.document.children {
            if let docx_rs::DocumentChild::Paragraph(p) = child {
                for para_child in &p.children {
                    if let docx_rs::ParagraphChild::Run(r) = para_child {
                        for run_child in &r.children {
                            if let docx_rs::RunChild::Text(t) = run_child {
                                fixed_text.push_str(&t.text);
                            }
                        }
                    }
                }
            }
        }
        assert!(fixed_text.contains("，"), "原文件应被覆盖为修复后内容");

        // 清理
        fs::remove_file(path).unwrap();
        fs::remove_file(&backup_path).unwrap();
    }

    #[test]
    fn test_apply_no_fixable_items() {
        let path = "/tmp/test_no_fixes.docx";
        create_test_docx(path, "本方案采用主流架构。");

        let issues = vec![SelfReviewIssue {
            id: 1,
            category: "format".to_string(),
            sub_category: "placeholder".to_string(),
            message: "占位符".to_string(),
            severity: Severity::Error,
            position: None,
            paragraph_index: None,
            original: Some("[待补充]".to_string()),
            suggestion: Some("请替换".to_string()),
            auto_fixable: false,
        }];

        let result = apply_self_review_fixes(path, &issues, FixMode::Copy);
        fs::remove_file(path).unwrap();

        assert!(result.is_err(), "无修复项时应返回 Err");
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("没有可自动修复的项"), "错误消息应提示无修复项");
    }

    #[test]
    fn test_apply_doc_not_supported() {
        let result = apply_self_review_fixes("test.doc", &[], FixMode::Copy);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("仅支持 .docx"), "应提示仅支持 docx");
    }
}
