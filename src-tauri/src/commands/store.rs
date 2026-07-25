use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub default_output_dir: String,
    pub api_configs: Vec<crate::commands::translator::ApiConfig>,
    pub active_api_index: usize,
    pub custom_prompt: String,
    pub history: Vec<HistoryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub timestamp: String,
    pub mod_name: String,
    pub output_path: String,
    pub entry_count: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_output_dir: String::new(),
            api_configs: vec![crate::commands::translator::ApiConfig::default()],
            active_api_index: 0,
            custom_prompt: crate::commands::translator::ApiConfig::default().custom_prompt,
            history: Vec::new(),
        }
    }
}

fn get_settings_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| std::env::temp_dir());
    path.push("modtrans");
    fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

fn get_term_db_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| std::env::temp_dir());
    path.push("modtrans");
    fs::create_dir_all(&path).ok();
    path.push("terms.json");
    path
}

pub fn load_settings() -> AppSettings {
    let path = get_settings_path();
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AppSettings::default()
    }
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn load_terms() -> HashMap<String, String> {
    let path = get_term_db_path();
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        let mut default = HashMap::new();
        default.insert("Creeper".to_string(), "爬行者".to_string());
        default.insert("Enderman".to_string(), "末影人".to_string());
        default.insert("Zombie".to_string(), "僵尸".to_string());
        default.insert("Skeleton".to_string(), "骷髅".to_string());
        default.insert("Spider".to_string(), "蜘蛛".to_string());
        default.insert("Cave Spider".to_string(), "洞穴蜘蛛".to_string());
        default.insert("Blaze".to_string(), "烈焰人".to_string());
        default.insert("Ender Dragon".to_string(), "末影龙".to_string());
        default.insert("Wither".to_string(), "凋灵".to_string());
        default.insert("Slime".to_string(), "史莱姆".to_string());
        default.insert("Magma Cube".to_string(), "岩浆怪".to_string());
        default.insert("Ghast".to_string(), "恶魂".to_string());
        default.insert("Piglin".to_string(), "猪灵".to_string());
        default.insert("Hoglin".to_string(), "疣猪兽".to_string());
        default.insert("Zoglin".to_string(), "僵尸疣猪兽".to_string());
        default.insert("Overworld".to_string(), "主世界".to_string());
        default.insert("Nether".to_string(), "下界".to_string());
        default.insert("The End".to_string(), "末地".to_string());
        default.insert("Enchantment".to_string(), "附魔".to_string());
        default.insert("Potion".to_string(), "药水".to_string());
        default.insert("Block".to_string(), "方块".to_string());
        default.insert("Item".to_string(), "物品".to_string());
        default.insert("Entity".to_string(), "实体".to_string());
        default.insert("Biome".to_string(), "生物群系".to_string());
        default.insert("Dimension".to_string(), "维度".to_string());
        default
    }
}

pub fn save_terms(terms: &HashMap<String, String>) -> Result<(), String> {
    let path = get_term_db_path();
    let json = serde_json::to_string_pretty(terms).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn add_history(mod_name: &str, output_path: &str, entry_count: usize) {
    let mut settings = load_settings();
    let now = chrono::Local::now();
    settings.history.push(HistoryRecord {
        timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        mod_name: mod_name.to_string(),
        output_path: output_path.to_string(),
        entry_count,
    });
    if settings.history.len() > 100 {
        settings.history.remove(0);
    }
    save_settings(&settings).ok();
}
