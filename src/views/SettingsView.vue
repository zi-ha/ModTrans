<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { ApiConfig } from '../types';

const apiConfigs = ref<ApiConfig[]>([]);
const activeIndex = ref(0);
const defaultOutputDir = ref('');
const history = ref<any[]>([]);
const terms = ref<Record<string, string>>({});
const newTermKey = ref('');
const newTermValue = ref('');
const activeSection = ref('api');
const testMessage = ref('');
const testing = ref(false);

const providerPresets: Record<string, { url: string; model: string }> = {
  openai: { url: 'https://api.openai.com/v1/chat/completions', model: 'gpt-4o-mini' },
  deepseek: { url: 'https://api.deepseek.com/chat/completions', model: 'deepseek-chat' },
  zhipu: { url: 'https://open.bigmodel.cn/api/paas/v4/chat/completions', model: 'glm-4-flash' },
  qwen: { url: 'https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions', model: 'qwen-plus' },
};

watch(
  () => apiConfigs.value[activeIndex.value]?.provider,
  (newProvider) => {
    if (!newProvider || !apiConfigs.value[activeIndex.value]) return;
    const preset = providerPresets[newProvider];
    if (preset && apiConfigs.value[activeIndex.value].api_url !== preset.url) {
      apiConfigs.value[activeIndex.value].api_url = preset.url;
      apiConfigs.value[activeIndex.value].model = preset.model;
    }
  }
);

watch(activeIndex, () => {
  const current = apiConfigs.value[activeIndex.value];
  if (!current) return;
  const preset = providerPresets[current.provider];
  if (preset && !current.api_url) {
    current.api_url = preset.url;
    current.model = preset.model;
  }
});

async function loadAll() {
  try {
    const s = await invoke<any>('get_settings');
    apiConfigs.value = s.api_configs || [];
    activeIndex.value = s.active_api_index ?? 0;
    defaultOutputDir.value = s.default_output_dir || '';
    history.value = s.history || [];
  } catch (e) { /* ignore */ }
  try {
    terms.value = await invoke<Record<string, string>>('get_terms');
  } catch (e) { /* ignore */ }
}

async function saveSettings() {
  try {
    await invoke('set_settings', {
      settings: {
        default_output_dir: defaultOutputDir.value,
        api_configs: apiConfigs.value,
        active_api_index: activeIndex.value,
        custom_prompt: apiConfigs.value[activeIndex.value]?.custom_prompt || '',
        history: history.value,
      },
    });
    testMessage.value = '';
    const el = document.createElement('div');
    el.style.cssText = 'position:fixed;top:20px;right:20px;background:#16A34A;color:#fff;padding:10px 20px;border-radius:6px;z-index:9999;font-size:14px;';
    el.textContent = '设置已保存';
    document.body.appendChild(el);
    setTimeout(() => el.remove(), 2000);
  } catch (e: any) {
    alert(`保存失败: ${e}`);
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
  const preset = providerPresets['openai'];
  apiConfigs.value.push({
    name: '新配置',
    provider: 'openai',
    api_url: preset.url,
    api_key: '',
    model: preset.model,
    custom_prompt: '你是一名 Minecraft 模组翻译专家。请将以下英文游戏文本翻译成简体中文，使用 Minecraft 官方中文译名。严格按照JSON格式返回，key保持不变，value为翻译结果。',
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
    const el = document.createElement('div');
    el.style.cssText = 'position:fixed;top:20px;right:20px;background:#16A34A;color:#fff;padding:10px 20px;border-radius:6px;z-index:9999;font-size:14px;';
    el.textContent = '词库已保存';
    document.body.appendChild(el);
    setTimeout(() => el.remove(), 2000);
  } catch (e: any) { alert(`保存失败: ${e}`); }
}

function addTerm() {
  if (!newTermKey.value || !newTermValue.value) return;
  terms.value[newTermKey.value] = newTermValue.value;
  newTermKey.value = '';
  newTermValue.value = '';
}

function removeTerm(key: string) { delete terms.value[key]; }

async function browseOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected) defaultOutputDir.value = selected as string;
}

onMounted(loadAll);
</script>

