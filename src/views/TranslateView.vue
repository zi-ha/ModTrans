<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../stores/app';
import type { LangEntry, ApiConfig } from '../types';

const router = useRouter();
const store = useAppStore();

const translating = ref(false);
const paused = ref(false);
const progress = ref({ current: 0, total: 0 });
const currentItem = ref<{ key: string; source: string; translation: string }>({ key: '', source: '', translation: '' });
const translateLog = ref<string[]>([]);
const apiConfigs = ref<ApiConfig[]>([]);
const selectedApiIndex = ref(0);
const showSettings = ref(false);
const showAdvanced = ref(false);
const selectedModIds = ref<string[]>([]);

const modIds = computed(() => store.currentFiles.map(f => f.mod_id));

const untranslatedTargets = computed(() => {
  const result: LangEntry[] = [];
  for (const key of Object.keys(store.editedEntries)) {
    if (selectedModIds.value.length > 0 && !selectedModIds.value.includes(key)) continue;
    for (const e of store.editedEntries[key]) {
      if (!e.translation) result.push(e);
    }
  }
  return result;
});

const totalEntries = computed(() => {
  let count = 0;
  for (const key of Object.keys(store.editedEntries)) {
    if (selectedModIds.value.length > 0 && !selectedModIds.value.includes(key)) continue;
    count += store.editedEntries[key].length;
  }
  return count;
});

const translatedCount = computed(() => {
  let count = 0;
  for (const key of Object.keys(store.editedEntries)) {
    if (selectedModIds.value.length > 0 && !selectedModIds.value.includes(key)) continue;
    count += store.editedEntries[key].filter(e => e.translation).length;
  }
  return count;
});

watch(() => store.currentStep, (step) => {
  if (step === 'review') router.push('/review');
});

async function loadSettings() {
  try {
    const s = await invoke<any>('get_settings');
    apiConfigs.value = s.api_configs || [];
    selectedApiIndex.value = s.active_api_index ?? 0;
  } catch (e) { /* ignore */ }
}

async function startTranslate() {
  const targets = untranslatedTargets.value;
  if (targets.length === 0 || translating.value) return;

  const apiCfg = apiConfigs.value[selectedApiIndex.value];
  if (!apiCfg?.api_key) {
    translateLog.value.push('请先在设置中配置 API Key（右上角齿轮图标）');
    return;
  }

  translating.value = true;
  paused.value = false;
  translateLog.value = [];
  progress.value = { current: 0, total: targets.length };
  currentItem.value = { key: '', source: '', translation: '' };

  const batchSize = 12;
  let current = 0;
  let consecutiveErrors = 0;

  while (current < targets.length && translating.value) {
    if (paused.value) {
      await sleep(200);
      continue;
    }

    const batch = targets.slice(current, current + batchSize);
    const batchTexts: [string, string][] = batch.map(e => [e.key, e.source]);

    try {
      const results = await invoke<any[]>('translate_entries', {
        entries: batchTexts,
        config: apiCfg,
      });

      let batchTranslated = 0;
      for (const res of results) {
        if (res.success && res.translation) {
          const modId = batch.find(e => e.key === res.key)?.mod_id || '';
          store.updateEntry(modId, res.key, res.translation);
          batchTranslated++;
          currentItem.value = {
            key: res.key,
            source: batch.find(e => e.key === res.key)?.source || '',
            translation: res.translation,
          };
        }
      }
      consecutiveErrors = 0;
      translateLog.value.push(`${current + 1}-${Math.min(current + batchSize, targets.length)}: 成功 ${batchTranslated}/${batch.length} 条`);
    } catch (e: any) {
      consecutiveErrors++;
      translateLog.value.push(`错误: ${e}`);
      if (consecutiveErrors >= 3) {
        translateLog.value.push('连续3次请求失败，已停止翻译。请检查 API 配置。');
        translating.value = false;
        break;
      }
    }

    current += batchSize;
    progress.value.current = Math.min(current, targets.length);
  }

  translating.value = false;
  translateLog.value.push(`完成 ${translatedCount.value}/${totalEntries.value}`);

  if (translatedCount.value > 0 && untranslatedTargets.value.length === 0) {
    translateLog.value.push('全部翻译完成！');
  }
}

function stopTranslate() { translating.value = false; paused.value = false; }
function togglePause() { paused.value = !paused.value; }

async function goToReview() {
  const nextPath = store.completeStep('translate');
  if (nextPath) router.push(nextPath);
}

function toggleAllMods() {
  if (selectedModIds.value.length === modIds.value.length) selectedModIds.value = [];
  else selectedModIds.value = [...modIds.value];
}

async function goToSettings() { router.push('/settings'); }

function sleep(ms: number) { return new Promise(r => setTimeout(r, ms)); }

onMounted(loadSettings);
</script>

