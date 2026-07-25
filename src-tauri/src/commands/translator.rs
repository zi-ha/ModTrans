use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub name: String,
    pub provider: String,
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub custom_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateResult {
    pub key: String,
    pub translation: String,
    pub success: bool,
    pub error: Option<String>,
}

pub fn get_provider_presets() -> Vec<ApiConfig> {
    vec![
        ApiConfig {
            name: "OpenAI".to_string(),
            provider: "openai".to_string(),
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            custom_prompt: String::new(),
        },
        ApiConfig {
            name: "DeepSeek".to_string(),
            provider: "deepseek".to_string(),
            api_url: "https://api.deepseek.com/chat/completions".to_string(),
            api_key: String::new(),
            model: "deepseek-chat".to_string(),
            custom_prompt: String::new(),
        },
        ApiConfig {
            name: "智谱 GLM".to_string(),
            provider: "zhipu".to_string(),
            api_url: "https://open.bigmodel.cn/api/paas/v4/chat/completions".to_string(),
            api_key: String::new(),
            model: "glm-4-flash".to_string(),
            custom_prompt: String::new(),
        },
        ApiConfig {
            name: "通义千问".to_string(),
            provider: "qwen".to_string(),
            api_url: "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".to_string(),
            api_key: String::new(),
            model: "qwen-plus".to_string(),
            custom_prompt: String::new(),
        },
    ]
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            name: "OpenAI".to_string(),
            provider: "openai".to_string(),
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            custom_prompt: TRANSLATION_SYSTEM_PROMPT.to_string(),
        }
    }
}

const TRANSLATION_SYSTEM_PROMPT: &str = r#"你是一名 Minecraft 模组翻译专家。将给定的JSON键值对中的英文value翻译为简体中文。

规则：
1. 使用 Minecraft 官方中文译名（如 Creeper=爬行者，Enderman=末影人，Nether=下界）
2. 保持游戏格式代码（§开头、%s、%d、%1$s 等变量占位符）
3. 翻译简洁自然，符合中文游戏语境
4. 不翻译原本就是中文的内容
5. 只返回JSON对象，key保持不变，value用翻译后的中文

返回格式示例：
{"item.diamond":"钻石","block.stone":"石头","entity.zombie":"僵尸"}"#;

pub async fn translate_batch(
    texts: Vec<(String, String)>,
    config: &ApiConfig,
    term_map: &HashMap<String, String>,
) -> Result<Vec<TranslateResult>, String> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }

    let term_hints = build_term_hints(term_map);

    let entries_json: Vec<String> = texts
        .iter()
        .map(|(k, v)| format!(r#"  "{}": "{}""#, escape_json_str(k), escape_json_str(v)))
        .collect();

    let user_prompt = format!(
        "{}{}\n\n请翻译以下JSON中的value为简体中文，只返回完整JSON对象：\n{{\n{}\n}}",
        config.custom_prompt,
        term_hints,
        entries_json.join(",\n")
    );

    let mut body_map = serde_json::Map::new();
    body_map.insert("model".to_string(), json!(config.model));
    body_map.insert("messages".to_string(), json!([
        {"role": "system", "content": TRANSLATION_SYSTEM_PROMPT},
        {"role": "user", "content": user_prompt}
    ]));
    body_map.insert("temperature".to_string(), json!(0.2));
    body_map.insert("max_tokens".to_string(), json!(4096));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let res = match client
        .post(&config.api_url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&body_map)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Err(format_reqwest_error(&e, &config.api_url)),
    };

    let status = res.status();
    let res_text = res.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format_status_error(status.as_u16(), &res_text, config));
    }

    let json: serde_json::Value =
        serde_json::from_str(&res_text).map_err(|e| format!("解析API响应失败: {}", e))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    if content.is_empty() {
        return Err("API返回空内容，请检查API Key或账户余额".to_string());
    }

    let parsed = parse_translation_response(&content, &texts);
    let mut results = Vec::new();
    for (key, _source) in texts {
        let translation = parsed.get(&key).cloned().unwrap_or_default();
        let success = !translation.is_empty();
        results.push(TranslateResult {
            key,
            translation,
            success,
            error: if success { None } else { Some("未能解析".to_string()) },
        });
    }

    Ok(results)
}

fn escape_json_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

fn build_term_hints(term_map: &HashMap<String, String>) -> String {
    if term_map.is_empty() {
        return String::new();
    }
    let mut hints = "\n\n术语对照（必须使用）：\n".to_string();
    for (en, zh) in term_map.iter().take(20) {
        hints.push_str(&format!("{} → {}\n", en, zh));
    }
    hints
}

