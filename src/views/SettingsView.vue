<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { showToast } from '../utils/toast';
import type { ApiConfig } from '../types';

const apiConfigs = ref<ApiConfig[]>([]);
const activeIndex = ref(0);
const history = ref<any[]>([]);
const terms = ref<Record<string, string>>({});
const newTermKey = ref('');
const newTermValue = ref('');
const testMessage = ref('');
const testing = ref(false);
const presets = ref<ApiConfig[]>([]);
const translationRules = ref<string[]>([]);
const showHistory = ref(false);

const RULE_OPTIONS = [
  { value: 'keep_vars', label: '保留变量（%s、%d、%1$s 等占位符）' },
  { value: 'keep_format_codes', label: '保留颜色代码（§ 格式码）' },
];

watch(
  () => apiConfigs.value[activeIndex.value]?.provider,
  (newProvider) => {
    if (!newProvider || !apiConfigs.value[activeIndex.value]) return;
    const preset = presets.value.find(p => p.provider === newProvider);
    if (preset && apiConfigs.value[activeIndex.value].api_url !== preset.api_url) {
      apiConfigs.value[activeIndex.value].api_url = preset.api_url;
      apiConfigs.value[activeIndex.value].model = preset.model;
    }
  }
);

watch(activeIndex, () => {
  const current = apiConfigs.value[activeIndex.value];
  if (!current) return;
  const preset = presets.value.find(p => p.provider === current.provider);
  if (preset && !current.api_url) {
    current.api_url = preset.api_url;
    current.model = preset.model;
  }
});

async function loadAll() {
  try {
    const s = await invoke<any>('get_settings');
    apiConfigs.value = s.api_configs || [];
    activeIndex.value = s.active_api_index ?? 0;
    history.value = s.history || [];
    translationRules.value = s.translation_rules || ['keep_vars', 'keep_format_codes'];
  } catch (e) { /* ignore */ }
  try {
    terms.value = await invoke<Record<string, string>>('get_terms');
  } catch (e) { /* ignore */ }
  try {
    presets.value = await invoke<ApiConfig[]>('get_provider_presets');
  } catch (e) { /* ignore */ }
}

async function saveSettings() {
  try {
    await invoke('set_settings', {
      settings: {
        api_configs: apiConfigs.value,
        active_api_index: activeIndex.value,
        custom_prompt: apiConfigs.value[activeIndex.value]?.custom_prompt || '',
        history: history.value,
        translation_rules: translationRules.value,
      },
    });
    testMessage.value = '';
    showToast('设置已保存');
  } catch (e: any) {
    showToast(`保存失败: ${e}`, 'error');
  }
}

async function testConnection() {
  const cfg = apiConfigs.value[activeIndex.value];
  if (!cfg || !cfg.api_key) {
    testMessage.value = '请先填入 API Key';
    return;
  }
  testing.value = true;
  testMessage.value = '';
  try {
    const result = await invoke<any>('translate_entries', {
      entries: [['test.hello', 'Hello World']],
      config: cfg,
    });
    if (result && result.length > 0 && result[0].success) {
      testMessage.value = `连接成功！测试翻译: "${result[0].translation}"`;
    } else {
      testMessage.value = '连接成功但未获得翻译结果';
    }
  } catch (e: any) {
    testMessage.value = `测试失败: ${e}`;
  }
  testing.value = false;
}

function addApiConfig() {
  const preset = presets.value.find(p => p.provider === 'openai') || presets.value[0];
  apiConfigs.value.push({
    name: '新配置',
    provider: preset?.provider || 'openai',
    api_url: preset?.api_url || '',
    api_key: '',
    model: preset?.model || '',
    custom_prompt: '',
  });
  activeIndex.value = apiConfigs.value.length - 1;
}

function removeApiConfig(index: number) {
  if (apiConfigs.value.length <= 1) return;
  apiConfigs.value.splice(index, 1);
  if (activeIndex.value >= apiConfigs.value.length) activeIndex.value = apiConfigs.value.length - 1;
}

async function saveTerms() {
  try {
    await invoke('set_terms', { terms: terms.value });
    showToast('词库已保存');
  } catch (e: any) { showToast(`保存失败: ${e}`, 'error'); }
}

function addTerm() {
  if (!newTermKey.value || !newTermValue.value) return;
  terms.value[newTermKey.value] = newTermValue.value;
  newTermKey.value = '';
  newTermValue.value = '';
}

function removeTerm(key: string) { delete terms.value[key]; }

onMounted(loadAll);
</script>

