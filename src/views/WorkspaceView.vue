<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '../stores/app';
import type { StepKey } from '../stores/app';
import type { ModLangFile, LangEntry, ApiConfig } from '../types';

const store = useAppStore();

// ---------- 页面切换 ----------

const sectionEls: Record<StepKey, any> = {
  import: ref(null),
  translate: ref(null),
  review: ref(null),
  generate: ref(null),
};

const currentStep = computed(() => store.currentStep);

function scrollToTop() {
  nextTick(() => {
    sectionEls[currentStep.value].value?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  });
}

// 返回/更换入口:切换到指定页面
function goToStep(step: StepKey) {
  if (step === store.currentStep) return;
  if (!store.canAccess(step)) return;
  store.setCurrentStep(step);
  scrollToTop();
}

// 完成当前阶段并进入下一页面
function finishStep(step: StepKey) {
  const next = store.completeStep(step);
  if (next) scrollToTop();
}

// 翻译完成后跳过校正直接导出
function skipToExport() {
  if (store.stepStatus['translate'] !== 'completed') store.completeStep('translate');
  store.completeStep('review');
  scrollToTop();
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

// 拖放:文件路径由 Tauri onDragDropEvent 提供(HTML5 dataTransfer 拿不到路径)
function handleDroppedPaths(paths: string[]) {
  for (const path of paths) {
    const name = path.split('\\').pop() || path.split('/').pop() || path;
    const ext = name.split('.').pop()?.toLowerCase() || '';
    if (!['jar', 'json', 'lang'].includes(ext)) continue;
    const type = ext === 'jar' ? 'Jar 模组' : `语言文件 (.${ext})`;
    if (!selectedFiles.value.find(f => f.path === path)) {
      selectedFiles.value.push({ path, name, type });
    }
  }
}

let unlistenDrag: (() => void) | null = null;

onMounted(async () => {
  loadSettings();
  try {
    const win = getCurrentWindow();
    unlistenDrag = await win.onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === 'enter' || p.type === 'over') {
        isDragging.value = true;
      } else if (p.type === 'drop') {
        isDragging.value = false;
        handleDroppedPaths(p.paths);
      } else if (p.type === 'leave') {
        isDragging.value = false;
      }
    });
  } catch (e) { /* 非 Tauri 环境忽略 */ }
});

onUnmounted(() => {
  unlistenDrag?.();
});

function removeFile(index: number) {
  selectedFiles.value.splice(index, 1);
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
const translationRules = ref<string[]>([]);

const RULE_TEXTS: Record<string, string> = {
  keep_vars: '保留变量占位符（%s、%d、%1$s 等），不要翻译或修改它们。',
  keep_format_codes: '保留 Minecraft 颜色/格式代码（§ 开头的代码），不要翻译或修改它们。',
};

const modIds = computed(() => store.currentFiles.map(f => f.mod_id));

const untranslatedTargets = computed(() => {
  const result: LangEntry[] = [];
  for (const key of Object.keys(store.editedEntries)) {
    for (const e of store.editedEntries[key]) {
      if (!e.translation) result.push(e);
    }
  }
  return result;
});

const totalEntries = computed(() => {
  let count = 0;
  for (const key of Object.keys(store.editedEntries)) count += store.editedEntries[key].length;
  return count;
});

const translatedCount = computed(() => {
  let count = 0;
  for (const key of Object.keys(store.editedEntries)) count += store.editedEntries[key].filter(e => e.translation).length;
  return count;
});

async function loadSettings() {
  try {
    const s = await invoke<any>('get_settings');
    apiConfigs.value = s.api_configs || [];
    selectedApiIndex.value = s.active_api_index ?? 0;
    translationRules.value = s.translation_rules || [];
  } catch (e) { /* ignore */ }
}

// 勾选的翻译规则拼入提示词,与自定义提示词一起发送
function buildConfigWithRules(apiCfg: ApiConfig): ApiConfig {
  const rulesText = translationRules.value
    .map(r => RULE_TEXTS[r])
    .filter(Boolean)
    .join('\n');
  if (!rulesText) return apiCfg;
  return { ...apiCfg, custom_prompt: [apiCfg.custom_prompt, rulesText].filter(Boolean).join('\n') };
}

async function startTranslate() {
  const targets = untranslatedTargets.value;
  if (targets.length === 0 || translating.value) return;

  const apiCfg = apiConfigs.value[selectedApiIndex.value];
  if (!apiCfg?.api_key) {
    translateLog.value.push('请先在设置中配置 API Key');
    return;
  }

  const config = buildConfigWithRules(apiCfg);

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
        config,
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

// ---------- ④ 导出 ----------

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
    alert(`导出失败: ${e}`);
  }
  generating.value = false;
}
</script>

