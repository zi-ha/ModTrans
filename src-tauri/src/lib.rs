mod commands;

use commands::parser::{extract_from_jar, extract_from_lang_file, serialize_lang_file, LangEntry, ModLangFile};
use commands::packer::{generate_resource_pack, merge_resource_packs, PackMeta};
use commands::store::{load_settings, save_settings, load_terms, save_terms, add_history, AppSettings};
use commands::translator::{translate_batch, ApiConfig, TranslateResult};
use std::collections::HashMap;

#[tauri::command]
async fn extract_jar(jar_path: String) -> Result<Vec<ModLangFile>, String> {
    tokio::task::spawn_blocking(move || extract_from_jar(&jar_path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn extract_lang(file_path: String) -> Result<Vec<ModLangFile>, String> {
    tokio::task::spawn_blocking(move || extract_from_lang_file(&file_path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn translate_entries(
    entries: Vec<(String, String)>,
    config: ApiConfig,
) -> Result<Vec<TranslateResult>, String> {
    let terms = load_terms();
    translate_batch(entries, &config, &terms).await
}

#[tauri::command]
fn generate_pack(
    meta: PackMeta,
    lang_files: HashMap<String, String>,
    output_dir: String,
) -> Result<String, String> {
    generate_resource_pack(&meta, &lang_files, &output_dir)
}

#[tauri::command]
fn merge_packs(
    pack_paths: Vec<String>,
    output_dir: String,
    meta: PackMeta,
) -> Result<String, String> {
    merge_resource_packs(pack_paths, &output_dir, &meta)
}

#[tauri::command]
fn get_settings() -> AppSettings {
    load_settings()
}

#[tauri::command]
fn set_settings(settings: AppSettings) -> Result<(), String> {
    save_settings(&settings)
}

#[tauri::command]
fn get_terms() -> HashMap<String, String> {
    load_terms()
}

#[tauri::command]
fn set_terms(terms: HashMap<String, String>) -> Result<(), String> {
    save_terms(&terms)
}

#[tauri::command]
fn add_history_record(mod_name: String, output_path: String, entry_count: usize) {
    add_history(&mod_name, &output_path, entry_count);
}

#[tauri::command]
fn serialize_lang(format_str: String, entries: Vec<LangEntry>) -> Result<String, String> {
    let format = match format_str.as_str() {
        "Json" => commands::parser::LangFormat::Json,
        "Lang" => commands::parser::LangFormat::Lang,
        _ => return Err("未知格式".to_string()),
    };
    serialize_lang_file(&format, &entries)
}

#[tauri::command]
fn get_provider_presets() -> Vec<ApiConfig> {
    commands::translator::get_provider_presets()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            extract_jar,
            extract_lang,
            translate_entries,
            generate_pack,
            merge_packs,
            get_settings,
            set_settings,
            get_terms,
            set_terms,
            add_history_record,
            serialize_lang,
            get_provider_presets
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
