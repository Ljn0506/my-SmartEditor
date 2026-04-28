use std::path::Path;

use crate::error::{AppError, Result};

/// 根据文件扩展名选择对应解析器，提取纯文本
pub fn parse_document(file_path: &str) -> Result<String> {
    let path = Path::new(file_path);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "docx" => parse_docx(file_path),
        "pdf" => parse_pdf(file_path),
        "xlsx" | "xls" => parse_xlsx(file_path),
        "txt" | "md" => parse_txt(file_path),
        _ => Err(AppError::Parse(format!("不支持的文件格式: {}", ext))),
    }
}

fn parse_docx(file_path: &str) -> Result<String> {
    let bytes = std::fs::read(file_path)?;
    let docx = docx_rs::read_docx(&bytes)
        .map_err(|e| AppError::Parse(format!("docx 解析失败: {:?}", e)))?;

    let mut text = String::new();
    for child in &docx.document.children {
        match child {
            docx_rs::DocumentChild::Paragraph(p) => {
                extract_paragraph_text(p, &mut text);
                text.push('\n');
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
                    text.push_str(&row_texts.join("\t"));
                    text.push('\n');
                }
            }
            _ => {}
        }
    }
    Ok(text)
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
}
