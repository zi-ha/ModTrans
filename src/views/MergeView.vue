<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

const packPaths = ref<{ path: string; name: string }[]>([]);
const outputName = ref('Minecraft汉化合集');
const mcVersion = ref('1.20');
const merging = ref(false);
const outputPath = ref('');
const isDragging = ref(false);

const mcVersions = ['1.12', '1.12.2', '1.16', '1.16.5', '1.18', '1.18.2', '1.19', '1.19.4', '1.20', '1.20.1', '1.21'];

async function browsePacks() {
  const selected = await open({
    multiple: true,
    filters: [{ name: 'ZIP 资源包', extensions: ['zip'] }],
  });
  if (selected) {
    for (const p of selected) {
      const name = p.split('\\').pop() || p.split('/').pop() || p;
      if (!packPaths.value.find(x => x.path === p)) {
        packPaths.value.push({ path: p, name });
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
    if (f.name.endsWith('.zip') && !packPaths.value.find(x => x.path === f.path)) {
      packPaths.value.push({ path: f.path, name: f.name });
    }
  }
}

function removePack(index: number) {
  packPaths.value.splice(index, 1);
}

async function doMerge() {
  if (packPaths.value.length < 2) return;
  merging.value = true;
  outputPath.value = '';

  try {
    const savePath = await save({
      defaultPath: `${outputName.value}.zip`,
      filters: [{ name: 'ZIP 压缩包', extensions: ['zip'] }],
    });

    if (!savePath) {
      merging.value = false;
      return;
    }

    const result: string = await invoke('merge_packs', {
      packPaths: packPaths.value.map(p => p.path),
      outputDir: savePath.substring(0, savePath.lastIndexOf('\\')),
      meta: {
        name: outputName.value,
        author: 'ModTrans',
        description: '多个汉化包合并',
        version: '1.0',
        mc_version: mcVersion.value,
        universal: false,
      },
    });

    outputPath.value = result;
  } catch (e: any) {
    alert(`合并失败: ${e}`);
  }
  merging.value = false;
}
</script>

<template>
  <div>
    <div
      class="drop-zone"
      :class="{ 'drag-over': isDragging }"
      style="padding: 40px;"
      @drop="handleDrop"
      @dragover="(e) => { e.preventDefault(); isDragging = true; }"
      @dragleave="() => isDragging = false"
      @click="browsePacks"
    >
      <div style="color: #D1D5DB; margin-bottom: 8px;">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="#D1D5DB" stroke-width="1.5" stroke-linecap="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="17 8 12 3 7 8"/>
          <line x1="12" y1="3" x2="12" y2="15"/>
        </svg>
      </div>
      <div style="font-size: 14px; color: #6B7280;">拖入多个汉化资源包 (.zip)</div>
      <div style="font-size: 12px; color: #9CA3AF; margin-top: 4px;">或点击浏览选择</div>
    </div>

    <div v-if="packPaths.length > 0" class="pack-list-card">
      <div class="pack-list-head">已选择 {{ packPaths.length }} 个资源包</div>
      <div class="pack-list">
        <div v-for="(p, i) in packPaths" :key="i" class="pack-row">
          <span style="font-size: 16px; color: #2563EB;">+</span>
          <span class="pack-name">{{ p.name }}</span>
          <button class="btn btn-secondary btn-xs" @click="removePack(i)">移除</button>
        </div>
      </div>
      <div class="pack-list-foot">
        <div class="form-col" style="flex: 1; min-width: 200px;">
          <label class="form-label">输出文件名</label>
          <input v-model="outputName" class="input" />
        </div>
        <div class="form-col" style="max-width: 150px;">
          <label class="form-label">MC 版本</label>
          <select v-model="mcVersion" class="select">
            <option v-for="v in mcVersions" :key="v" :value="v">{{ v }}</option>
          </select>
        </div>
        <button class="btn btn-primary btn-lg" @click="doMerge" :disabled="merging || packPaths.length < 2" style="flex-shrink: 0;">
          {{ merging ? '合并中...' : '开始合并' }}
        </button>
      </div>
      <div v-if="packPaths.length < 2" class="merge-hint">请再添加 {{ 2 - packPaths.length }} 个资源包（至少 2 个）才能合并</div>
    </div>

    <div v-if="outputPath" class="card success-card">
      <div style="font-size: 13px; font-weight: 600; color: #16A34A; margin-bottom: 6px;">✓ 合并成功</div>
      <div style="font-size: 12px; color: #6B7280; word-break: break-all;">{{ outputPath }}</div>
    </div>
  </div>
</template>

<style scoped>
.pack-list-card {
  margin-top: 16px;
  border: 1px solid #E5E7EB;
  border-radius: 8px;
  overflow: hidden;
}

.pack-list-head {
  padding: 12px 16px;
  font-size: 13px;
  font-weight: 600;
  color: #1F2937;
  border-bottom: 1px solid #F3F4F6;
}

.pack-list {
  max-height: 220px;
  overflow: auto;
  padding: 0 16px;
}

.pack-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 0;
  border-bottom: 1px solid #F3F4F6;
}

.pack-row:last-child {
  border-bottom: none;
}

.pack-name {
  flex: 1;
  font-size: 13px;
}

.pack-list-foot {
  display: flex;
  gap: 12px;
  align-items: flex-end;
  flex-wrap: wrap;
  padding: 14px 16px;
  border-top: 1px solid #E5E7EB;
}

.btn-xs {
  padding: 3px 10px;
  font-size: 11px;
}

.merge-hint {
  padding: 8px 16px;
  font-size: 12px;
  color: #D97706;
  background: #FFFBEB;
  border-top: 1px solid #F3F4F6;
}

.success-card {
  margin-top: 16px;
  border-color: #16A34A;
  background: #F0FDF4;
}
</style>