<template>
  <div>
    <div class="page-head">
      <h2>设置</h2>
    </div>

    <!-- AI 服务 -->
    <div class="panel">
      <div class="panel-title">AI 服务</div>

      <div style="display: flex; gap: 8px; margin-bottom: 16px; align-items: center; flex-wrap: wrap;">
        <select v-model="activeIndex" class="select" style="max-width: 220px;">
          <option v-for="(cfg, i) in apiConfigs" :key="i" :value="i">{{ cfg.name }}</option>
        </select>
        <button class="btn btn-secondary btn-xs" @click="addApiConfig">+ 新增</button>
        <button class="btn btn-danger btn-xs" @click="removeApiConfig(activeIndex)" :disabled="apiConfigs.length <= 1">删除</button>
        <button class="btn btn-secondary btn-xs" style="margin-left: auto;" @click="testConnection" :disabled="testing">
          {{ testing ? '测试中...' : '测试连接' }}
        </button>
      </div>

      <div v-if="testMessage" style="margin-bottom: 14px; padding: 10px 14px; border-radius: 6px; font-size: 13px;"
        :style="{ background: testMessage.includes('失败') || testMessage.includes('错误') ? '#FFF1F0' : '#F6FFED', color: testMessage.includes('失败') || testMessage.includes('错误') ? '#FF4D4F' : '#52C41A', border: '1px solid ' + (testMessage.includes('失败') || testMessage.includes('错误') ? '#FFCCC7' : '#B7EB8F') }"
      >
        {{ testMessage }}
      </div>

      <div v-if="apiConfigs[activeIndex]">
        <div class="form-row">
          <div class="form-col">
            <label class="form-label">配置名称</label>
            <input v-model="apiConfigs[activeIndex].name" class="input" placeholder="例如: 我的DeepSeek" />
          </div>
          <div class="form-col">
            <label class="form-label">提供商</label>
            <select v-model="apiConfigs[activeIndex].provider" class="select">
              <option value="openai">OpenAI</option>
              <option value="deepseek">DeepSeek</option>
              <option value="zhipu">智谱 GLM</option>
              <option value="qwen">通义千问</option>
              <option value="custom">自定义</option>
            </select>
            <div class="form-hint">切换提供商会自动填写 API 地址和推荐模型</div>
          </div>
        </div>
        <div class="form-row">
          <div class="form-col">
            <label class="form-label">API 地址</label>
            <input v-model="apiConfigs[activeIndex].api_url" class="input font-mono" style="font-size: 12px;" />
          </div>
          <div class="form-col">
            <label class="form-label">模型</label>
            <input v-model="apiConfigs[activeIndex].model" class="input" />
          </div>
        </div>
        <div class="form-row">
          <div class="form-col">
            <label class="form-label">API Key</label>
            <input v-model="apiConfigs[activeIndex].api_key" class="input" type="password" placeholder="sk-..." />
            <div class="form-hint">密钥仅保存在本地，不会上传</div>
          </div>
        </div>
        <div style="margin-top: 14px;">
          <label class="form-label">自定义翻译提示词（可选，留空使用内置默认提示词）</label>
          <textarea v-model="apiConfigs[activeIndex].custom_prompt" class="textarea" style="min-height: 100px;"></textarea>
        </div>
      </div>
    </div>

    <!-- 翻译规则 -->
    <div class="panel">
      <div class="panel-title">翻译规则</div>
      <label v-for="opt in RULE_OPTIONS" :key="opt.value" class="checkbox-label">
        <input type="checkbox" :value="opt.value" v-model="translationRules" />
        {{ opt.label }}
      </label>
      <div class="form-hint">勾选后将在翻译请求中附加对应规则，随"保存设置"一起生效</div>
    </div>

    <!-- 词典管理 -->
    <div class="panel">
      <div class="panel-title">词典管理</div>
      <div style="display: flex; gap: 8px; margin-bottom: 16px;">
        <input v-model="newTermKey" class="input" placeholder="英文术语" @keydown.enter="addTerm" />
        <input v-model="newTermValue" class="input" placeholder="中文译名" @keydown.enter="addTerm" />
        <button class="btn btn-primary" style="white-space: nowrap;" @click="addTerm">添加</button>
      </div>
      <div style="max-height: 280px; overflow: auto; border: 1px solid #EBEDF0; border-radius: 6px;">
        <table class="table">
          <thead><tr><th>英文</th><th>中文</th><th></th></tr></thead>
          <tbody>
            <tr v-for="(val, key) in terms" :key="key">
              <td>{{ key }}</td><td>{{ val }}</td>
              <td style="width: 60px;"><button class="btn btn-danger btn-xs" @click="removeTerm(key)">删除</button></td>
            </tr>
            <tr v-if="Object.keys(terms).length === 0">
              <td colspan="3" style="text-align: center; padding: 30px; color: #9CA3AF;">暂无术语</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div style="margin-top: 12px; text-align: right;">
        <button class="btn btn-secondary" @click="saveTerms">保存词库</button>
      </div>
    </div>

    <!-- 历史记录 -->
    <div class="panel">
      <div class="collapse-header" style="margin-bottom: 0;" @click="showHistory = !showHistory">
        <span>历史记录</span>
        <span>{{ showHistory ? '▲' : '▼' }}</span>
      </div>
      <div v-if="showHistory" class="collapse-body" style="margin-top: 12px;">
        <div v-if="history.length === 0" style="text-align: center; padding: 30px; color: #9CA3AF;">暂无记录</div>
        <table v-else class="table">
          <thead><tr><th>时间</th><th>名称</th><th>输出路径</th><th>词条数</th></tr></thead>
          <tbody>
            <tr v-for="(rec, i) in history" :key="i">
              <td style="font-size: 11px; white-space: nowrap;">{{ rec.timestamp }}</td>
              <td>{{ rec.mod_name }}</td>
              <td style="font-size: 11px; font-family: monospace; max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" :title="rec.output_path">{{ rec.output_path }}</td>
              <td>{{ rec.entry_count }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="save-row">
      <button class="btn btn-primary btn-lg" @click="saveSettings">保存设置</button>
    </div>
  </div>
</template>
