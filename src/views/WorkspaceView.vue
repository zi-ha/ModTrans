<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '../stores/app';
import type { StepKey } from '../stores/app';
import type { ModLangFile, LangEntry, ApiConfig } from '../types';

const router = useRouter();
const store = useAppStore();

// ---------- 步骤切换 ----------

const steps: { key: StepKey; label: string; num: string }[] = [
  { key: 'import', label: '导入', num: '①' },
  { key: 'translate', label: '翻译', num: '②' },
  { key: 'review', label: '校正', num: '③' },
  { key: 'generate', label: '生成', num: '④' },
];

const sectionEls: Record<StepKey, any> = {
  import: ref(null),
  translate: ref(null),
  review: ref(null),
  generate: ref(null),
};

// 当前显示的步骤,由 store.currentStep 唯一驱动
const currentStep = computed(() => store.currentStep);

function stepSymbol(step: StepKey): string {
  const status = store.stepStatus[step];
  if (step === store.currentStep) return '●';
  if (status === 'completed') return '✓';
  return '○';
}

function stepClass(step: StepKey): string {
  const status = store.stepStatus[step];
  if (step === store.currentStep) return 'step-active';
  if (status === 'completed') return 'step-done';
  return 'step-pending';
}

function scrollToTop() {
  nextTick(() => {
    sectionEls[currentStep.value].value?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  });
}

// 点击步骤条:切换到该步骤
function goToStep(step: StepKey) {
  if (step === store.currentStep) return;
  if (!store.canAccess(step)) return;
  store.setCurrentStep(step);
  scrollToTop();
}

// 完成当前步骤:解锁并切到下一步
function finishStep(step: StepKey) {
  const next = store.completeStep(step);
  if (next) scrollToTop();
}

// ---------- ① 导入 ----------

const isDragging = ref(false);
const loading = ref(false);
const selectedFiles = ref<{ path: string; name: string; type: string }[]>([]);
const extractError = ref('');

async function handleBrowse() {
  const selected = await open({
    multiple: true,
    filters: [
      { name: 'Minecraft Mod', extensions: ['jar'] },
      { name: 'Language File', extensions: ['json', 'lang'] },
    ],
  });
  if (selected) {
    for (const path of selected) {
      const name = path.split('\\').pop() || path.split('/').pop() || path;
      const ext = name.split('.').pop()?.toLowerCase() || '';
      const type = ext === 'jar' ? 'Jar 模组' : `语言文件 (.${ext})`;
      if (!selectedFiles.value.find(f => f.path === path)) {
        selectedFiles.value.push({ path, name, type });
      }
    }
  }
}

function handleDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
  const files = e.dataTransfer?.files;
  if (!files) return;
  for (let i = 0; i < files.length; i++) {
    const f = files[i] as any;
    const name = f.name;
    const ext = name.split('.').pop()?.toLowerCase() || '';
    if (!['jar', 'json', 'lang'].includes(ext)) continue;
    const type = ext === 'jar' ? 'Jar 模组' : `语言文件 (.${ext})`;
    if (!selectedFiles.value.find(x => x.path === f.path)) {
      selectedFiles.value.push({ path: f.path, name: f.name, type });
    }
  }
}

function removeFile(index: number) {
  selectedFiles.value.splice(index, 1);
}

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  isDragging.value = true;
}

async function startExtract() {
  if (selectedFiles.value.length === 0) return;
  loading.value = true;
  extractError.value = '';
  const allEntries: ModLangFile[] = [];

  for (const file of selectedFiles.value) {
    try {
      const isJar = file.path.endsWith('.jar');
      const result: ModLangFile[] = await invoke(
        isJar ? 'extract_jar' : 'extract_lang',
        isJar ? { jarPath: file.path } : { filePath: file.path }
      );
      if (result) allEntries.push(...result);
    } catch (e: any) {
      extractError.value += `${file.name}: ${e}\n`;
    }
  }

  loading.value = false;

  if (allEntries.length > 0) {
    store.setFiles(allEntries);
    finishStep('import');
  }
}

