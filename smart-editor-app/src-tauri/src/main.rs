// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
#[cfg(feature = "docx")]
mod apply_self_review_fixes;
mod category;
mod clipboard;
mod config;
mod db;
mod desensitize;
mod deviation;
mod error;
mod models;
mod nas_scanner;
mod parser;
mod punctuation;
mod search;
mod self_review;
mod utils;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{Emitter, Manager};

use crate::ai::AiClient;
use crate::db::Database;
use crate::desensitize::DesensitizeHit;
use crate::models::{
    AiConfig, AiProvider, DeviationReport, DocumentType, ParsedDocument, ParsedDocumentStructured, ParsedRequirements,
    SearchResult, SelfReviewReport, Template,
};
use crate::nas_scanner::{scan_directory, ImportResult};
use crate::search::SearchEngine;

pub struct AppState {
    db: Arc<Mutex<Database>>,
    search: Option<SearchEngine>,
    ai: Mutex<AiClient>,
    config_path: PathBuf,

}

impl AppState {
    fn ai_client(&self) -> Result<AiClient, String> {
        let guard = self.ai.lock().map_err(|e| format!("AI 客户端锁定失败: {}", e))?;
        Ok(guard.clone())
    }
}

// --- 文档解析命令 ---

#[tauri::command]
fn parse_document(file_path: String) -> Result<ParsedDocumentStructured, String> {
    crate::utils::validate_path(&file_path).map_err(|e| e.to_string())?;
    let meta = std::fs::metadata(&file_path).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("路径不是有效的文件".to_string());
    }
    parser::parse_document_structured(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn parse_requirement_file(file_path: String) -> Result<ParsedDocument, String> {
    crate::utils::validate_path(&file_path).map_err(|e| e.to_string())?;
    let meta = std::fs::metadata(&file_path).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("路径不是有效的文件".to_string());
    }
    let text = parser::parse_document(&file_path).map_err(|e| e.to_string())?;
    let path = std::path::Path::new(&file_path);
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let document_type = nas_scanner::detect_document_type(&file_name);
    Ok(ParsedDocument {
        file_path,
        file_name,
        document_type,
        text,
    })
}

#[tauri::command]
async fn parse_and_extract(
    state: tauri::State<'_, AppState>,
    file_path: String,
) -> Result<ParsedRequirements, String> {
    let parsed = parse_requirement_file(file_path).map_err(|e| e.to_string())?;
    let ai = state.ai_client()?;
    ai.extract_requirements(&parsed.text, parsed.document_type)
        .await
        .map_err(|e| e.to_string())
}

// --- 模板库命令 ---

