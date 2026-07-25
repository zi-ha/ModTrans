<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { save } from '@tauri-apps/plugin-dialog';

const packPaths = ref<{ path: string; name: string }[]>([]);
const outputName = ref('Minecraft汉化合集');
const mcVersion = ref('1.20');
const merging = ref(false);
const outputPath = ref('');
const isDragging = ref(false);

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
    <h2 style="font-size: 18px; font-weight: 600; margin-bottom: 20px; color: #1F2937;">资源包合并</h2>

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

    <div v-if="packPaths.length > 0" class="card" style="margin-top: 16px;">
      <div class="card-title" style="font-size: 13px;">
        已选择 {{ packPaths.length }} 个资源包
      </div>
      <div v-for="(p, i) in packPaths" :key="i" style="display: flex; align-items: center; gap: 10px; padding: 8px 0; border-bottom: 1px solid #F3F4F6;">
        <span style="font-size: 16px; color: #2563EB;">+</span>
        <span style="flex: 1; font-size: 13px;">{{ p.name }}</span>
        <button class="btn btn-secondary" style="padding: 3px 10px; font-size: 11px;" @click="removePack(i)">移除</button>
      </div>

      <div style="margin-top: 16px; display: flex; gap: 12px; align-items: flex-end;">
        <div class="form-col">
          <label class="form-label">输出文件名</label>
          <input v-model="outputName" class="input" />
        </div>
        <div class="form-col" style="max-width: 140px;">
          <label class="form-label">MC 版本</label>
          <select v-model="mcVersion" class="select">
            <option value="1.12">1.12.x</option>
            <option value="1.16">1.16.x</option>
            <option value="1.18">1.18.x</option>
            <option value="1.19">1.19.x</option>
            <option value="1.20">1.20.x</option>
            <option value="1.21">1.21</option>
          </select>
        </div>
      </div>

      <div style="text-align: right; margin-top: 16px;">
        <button class="btn btn-primary btn-lg" @click="doMerge" :disabled="merging || packPaths.length < 2">
          {{ merging ? '合并中...' : '开始合并' }}
        </button>
      </div>
    </div>

    <div v-if="outputPath" class="card" style="border-color: #16A34A; background: #F0FDF4; margin-top: 16px;">
      <div style="font-size: 13px; font-weight: 600; color: #16A34A; margin-bottom: 6px;">✓ 合并成功</div>
      <div style="font-size: 12px; color: #6B7280; word-break: break-all;">{{ outputPath }}</div>
    </div>
  </div>
</template>