<template>
  <div>
    <h2 style="font-size: 18px; font-weight: 600; margin-bottom: 20px; color: #1F2937;">翻译</h2>

    <div v-if="store.currentFiles.length === 0" class="empty-state">
      <div class="empty-state-icon">📦</div>
      <div class="empty-state-text">请先导入模组文件</div>
      <button class="btn btn-primary" @click="router.push('/import')">去导入</button>
    </div>

    <template v-else>
      <!-- Mod selector -->
      <div class="card" style="padding: 14px 20px;">
        <div style="display: flex; gap: 12px; align-items: center; flex-wrap: wrap;">
          <span style="font-size: 12px; color: #6B7280;">翻译模组:</span>
          <button class="btn btn-secondary" style="padding: 3px 10px; font-size: 11px;" @click="toggleAllMods">
            {{ selectedModIds.length === modIds.length ? '取消全选' : '全选' }}
          </button>
          <label v-for="id in modIds" :key="id" style="display: flex; align-items: center; gap: 3px; font-size: 12px; color: #374151; cursor: pointer;">
            <input type="checkbox" :value="id" v-model="selectedModIds" style="accent-color: #2563EB;" /> {{ id }}
          </label>
        </div>
      </div>

      <!-- Progress card -->
      <div class="card">
        <div style="font-weight: 600; font-size: 14px; margin-bottom: 4px;">
          {{ selectedModIds.length > 0 ? selectedModIds.join(' + ') : '全部模组' }}
        </div>

        <div class="progress-bar" style="height: 8px; margin: 14px 0;">
          <div class="progress-fill" :style="{ width: totalEntries > 0 ? (translatedCount / totalEntries * 100) + '%' : '0%' }"></div>
        </div>
        <div style="font-size: 28px; font-weight: 700; color: #2563EB;">
          {{ totalEntries > 0 ? Math.round(translatedCount / totalEntries * 100) : 0 }}%
        </div>

        <!-- Current item -->
        <div v-if="currentItem.key" style="background: #F9FAFB; padding: 14px 16px; border-radius: 8px; margin-top: 14px;">
          <div style="font-size: 11px; color: #9CA3AF; margin-bottom: 3px;">当前原文</div>
          <div style="font-size: 13px; font-weight: 500; margin-bottom: 8px;">{{ currentItem.source }}</div>
          <div style="font-size: 11px; color: #9CA3AF; margin-bottom: 3px;">AI 翻译</div>
          <div style="font-size: 14px; color: #2563EB; font-weight: 500;">{{ currentItem.translation }}</div>
        </div>

        <div style="display: flex; gap: 24px; margin-top: 14px;">
          <div>
            <div style="font-size: 11px; color: #9CA3AF;">已完成</div>
            <div style="font-size: 16px; font-weight: 600;">{{ translatedCount }} / {{ totalEntries }}</div>
          </div>
          <div>
            <div style="font-size: 11px; color: #9CA3AF;">剩余</div>
            <div style="font-size: 16px; font-weight: 600;" :style="{ color: untranslatedTargets.length === 0 ? '#16A34A' : '#D97706' }">
              {{ untranslatedTargets.length }}
            </div>
          </div>
        </div>

        <div style="display: flex; gap: 10px; margin-top: 16px;">
          <button v-if="!translating" class="btn btn-primary btn-lg" @click="startTranslate" :disabled="untranslatedTargets.length === 0">
            开始翻译 →
          </button>
          <template v-else>
            <button class="btn btn-lg" :class="paused ? 'btn-success' : 'btn-warning'" @click="togglePause">
              {{ paused ? '继续翻译' : '暂停' }}
            </button>
            <button class="btn btn-danger btn-lg" @click="stopTranslate">停止</button>
          </template>
        </div>

        <div v-if="translateLog.length > 0" style="margin-top: 14px; max-height: 100px; overflow: auto; background: #F9FAFB; padding: 8px 12px; border-radius: 6px; font-size: 11px; font-family: monospace;">
          <div v-for="(log, i) in translateLog.slice(-8)" :key="i" :style="{ color: log.startsWith('错误') || log.startsWith('连续') ? '#DC2626' : '#6B7280' }">{{ log }}</div>
        </div>
      </div>

      <!-- Settings -->
      <div class="collapse-header" @click="showSettings = !showSettings">
        <span>翻译设置</span>
        <span style="font-size: 11px; color: #9CA3AF; margin-left: 8px;">
          {{ apiConfigs[selectedApiIndex]?.name || '未配置' }} · {{ apiConfigs[selectedApiIndex]?.model || '' }}
        </span>
        <span style="margin-left: auto;">{{ showSettings ? '▲' : '▼' }}</span>
      </div>
      <div v-if="showSettings" class="collapse-body">
        <div class="card" style="margin-top: 0;">
          <div class="form-row">
            <div class="form-col">
              <label class="form-label">API 配置</label>
              <select v-model="selectedApiIndex" class="select">
                <option v-for="(cfg, i) in apiConfigs" :key="i" :value="i">
                  {{ cfg.name }} — {{ cfg.model }}
                </option>
              </select>
            </div>
            <div class="form-col">
              <label class="form-label">语言</label>
              <select class="select"><option>简体中文</option></select>
            </div>
          </div>
          <div v-if="!apiConfigs[selectedApiIndex]?.api_key" style="padding: 10px 14px; background: #FFFBEB; color: #D97706; border-radius: 6px; font-size: 12px;">
            请先在设置中配置 API Key
            <button class="btn btn-secondary" style="padding: 3px 10px; font-size: 11px; margin-left: 8px;" @click="goToSettings">去设置</button>
          </div>
          <div class="collapse-header" style="margin-top: 10px;" @click="showAdvanced = !showAdvanced">
            <span>高级设置</span>
            <span>{{ showAdvanced ? '▲' : '▼' }}</span>
          </div>
          <div v-if="showAdvanced" class="collapse-body">
            <div class="form-col">
              <label class="form-label">API 地址</label>
              <input :value="apiConfigs[selectedApiIndex]?.api_url || ''" class="input font-mono" style="font-size: 11px;" disabled />
            </div>
          </div>
        </div>
      </div>

      <!-- Bottom bar -->
      <div class="bottom-bar" v-if="translatedCount > 0 && !translating">
        <div class="bottom-bar-info">
          已翻译 <strong>{{ translatedCount }}</strong> 条 · 剩余 <strong>{{ untranslatedTargets.length }}</strong> 条
        </div>
        <button class="btn btn-primary btn-lg" @click="goToReview">
          进入校正 →
        </button>
      </div>
    </template>
  </div>
</template>