// ---------- ② 翻译 ----------

const translating = ref(false);
const paused = ref(false);
const progress = ref({ current: 0, total: 0 });
const currentItem = ref<{ key: string; source: string; translation: string }>({ key: '', source: '', translation: '' });
const translateLog = ref<string[]>([]);
const apiConfigs = ref<ApiConfig[]>([]);
const selectedApiIndex = ref(0);
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
    translateLog.value.push('请先在设置中配置 API Key');
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

function toggleAllMods() {
  if (selectedModIds.value.length === modIds.value.length) selectedModIds.value = [];
  else selectedModIds.value = [...modIds.value];
}

function goToSettings() { router.push('/settings'); }

function sleep(ms: number) { return new Promise(r => setTimeout(r, ms)); }

// ---------- ③ 校正 ----------

const searchQuery = ref('');
const activeTab = ref<'all' | 'unconfirmed' | 'ai'>('all');
const editingKey = ref('');
const editingModId = ref('');
const editText = ref('');
const batchFrom = ref('');
const batchTo = ref('');
const selectedModId = ref('');

const allEntriesFlat = computed(() => {
  const result: LangEntry[] = [];
  for (const key of Object.keys(store.editedEntries)) result.push(...store.editedEntries[key]);
  return result;
});

const filteredEntries = computed(() => {
  let entries = allEntriesFlat.value;
  if (selectedModId.value) entries = entries.filter(e => e.mod_id === selectedModId.value);
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    entries = entries.filter(e =>
      e.key.toLowerCase().includes(q) || e.source.toLowerCase().includes(q) ||
      (e.translation && e.translation.toLowerCase().includes(q))
    );
  }
  if (activeTab.value === 'unconfirmed') entries = entries.filter(e => !e.translation);
  else if (activeTab.value === 'ai') entries = entries.filter(e => e.translation);
  return entries;
});

const translationsWithContent = computed(() => allEntriesFlat.value.filter(e => e.translation));
const unconfirmedCount = computed(() => allEntriesFlat.value.filter(e => !e.translation).length);

function startEdit(entry: LangEntry) {
  editingKey.value = entry.key;
  editingModId.value = entry.mod_id;
  editText.value = entry.translation;
}
function saveEdit() {
  if (editingKey.value) store.updateEntry(editingModId.value, editingKey.value, editText.value);
  editingKey.value = ''; editingModId.value = '';
}
function cancelEdit() { editingKey.value = ''; editingModId.value = ''; }

function handleBatchReplace() {
  if (!batchFrom.value) return;
  const target = selectedModId.value;
  if (target) store.batchReplace(target, batchFrom.value, batchTo.value);
  else for (const id of Object.keys(store.editedEntries)) store.batchReplace(id, batchFrom.value, batchTo.value);
  batchFrom.value = ''; batchTo.value = '';
}

// ---------- ④ 生成 ----------

const packName = ref('汉化资源包');
const packAuthor = ref('ModTrans');
const packVersion = ref('1.0');
const mcVersion = ref('1.20');
const generating = ref(false);
const outputPath = ref('');
const includeModIds = ref<string[]>([]);

const mcVersions = ['1.12', '1.12.2', '1.16', '1.16.5', '1.18', '1.18.2', '1.19', '1.19.4', '1.20', '1.20.1', '1.21'];

const packTranslatedCount = computed(() => {
  let count = 0;
  for (const key of Object.keys(store.editedEntries)) {
    if (includeModIds.value.length > 0 && !includeModIds.value.includes(key)) continue;
    count += store.editedEntries[key].filter(e => e.translation).length;
  }
  return count;
});

function toggleAllPackMods() {
  if (includeModIds.value.length === modIds.value.length) {
    includeModIds.value = [];
  } else {
    includeModIds.value = [...modIds.value];
  }
}