<template>
  <div>
    <h2 style="font-size: 18px; font-weight: 600; margin-bottom: 20px; color: #1F2937;">设置</h2>

    <div class="tab-bar">
      <button v-for="s in [
        { key: 'api', label: 'AI 服务' },
        { key: 'terms', label: '术语库' },
        { key: 'history', label: '历史' },
        { key: 'general', label: '文件' },
      ]" :key="s.key" class="tab-item" :class="{ active: activeSection === s.key }" @click="activeSection = s.key">{{ s.label }}</button>
    </div>

    <div v-if="activeSection === 'api'">
      <div class="card">
        <div class="card-title">AI 服务配置</div>

        <div style="display: flex; gap: 8px; margin-bottom: 16px; align-items: center;">
          <select v-model="activeIndex" class="select" style="max-width: 240px;">
            <option v-for="(cfg, i) in apiConfigs" :key="i" :value="i">{{ cfg.name }}</option>
          </select>
          <button class="btn btn-secondary" style="padding: 5px 14px; font-size: 12px;" @click="addApiConfig">+ 新增</button>
          <button class="btn btn-danger" style="padding: 5px 14px; font-size: 12px;" @click="removeApiConfig(activeIndex)" :disabled="apiConfigs.length <= 1">删除</button>
          <button class="btn btn-secondary" style="padding: 5px 14px; font-size: 12px; margin-left: auto;" @click="testConnection" :disabled="testing">
            {{ testing ? '测试中...' : '测试连接' }}
          </button>
        </div>

        <div v-if="testMessage" style="margin-bottom: 14px; padding: 10px 14px; border-radius: 6px; font-size: 13px;"
          :style="{ background: testMessage.includes('失败') || testMessage.includes('错误') ? '#FEF2F2' : '#F0FDF4', color: testMessage.includes('失败') || testMessage.includes('错误') ? '#DC2626' : '#16A34A', border: '1px solid ' + (testMessage.includes('失败') || testMessage.includes('错误') ? '#FECACA' : '#BBF7D0') }"
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
            <label class="form-label">自定义系统提示词（高级）</label>
            <textarea v-model="apiConfigs[activeIndex].custom_prompt" class="textarea" style="min-height: 120px;"></textarea>
          </div>
        </div>

        <div style="margin-top: 16px; text-align: right;">
          <button class="btn btn-primary" @click="saveSettings">保存设置</button>
        </div>
      </div>
    </div>

    <div v-if="activeSection === 'terms'">
      <div class="card">
        <div class="card-title">术语词库</div>
        <div style="display: flex; gap: 8px; margin-bottom: 16px;">
          <input v-model="newTermKey" class="input" placeholder="英文术语" @keydown.enter="addTerm" />
          <input v-model="newTermValue" class="input" placeholder="中文译名" @keydown.enter="addTerm" />
          <button class="btn btn-primary" style="white-space: nowrap;" @click="addTerm">添加</button>
        </div>
        <div style="max-height: 350px; overflow: auto; border: 1px solid #E5E7EB; border-radius: 6px;">
          <table class="table">
            <thead><tr><th>英文</th><th>中文</th><th></th></tr></thead>
            <tbody>
              <tr v-for="(val, key) in terms" :key="key">
                <td>{{ key }}</td><td>{{ val }}</td>
                <td style="width: 60px;"><button class="btn btn-danger" style="padding: 3px 8px; font-size: 11px;" @click="removeTerm(key)">删除</button></td>
              </tr>
              <tr v-if="Object.keys(terms).length === 0">
                <td colspan="3" style="text-align: center; padding: 30px; color: #9CA3AF;">暂无术语</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div style="margin-top: 12px; text-align: right;"><button class="btn btn-primary" @click="saveTerms">保存词库</button></div>
      </div>
    </div>

    <div v-if="activeSection === 'history'">
      <div class="card">
        <div class="card-title">操作历史</div>
        <div v-if="history.length === 0" style="text-align: center; padding: 40px; color: #9CA3AF;">暂无记录</div>
        <table v-else class="table">
          <thead><tr><th>时间</th><th>名称</th><th>输出路径</th><th>词条数</th></tr></thead>
          <tbody>
            <tr v-for="(rec, i) in history" :key="i">
              <td style="font-size: 11px; white-space: nowrap;">{{ rec.timestamp }}</td>
              <td>{{ rec.mod_name }}</td>
              <td style="font-size: 11px; font-family: monospace; max-width: 260px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" :title="rec.output_path">{{ rec.output_path }}</td>
              <td>{{ rec.entry_count }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div v-if="activeSection === 'general'">
      <div class="card">
        <div class="card-title">文件</div>
        <div class="form-row">
          <div class="form-col">
            <label class="form-label">默认输出目录</label>
            <div style="display: flex; gap: 8px;">
              <input v-model="defaultOutputDir" class="input" placeholder="留空使用默认目录" />
              <button class="btn btn-secondary" style="white-space: nowrap;" @click="browseOutputDir">浏览</button>
            </div>
          </div>
        </div>
        <div style="margin-top: 12px; text-align: right;"><button class="btn btn-primary" @click="saveSettings">保存设置</button></div>
      </div>
    </div>
  </div>
</template>
