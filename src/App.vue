<script setup lang="ts">
import { useRoute } from 'vue-router';

const route = useRoute();
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <nav class="sidebar-nav">
        <router-link to="/workspace" class="sidebar-item" :class="{ active: route.path === '/workspace' }">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="7" height="7" rx="1.5"/>
            <rect x="14" y="3" width="7" height="7" rx="1.5"/>
            <rect x="3" y="14" width="7" height="7" rx="1.5"/>
            <rect x="14" y="14" width="7" height="7" rx="1.5"/>
          </svg>
          <span>工作台</span>
        </router-link>
        <router-link to="/merge" class="sidebar-item" :class="{ active: route.path === '/merge' }">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="12 2 21 7 12 12 3 7 12 2"/>
            <polygon points="12 12 21 17 12 22 3 17 12 12"/>
          </svg>
          <span>合并</span>
        </router-link>
      </nav>
      <div class="sidebar-footer">
        <router-link to="/settings" class="sidebar-item" :class="{ active: route.path === '/settings' }">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
          </svg>
          <span>设置</span>
        </router-link>
      </div>
    </aside>
    <main class="workspace">
      <router-view />
    </main>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background: #FFFFFF;
  color: #1F2937;
  font-size: 14px;
  line-height: 1.5;
}

.app-shell {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

/* 侧边栏 */
.sidebar {
  width: 134px;
  background: #FFFFFF;
  border-right: 1px solid #EAECEF;
  display: flex;
  flex-direction: column;
  padding: 14px 10px;
  flex-shrink: 0;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sidebar-footer {
  margin-top: auto;
  padding-top: 10px;
  border-top: 1px solid #F3F4F6;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sidebar-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-radius: 6px;
  text-decoration: none;
  color: #6B7280;
  font-size: 13px;
  transition: all 0.15s;
  user-select: none;
}

.sidebar-item svg {
  width: 17px;
  height: 17px;
  flex-shrink: 0;
}

.sidebar-item:hover {
  background: #F3F4F6;
  color: #374151;
}

.sidebar-item.active {
  background: #EFF6FF;
  color: #2563EB;
  font-weight: 500;
}

.workspace {
  flex: 1;
  background: #FAFAFA;
  overflow-y: auto;
  padding: 32px 40px;
}

/* Shared styles */
.card {
  background: #FFFFFF;
  border: 1px solid #E5E7EB;
  border-radius: 8px;
  padding: 24px;
  margin-bottom: 16px;
}

.card-title {
  font-size: 15px;
  font-weight: 600;
  color: #1F2937;
  margin-bottom: 16px;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 18px;
  border-radius: 6px;
  border: none;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  font-family: inherit;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: #2563EB;
  color: #FFFFFF;
}

.btn-primary:hover:not(:disabled) {
  background: #1D4ED8;
}

.btn-secondary {
  background: #F3F4F6;
  color: #374151;
  border: 1px solid #D1D5DB;
}

.btn-secondary:hover:not(:disabled) {
  background: #E5E7EB;
}

.btn-success {
  background: #16A34A;
  color: #FFFFFF;
}

.btn-success:hover:not(:disabled) {
  background: #15803D;
}

.btn-danger {
  background: #EF4444;
  color: #FFFFFF;
}

.btn-danger:hover:not(:disabled) {
  background: #DC2626;
}

.btn-warning {
  background: #D97706;
  color: #FFFFFF;
}

.btn-warning:hover:not(:disabled) {
  background: #B45309;
}

.btn-lg {
  padding: 12px 28px;
  font-size: 15px;
}

.input {
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid #D1D5DB;
  background: #FFFFFF;
  color: #1F2937;
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s;
  font-family: inherit;
}

.input:focus {
  border-color: #2563EB;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

.input::placeholder {
  color: #9CA3AF;
}

.select {
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid #D1D5DB;
  background: #FFFFFF;
  color: #1F2937;
  font-size: 13px;
  outline: none;
  cursor: pointer;
  font-family: inherit;
}

.select:focus {
  border-color: #2563EB;
}

.textarea {
  width: 100%;
  padding: 10px 12px;
  border-radius: 6px;
  border: 1px solid #D1D5DB;
  background: #FFFFFF;
  color: #1F2937;
  font-size: 13px;
  outline: none;
  resize: vertical;
  min-height: 80px;
  font-family: inherit;
}

.textarea:focus {
  border-color: #2563EB;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

.progress-bar {
  width: 100%;
  height: 6px;
  background: #E5E7EB;
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #2563EB;
  border-radius: 3px;
  transition: width 0.3s;
}

.table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.table th {
  text-align: left;
  padding: 10px 14px;
  background: #F9FAFB;
  color: #6B7280;
  font-weight: 500;
  border-bottom: 1px solid #E5E7EB;
  position: sticky;
  top: 0;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.table td {
  padding: 9px 14px;
  border-bottom: 1px solid #F3F4F6;
  color: #374151;
}

.table tr:hover td {
  background: #F9FAFB;
}

.tag {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.tag-vanilla {
  background: #EDE9FE;
  color: #7C3AED;
}

.tag-mod {
  background: #D1FAE5;
  color: #059669;
}

.tag-ai {
  background: #DBEAFE;
  color: #2563EB;
}

.drop-zone {
  border: 2px dashed #D1D5DB;
  border-radius: 10px;
  padding: 48px 24px;
  text-align: center;
  transition: all 0.2s;
  background: #FAFAFA;
  cursor: pointer;
}

.drop-zone:hover,
.drop-zone.drag-over {
  border-color: #2563EB;
  background: #EFF6FF;
}

.form-row {
  display: flex;
  gap: 16px;
  margin-bottom: 14px;
}

.form-col {
  flex: 1;
}

.form-label {
  display: block;
  font-size: 12px;
  color: #6B7280;
  margin-bottom: 5px;
  font-weight: 500;
}

.form-hint {
  font-size: 11px;
  color: #9CA3AF;
  margin-top: 3px;
}

.bottom-bar {
  margin-top: 24px;
  padding: 16px 0;
  border-top: 1px solid #E5E7EB;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.bottom-bar-info {
  font-size: 13px;
  color: #6B7280;
}

.bottom-bar-info strong {
  color: #2563EB;
  font-weight: 600;
}

.collapse-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: #F9FAFB;
  border: 1px solid #E5E7EB;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  color: #6B7280;
  margin-bottom: 12px;
  user-select: none;
}

.collapse-header:hover {
  background: #F3F4F6;
}

.collapse-body {
  padding: 0 0 8px;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #9CA3AF;
}

.empty-state-icon {
  font-size: 40px;
  margin-bottom: 12px;
}

.empty-state-text {
  font-size: 14px;
  margin-bottom: 16px;
}

.tab-bar {
  display: flex;
  gap: 0;
  margin-bottom: 16px;
  border-bottom: 2px solid #E5E7EB;
}

.tab-item {
  padding: 8px 18px;
  border: none;
  background: none;
  font-size: 13px;
  color: #6B7280;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -2px;
  transition: all 0.15s;
}

.tab-item:hover {
  color: #374151;
}

.tab-item.active {
  color: #2563EB;
  border-bottom-color: #2563EB;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #374151;
  cursor: pointer;
  padding: 6px 0;
}

.checkbox-label input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: #2563EB;
}

.text-muted {
  color: #9CA3AF;
}

.text-sm {
  font-size: 12px;
}

.font-mono {
  font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace;
}
</style>
