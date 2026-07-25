<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useAppStore } from '../stores/app';
import type { LangEntry } from '../types';

const router = useRouter();
const store = useAppStore();

const searchQuery = ref('');
const activeTab = ref<'all' | 'unconfirmed' | 'ai'>('all');
const editingKey = ref('');
const editingModId = ref('');
const editText = ref('');
const batchFrom = ref('');
const batchTo = ref('');
const selectedModId = ref('');

watch(() => store.currentStep, (step) => {
  if (step === 'generate') router.push('/generate');
});

const modIds = computed(() => store.currentFiles.map(f => f.mod_id));

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

function goToGenerate() {
  const nextPath = store.completeStep('review');
  if (nextPath) router.push(nextPath);
}
</script>

<template>
  <div>
    <h2 style="font-size: 18px; font-weight: 600; margin-bottom: 20px; color: #1F2937;">校正翻译</h2>

    <div v-if="store.currentFiles.length === 0" class="empty-state">
      <div class="empty-state-icon">📝</div>
      <div class="empty-state-text">请先导入并翻译模组文件</div>
      <button class="btn btn-primary" @click="router.push('/import')">去导入</button>
    </div>

    <template v-else>
      <!-- Search / filter -->
      <div style="display: flex; gap: 12px; margin-bottom: 16px; align-items: center;">
        <div style="flex: 1; min-width: 180px; position: relative;">
          <input v-model="searchQuery" class="input" placeholder="搜索词条..." style="padding-left: 32px;" />
          <span style="position: absolute; left: 10px; top: 50%; transform: translateY(-50%); color: #9CA3AF; font-size: 14px;">🔍</span>
        </div>
        <select v-model="selectedModId" class="select" style="max-width: 160px;">
          <option value="">全部模组</option>
          <option v-for="id in modIds" :key="id" :value="id">{{ id }}</option>
        </select>
      </div>

      <!-- Tabs -->
      <div class="tab-bar">
        <button v-for="tab in [{k:'all',l:'全部'},{k:'unconfirmed',l:`未确认 (${unconfirmedCount})`},{k:'ai',l:'AI 建议'}]" :key="tab.k"
          class="tab-item" :class="{ active: activeTab === tab.k }" @click="activeTab = tab.k as any">{{ tab.l }}</button>
      </div>

      <!-- Batch replace -->
      <div style="display: flex; gap: 8px; margin-bottom: 16px; align-items: center; padding: 8px 14px; background: #F9FAFB; border-radius: 6px;">
        <span style="font-size: 12px; color: #6B7280; white-space: nowrap;">批量替换:</span>
        <input v-model="batchFrom" class="input" placeholder="查找" style="max-width: 130px; padding: 5px 10px; font-size: 12px;" />
        <span style="color: #9CA3AF;">→</span>
        <input v-model="batchTo" class="input" placeholder="替换为" style="max-width: 130px; padding: 5px 10px; font-size: 12px;" />
        <button class="btn btn-secondary" style="padding: 4px 12px; font-size: 12px;" @click="handleBatchReplace" :disabled="!batchFrom">替换</button>
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

      <!-- Bottom bar -->
      <div class="bottom-bar">
        <div class="bottom-bar-info">
          已确认 <strong>{{ translationsWithContent.length }}</strong> 条 · 未翻译 <strong style="color: #EF4444;">{{ unconfirmedCount }}</strong> 条 · 已修改 <strong>{{ store.reviewedCount }}</strong> 条
        </div>
        <button class="btn btn-primary btn-lg" :disabled="translationsWithContent.length === 0" @click="goToGenerate">
          保存并生成资源包 →
        </button>
      </div>
    </template>
  </div>
</template>
