use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangEntry {
    pub key: String,
    pub source: String,
    pub translation: String,
    pub is_vanilla: bool,
    pub mod_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLangFile {
    pub mod_id: String,
    pub file_path: String,
    pub entries: Vec<LangEntry>,
    pub format: LangFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LangFormat {
    Json,
    Lang,
}

pub fn extract_from_jar(jar_path: &str) -> Result<Vec<ModLangFile>, String> {
    let file = fs::File::open(jar_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let jar_name = extract_mod_id_from_jar_name(jar_path);
    let mut result = Vec::new();
    let mut lang_files: HashMap<String, Vec<(String, Vec<u8>)>> = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();

        if name.contains("/lang/") && (name.ends_with(".json") || name.ends_with(".lang")) {
            let mod_id = extract_mod_id_from_path(&name);
            // Fallback to jar name if path-based extraction failed
            let mod_id = if mod_id == "unknown" { jar_name.clone() } else { mod_id };
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            lang_files
                .entry(mod_id)
                .or_default()
                .push((name.clone(), buf));
        }
    }

    if lang_files.is_empty() {
        return Err("未在模组中找到语言文件".to_string());
    }

    for (mod_id, files) in lang_files {
        for (path, content) in files {
            if path.ends_with(".json") {
                if let Ok(entries) = parse_json_lang(&content, &mod_id) {
                    result.push(ModLangFile {
                        mod_id: mod_id.clone(),
                        file_path: path.clone(),
                        entries,
                        format: LangFormat::Json,
                    });
                }
            } else if path.ends_with(".lang") {
                if let Ok(entries) = parse_legacy_lang(&content, &mod_id) {
                    result.push(ModLangFile {
                        mod_id: mod_id.clone(),
                        file_path: path.clone(),
                        entries,
                        format: LangFormat::Lang,
                    });
                }
            }
        }
    }

    Ok(result)
}

pub fn extract_from_lang_file(file_path: &str) -> Result<Vec<ModLangFile>, String> {
    let content = fs::read(file_path).map_err(|e| e.to_string())?;
    let path = Path::new(file_path);

    // Try to extract mod_id from parent directory (e.g., .../create/lang/en_us.json -> create)
    let mod_id = path
        .parent()
        .and_then(|p| {
            let parent_name = p.file_stem()?.to_str()?;
            if parent_name == "lang" || parent_name == "langs" {
                p.parent()?.file_stem()?.to_str().map(|s| s.to_string())
            } else {
                Some(parent_name.to_string())
            }
        })
        .or_else(|| path.file_stem().and_then(|s| s.to_str()).map(|s| {
            // Strip language codes from filename: en_us, zh_cn, etc.
            if s.len() > 5 && s.contains('_') {
                s.split('_').next().unwrap_or(s).to_string()
            } else {
                s.to_string()
            }
        }))
        .unwrap_or_else(|| "unknown".to_string());

    let entries = if file_path.ends_with(".json") {
        parse_json_lang(&content, &mod_id)?
    } else {
        parse_legacy_lang(&content, &mod_id)?
    };

    Ok(vec![ModLangFile {
        mod_id: mod_id.clone(),
        file_path: file_path.to_string(),
        entries,
        format: if file_path.ends_with(".json") {
            LangFormat::Json
        } else {
            LangFormat::Lang
        },
    }])
}

fn extract_mod_id_from_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();

    // Standard: assets/<mod_id>/lang/en_us.json
    if parts.len() >= 3 {
        // Check for "assets/<id>/..." pattern
        for i in 0..parts.len().saturating_sub(2) {
            if parts[i] == "assets" && i + 1 < parts.len() {
                return parts[i + 1].to_string();
            }
        }
        // Check for "data/<id>/..." pattern (some mods use data folder)
        for i in 0..parts.len().saturating_sub(2) {
            if parts[i] == "data" && i + 1 < parts.len() {
                let id = parts[i + 1].to_string();
                // Skip "minecraft" data namespace
                if id != "minecraft" {
                    return id;
                }
            }
        }
    }

    // Fallback: try to extract from path segments near "lang"
    if let Some(lang_pos) = parts.iter().position(|&p| p == "lang") {
        if lang_pos > 0 {
            let prev = parts[lang_pos - 1];
            if prev != "assets" && prev != "data" {
                return prev.to_string();
            }
            if lang_pos > 1 {
                return parts[lang_pos - 2].to_string();
            }
        }
    }

    "unknown".to_string()
}

fn extract_mod_id_from_jar_name(jar_path: &str) -> String {
    let name = std::path::Path::new(jar_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // Strip common version suffixes: -1.20.1, -1.20, -forge, -fabric, etc
    let stripped = name
        .split('-')
        .filter(|part| {
            let p = part.to_lowercase();
            !p.starts_with("1.") && !p.starts_with("v") && p != "forge" 
                && p != "fabric" && p != "neoforge" && p != "mc"
                && !p.chars().all(|c| c.is_numeric() || c == '.')
        })
        .collect::<Vec<_>>()
        .join("-");

    if stripped.is_empty() { name.to_string() } else { stripped }
}

fn parse_json_lang(content: &[u8], mod_id: &str) -> Result<Vec<LangEntry>, String> {
    let text = String::from_utf8_lossy(content);
    let map: HashMap<String, String> =
        serde_json::from_str(&text).map_err(|e| format!("JSON解析错误: {}", e))?;

    let mut entries = Vec::new();
    for (key, value) in map {
        if should_skip(&key, &value) {
            continue;
        }
        entries.push(LangEntry {
            key: key.clone(),
            source: clean_text(&value),
            translation: String::new(),
            is_vanilla: key.starts_with("minecraft."),
            mod_id: mod_id.to_string(),
        });
    }
    Ok(entries)
}

fn parse_legacy_lang(content: &[u8], mod_id: &str) -> Result<Vec<LangEntry>, String> {
    let text = String::from_utf8_lossy(content);
    let reader = BufReader::new(text.as_bytes());
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim().to_string();
            let value = line[pos + 1..].trim().to_string();
            if should_skip(&key, &value) {
                continue;
            }
            entries.push(LangEntry {
                key: key.clone(),
                source: clean_text(&value),
                translation: String::new(),
                is_vanilla: key.starts_with("minecraft."),
                mod_id: mod_id.to_string(),
            });
        }
    }
    Ok(entries)
}

fn should_skip(key: &str, value: &str) -> bool {
    if key.is_empty() || value.is_empty() {
        return true;
    }
    let skip_patterns = ["/", "\\u", "§", "%s", "%d", "%1$", "%2$", "%3$"];
    if skip_patterns.iter().any(|p| value.contains(p)) && value.len() < 5 {
        return true;
    }
    false
}

fn clean_text(text: &str) -> String {
    text.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\\\", "\\")
}

pub fn serialize_lang_file(format: &LangFormat, entries: &[LangEntry]) -> Result<String, String> {
    match format {
        LangFormat::Json => {
            let mut map = HashMap::new();
            for entry in entries {
                let text = if entry.translation.is_empty() {
                    &entry.source
                } else {
                    &entry.translation
                };
                map.insert(&entry.key, text);
            }
            serde_json::to_string_pretty(&map).map_err(|e| e.to_string())
        }
        LangFormat::Lang => {
            let mut lines = Vec::new();
            for entry in entries {
                let text = if entry.translation.is_empty() {
                    &entry.source
                } else {
                    &entry.translation
                };
                lines.push(format!("{}={}", entry.key, text));
            }
            Ok(lines.join("\n"))
        }
    }
}