async function generatePack() {
  if (packTranslatedCount.value === 0) return;
  generating.value = true;
  outputPath.value = '';

  try {
    const langFiles: Record<string, string> = {};

    for (const file of store.currentFiles) {
      const modId = file.mod_id;
      if (includeModIds.value.length > 0 && !includeModIds.value.includes(modId)) continue;

      const entries = store.editedEntries[modId] || [];
      const translated = entries.filter(e => e.translation);
      if (translated.length === 0) continue;

      const format = file.format === 'Json' ? 'Json' : 'Lang';
      const content: string = await invoke('serialize_lang', {
        formatStr: format,
        entries: translated,
      });

      const ext = file.format === 'Json' ? '.json' : '.lang';
      const path = `assets/${modId}/lang/zh_cn${ext}`;
      langFiles[path] = content;
    }

    if (Object.keys(langFiles).length === 0) {
      alert('没有可导出的翻译内容');
      generating.value = false;
      return;
    }

    const savePath = await save({
      defaultPath: `${packName.value}_v${packVersion.value}.zip`,
      filters: [{ name: 'ZIP 压缩包', extensions: ['zip'] }],
    });

    if (!savePath) {
      generating.value = false;
      return;
    }

    const result: string = await invoke('generate_pack', {
      meta: {
        name: packName.value,
        author: packAuthor.value,
        description: `由 ModTrans 生成的汉化资源包`,
        version: packVersion.value,
        mc_version: mcVersion.value,
        universal: false,
      },
      langFiles,
      outputDir: savePath.substring(0, savePath.lastIndexOf('\\')),
    });

    outputPath.value = result;

    await invoke('add_history_record', {
      modName: packName.value,
      outputPath: result,
      entryCount: packTranslatedCount.value,
    });
  } catch (e: any) {
    alert(`生成失败: ${e}`);
  }
  generating.value = false;
}

onMounted(loadSettings);
</script>

