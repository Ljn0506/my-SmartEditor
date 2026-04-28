use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize, Clone)]
pub enum AppError {
    Database(String),
    Search(String),
    Parse(String),
    Ai(String),
    Io(String),
    // Config(String),   // 预留：配置错误处理
    // NotFound(String), // 预留：资源未找到处理
    Validation(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "数据库错误: {}", msg),
            AppError::Search(msg) => write!(f, "搜索错误: {}", msg),
            AppError::Parse(msg) => write!(f, "解析错误: {}", msg),
            AppError::Ai(msg) => write!(f, "AI 错误: {}", msg),
            AppError::Io(msg) => write!(f, "IO 错误: {}", msg),
            // AppError::Config(msg) => write!(f, "配置错误: {}", msg),
            // AppError::NotFound(msg) => write!(f, "未找到: {}", msg),
            AppError::Validation(msg) => write!(f, "校验错误: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<meilisearch_sdk::errors::Error> for AppError {
    fn from(err: meilisearch_sdk::errors::Error) -> Self {
        AppError::Search(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Parse(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Ai(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
