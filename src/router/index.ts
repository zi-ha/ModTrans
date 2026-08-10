import { createRouter, createWebHashHistory } from 'vue-router';
import WorkspaceView from '../views/WorkspaceView.vue';
import MergeView from '../views/MergeView.vue';
import SettingsView from '../views/SettingsView.vue';

const routes = [
  { path: '/', redirect: '/workspace' },
  { path: '/workspace', name: 'workspace', component: WorkspaceView },
  { path: '/merge', name: 'merge', component: MergeView },
  { path: '/settings', name: 'settings', component: SettingsView },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
