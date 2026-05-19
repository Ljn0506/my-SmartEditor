use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::models::{AiConfig, AiProvider};

/// 应用全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai_provider: AiProvider,
    pub ai_base_url: String,
    pub ai_api_key: Option<String>,
    pub ai_model: String,
    #[serde(default)]
    pub ai_timeout_secs: Option<u64>,
    pub meili_host: String,
    pub meili_api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai_provider: AiProvider::Ollama,
            ai_base_url: "http://localhost:11434".to_string(),
            ai_api_key: None,
            ai_model: "qwen2.5:14b".to_string(),
            ai_timeout_secs: None,
            meili_host: "http://localhost:7700".to_string(),
            meili_api_key: None,
        }
    }
}

impl AppConfig {
    pub fn to_ai_config(&self) -> AiConfig {
        AiConfig {
            provider: self.ai_provider,
            base_url: self.ai_base_url.clone(),
            api_key: self.ai_api_key.clone(),
            model: self.ai_model.clone(),
            timeout_secs: self.ai_timeout_secs,
        }
    }

    #[allow(dead_code)]
    pub fn from_ai_config(ai: &AiConfig) -> Self {
        Self {
            ai_provider: ai.provider,
            ai_base_url: ai.base_url.clone(),
            ai_api_key: ai.api_key.clone(),
            ai_model: ai.model.clone(),
            ai_timeout_secs: ai.timeout_secs,
            meili_host: "http://localhost:7700".to_string(),
            meili_api_key: None,
        }
    }
}

/// 从指定路径加载配置；若文件不存在，则创建默认配置并写入。
pub fn load_or_create(path: &Path) -> Result<AppConfig> {
    if path.exists() {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Io(format!("读取配置失败: {}", e)))?;
        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| AppError::Parse(format!("配置解析失败: {}", e)))?;
        Ok(config)
    } else {
        let config = AppConfig::default();
        save(path, &config)?;
        Ok(config)
    }
}

/// 将配置写入指定路径。
pub fn save(path: &Path, config: &AppConfig) -> Result<()> {
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Parse(format!("配置序列化失败: {}", e)))?;
    std::fs::write(path, content).map_err(|e| AppError::Io(format!("写入配置失败: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.ai_provider, AiProvider::Ollama);
        assert_eq!(cfg.ai_base_url, "http://localhost:11434");
        assert_eq!(cfg.ai_model, "qwen2.5:14b");
        assert_eq!(cfg.meili_host, "http://localhost:7700");
        assert!(cfg.ai_api_key.is_none());
        assert!(cfg.meili_api_key.is_none());
    }

    #[test]
    fn test_to_ai_config() {
        let app_cfg = AppConfig {
            ai_provider: AiProvider::Claude,
            ai_base_url: "https://api.anthropic.com".to_string(),
            ai_api_key: Some("sk-test".to_string()),
            ai_model: "claude-3".to_string(),
            ai_timeout_secs: None,
            meili_host: "http://localhost:7700".to_string(),
            meili_api_key: None,
        };
        let ai_cfg = app_cfg.to_ai_config();
        assert_eq!(ai_cfg.provider, AiProvider::Claude);
        assert_eq!(ai_cfg.base_url, "https://api.anthropic.com");
        assert_eq!(ai_cfg.api_key, Some("sk-test".to_string()));
        assert_eq!(ai_cfg.model, "claude-3");
    }

    #[test]
    fn test_save_and_load() {
        let tmp_dir = std::env::temp_dir();
        let path = tmp_dir.join("smart_editor_test_config.json");
        let _ = std::fs::remove_file(&path);

        let cfg = AppConfig {
            ai_provider: AiProvider::DeepSeek,
            ai_base_url: "https://api.deepseek.com".to_string(),
            ai_api_key: Some("sk-deep".to_string()),
            ai_model: "deepseek-chat".to_string(),
            ai_timeout_secs: None,
            meili_host: "http://localhost:7700".to_string(),
            meili_api_key: Some("master-key".to_string()),
        };

        save(&path, &cfg).unwrap();
        assert!(path.exists());

        let loaded = load_or_create(&path).unwrap();
        assert_eq!(loaded.ai_provider, AiProvider::DeepSeek);
        assert_eq!(loaded.ai_base_url, "https://api.deepseek.com");
        assert_eq!(loaded.ai_api_key, Some("sk-deep".to_string()));
        assert_eq!(loaded.ai_model, "deepseek-chat");
        assert_eq!(loaded.meili_api_key, Some("master-key".to_string()));

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_load_or_create_creates_default_when_missing() {
        let tmp_dir = std::env::temp_dir();
        let path = tmp_dir.join("smart_editor_test_missing_config.json");
        let _ = std::fs::remove_file(&path);

        assert!(!path.exists());
        let cfg = load_or_create(&path).unwrap();
        assert!(path.exists());
        assert_eq!(cfg.ai_provider, AiProvider::Ollama);

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_load_invalid_json_fails() {
        let tmp_dir = std::env::temp_dir();
        let path = tmp_dir.join("smart_editor_test_bad_config.json");
        let mut file = std::fs::File::create(&path).unwrap();
        write!(file, "{{ invalid json").unwrap();
        drop(file);

        let result = load_or_create(&path);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("配置解析失败"));

        std::fs::remove_file(&path).unwrap();
    }
}