<template>
  <div>
    <!-- 步骤条 -->
    <div class="step-bar card">
      <template v-for="(s, i) in steps" :key="s.key">
        <div
          class="step-bar-item"
          :class="stepClass(s.key)"
          @click="goToStep(s.key)"
        >
          <span class="step-bar-dot">{{ stepSymbol(s.key) }}</span>
          <span class="step-bar-label">{{ s.num }} {{ s.label }}</span>
        </div>
        <span v-if="i < steps.length - 1" class="step-bar-arrow">→</span>
      </template>
    </div>

    <!-- ① 导入 -->
    <section v-if="currentStep === 'import'" ref="sectionEls.import" class="card work-section">
      <div class="work-section-head">
        <span class="work-section-title">① 导入 Mod</span>
      </div>

      <div
        class="drop-zone"
        :class="{ 'drag-over': isDragging }"
        @drop="handleDrop"
        @dragover="handleDragOver"
        @dragleave="() => isDragging = false"
        @click="handleBrowse"
      >
        <div style="color: #D1D5DB; margin-bottom: 12px;">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#D1D5DB" stroke-width="1.5" stroke-linecap="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
        </div>
        <div style="font-size: 15px; font-weight: 500; color: #6B7280; margin-bottom: 4px;">
          拖入 .jar 文件或点击选择
        </div>
        <div style="font-size: 12px; color: #9CA3AF;">
          支持 .jar .lang .json
        </div>
      </div>

      <div v-if="selectedFiles.length > 0" class="file-list-card">
        <div class="file-list-head">已选择 {{ selectedFiles.length }} 个文件</div>
        <div class="file-list">
          <div v-for="(f, i) in selectedFiles" :key="i" class="file-row">
            <div>
              <div class="file-name">{{ f.name }}</div>
              <div class="file-type">{{ f.type }}</div>
            </div>
            <button class="btn btn-secondary btn-xs" @click="removeFile(i)">移除</button>
          </div>
        </div>
        <div class="file-list-foot">
          <span class="bottom-bar-info">共 <strong>{{ selectedFiles.length }}</strong> 个文件</span>
          <button class="btn btn-primary btn-lg" @click="startExtract" :disabled="loading">
            {{ loading ? '提取中...' : '开始提取 →' }}
          </button>
        </div>
      </div>

      <div v-if="extractError" class="error-box">{{ extractError }}</div>
    </section>

    <!-- ② 翻译 -->
    <section v-if="currentStep === 'translate'" ref="sectionEls.translate" class="card work-section">
      <div class="work-section-head">
        <span class="work-section-title">② 翻译</span>
      </div>

      <!-- 工具栏:模组选择 + API 配置 -->
      <div class="card toolbar-card">
        <div class="toolbar-row">
          <span class="toolbar-label">翻译模组</span>
          <button class="btn btn-secondary btn-xs" @click="toggleAllMods">
            {{ selectedModIds.length === modIds.length ? '取消全选' : '全选' }}
          </button>
          <label v-for="id in modIds" :key="id" class="mod-check">
            <input type="checkbox" :value="id" v-model="selectedModIds" /> {{ id }}
          </label>
          <span class="toolbar-sep"></span>
          <span class="toolbar-label">API</span>
          <select v-model="selectedApiIndex" class="select" style="width: 240px;">
            <option v-for="(cfg, i) in apiConfigs" :key="i" :value="i">
              {{ cfg.name }} — {{ cfg.model }}
            </option>
          </select>
        </div>
        <div v-if="!apiConfigs[selectedApiIndex]?.api_key" class="warn-box">
          请先在设置中配置 API Key
          <button class="btn btn-secondary btn-xs" style="margin-left: 8px;" @click="goToSettings">去设置</button>
        </div>
      </div>

      <!-- 进度 -->
      <div class="card">
        <div class="progress-head">
          {{ selectedModIds.length > 0 ? selectedModIds.join(' + ') : '全部模组' }}
        </div>

        <div class="progress-bar" style="height: 8px; margin: 12px 0;">
          <div class="progress-fill" :style="{ width: totalEntries > 0 ? (translatedCount / totalEntries * 100) + '%' : '0%' }"></div>
        </div>
        <div class="progress-pct">
          {{ totalEntries > 0 ? Math.round(translatedCount / totalEntries * 100) : 0 }}%
        </div>

        <div class="stats-row">
          <div>
            <div class="stat-label">已完成</div>
            <div class="stat-value">{{ translatedCount }} / {{ totalEntries }}</div>
          </div>
          <div>
            <div class="stat-label">剩余</div>
            <div class="stat-value" :style="{ color: untranslatedTargets.length === 0 ? '#16A34A' : '#D97706' }">
              {{ untranslatedTargets.length }}
            </div>
          </div>
        </div>

        <!-- 当前词条 -->
        <div v-if="currentItem.key" class="current-item">
          <div class="current-label">当前原文</div>
          <div class="current-source">{{ currentItem.source }}</div>
          <div class="current-label">AI 翻译</div>
          <div class="current-translation">{{ currentItem.translation }}</div>
        </div>

        <div class="ctrl-row">
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

        <div v-if="translateLog.length > 0" class="log-box">
          <div v-for="(log, i) in translateLog.slice(-8)" :key="i" :style="{ color: log.startsWith('错误') || log.startsWith('连续') ? '#DC2626' : '#6B7280' }">{{ log }}</div>
        </div>
      </div>

      <!-- 底部操作 -->
      <div class="bottom-bar" v-if="translatedCount > 0 && !translating">
        <div class="bottom-bar-info">
          已翻译 <strong>{{ translatedCount }}</strong> 条 · 剩余 <strong>{{ untranslatedTargets.length }}</strong> 条
        </div>
        <button class="btn btn-primary btn-lg" @click="finishStep('translate')">
          进入校正 →
        </button>
      </div>
    </section>

    <!-- ③ 校正 -->
    <section v-if="currentStep === 'review'" ref="sectionEls.review" class="card work-section">
      <div class="work-section-head">
        <span class="work-section-title">③ 校正</span>
      </div>

      <!-- 工具栏:搜索 + 筛选 + 批量替换 -->
      <div class="card toolbar-card">
        <div class="toolbar-row">
          <div class="search-wrap">
            <input v-model="searchQuery" class="input" placeholder="搜索词条..." style="padding-left: 32px;" />
            <span class="search-icon">🔍</span>
          </div>
          <select v-model="selectedModId" class="select" style="max-width: 150px;">
            <option value="">全部模组</option>
            <option v-for="id in modIds" :key="id" :value="id">{{ id }}</option>
          </select>
          <span class="toolbar-sep"></span>
          <span class="toolbar-label">批量替换</span>
          <input v-model="batchFrom" class="input" placeholder="查找" style="max-width: 120px; padding: 6px 10px; font-size: 12px;" />
          <span style="color: #9CA3AF;">→</span>
          <input v-model="batchTo" class="input" placeholder="替换为" style="max-width: 120px; padding: 6px 10px; font-size: 12px;" />
          <button class="btn btn-secondary" style="padding: 4px 12px; font-size: 12px;" @click="handleBatchReplace" :disabled="!batchFrom">替换</button>
        </div>
      </div>

      <!-- Tabs -->
      <div class="tab-bar">
        <button v-for="tab in [{k:'all',l:'全部'},{k:'unconfirmed',l:`未翻译 (${unconfirmedCount})`},{k:'ai',l:'AI 建议'}]" :key="tab.k"
          class="tab-item" :class="{ active: activeTab === tab.k }" @click="activeTab = tab.k as any">{{ tab.l }}</button>
      </div>

      <!-- Table -->
      <div class="card" style="padding: 0; overflow: hidden;">
        <div style="max-height: 460px; overflow: auto;">
          <table class="table">
            <thead><tr>
              <th style="width: 28%;">Key</th><th style="width: 28%;">原文</th><th style="width: 34%;">翻译</th><th style="width: 10%;">类型</th>
            </tr></thead>
            <tbody>
              <tr v-for="entry in filteredEntries.slice(0, 300)" :key="entry.key + entry.mod_id">
                <td style="font-family: monospace; font-size: 11px; max-width: 190px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" :title="entry.key">
                  {{ entry.key.split('.').pop() || entry.key }}
                </td>
                <td style="max-width: 190px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" :title="entry.source">{{ entry.source }}</td>
                <td @dblclick="startEdit(entry)" style="cursor: pointer;">
                  <template v-if="editingKey === entry.key && editingModId === entry.mod_id">
                    <div style="display: flex; gap: 4px;">
                      <input v-model="editText" class="input" style="flex: 1; padding: 4px 8px; font-size: 12px;"
                        @keydown.enter="saveEdit" @keydown.escape="cancelEdit" />
                      <button class="btn btn-success" style="padding: 4px 8px; font-size: 11px;" @click="saveEdit">✓</button>
                      <button class="btn btn-danger" style="padding: 4px 8px; font-size: 11px;" @click="cancelEdit">✕</button>
                    </div>
                  </template>
                  <template v-else>
                    <span v-if="entry.translation" style="color: #374151;">{{ entry.translation }}</span>
                    <span v-else style="color: #EF4444; font-style: italic; font-size: 12px;">未翻译（双击编辑）</span>
                  </template>
                </td>
                <td>
                  <span :class="['tag', entry.is_vanilla ? 'tag-vanilla' : entry.translation ? 'tag-ai' : 'tag-mod']">
                    {{ entry.is_vanilla ? '原生' : entry.translation ? 'AI' : '模组' }}
                  </span>
                </td>
              </tr>
              <tr v-if="filteredEntries.length === 0"><td colspan="4" style="text-align: center; padding: 40px; color: #9CA3AF;">无匹配词条</td></tr>
            </tbody>
          </table>
        </div>
        <div v-if="filteredEntries.length > 300" style="padding: 10px 14px; font-size: 12px; color: #9CA3AF; border-top: 1px solid #E5E7EB;">
          显示前 300 条（共 {{ filteredEntries.length }} 条），请使用搜索缩小范围
        </div>
      </div>

      <!-- 底部操作 -->
      <div class="bottom-bar">
        <div class="bottom-bar-info">
          已翻译 <strong>{{ translationsWithContent.length }}</strong> 条 · 未翻译 <strong style="color: #EF4444;">{{ unconfirmedCount }}</strong> 条 · 已修改 <strong>{{ store.reviewedCount }}</strong> 条
        </div>
        <button class="btn btn-primary btn-lg" :disabled="translationsWithContent.length === 0" @click="finishStep('review')">
          进入生成 →
        </button>
      </div>
    </section>

    <!-- ④ 生成 -->
    <section v-if="currentStep === 'generate'" ref="sectionEls.generate" class="card work-section">
      <div class="work-section-head">
        <span class="work-section-title">④ 生成资源包</span>
      </div>

      <div class="card">
        <div class="card-title" style="font-size: 13px;">资源包信息</div>
        <div class="form-row">
          <div class="form-col">
            <label class="form-label">Minecraft 版本</label>
            <select v-model="mcVersion" class="select">
              <option v-for="v in mcVersions" :key="v" :value="v">{{ v }}</option>
            </select>
          </div>
          <div class="form-col">
            <label class="form-label">资源包名称</label>
            <input v-model="packName" class="input" placeholder="汉化资源包" />
          </div>
        </div>
        <div class="form-row">
          <div class="form-col">
            <label class="form-label">作者</label>
            <input v-model="packAuthor" class="input" placeholder="ModTrans" />
          </div>
          <div class="form-col">
            <label class="form-label">版本号</label>
            <input v-model="packVersion" class="input" placeholder="1.0" />
          </div>
        </div>

        <div class="gen-divider"></div>

        <div class="card-title" style="font-size: 13px;">包含 Mod</div>
        <div class="gen-mods">
          <button class="btn btn-secondary btn-xs" @click="toggleAllPackMods">
            {{ includeModIds.length === modIds.length ? '取消全选' : '全选' }}
          </button>
          <label v-for="id in modIds" :key="id" class="mod-check">
            <input type="checkbox" :value="id" v-model="includeModIds" />
            {{ id }}
          </label>
        </div>

        <div class="gen-stats">
          <div>
            <div class="stat-label">翻译词条</div>
            <div class="stat-value" style="color: #2563EB;">{{ packTranslatedCount }}</div>
          </div>
          <div>
            <div class="stat-label">包含 Mod</div>
            <div class="stat-value" style="color: #16A34A;">{{ includeModIds.length || modIds.length }}</div>
          </div>
        </div>
      </div>

      <div class="bottom-bar">
        <div class="bottom-bar-info">
          将生成 <strong>{{ mcVersion }}</strong> 版本的资源包（含 pack.mcmeta）
        </div>
        <button class="btn btn-success btn-lg" @click="generatePack" :disabled="generating || packTranslatedCount === 0">
          {{ generating ? '生成中...' : '生成 ZIP' }}
        </button>
      </div>

      <div v-if="outputPath" class="card success-card">
        <div style="font-size: 13px; font-weight: 600; color: #16A34A; margin-bottom: 6px;">✓ 生成成功</div>
        <div style="font-size: 12px; color: #6B7280; word-break: break-all;">{{ outputPath }}</div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.step-bar {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
  margin: -32px -40px 20px;
  padding: 14px 40px 12px;
  border-radius: 0;
  border-left: none;
  border-right: none;
  border-top: none;
}