#[tauri::command]
async fn create_template(
    state: tauri::State<'_, AppState>,
    mut template: Template,
) -> Result<i64, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.insert_template(&mut template).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_template(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<Option<Template>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.get_template(id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn update_template(
    state: tauri::State<'_, AppState>,
    template: Template,
) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.update_template(&template).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn delete_template(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.delete_template(id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn list_all_templates(
    state: tauri::State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<Template>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let db = db.lock().map_err(|e| e.to_string())?;
        let filter = crate::models::TemplateFilter {
            limit: limit.unwrap_or(1000),
            ..Default::default()
        };
        db.search_templates(&filter).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn search_templates_db(
    state: tauri::State<'_, AppState>,
    filter: crate::models::TemplateFilter,
) -> Result<Vec<Template>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.search_templates(&filter).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

// --- Meilisearch 命令 ---

#[tauri::command]
async fn search_templates_meili(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<SearchResult>, String> {
    let search = state
        .search
        .as_ref()
        .ok_or("搜索服务未初始化，请检查 Meilisearch 配置")?;
    search
        .search(&query, limit.unwrap_or(10), offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn init_search_index(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let search = state
        .search
        .as_ref()
        .ok_or("搜索服务未初始化，请检查 Meilisearch 配置")?;
    search.init_index().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn index_templates(
    state: tauri::State<'_, AppState>,
    templates: Vec<Template>,
) -> Result<(), String> {
    let search = state
        .search
        .as_ref()
        .ok_or("搜索服务未初始化，请检查 Meilisearch 配置")?;
    search
        .add_or_update_templates(&templates)
        .await
        .map_err(|e| e.to_string())
}

// --- AI 命令 ---

#[tauri::command]
fn get_ai_config(state: tauri::State<'_, AppState>) -> Result<AiConfig, String> {
    let app_config = config::load_or_create(&state.config_path).map_err(|e| e.to_string())?;
    let mut ai = app_config.to_ai_config();
    // 脱敏：避免 API Key 泄露到前端
    ai.api_key = ai.api_key.map(|k| {
        if k.len() > 8 {
            format!("{}****", &k[..4])
        } else {
            "****".to_string()
        }
    });
    Ok(ai)
}

#[tauri::command]
fn update_ai_config(state: tauri::State<'_, AppState>, ai: AiConfig) -> Result<(), String> {
    let mut app_config = config::load_or_create(&state.config_path).map_err(|e| e.to_string())?;
    app_config.ai_provider = ai.provider;
    app_config.ai_base_url = ai.base_url.clone();
    app_config.ai_api_key = ai.api_key.clone();
    app_config.ai_model = ai.model.clone();
    app_config.ai_timeout_secs = ai.timeout_secs;
    config::save(&state.config_path, &app_config).map_err(|e| e.to_string())?;
    let new_ai = AiClient::new(ai).map_err(|e| e.to_string())?;
    let mut guard = state.ai.lock().map_err(|e| e.to_string())?;
    *guard = new_ai;
    Ok(())
}

#[tauri::command]
async fn extract_requirements(
    state: tauri::State<'_, AppState>,
    text: String,
    doc_type: DocumentType,
) -> Result<ParsedRequirements, String> {
    let ai = state.ai_client()?;
    ai.extract_requirements(&text, doc_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn generate_draft(
    state: tauri::State<'_, AppState>,
    requirements: String,
    references: Vec<String>,
    doc_type: DocumentType,
) -> Result<String, String> {
    let ai = state.ai_client()?;
    ai.generate_draft(&requirements, &references, doc_type)
        .await
        .map_err(|e| e.to_string())
}

// --- NAS 扫描命令 ---

#[tauri::command]
fn scan_nas_directory(path: String) -> Result<Vec<String>, String> {
    scan_directory(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_nas_files(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
) -> Result<ImportResult, String> {
    let total = paths.len();
    let app_emit = app.clone();

    // 1. rayon 并行解析文件（CPU-bound）+ 流式进度
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (templates, mut failed_files) = tokio::task::spawn_blocking(move || {
        use rayon::prelude::*;
        let results: Vec<(Option<Template>, Option<String>)> = paths
            .into_par_iter()
            .map(|path| {
                if let Err(e) = crate::utils::validate_path(&path) {
                    return (None, Some(format!("路径验证失败: {}", e)));
                }
                let result = nas_scanner::file_to_template(&path);
                let current = progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                if current % 5 == 0 || current == total {
                    let _ = app_emit.emit(
                        "import_progress",
                        serde_json::json!({
                            "current": current,
                            "total": total,
                            "phase": "parse",
                            "file": path,
                        }),
                    );
                }
                match result {
                    Ok(t) => (Some(t), None),
                    Err(e) => {
                        log::error!("解析文件失败 {}: {}", path, e);
                        (None, Some(path))
                    }
                }
            })
            .collect();
        let mut templates = Vec::new();
        let mut failed = Vec::new();
        for (t, f) in results {
            if let Some(t) = t {
                templates.push(t);
            }
            if let Some(f) = f {
                failed.push(f);
            }
        }
        (templates, failed)
    })
    .await
    .map_err(|e| e.to_string())?;

    // 2. spawn_blocking 入库 SQLite（IO-bound，不卡 IPC）
    let db = state.db.clone();
    let templates_for_db = templates.clone();
    let sqlite_failed = tokio::task::spawn_blocking(move || {
        let mut sqlite_failed = Vec::new();
        let db = match db.lock() {
            Ok(g) => g,
            Err(e) => {
                log::error!("获取数据库锁失败: {}", e);
                return sqlite_failed;
            }
        };
        for mut t in templates_for_db {
            if let Err(e) = db.insert_template(&mut t) {
                log::error!("SQLite 入库失败: {}", e);
                if let Some(ref path) = t.source_file {
                    sqlite_failed.push(path.clone());
                }
            }
        }
        sqlite_failed
    })
    .await
    .map_err(|e| e.to_string())?;

    failed_files.extend(sqlite_failed);

    // 过滤出成功入库且有 id 的模板，用于 Meilisearch 索引
    let templates_to_index: Vec<Template> = templates
        .into_iter()
        .filter(|t| {
            t.id.is_some()
                && t.source_file.as_ref().is_none_or(|sf| !failed_files.contains(sf))
        })
        .collect();

    if !templates_to_index.is_empty() {
        let _ = app.emit(
            "import_progress",
            serde_json::json!({ "current": total, "total": total, "phase": "index" }),
        );
        if let Some(search) = state.search.as_ref() {
            if let Err(e) = search.add_or_update_templates(&templates_to_index).await {
                log::error!("Meilisearch 索引失败: {}", e);
            }
        } else {
            log::warn!("搜索服务未初始化，跳过 Meilisearch 索引");
        }
    }

    Ok(ImportResult {
        success_count: templates_to_index.len(),
        failed_count: failed_files.len(),
        failed_files,
    })
}

#[tauri::command]
fn detect_document_type(file_name: String) -> DocumentType {
    nas_scanner::detect_document_type(&file_name)
}

async fn spawn_blocking_cmd<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(f).await.map_err(|e| e.to_string())
}

// --- 偏离检查命令 ---

#[tauri::command]
async fn check_deviation(bid_text: String, req_text: String) -> Result<DeviationReport, String> {
    spawn_blocking_cmd(move || deviation::check_deviation(&bid_text, &req_text)).await
}

#[tauri::command]
async fn check_deviation_files(bid_path: String, req_path: String) -> Result<DeviationReport, String> {
    crate::utils::validate_path(&bid_path).map_err(|e| e.to_string())?;
    crate::utils::validate_path(&req_path).map_err(|e| e.to_string())?;
    tokio::task::spawn_blocking(move || {
        let bid_text = parser::parse_document(&bid_path).map_err(|e| e.to_string())?;
        let req_text = parser::parse_document(&req_path).map_err(|e| e.to_string())?;
        Ok(deviation::check_deviation(&bid_text, &req_text))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn check_deviation_items(
    req_items: Vec<crate::models::RequirementItem>,
    bid_text: String,
) -> Result<DeviationReport, String> {
    spawn_blocking_cmd(move || deviation::check_deviation_items(&req_items, &bid_text)).await
}

// --- 脱敏命令 ---

#[tauri::command]
fn desensitize_text(text: String) -> Result<String, String> {
    let engine = desensitize::Desensitizer::new();
    engine.desensitize(&text).map_err(|e| e.to_string())
}

#[tauri::command]
fn analyze_desensitize(text: String) -> Result<Vec<DesensitizeHit>, String> {
    let engine = desensitize::Desensitizer::new();
    Ok(engine.analyze(&text))
}

// --- 剪贴板命令 ---

#[tauri::command]
fn write_clipboard_html(html: String) -> Result<(), String> {
    clipboard::write_html_to_clipboard(&html).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_clipboard_text(text: String) -> Result<(), String> {
    clipboard::write_text_to_clipboard(&text).map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_fatal_risks_text(text: String) -> Result<Vec<crate::models::FatalRisk>, String> {
    spawn_blocking_cmd(move || deviation::check_fatal_risks(&text)).await
}

#[tauri::command]
fn export_deviation_report_markdown(report: crate::models::DeviationReport) -> String {
    report.to_markdown()
}

#[tauri::command]
fn export_deviation_report_html(report: crate::models::DeviationReport) -> String {
    report.to_html()
}

#[tauri::command]
fn export_deviation_report_json(report: crate::models::DeviationReport) -> String {
    report.to_json()
}

#[tauri::command]
async fn check_punctuation(text: String) -> Result<Vec<punctuation::PunctuationIssue>, String> {
    spawn_blocking_cmd(move || punctuation::check_punctuation(&text)).await
}

#[tauri::command]
async fn check_self_review(text: String) -> Result<SelfReviewReport, String> {
    spawn_blocking_cmd(move || self_review::check_self_review(&text)).await
}

/// T5: 完整的投标文件自查（本地 + AI）
#[tauri::command]
async fn check_self_review_async(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<SelfReviewReport, String> {
    // 1. 本地检查（重复 / 敏感信息 / 占位符 / 标点）
    let mut report = self_review::check_self_review(&text);

    // 2. AI 检查（矛盾 / 逻辑）并行执行，带 60s 超时
    let ai = state.ai_client().map_err(|e| e.to_string())?;
    let (contradiction_result, logic_result) = tokio::join!(
        tokio::time::timeout(Duration::from_secs(60), ai.check_contradictions(&text)),
        tokio::time::timeout(Duration::from_secs(60), ai.check_context_logic(&text)),
    );
    match contradiction_result {
        Ok(Ok(mut issues)) => report.issues.append(&mut issues),
        Ok(Err(e)) => log::warn!("AI 矛盾检测失败: {}", e),
        Err(_) => log::warn!("AI 矛盾检测超时（60s），已降级为仅本地检查"),
    }
    match logic_result {
        Ok(Ok(mut issues)) => report.issues.append(&mut issues),
        Ok(Err(e)) => log::warn!("AI 逻辑检测失败: {}", e),
        Err(_) => log::warn!("AI 逻辑检测超时（60s），已降级为仅本地检查"),
    }

    // 3. 重新编号 + 按 severity 排序
    for (i, issue) in report.issues.iter_mut().enumerate() {
        issue.id = (i + 1) as i64;
    }
    report.issues.sort_by_key(|i| match i.severity {
        crate::models::Severity::Error => 0,
        crate::models::Severity::Warning => 1,
        crate::models::Severity::Info => 2,
    });

    Ok(report)
}

#[cfg(feature = "docx")]
#[tauri::command]
fn apply_self_review_fixes(
    file_path: String,
    issues: Vec<crate::models::SelfReviewIssue>,
    mode: crate::models::FixMode,
) -> Result<crate::models::FixResult, String> {
    crate::utils::validate_path(&file_path).map_err(|e| e.to_string())?;
    crate::apply_self_review_fixes::apply_self_review_fixes(&file_path, &issues, mode)
        .map_err(|e| e.to_string())
}

#[cfg(not(feature = "docx"))]
#[tauri::command]
fn apply_self_review_fixes(
    _file_path: String,
    _issues: Vec<crate::models::SelfReviewIssue>,
    _mode: crate::models::FixMode,
) -> Result<crate::models::FixResult, String> {
    Err("docx 修复功能未启用，请使用 --features docx 编译".to_string())
}

// --- Phase 2: 智能生成命令 ---

#[tauri::command]
async fn generate_outline(
    state: tauri::State<'_, AppState>,
    requirements_text: String,
    doc_type: crate::models::DocumentType,
) -> Result<Vec<crate::models::CardOutline>, String> {
    // 从知识库检索相关素材
    let mut refs: Vec<String> = vec![];
    if let Some(search) = state.search.as_ref() {
        // 简单提取前几个关键词组合进行搜索
        let queries: Vec<&str> = requirements_text.split_whitespace().take(5).collect();
        for q in queries {
            if q.len() >= 2 {
                match search.search(q, 2, None).await {
                    Ok(results) => {
                        for r in results {
                            if !r.content.is_empty() {
                                refs.push(format!("{}
{}", r.title, r.content));
                            }
                        }
                    }
                    Err(e) => log::warn!("知识库搜索失败: {}", e),
                }
            }
        }
        refs.sort();
        refs.dedup();
        refs.truncate(5);
    }

    let ai = state.ai_client().map_err(|e| e.to_string())?;
    ai.generate_outline(&requirements_text, doc_type, &refs)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn generate_card(
    state: tauri::State<'_, AppState>,
    outline: crate::models::CardOutline,
    requirements_text: String,
    _references: Vec<String>,
    doc_type: crate::models::DocumentType,
    global_params: Option<crate::models::GlobalParams>,
) -> Result<crate::models::Card, String> {
    // 从知识库搜索相关素材
    let mut refs: Vec<String> = vec![];
    let mut source_files: Vec<String> = vec![];
    if let Some(search) = state.search.as_ref() {
        match search.search(&outline.title, 3, None).await {
            Ok(results) => {
                for r in results {
                    if !r.content.is_empty() {
                        refs.push(format!("{}
{}", r.title, r.content));
                    }
                    if let Some(sf) = r.source_file {
                        if !sf.is_empty() {
                            source_files.push(sf);
                        }
                    }
                }
            }
            Err(e) => log::warn!("知识库搜索失败: {}", e),
        }
    }
    source_files.sort();
    source_files.dedup();

    let ai = state.ai_client().map_err(|e| e.to_string())?;
    let (content, risk_flags) = ai
        .generate_chapter(&outline.title, &outline.chapter, &requirements_text, &refs, doc_type, global_params.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    // 提取 [PARAM:xxx] 占位符
    let param_placeholders = crate::ai::extract_param_placeholders(&content);

    Ok(crate::models::Card {
        id: outline.id,
        chapter: outline.chapter,
        title: outline.title,
        content,
        source_refs: source_files,
        document_target: outline.document_target,
        status: crate::models::CardStatus::Draft,
        generated_by: "ai".to_string(),
        related_cards: vec![],
        param_placeholders,
        risk_flags,
    })
}

#[tauri::command]
fn save_cards(
    state: tauri::State<'_, AppState>,
    document_target: String,
    cards: Vec<crate::models::Card>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_cards(&document_target, &cards).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_cards(
    state: tauri::State<'_, AppState>,
    document_target: String,
) -> Result<Vec<crate::models::Card>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_cards(&document_target).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_card(
    state: tauri::State<'_, AppState>,
    _document_target: String,
    card: crate::models::Card,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_card(&card).map_err(|e| e.to_string())
}

#[tauri::command]
fn confirm_all_cards(
    state: tauri::State<'_, AppState>,
    document_target: String,
) -> Result<(), String> {
    if document_target == "business" {
        return Err("商务卡片涉及敏感条款，必须逐张审核，不支持一键确认".to_string());
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut cards = db.get_cards(&document_target).map_err(|e| e.to_string())?;
    for card in cards.iter_mut() {
        if card.status == crate::models::CardStatus::Draft {
            card.status = crate::models::CardStatus::Confirmed;
        }
    }
    db.save_cards(&document_target, &cards).map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_consistency(
    state: tauri::State<'_, AppState>,
    cards: Vec<crate::models::Card>,
) -> Result<crate::models::ConsistencyReport, String> {
    let ai = state.ai_client().map_err(|e| e.to_string())?;
    ai.check_consistency(&cards)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_global_params(
    state: tauri::State<'_, AppState>,
) -> Result<Option<crate::models::GlobalParams>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_global_params().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_global_params(
    state: tauri::State<'_, AppState>,
    params: crate::models::GlobalParams,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_global_params(&params).map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_cross_document_consistency(
    state: tauri::State<'_, AppState>,
    global_params: crate::models::GlobalParams,
    technical_cards: Vec<crate::models::Card>,
    business_cards: Vec<crate::models::Card>,
) -> Result<crate::models::ConsistencyReport, String> {
    let ai = state.ai_client().map_err(|e| e.to_string())?;
    ai.check_cross_document_consistency(&global_params, &technical_cards, &business_cards)
        .await
        .map_err(|e| e.to_string())
}

// --- 应用入口 ---

fn main() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            std::fs::create_dir_all(&app_data_dir).ok();

            let db_path = app_data_dir.join("smart_editor.db");
            let db = Database::new(db_path.to_str().unwrap()).unwrap_or_else(|e| {
                log::error!("数据库初始化失败: {}，应用将无法正常运行", e);
                panic!("数据库初始化失败: {}", e);
            });

            let config_path = app_data_dir.join("config.json");
            let app_config = config::load_or_create(&config_path).unwrap_or_else(|e| {
                log::warn!("配置加载失败: {}，使用默认配置", e);
                config::AppConfig::default()
            });

            let search = match SearchEngine::new(
                &app_config.meili_host,
                app_config.meili_api_key.as_deref(),
            ) {
                Ok(s) => Some(s),
                Err(e) => {
                    log::warn!("搜索引擎初始化失败: {}，搜索功能将不可用", e);
                    None
                }
            };

            let ai = AiClient::new(app_config.to_ai_config()).unwrap_or_else(|e| {
                log::warn!("AI 客户端初始化失败（URL 校验不通过）: {}，使用默认配置重试", e);
                let default = config::AppConfig::default();
                AiClient::new(default.to_ai_config()).unwrap_or_else(|e2| {
                    log::warn!("默认配置也无法通过 URL 校验: {}，AI 功能将不可用，请前往设置配置有效的 AI 地址", e2);
                    // 降级：使用公网地址创建 client（不会实际工作，但保证应用能启动）
                    AiClient::new(AiConfig {
                        provider: AiProvider::Ollama,
                        base_url: "https://api.openai.com".to_string(),
                        model: "dummy".to_string(),
                        api_key: None,
                        timeout_secs: None,
                    }).expect("降级配置应合法")
                })
            });

            app.manage(AppState {
                db: Arc::new(Mutex::new(db)),
                search,
                ai: Mutex::new(ai),
                config_path,

            });

            let app_handle = app.handle().clone();
            let window = app.get_webview_window("main").unwrap();
            window.on_window_event(move |event| {
                let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event else { return; };
                if paths.is_empty() { return; }
                let files: Vec<serde_json::Value> = paths.iter().filter_map(|path| {
                    let path_str = path.to_string_lossy().to_string();
                    if crate::utils::validate_path(&path_str).is_err() {
                        return None;
                    }
                    let name = path.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    Some(serde_json::json!({ "path": path_str, "name": name }))
                }).collect();
                let _ = app_handle.emit("file-dropped", serde_json::json!({ "files": files }));
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            parse_document,
            parse_requirement_file,
            parse_and_extract,
            create_template,
            get_template,
            update_template,
            delete_template,
            list_all_templates,
            search_templates_db,
            search_templates_meili,
            init_search_index,
            index_templates,
            get_ai_config,
            update_ai_config,
            extract_requirements,
            generate_draft,
            scan_nas_directory,
            import_nas_files,
            detect_document_type,
            check_deviation,
            check_deviation_files,
            check_deviation_items,
            desensitize_text,
            analyze_desensitize,
            write_clipboard_html,
            write_clipboard_text,
            check_fatal_risks_text,
            export_deviation_report_markdown,
            export_deviation_report_html,
            export_deviation_report_json,
            check_punctuation,
            check_self_review,
            check_self_review_async,
            apply_self_review_fixes,
            generate_outline,
            generate_card,
            save_cards,
            get_cards,
            update_card,
            confirm_all_cards,
            check_consistency,
            get_global_params,
            save_global_params,
            check_cross_document_consistency,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_document_blocks_path_traversal() {
        let res = parse_document("../../../etc/passwd".to_string());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("非法文件路径"));
    }

    #[test]
    fn test_parse_document_blocks_empty_path() {
        let res = parse_document("".to_string());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("路径不能为空"));
    }

    #[test]
    fn test_parse_document_blocks_system_path() {
        let res = parse_document("/etc/passwd".to_string());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("非法文件路径"));
    }

    #[test]
    fn test_parse_requirement_file_blocks_path_traversal() {
        let res = parse_requirement_file("../secret.docx".to_string());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("非法文件路径"));
    }

    #[tokio::test]
    async fn test_check_deviation_files_blocks_path_traversal() {
        let res = check_deviation_files(
            "../../../etc/passwd".to_string(),
            "../secret.docx".to_string(),
        )
        .await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("非法文件路径"));
    }

    #[test]
    #[cfg(feature = "docx")]
    fn test_apply_self_review_fixes_blocks_path_traversal() {
        let res = apply_self_review_fixes(
            "../template.docx".to_string(),
            vec![],
            crate::models::FixMode::Copy,
        );
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("非法文件路径"));
    }
}
