export interface LangEntry {
  key: string;
  source: string;
  translation: string;
  is_vanilla: boolean;
  mod_id: string;
}

export interface ModLangFile {
  mod_id: string;
  file_path: string;
  entries: LangEntry[];
  format: 'Json' | 'Lang';
}

export interface ApiConfig {
  name: string;
  provider: string;
  api_url: string;
  api_key: string;
  model: string;
  custom_prompt: string;
}

export interface PackMeta {
  name: string;
  author: string;
  description: string;
  version: string;
  mc_version: string | null;
  universal: boolean;
}

export interface HistoryRecord {
  timestamp: string;
  mod_name: string;
  output_path: string;
  entry_count: number;
}

export interface AppSettings {
  api_configs: ApiConfig[];
  active_api_index: number;
  custom_prompt: string;
  history: HistoryRecord[];
}

export interface TranslateResult {
  key: string;
  translation: string;
  success: boolean;
  error: string | null;
}