.step-bar-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  user-select: none;
  transition: all 0.12s;
}

.step-bar-item:hover {
  background: #F3F4F6;
}

.step-bar-item.step-active {
  color: #2563EB;
  background: #EFF6FF;
  font-weight: 500;
}

.step-bar-item.step-done {
  color: #16A34A;
}

.step-bar-item.step-pending {
  color: #9CA3AF;
  cursor: default;
}

.step-bar-item.step-pending:hover {
  background: transparent;
}

.step-bar-dot {
  font-size: 11px;
  width: 14px;
  text-align: center;
}

.step-bar-arrow {
  color: #D1D5DB;
  font-size: 13px;
  padding: 0 2px;
}

.work-section {
  scroll-margin-top: 20px;
}

.work-section-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 18px;
}

.work-section-title {
  font-size: 15px;
  font-weight: 600;
  color: #1F2937;
}

/* 导入:文件列表 */
.file-list-card {
  margin-top: 16px;
  border: 1px solid #E5E7EB;
  border-radius: 8px;
  overflow: hidden;
}

.file-list-head {
  padding: 12px 16px;
  font-size: 13px;
  font-weight: 600;
  color: #1F2937;
  border-bottom: 1px solid #F3F4F6;
}

.file-list {
  max-height: 200px;
  overflow: auto;
  padding: 0 16px;
}

