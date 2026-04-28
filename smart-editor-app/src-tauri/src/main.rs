// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
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

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

use crate::ai::AiClient;
use crate::db::Database;
use crate::desensitize::DesensitizeHit;
use crate::models::{
    AiConfig, DeviationReport, DocumentType, ParsedDocument, ParsedRequirements, SearchResult,
    Template,
};
use crate::nas_scanner::{scan_directory, ImportResult};
use crate::search::SearchEngine;

pub struct AppState {
    db: Mutex<Database>,
    search: Option<SearchEngine>,
    ai: AiClient,
    config_path: PathBuf,
}

// --- 文档解析命令 ---

#[tauri::command]
fn parse_document(file_path: String) -> Result<String, String> {
    parser::parse_document(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn parse_requirement_file(file_path: String) -> Result<ParsedDocument, String> {
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
    state
        .ai
        .extract_requirements(&parsed.text, parsed.document_type)
        .await
        .map_err(|e| e.to_string())
}

// --- 模板库命令 ---

#[tauri::command]
fn create_template(
    state: tauri::State<'_, AppState>,
    mut template: Template,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.insert_template(&mut template).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_template(state: tauri::State<'_, AppState>, id: i64) -> Result<Option<Template>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_template(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_template(state: tauri::State<'_, AppState>, template: Template) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_template(&template).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_template(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_template(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_all_templates(
    state: tauri::State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<Template>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_templates(None, None, None, None, None, limit.unwrap_or(1000))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn search_templates_db(
    state: tauri::State<'_, AppState>,
    doc_attr: Option<String>,
    business_domain: Option<String>,
    content_module: Option<String>,
    project_phase: Option<String>,
    keyword: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<Template>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_templates(
        doc_attr.as_deref(),
        business_domain.as_deref(),
        content_module.as_deref(),
        project_phase.as_deref(),
        keyword.as_deref(),
        limit.unwrap_or(50),
    )
    .map_err(|e| e.to_string())
}

// --- Meilisearch 命令 ---

#[tauri::command]
async fn search_templates_meili(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SearchResult>, String> {
    let search = state
        .search
        .as_ref()
        .ok_or("搜索服务未初始化，请检查 Meilisearch 配置")?;
    search
        .search(&query, limit.unwrap_or(10))
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
    Ok(app_config.to_ai_config())
}

#[tauri::command]
fn update_ai_config(state: tauri::State<'_, AppState>, ai: AiConfig) -> Result<(), String> {
    let mut app_config = config::load_or_create(&state.config_path).map_err(|e| e.to_string())?;
    app_config.ai_provider = ai.provider;
    app_config.ai_base_url = ai.base_url;
    app_config.ai_api_key = ai.api_key;
    app_config.ai_model = ai.model;
    config::save(&state.config_path, &app_config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn extract_requirements(
    state: tauri::State<'_, AppState>,
    text: String,
    doc_type: DocumentType,
) -> Result<ParsedRequirements, String> {
    state
        .ai
        .extract_requirements(&text, doc_type)
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
    state
        .ai
        .generate_draft(&requirements, &references, doc_type)
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
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
) -> Result<ImportResult, String> {
    let mut templates = Vec::new();
    let mut failed_files = Vec::new();

    for path in &paths {
        match nas_scanner::file_to_template(path) {
            Ok(t) => templates.push(t),
            Err(e) => {
                log::error!("解析文件失败 {}: {}", path, e);
                failed_files.push(path.clone());
            }
        }
    }

    // 入库 SQLite
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        for t in &mut templates {
            if let Err(e) = db.insert_template(t) {
                log::error!("SQLite 入库失败: {}", e);
                if let Some(ref path) = t.source_file {
                    failed_files.push(path.clone());
                }
            }
        }
    }

    // 过滤出成功入库且有 id 的模板，用于 Meilisearch 索引
    let templates_to_index: Vec<Template> = templates
        .into_iter()
        .filter(|t| {
            t.id.is_some()
                && !failed_files.contains(t.source_file.as_ref().unwrap_or(&String::new()))
        })
        .collect();

    if !templates_to_index.is_empty() {
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

// --- 偏离检查命令 ---

#[tauri::command]
fn check_deviation(bid_text: String, req_text: String) -> DeviationReport {
    deviation::check_deviation(&bid_text, &req_text)
}

#[tauri::command]
fn check_deviation_files(bid_path: String, req_path: String) -> Result<DeviationReport, String> {
    let bid_text = parser::parse_document(&bid_path).map_err(|e| e.to_string())?;
    let req_text = parser::parse_document(&req_path).map_err(|e| e.to_string())?;
    Ok(deviation::check_deviation(&bid_text, &req_text))
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
fn check_fatal_risks_text(text: String) -> Vec<crate::models::FatalRisk> {
    deviation::check_fatal_risks(&text)
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
fn check_punctuation(text: String) -> Vec<punctuation::PunctuationIssue> {
    punctuation::check_punctuation(&text)
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

            let ai = AiClient::new(app_config.to_ai_config());

            app.manage(AppState {
                db: Mutex::new(db),
                search,
                ai,
                config_path,
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
            desensitize_text,
            analyze_desensitize,
            write_clipboard_html,
            write_clipboard_text,
            check_fatal_risks_text,
            export_deviation_report_markdown,
            export_deviation_report_html,
            export_deviation_report_json,
            check_punctuation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
