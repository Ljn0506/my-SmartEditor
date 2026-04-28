use arboard::Clipboard;

use crate::error::{AppError, Result};

/// 将富文本 HTML 写入系统剪贴板
pub fn write_html_to_clipboard(html: &str) -> Result<()> {
    let mut clipboard =
        Clipboard::new().map_err(|e| AppError::Io(format!("剪贴板访问失败: {}", e)))?;
    clipboard
        .set_html(html, Some(html))
        .map_err(|e| AppError::Io(format!("剪贴板写入失败: {}", e)))?;
    Ok(())
}

/// 将纯文本写入系统剪贴板
pub fn write_text_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard =
        Clipboard::new().map_err(|e| AppError::Io(format!("剪贴板访问失败: {}", e)))?;
    clipboard
        .set_text(text)
        .map_err(|e| AppError::Io(format!("剪贴板写入失败: {}", e)))?;
    Ok(())
}