.file-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 9px 0;
  border-bottom: 1px solid #F3F4F6;
}

.file-row:last-child {
  border-bottom: none;
}

.file-name {
  font-size: 13px;
  font-weight: 500;
}

.file-type {
  font-size: 11px;
  color: #9CA3AF;
}

.file-list-foot {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-top: 1px solid #E5E7EB;
}

.error-box {
  margin-top: 12px;
  padding: 10px 14px;
  background: #FEF2F2;
  border: 1px solid #FECACA;
  border-radius: 6px;
  font-size: 12px;
  color: #DC2626;
  white-space: pre-wrap;
}

/* 通用工具栏 */
.toolbar-card {
  padding: 14px 20px;
  margin-bottom: 16px;
}

.toolbar-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.toolbar-label {
  font-size: 12px;
  color: #6B7280;
  white-space: nowrap;
}

.toolbar-sep {
  width: 1px;
  height: 18px;
  background: #E5E7EB;
  margin: 0 4px;
}

.btn-xs {
  padding: 3px 10px;
  font-size: 11px;
}

.mod-check {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 12px;
  color: #374151;
  cursor: pointer;
}

.mod-check input[type="checkbox"] {
  accent-color: #2563EB;
}

.warn-box {
  margin-top: 10px;
  padding: 8px 12px;
  background: #FFFBEB;
  color: #D97706;
  border-radius: 6px;
  font-size: 12px;
}

