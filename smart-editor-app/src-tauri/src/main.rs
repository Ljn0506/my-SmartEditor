// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

// 在此处定义 Tauri 命令（前端可调用的 Rust 函数）

/// 示例：返回应用版本
#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 示例：解析文件内容（后续实现 docx/pdf/xlsx 解析）
#[tauri::command]
fn parse_document(file_path: String) -> Result<String, String> {
    // TODO: 根据文件扩展名调用不同解析器
    // .docx -> docx-rs
    // .pdf  -> pdf-extract
    // .xlsx -> calamine
    // .txt  -> 直接读取
    Ok(format!("已收到文件路径: {}", file_path))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![get_app_version, parse_document])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
