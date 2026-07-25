import { createRouter, createWebHashHistory } from 'vue-router';
import ImportView from '../views/ImportView.vue';
import TranslateView from '../views/TranslateView.vue';
import ReviewView from '../views/ReviewView.vue';
import GenerateView from '../views/GenerateView.vue';
import MergeView from '../views/MergeView.vue';
import SettingsView from '../views/SettingsView.vue';
import { useAppStore } from '../stores/app';
import type { StepKey } from '../stores/app';

const routes = [
  { path: '/', redirect: '/import' },
  { path: '/import', name: 'import', component: ImportView, meta: { step: 'import' as StepKey } },
  { path: '/translate', name: 'translate', component: TranslateView, meta: { step: 'translate' as StepKey } },
  { path: '/review', name: 'review', component: ReviewView, meta: { step: 'review' as StepKey } },
  { path: '/generate', name: 'generate', component: GenerateView, meta: { step: 'generate' as StepKey } },
  { path: '/merge', name: 'merge', component: MergeView },
  { path: '/settings', name: 'settings', component: SettingsView },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

router.beforeEach((to, _from, next) => {
  const step = to.meta?.step as StepKey | undefined;
  if (!step) {
    next();
    return;
  }

  const store = useAppStore();
  if (store.canAccess(step)) {
    store.setCurrentStep(step);
    next();
  } else {
    // Redirect to current step
    const currentPath = `/${store.currentStep}`;
    if (to.path !== currentPath) {
      next(currentPath);
    } else {
      next();
    }
  }
});

export default router;