.search-wrap {
  position: relative;
  flex: 1;
  min-width: 180px;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: #9CA3AF;
  font-size: 14px;
}

/* 翻译:进度 */
.progress-head {
  font-weight: 600;
  font-size: 14px;
}

.progress-pct {
  font-size: 28px;
  font-weight: 700;
  color: #2563EB;
}

.stats-row {
  display: flex;
  gap: 24px;
  margin-top: 12px;
}

.stat-label {
  font-size: 11px;
  color: #9CA3AF;
}

.stat-value {
  font-size: 16px;
  font-weight: 600;
}

.current-item {
  background: #F9FAFB;
  padding: 14px 16px;
  border-radius: 8px;
  margin-top: 14px;
}

.current-label {
  font-size: 11px;
  color: #9CA3AF;
  margin-bottom: 3px;
}

.current-source {
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 8px;
}

.current-translation {
  font-size: 14px;
  color: #2563EB;
  font-weight: 500;
}

.ctrl-row {
  display: flex;
  gap: 10px;
  margin-top: 16px;
}

.log-box {
  margin-top: 14px;
  max-height: 100px;
  overflow: auto;
  background: #F9FAFB;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 11px;
  font-family: monospace;
}

/* 生成 */
.gen-divider {
  height: 1px;
  background: #F3F4F6;
  margin: 16px 0;
}

.gen-mods {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  align-items: center;
}

.gen-stats {
  display: flex;
  gap: 24px;
  margin-top: 16px;
}

.success-card {
  border-color: #16A34A;
  background: #F0FDF4;
}
</style>