fn format_reqwest_error(e: &reqwest::Error, url: &str) -> String {
    let msg = e.to_string().to_lowercase();
    if msg.contains("404") || msg.contains("not found") {
        format!(
            "API地址404不存在。\n当前地址: {}\n请检查：\n1. 提供商选择是否正确\n2. API地址是否拼写正确\n3. 是否包含了多余的路径后缀",
            url
        )
    } else if msg.contains("401") || msg.contains("unauthorized") {
        "API Key 无效（401），请在设置中重新填写".to_string()
    } else if msg.contains("403") || msg.contains("forbidden") {
        "无权限（403），请检查Key或账户状态".to_string()
    } else if msg.contains("timeout") || msg.contains("timed out") {
        "请求超时，请检查网络连接".to_string()
    } else {
        format!("网络错误: {}\n地址: {}", e, url)
    }
}

fn format_status_error(code: u16, body: &str, config: &ApiConfig) -> String {
    let prefix = match code {
        401 => "API Key 无效（401）",
        403 => "无权限访问（403）",
        404 => "API端点不存在（404）",
        429 => "请求太频繁（429），请稍后重试",
        500..=599 => "服务器错误",
        _ => "API请求失败",
    };

    let mut detail = String::new();
    if code == 404 {
        detail.push_str(&format!(
            "\n当前地址: {}\n提供商: {}\n请确认地址与提供商一致。",
            config.api_url, config.provider
        ));
    }

    if let Ok(err_json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = err_json["error"]["message"].as_str() {
            detail.push_str(&format!("\n服务器返回: {}", msg));
        } else if let Some(msg) = err_json["message"].as_str() {
            detail.push_str(&format!("\n服务器返回: {}", msg));
        }
    } else if body.len() < 300 {
        detail.push_str(&format!("\n响应: {}", body));
    }

    format!("{}{}", prefix, detail)
}

fn parse_translation_response(content: &str, texts: &[(String, String)]) -> HashMap<String, String> {

    // 1: direct JSON
    if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(content) {
        if !map.is_empty() { return map; }
    }
    if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(content) {
        let result: HashMap<String, String> = map.into_iter()
            .map(|(k, v)| (k, v.as_str().unwrap_or("").to_string()))
            .collect();
        if !result.is_empty() { return result; }
    }

    // 2: markdown code block
    for (tag, inner) in extract_markdown_code(content) {
        let block = if tag == "json" { inner } else { inner };
        if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(block) {
            if !map.is_empty() { return map; }
        }
        if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(block) {
            let result: HashMap<String, String> = map.into_iter()
                .map(|(k, v)| (k, as_str(&v), )).collect();
            if !result.is_empty() { return result; }
        }
    }

    // 3: extract JSON object between { }
    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            let inner = &content[start..=end];
            if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(inner) {
                let result: HashMap<String, String> = map.into_iter()
                    .map(|(k, v)| (k, as_str(&v)))
                    .collect();
                if !result.is_empty() { return result; }
            }
        }
    }

    // 4: line-by-line key:value
    let mut result = HashMap::new();
    for line in content.lines() {
        let line = line.trim().trim_matches(',').trim();
        if line.is_empty() || line == "{" || line == "}" { continue; }

        // Try "key": "value" or key: value patterns
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().trim_matches('"').trim().to_string();
            let val = line[pos + 1..].trim().trim_matches('"').trim_end_matches(',').to_string();
            if texts.iter().any(|(k, _)| *k == key) && !val.is_empty() {
                result.insert(key, val);
            }
        }

        // Try key|||value legacy format
        if let Some(pos) = line.find("|||") {
            let key = line[..pos].trim().trim_matches('"').to_string();
            let val = line[pos + 3..].trim().trim_matches('"').trim_end_matches(',').to_string();
            if texts.iter().any(|(k, _)| *k == key) {
                result.insert(key, val);
            }
        }
    }

    result
}

fn extract_markdown_code(content: &str) -> Vec<(String, &str)> {
    let mut results = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("```") {
        let after_start = &rest[start + 3..];
        let tag_end = after_start.find('\n').unwrap_or(0);
        let tag = after_start[..tag_end].trim().to_string();
        let after_tag = &after_start[tag_end..];
        if let Some(end) = after_tag.find("```") {
            results.push((tag, after_tag[..end].trim()));
            rest = &after_tag[end + 3..];
        } else {
            break;
        }
    }
    results
}

fn as_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => v.to_string().trim_matches('"').to_string(),
    }
}