<template>
  <div>
    <!-- 导入 -->
    <div v-if="currentStep === 'import'" ref="sectionEls.import">
      <div class="page-head">
        <h2>导入 Mod</h2>
      </div>

      <div
        class="drop-bar"
        :class="{ 'drag-over': isDragging }"
        @dragover.prevent
        @drop.prevent
        @click="handleBrowse"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="17 8 12 3 7 8"/>
          <line x1="12" y1="3" x2="12" y2="15"/>
        </svg>
        <span class="drop-bar-text">拖入 .jar 文件</span>
        <button class="btn btn-secondary" @click.stop="handleBrowse">选择文件</button>
      </div>
      <div class="page-hint">支持 .jar .lang .json 文件</div>

      <div v-if="selectedFiles.length > 0" class="panel">
        <div class="file-list">
          <div v-for="(f, i) in selectedFiles" :key="i" class="file-row">
            <div>
              <div class="file-name">{{ f.name }}</div>
              <div class="file-type">{{ f.type }}</div>
            </div>
            <button class="btn btn-secondary btn-xs" @click="removeFile(i)">移除</button>
          </div>
        </div>
        <div class="panel-foot">
          <span class="panel-foot-text">已选择 {{ selectedFiles.length }} 个文件</span>
          <button class="btn btn-primary" @click="startExtract" :disabled="loading">
            {{ loading ? '提取中...' : '开始提取' }}
          </button>
        </div>
      </div>

      <div v-if="extractError" class="error-box">{{ extractError }}</div>
    </div>

    <!-- 翻译 -->
    <div v-else-if="currentStep === 'translate'" ref="sectionEls.translate">
      <div class="page-head">
        <h2>翻译</h2>
        <button class="link-btn" @click="goToStep('import')">← 更换 Mod</button>
      </div>

      <!-- 翻译中 -->
      <div v-if="translating" class="panel">
        <div class="field-label">正在翻译</div>
        <div class="progress-bar" style="height: 8px; margin: 12px 0;">
          <div class="progress-fill" :style="{ width: progress.total > 0 ? (progress.current / progress.total * 100) + '%' : '0%' }"></div>
        </div>
        <div class="progress-num">{{ progress.current }} / {{ progress.total }}</div>

        <div v-if="currentItem.key" class="current-pair">
          <div class="pair-row">
            <span class="pair-label">当前原文</span>
            <span>{{ currentItem.source }}</span>
          </div>
          <div class="pair-row">
            <span class="pair-label">译文</span>
            <span class="pair-trans">{{ currentItem.translation }}</span>
          </div>
        </div>

        <div class="ctrl-row">
          <button class="btn" :class="paused ? 'btn-success' : 'btn-warning'" @click="togglePause">
            {{ paused ? '继续翻译' : '暂停' }}
          </button>
          <button class="btn btn-danger" @click="stopTranslate">停止</button>
        </div>
      </div>

      <!-- 待翻译 / 已完成 -->
      <div v-else class="panel">
        <div v-if="translatedCount > 0 && untranslatedTargets.length === 0" class="field-label" style="color: #52C41A;">✓ 翻译完成</div>

        <div class="progress-bar" style="height: 8px; margin: 12px 0;">
          <div class="progress-fill" :style="{ width: totalEntries > 0 ? (translatedCount / totalEntries * 100) + '%' : '0%' }"></div>
        </div>
        <div class="progress-num">{{ translatedCount }} / {{ totalEntries }}</div>
        <div v-if="translatedCount > 0 && untranslatedTargets.length === 0" class="field-value" style="margin-top: 4px;">
          {{ translatedCount }} 条文本已完成
        </div>

        <div class="stats-row">
          <div>
            <div class="stat-label">已完成</div>
            <div class="stat-value">{{ translatedCount }} / {{ totalEntries }}</div>
          </div>
          <div>
            <div class="stat-label">剩余</div>
            <div class="stat-value" :style="{ color: untranslatedTargets.length === 0 ? '#52C41A' : '#FA8C16' }">
              {{ untranslatedTargets.length }}
            </div>
          </div>
        </div>

        <div class="ctrl-row">
          <button v-if="untranslatedTargets.length > 0" class="btn btn-primary btn-lg" @click="startTranslate">
            开始翻译
          </button>
          <template v-if="translatedCount > 0">
            <button class="btn btn-primary btn-lg" @click="finishStep('translate')">查看校正</button>
            <button class="btn btn-success btn-lg" @click="skipToExport">导出资源包</button>
          </template>
        </div>
      </div>

      <div v-if="translateLog.length > 0" class="log-box">
        <div v-for="(log, i) in translateLog.slice(-8)" :key="i" :style="{ color: log.startsWith('错误') || log.startsWith('连续') ? '#FF4D4F' : '#6B7280' }">{{ log }}</div>
      </div>
    </div>

    <!-- 校正 -->
    <div v-else-if="currentStep === 'review'" ref="sectionEls.review">
      <div class="page-head">
        <h2>校正</h2>
        <button class="link-btn" @click="goToStep('translate')">← 返回翻译</button>
      </div>

      <div class="panel">
        <div class="toolbar-row">
          <div class="search-wrap">
            <input v-model="searchQuery" class="input" placeholder="搜索词条..." style="padding-left: 32px;" />
            <span class="search-icon">🔍</span>
          </div>
          <select v-model="selectedModId" class="select" style="max-width: 140px;">
            <option value="">全部模组</option>
            <option v-for="id in modIds" :key="id" :value="id">{{ id }}</option>
          </select>
          <span class="field-sep"></span>
          <span class="field-label">批量替换</span>
          <input v-model="batchFrom" class="input" placeholder="查找" style="max-width: 110px; padding: 6px 10px; font-size: 12px;" />
          <span style="color: #B7BCC4;">→</span>
          <input v-model="batchTo" class="input" placeholder="替换为" style="max-width: 110px; padding: 6px 10px; font-size: 12px;" />
          <button class="btn btn-secondary btn-xs" @click="handleBatchReplace" :disabled="!batchFrom">替换</button>
        </div>
        <div class="tab-bar" style="margin-top: 14px;">
          <button v-for="tab in [{k:'all',l:'全部'},{k:'unconfirmed',l:`未翻译 (${unconfirmedCount})`},{k:'ai',l:'AI 建议'}]" :key="tab.k"
            class="tab-item" :class="{ active: activeTab === tab.k }" @click="activeTab = tab.k as any">{{ tab.l }}</button>
        </div>
      </div>

      <div class="panel" style="padding: 0; overflow: hidden;">
        <div style="max-height: 400px; overflow: auto;">
          <table class="table">
            <thead><tr>
              <th style="width: 42%;">原文</th><th>中文</th>
            </tr></thead>
            <tbody>
              <tr v-for="entry in filteredEntries.slice(0, 300)" :key="entry.key + entry.mod_id">
                <td class="src-cell">
                  <div class="src-text">{{ entry.source }}</div>
                  <div class="src-key">{{ entry.key.split('.').pop() || entry.key }}</div>
                </td>
                <td class="zh-cell" @click="startEdit(entry)">
                  <template v-if="editingKey === entry.key && editingModId === entry.mod_id">
                    <div style="display: flex; gap: 4px;">
                      <input v-model="editText" class="input" style="flex: 1; padding: 4px 8px; font-size: 12px;"
                        @keydown.enter="saveEdit" @keydown.escape="cancelEdit" />
                      <button class="btn btn-success" style="padding: 4px 8px; font-size: 11px;" @click="saveEdit">✓</button>
                      <button class="btn btn-danger" style="padding: 4px 8px; font-size: 11px;" @click="cancelEdit">✕</button>
                    </div>
                  </template>
                  <template v-else>
                    <span v-if="entry.translation" class="zh-text">{{ entry.translation }}</span>
                    <span v-else class="zh-empty">{{ entry.is_vanilla ? '原生文本' : '待翻译' }}</span>
                    <span v-if="entry.is_vanilla" class="zh-tag">原生</span>
                  </template>
                </td>
              </tr>
              <tr v-if="filteredEntries.length === 0"><td colspan="2" style="text-align: center; padding: 40px; color: #9CA3AF;">无匹配词条</td></tr>
            </tbody>
          </table>
        </div>
        <div v-if="filteredEntries.length > 300" style="padding: 10px 14px; font-size: 12px; color: #9CA3AF; border-top: 1px solid #EBEDF0;">
          显示前 300 条（共 {{ filteredEntries.length }} 条），请使用搜索缩小范围
        </div>
      </div>

      <div class="panel">
        <div style="display: flex; justify-content: space-between; align-items: center;">
          <span class="panel-foot-text">
            已翻译 <strong>{{ translationsWithContent.length }}</strong> 条 · 未翻译 <strong style="color: #FF4D4F;">{{ unconfirmedCount }}</strong> 条
          </span>
          <button class="btn btn-primary btn-lg" :disabled="translationsWithContent.length === 0" @click="finishStep('review')">
            导出资源包
          </button>
        </div>
      </div>
    </div>

    <!-- 导出 -->
    <div v-else ref="sectionEls.generate">
      <div class="page-head">
        <h2>导出资源包</h2>
        <button class="link-btn" @click="goToStep('review')">← 返回校正</button>
      </div>

      <div class="panel">
        <div class="form-row">
          <div class="form-col">
            <label class="form-label">资源包名称</label>
            <input v-model="packName" class="input" placeholder="例如: Sodium 中文包" />
          </div>
          <div class="form-col">
            <label class="form-label">Minecraft 版本</label>
            <select v-model="mcVersion" class="select">
              <option v-for="v in mcVersions" :key="v" :value="v">{{ v }}</option>
            </select>
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

        <div class="field-row" style="margin-top: 4px;">
          <span class="field-label">包含</span>
          <button class="btn btn-secondary btn-xs" @click="toggleAllPackMods">
            {{ includeModIds.length === modIds.length ? '取消全选' : '全选' }}
          </button>
          <label v-for="id in modIds" :key="id" class="mod-check">
            <input type="checkbox" :value="id" v-model="includeModIds" /> {{ id }}
          </label>
        </div>
      </div>

      <div class="panel">
        <div style="display: flex; justify-content: space-between; align-items: center;">
          <span class="panel-foot-text">
            将导出 <strong>{{ packTranslatedCount }}</strong> 条翻译 → {{ mcVersion }} 资源包
          </span>
          <button class="btn btn-success btn-lg" @click="generatePack" :disabled="generating || packTranslatedCount === 0">
            {{ generating ? '导出中...' : '导出' }}
          </button>
        </div>
      </div>

      <div v-if="outputPath" class="success-card">
        <div style="font-size: 13px; font-weight: 600; color: #52C41A; margin-bottom: 4px;">✓ 导出成功</div>
        <div style="font-size: 12px; color: #6B7280; word-break: break-all;">{{ outputPath }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
</style>
