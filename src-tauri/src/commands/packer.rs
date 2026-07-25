use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use zip::write::FileOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackMeta {
    pub name: String,
    pub author: String,
    pub description: String,
    pub version: String,
    pub mc_version: Option<String>,
    pub universal: bool,
}

impl Default for PackMeta {
    fn default() -> Self {
        Self {
            name: "汉化资源包".to_string(),
            author: "ModTrans".to_string(),
            description: "由ModTrans生成的汉化资源包".to_string(),
            version: "1.0".to_string(),
            mc_version: Some("1.20".to_string()),
            universal: false,
        }
    }
}

pub fn generate_resource_pack(
    meta: &PackMeta,
    lang_files: &HashMap<String, String>,
    output_dir: &str,
) -> Result<String, String> {
    let pack_name = format!("{}_v{}.zip", sanitize_filename(&meta.name), &meta.version);
    let output_path = Path::new(output_dir).join(&pack_name);

    let buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // pack.mcmeta
    let pack_format = if meta.universal {
        15
    } else {
        match meta.mc_version.as_deref() {
            Some("1.12") | Some("1.12.2") => 3,
            Some("1.16") | Some("1.16.5") => 6,
            Some("1.18") | Some("1.18.2") => 8,
            Some("1.19") | Some("1.19.4") => 13,
            Some("1.20") | Some("1.20.1") => 15,
            Some("1.21") => 34,
            _ => 15,
        }
    };

    let mcmeta = serde_json::json!({
        "pack": {
            "pack_format": pack_format,
            "description": format!("{}\n作者: {}\n{}", meta.description, meta.author, meta.version)
        }
    });

    zip.start_file("pack.mcmeta", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(mcmeta.to_string().as_bytes())
        .map_err(|e| e.to_string())?;

    for (path, content) in lang_files {
        let zip_path = if path.starts_with("assets/") {
            path.clone()
        } else {
            format!("assets/{}", path)
        };
        zip.start_file(&zip_path, options)
            .map_err(|e| e.to_string())?;
        zip.write_all(content.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    let buf = zip.finish().map_err(|e| e.to_string())?;
    fs::write(&output_path, buf.into_inner()).map_err(|e| e.to_string())?;

    Ok(output_path.to_string_lossy().to_string())
}

pub fn merge_resource_packs(
    pack_paths: Vec<String>,
    output_dir: &str,
    meta: &PackMeta,
) -> Result<String, String> {
    let pack_name = format!("{}_merged_v{}.zip", sanitize_filename(&meta.name), &meta.version);
    let output_path = Path::new(output_dir).join(&pack_name);

    let buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut merged_files: HashMap<String, String> = HashMap::new();

    for pack_path in pack_paths {
        let file = fs::File::open(&pack_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();

            if name == "pack.mcmeta" {
                continue;
            }

            if name.ends_with("/") {
                continue;
            }

            let mut buf = Vec::new();
            std::io::copy(&mut entry, &mut buf).map_err(|e| e.to_string())?;
            let text = String::from_utf8_lossy(&buf).to_string();

            if name.ends_with(".json") && name.contains("/lang/") {
                if let Some(existing) = merged_files.get(&name) {
                    let merged = merge_json_lang(existing, &text)?;
                    merged_files.insert(name.clone(), merged);
                } else {
                    merged_files.insert(name.clone(), text);
                }
            } else {
                merged_files.insert(name.clone(), text);
            }
        }
    }

    // Write merged pack.mcmeta
    let pack_format = match meta.mc_version.as_deref() {
        Some("1.12") | Some("1.12.2") => 3,
        Some("1.16") | Some("1.16.5") => 6,
        Some("1.18") | Some("1.18.2") => 8,
        Some("1.19") | Some("1.19.4") => 13,
        Some("1.20") | Some("1.20.1") => 15,
        Some("1.21") => 34,
        _ => 15,
    };

    let mcmeta = serde_json::json!({
        "pack": {
            "pack_format": pack_format,
            "description": format!("{}\n作者: {}\n{}", meta.description, meta.author, meta.version)
        }
    });

    zip.start_file("pack.mcmeta", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(mcmeta.to_string().as_bytes())
        .map_err(|e| e.to_string())?;

    for (path, content) in merged_files {
        zip.start_file(&path, options)
            .map_err(|e| e.to_string())?;
        zip.write_all(content.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    let buf = zip.finish().map_err(|e| e.to_string())?;
    fs::write(&output_path, buf.into_inner()).map_err(|e| e.to_string())?;

    Ok(output_path.to_string_lossy().to_string())
}

fn merge_json_lang(existing: &str, new: &str) -> Result<String, String> {
    let mut map1: HashMap<String, String> =
        serde_json::from_str(existing).map_err(|e| e.to_string())?;
    let map2: HashMap<String, String> =
        serde_json::from_str(new).map_err(|e| e.to_string())?;

    for (k, v) in map2 {
        map1.insert(k, v);
    }

    serde_json::to_string_pretty(&map1).map_err(|e| e.to_string())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect()
}
