use std::path::Path;

use crate::error::{AppError, Result};
use crate::models::{Paragraph, ParsedDocumentStructured};

/// 根据文件扩展名选择对应解析器，提取纯文本
pub fn parse_document(file_path: &str) -> Result<String> {
    let path = Path::new(file_path);
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(AppError::Validation("非法文件路径".to_string()));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "docx" => parse_docx(file_path).map(|(text, _)| text),
        "doc" => Err(AppError::Parse(
            "doc 格式不支持，请转换为 .docx 后重试".to_string(),
        )),
        "pdf" => parse_pdf(file_path),
        "xlsx" | "xls" => parse_xlsx(file_path),
        "txt" | "md" => parse_txt(file_path),
        _ => Err(AppError::Parse(format!("不支持的文件格式: {}", ext))),
    }
}

/// 结构化解析：返回纯文本 + 段落列表（T6）
/// .docx 按 docx-rs 原始段落单元切分，其他格式 fallback 到按行切分
pub fn parse_document_structured(file_path: &str) -> Result<ParsedDocumentStructured> {
    let path = Path::new(file_path);
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(AppError::Validation("非法文件路径".to_string()));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "docx" => {
            let (text, paragraphs) = parse_docx(file_path)?;
            Ok(ParsedDocumentStructured { text, paragraphs })
        }
        "doc" => Err(AppError::Parse(
            "doc 格式不支持，请转换为 .docx 后重试".to_string(),
        )),
        _ => {
            // fallback: 纯文本 + 按行切分（跳过空行）
            let text = parse_document(file_path)?;
            let mut paragraphs = Vec::new();
            let mut char_offset = 0usize;
            let mut idx = 0usize;

            for line in text.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    paragraphs.push(Paragraph {
                        index: idx,
                        text: trimmed.to_string(),
                        char_offset,
                    });
                    idx += 1;
                }
                char_offset += line.chars().count() + 1; // +1 for '\n'
            }

            Ok(ParsedDocumentStructured { text, paragraphs })
        }
    }
}

/// 解析 docx，返回 (纯文本, 段落列表)
/// 段落按 docx-rs 原始结构提取：Paragraph 和 TableRow 各为一个段落单元
fn parse_docx(file_path: &str) -> Result<(String, Vec<Paragraph>)> {
    let bytes = std::fs::read(file_path)?;
    let docx = docx_rs::read_docx(&bytes)
        .map_err(|e| AppError::Parse(format!("docx 解析失败: {:?}", e)))?;

    let mut text = String::new();
    let mut paragraphs = Vec::new();
    let mut char_offset = 0usize;
    let mut idx = 0usize;

    for child in &docx.document.children {
        match child {
            docx_rs::DocumentChild::Paragraph(p) => {
                let mut para_text = String::new();
                extract_paragraph_text(p, &mut para_text);

                let para_len = para_text.chars().count();
                if para_len > 0 {
                    paragraphs.push(Paragraph {
                        index: idx,
                        text: para_text.clone(),
                        char_offset,
                    });
                    idx += 1;
                }

                text.push_str(&para_text);
                text.push('\n');
                char_offset += para_len + 1; // +1 for '\n'
            }
            docx_rs::DocumentChild::Table(t) => {
                for row_child in &t.rows {
                    let docx_rs::TableChild::TableRow(row) = row_child;
                    let mut row_texts = Vec::new();
                    for cell_child in &row.cells {
                        let docx_rs::TableRowChild::TableCell(cell) = cell_child;
                        let mut cell_text = String::new();
                        for cell_content in &cell.children {
                            if let docx_rs::TableCellContent::Paragraph(p) = cell_content {
                                extract_paragraph_text(p, &mut cell_text);
                            }
                        }
                        row_texts.push(cell_text);
                    }
                    let row_text = row_texts.join("\t");
                    let row_len = row_text.chars().count();

                    if row_len > 0 {
                        paragraphs.push(Paragraph {
                            index: idx,
                            text: row_text.clone(),
                            char_offset,
                        });
                        idx += 1;
                    }

                    text.push_str(&row_text);
                    text.push('\n');
                    char_offset += row_len + 1;
                }
            }
            _ => {}
        }
    }

    Ok((text, paragraphs))
}

fn extract_paragraph_text(p: &docx_rs::Paragraph, buf: &mut String) {
    for para_child in &p.children {
        if let docx_rs::ParagraphChild::Run(r) = para_child {
            for run_child in &r.children {
                if let docx_rs::RunChild::Text(t) = run_child {
                    buf.push_str(&t.text);
                }
            }
        }
    }
}

fn parse_pdf(file_path: &str) -> Result<String> {
    let text = pdf_extract::extract_text(file_path)
        .map_err(|e| AppError::Parse(format!("pdf 解析失败: {:?}", e)))?;
    Ok(text)
}

