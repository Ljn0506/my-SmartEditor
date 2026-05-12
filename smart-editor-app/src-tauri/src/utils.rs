use crate::error::{AppError, Result};

pub fn validate_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(AppError::Validation("路径不能为空".to_string()));
    }
    let p = std::path::Path::new(path);
    if p.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(AppError::Validation("非法文件路径".to_string()));
    }
    Ok(())
}
