<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '../stores/app';
import type { ModLangFile } from '../types';

const router = useRouter();
const store = useAppStore();

const isDragging = ref(false);
const loading = ref(false);
const selectedFiles = ref<{ path: string; name: string; type: string }[]>([]);
const extractError = ref('');

watch(() => store.currentStep, (step) => {
  if (step === 'translate') router.push('/translate');
  else if (step === 'review') router.push('/review');
  else if (step === 'generate') router.push('/generate');
});

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
    const nextPath = store.completeStep('import');
    if (nextPath) router.push(nextPath);
  }
}

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  isDragging.value = true;
}
</script>

<template>
  <div>
    <h2 style="font-size: 18px; font-weight: 600; margin-bottom: 20px; color: #1F2937;">导入 Mod</h2>

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

    <div v-if="selectedFiles.length > 0" class="card" style="margin-top: 16px;">
      <div class="card-title" style="font-size: 13px; margin-bottom: 12px;">
        已选择 {{ selectedFiles.length }} 个文件
      </div>
      <div style="max-height: 200px; overflow: auto;">
        <div v-for="(f, i) in selectedFiles" :key="i"
          style="display: flex; align-items: center; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid #F3F4F6;">
          <div>
            <div style="font-size: 13px; font-weight: 500;">{{ f.name }}</div>
            <div style="font-size: 11px; color: #9CA3AF;">{{ f.type }}</div>
          </div>
          <button class="btn btn-secondary" style="padding: 4px 10px; font-size: 11px;" @click.stop="removeFile(i)">移除</button>
        </div>
      </div>
    </div>

    <div v-if="extractError" style="margin-top: 12px; padding: 10px 14px; background: #FEF2F2; border: 1px solid #FECACA; border-radius: 6px; font-size: 12px; color: #DC2626; white-space: pre-wrap;">
      {{ extractError }}
    </div>

    <div class="bottom-bar" v-if="selectedFiles.length > 0">
      <div class="bottom-bar-info">
        已选择 <strong>{{ selectedFiles.length }}</strong> 个文件
      </div>
      <button class="btn btn-primary btn-lg" @click="startExtract" :disabled="loading">
        {{ loading ? '提取中...' : '开始提取 →' }}
      </button>
    </div>
  </div>
</template>