fn parse_xlsx(file_path: &str) -> Result<String> {
    use calamine::{open_workbook, Reader, Xlsx};

    let mut workbook: Xlsx<_> =
        open_workbook(file_path).map_err(|e| AppError::Parse(format!("xlsx 打开失败: {:?}", e)))?;

    let mut text = String::new();
    let sheet_names = workbook.sheet_names().to_vec();
    let max_sheets = sheet_names.len().min(10); // 设计规格：最多前 10 个工作表

    for sheet_name in &sheet_names[..max_sheets] {
        text.push_str(&format!("--- Sheet: {} ---\n", sheet_name));
        match workbook.worksheet_range(sheet_name) {
            Ok(range) => {
                for row in range.rows() {
                    let row_text: Vec<String> = row.iter().map(|c| c.to_string()).collect();
                    text.push_str(&row_text.join("\t"));
                    text.push('\n');
                }
            }
            Err(e) => {
                log::warn!("读取工作表 {} 失败: {:?}", sheet_name, e);
            }
        }
    }
    Ok(text)
}

fn parse_txt(file_path: &str) -> Result<String> {
    std::fs::read_to_string(file_path).map_err(|e| AppError::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    ///  helper：创建一个包含两个段落的测试 docx 文件
    fn create_test_docx(path: &str) {
        let mut docx = docx_rs::Docx::new();
        docx = docx.add_paragraph(docx_rs::Paragraph::new().add_run(docx_rs::Run::new().add_text("第一段测试内容。")));
        docx = docx.add_paragraph(docx_rs::Paragraph::new().add_run(docx_rs::Run::new().add_text("第二段测试内容。")));
        let file = fs::File::create(path).unwrap();
        docx.build().pack(file).unwrap();
    }

    #[test]
    fn test_parse_document_txt() {
        let path = "/tmp/test_parse_document.txt";
        fs::write(path, "这是一段测试文本。\n第二行内容。").unwrap();
        let result = parse_document(path);
        fs::remove_file(path).unwrap();
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("测试文本"));
        assert!(text.contains("第二行内容"));
    }

    #[test]
    fn test_parse_document_unsupported_format() {
        let result = parse_document("test.xyz");
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("不支持的文件格式"));
    }

    #[test]
    fn test_parse_document_docx_route() {
        // 验证 docx 扩展名走正确路由（文件不存在时返回 IO 错误而非不支持格式）
        let result = parse_document("nonexistent.docx");
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(!err_msg.contains("不支持的文件格式"));
    }

    #[test]
    fn test_parse_document_xlsx_route() {
        let result = parse_document("nonexistent.xlsx");
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(!err_msg.contains("不支持的文件格式"));
    }

    /// B1: parse_document_structured(".docx") 返回 paragraphs 数量 > 0 且 char_offset 严格递增
    #[test]
    fn test_parse_document_structured_docx_paragraphs() {
        let path = "/tmp/test_parse_document_structured.docx";
        create_test_docx(path);
        let result = parse_document_structured(path);
        fs::remove_file(path).unwrap();

        assert!(result.is_ok(), "解析应成功: {:?}", result.err());
        let doc = result.unwrap();

        // paragraphs 数量 > 0
        assert!(
            doc.paragraphs.len() > 0,
            "paragraphs 数量应 > 0，实际: {}",
            doc.paragraphs.len()
        );

        // char_offset 严格递增
        for i in 1..doc.paragraphs.len() {
            let prev = &doc.paragraphs[i - 1];
            let curr = &doc.paragraphs[i];
            assert!(
                curr.char_offset > prev.char_offset,
                "char_offset 应严格递增: paragraphs[{}].char_offset={} >= paragraphs[{}].char_offset={}",
                i,
                curr.char_offset,
                i - 1,
                prev.char_offset
            );
        }

        // index 从 0 开始连续递增
        for (i, p) in doc.paragraphs.iter().enumerate() {
            assert_eq!(p.index, i, "paragraph index 应等于数组位置");
        }

        // text 应包含段落内容
        assert!(doc.text.contains("第一段测试内容"));
        assert!(doc.text.contains("第二段测试内容"));
    }

    /// B2: parse_document_structured(".doc") 返回明确 Err("unsupported_format")
    #[test]
    fn test_parse_document_structured_doc_unsupported() {
        let result = parse_document_structured("legacy.doc");
        assert!(result.is_err(), ".doc 应返回 Err");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("doc 格式不支持") || err_msg.contains("请转换为 .docx"),
            ".doc 错误消息应给出可操作建议，当前: {:?}",
            err_msg
        );
    }

    /// T9-B2（遗留）：.doc 纯文本解析也应返回明确 actionable 错误
    #[test]
    fn test_parse_document_doc_actionable_error() {
        let result = parse_document("legacy.doc");
        assert!(result.is_err(), ".doc 应返回 Err");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("doc 格式不支持") || err_msg.contains("请转换为 .docx"),
            "T9-B2: .doc 错误消息应给出可操作建议，当前: {:?}",
            err_msg
        );
    }
}
