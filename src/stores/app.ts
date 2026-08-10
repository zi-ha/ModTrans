import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ModLangFile, ApiConfig, PackMeta, AppSettings, LangEntry } from '../types';

export type StepKey = 'import' | 'translate' | 'review' | 'generate';

export function getNextStep(current: StepKey): StepKey | null {
  const order: StepKey[] = ['import', 'translate', 'review', 'generate'];
  const idx = order.indexOf(current);
  return idx < order.length - 1 ? order[idx + 1] : null;
}

export const useAppStore = defineStore('app', () => {
  const currentFiles = ref<ModLangFile[]>([]);
  const editedEntries = ref<Record<string, LangEntry[]>>({});
  const apiConfigs = ref<ApiConfig[]>([]);
  const activeApiIndex = ref(0);
  const packMeta = ref<PackMeta>({
    name: '汉化资源包',
    author: 'ModTrans',
    description: '由ModTrans生成的汉化资源包',
    version: '1.0',
    mc_version: '1.20',
    universal: false,
  });
  const history = ref<AppSettings['history']>([]);
  const isTranslating = ref(false);
  const translateProgress = ref({ current: 0, total: 0 });
  const reviewedCount = ref(0);

  const steps: StepKey[] = ['import', 'translate', 'review', 'generate'];
  const currentStep = ref<StepKey>('import');
  const stepStatus = ref<Record<StepKey, 'pending' | 'active' | 'completed'>>({
    import: 'active',
    translate: 'pending',
    review: 'pending',
    generate: 'pending',
  });

  const totalEntries = computed(() => {
    let count = 0;
    for (const key of Object.keys(editedEntries.value)) count += editedEntries.value[key].length;
    return count;
  });

  const translatedCount = computed(() => {
    let count = 0;
    for (const key of Object.keys(editedEntries.value)) count += editedEntries.value[key].filter(e => e.translation).length;
    return count;
  });

  const allTranslated = computed(() => totalEntries.value > 0 && translatedCount.value >= totalEntries.value);

  function canAccess(step: StepKey): boolean {
    const idx = steps.indexOf(step);
    for (let i = 0; i < idx; i++) {
      if (stepStatus.value[steps[i]] !== 'completed') return false;
    }
    return true;
  }

  function completeStep(step: StepKey): StepKey | null {
    stepStatus.value[step] = 'completed';
    const next = getNextStep(step);
    if (next) {
      if (stepStatus.value[next] === 'pending') stepStatus.value[next] = 'active';
      currentStep.value = next;
      return next;
    }
    return null;
  }

  function setCurrentStep(step: StepKey) {
    if (canAccess(step)) {
      currentStep.value = step;
      if (stepStatus.value[step] === 'pending') stepStatus.value[step] = 'active';
    }
  }

  function setFiles(files: ModLangFile[]) {
    currentFiles.value = files;
    editedEntries.value = {};
    for (const f of files) {
      editedEntries.value[f.mod_id] = JSON.parse(JSON.stringify(f.entries));
    }
    reviewedCount.value = 0;
    stepStatus.value = { import: 'active', translate: 'pending', review: 'pending', generate: 'pending' };
    currentStep.value = 'import';
  }

  function updateEntry(modId: string, key: string, translation: string) {
    const list = editedEntries.value[modId];
    if (!list) return;
    const entry = list.find(e => e.key === key);
    if (entry) {
      const had = !!entry.translation;
      entry.translation = translation;
      if (!had && translation) reviewedCount.value++;
    }
  }

  function batchReplace(modId: string, from: string, to: string) {
    const list = editedEntries.value[modId];
    if (!list) return;
    for (const entry of list) {
      if (entry.translation.includes(from)) entry.translation = entry.translation.split(from).join(to);
    }
  }

  function hasAnyTranslation(): boolean { return translatedCount.value > 0; }

  return {
    currentFiles, editedEntries, apiConfigs, activeApiIndex,
    packMeta, history, isTranslating, translateProgress,
    steps, currentStep, stepStatus, totalEntries, translatedCount,
    reviewedCount, allTranslated,
    canAccess, completeStep, setCurrentStep, setFiles,
    updateEntry, batchReplace, hasAnyTranslation,
  };
});
