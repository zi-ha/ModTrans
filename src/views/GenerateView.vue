<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '../stores/app';

const store = useAppStore();

const packName = ref('汉化资源包');
const packAuthor = ref('ModTrans');
const packVersion = ref('1.0');
const mcVersion = ref('1.20');
const showAdvanced = ref(false);
const genPackMcmeta = ref(true);
const utf8Fix = ref(true);
const mergeOrigin = ref(false);
const generating = ref(false);
const outputPath = ref('');
const includeModIds = ref<string[]>([]);

const mcVersions = ['1.12', '1.12.2', '1.16', '1.16.5', '1.18', '1.18.2', '1.19', '1.19.4', '1.20', '1.20.1', '1.21'];

const modIds = computed(() => store.currentFiles.map(f => f.mod_id));

const translatedCount = computed(() => {
  let count = 0;
  for (const key of Object.keys(store.editedEntries)) {
    if (includeModIds.value.length > 0 && !includeModIds.value.includes(key)) continue;
    count += store.editedEntries[key].filter(e => e.translation).length;
  }
  return count;
});

function toggleAllMods() {
  if (includeModIds.value.length === modIds.value.length) {
    includeModIds.value = [];
  } else {
    includeModIds.value = [...modIds.value];
  }
}

async function generatePack() {
  if (translatedCount.value === 0) return;
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
      entryCount: translatedCount.value,
    });
  } catch (e: any) {
    alert(`生成失败: ${e}`);
  }
  generating.value = false;
}
</script>

<template>
  <div>
    <h2 style="font-size: 18px; font-weight: 600; margin-bottom: 20px; color: #1F2937;">生成汉化资源包</h2>

    <div v-if="!store.hasAnyTranslation()" class="empty-state">
      <div class="empty-state-icon">💾</div>
      <div class="empty-state-text">请先完成翻译和校正</div>
      <button class="btn btn-primary" @click="$router.push('/review')">去校正</button>
    </div>

    <template v-else>
      <!-- Mod selector -->
      <div class="card">
        <div class="card-title" style="font-size: 13px;">选择包含的 Mod</div>
        <div style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center;">
          <button class="btn btn-secondary" style="padding: 4px 12px; font-size: 12px;" @click="toggleAllMods">
            {{ includeModIds.length === modIds.length ? '取消全选' : '全选' }}
          </button>
          <label v-for="id in modIds" :key="id" style="display: flex; align-items: center; gap: 4px; font-size: 12px; cursor: pointer;">
            <input type="checkbox" :value="id" v-model="includeModIds" style="accent-color: #2563EB;" />
            {{ id }}
          </label>
        </div>
      </div>

      <!-- Main config -->
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

        <div style="margin-top: 8px; display: flex; gap: 24px;">
          <div>
            <div style="font-size: 12px; color: #9CA3AF;">翻译词条</div>
            <div style="font-size: 18px; font-weight: 600; color: #2563EB;">{{ translatedCount }}</div>
          </div>
          <div>
            <div style="font-size: 12px; color: #9CA3AF;">包含 Mod</div>
            <div style="font-size: 18px; font-weight: 600; color: #16A34A;">{{ includeModIds.length || modIds.length }}</div>
          </div>
        </div>
      </div>

      <!-- Advanced -->
      <div class="collapse-header" @click="showAdvanced = !showAdvanced">
        <span>高级选项</span>
        <span>{{ showAdvanced ? '▲' : '▼' }}</span>
      </div>
      <div v-if="showAdvanced" class="collapse-body">
        <div class="card" style="margin-top: 0;">
          <label class="checkbox-label">
            <input type="checkbox" v-model="genPackMcmeta" />
            生成 pack.mcmeta
          </label>
          <label class="checkbox-label">
            <input type="checkbox" v-model="utf8Fix" />
            UTF-8 编码修复
          </label>
          <label class="checkbox-label">
            <input type="checkbox" v-model="mergeOrigin" />
            合并原语言文件
          </label>
        </div>
      </div>

      <div class="bottom-bar">
        <div class="bottom-bar-info">
          将生成 <strong>{{ mcVersion }}</strong> 版本的资源包
        </div>
        <button class="btn btn-success btn-lg" @click="generatePack" :disabled="generating || translatedCount === 0">
          {{ generating ? '生成中...' : '生成 ZIP' }}
        </button>
      </div>

      <div v-if="outputPath" class="card" style="border-color: #16A34A; background: #F0FDF4;">
        <div style="font-size: 13px; font-weight: 600; color: #16A34A; margin-bottom: 6px;">✓ 生成成功</div>
        <div style="font-size: 12px; color: #6B7280; word-break: break-all;">{{ outputPath }}</div>
      </div>
    </template>
  </div>
</template>
