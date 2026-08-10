<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open, save } from '@tauri-apps/plugin-dialog';

const packPaths = ref<{ path: string; name: string }[]>([]);
const outputName = ref('整合中文包');
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

// 拖放:文件路径由 Tauri onDragDropEvent 提供(HTML5 dataTransfer 拿不到路径)
function handleDroppedPaths(paths: string[]) {
  for (const path of paths) {
    if (!path.endsWith('.zip')) continue;
    const name = path.split('\\').pop() || path.split('/').pop() || path;
    if (!packPaths.value.find(x => x.path === path)) {
      packPaths.value.push({ path, name });
    }
  }
}

let unlistenDrag: (() => void) | null = null;

onMounted(async () => {
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
    <div class="page-head">
      <h2>合并资源包</h2>
    </div>

    <div
      class="drop-bar"
      :class="{ 'drag-over': isDragging }"
      @dragover.prevent
      @drop.prevent
      @click="browsePacks"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
        <polyline points="17 8 12 3 7 8"/>
        <line x1="12" y1="3" x2="12" y2="15"/>
      </svg>
      <span class="drop-bar-text">拖入多个汉化资源包 (.zip)</span>
      <button class="btn btn-secondary" @click.stop="browsePacks">选择文件</button>
    </div>
    <div class="page-hint">重复的语言文件会自动合并覆盖</div>

    <div v-if="packPaths.length > 0" class="panel">
      <div class="file-list">
        <div v-for="(p, i) in packPaths" :key="i" class="file-row">
          <div style="display: flex; align-items: center; gap: 8px; min-width: 0;">
            <span style="font-size: 15px; color: #1677FF; flex-shrink: 0;">+</span>
            <span class="file-name" style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{{ p.name }}</span>
          </div>
          <button class="btn btn-secondary btn-xs" @click="removePack(i)">移除</button>
        </div>
      </div>

      <div class="field-row" style="margin-top: 14px;">
        <span class="field-label">输出名称</span>
        <input v-model="outputName" class="input" style="max-width: 220px;" />
        <span class="field-label" style="margin-left: 8px;">MC 版本</span>
        <select v-model="mcVersion" class="select" style="max-width: 130px;">
          <option v-for="v in mcVersions" :key="v" :value="v">{{ v }}</option>
        </select>
      </div>

      <div class="panel-foot">
        <span class="panel-foot-text">
          {{ packPaths.length < 2 ? `还需 ${2 - packPaths.length} 个资源包（至少 2 个）` : `共 ${packPaths.length} 个资源包` }}
        </span>
        <button class="btn btn-primary" @click="doMerge" :disabled="merging || packPaths.length < 2">
          {{ merging ? '合并中...' : '合并' }}
        </button>
      </div>
    </div>

    <div v-if="outputPath" class="success-card">
      <div style="font-size: 13px; font-weight: 600; color: #52C41A; margin-bottom: 4px;">✓ 合并成功</div>
      <div style="font-size: 12px; color: #6B7280; word-break: break-all;">{{ outputPath }}</div>
    </div>
  </div>
</template>
