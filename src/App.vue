<script setup lang="ts">
import { useRoute } from 'vue-router';

const route = useRoute();
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="sidebar-brand">ModTrans</div>
      <nav class="sidebar-nav">
        <router-link to="/translate" class="sidebar-item" :class="{ active: route.path === '/translate' }">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
            <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
          </svg>
          <span>翻译</span>
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
  background: #F7F8FA;
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

/* ---------- 侧边栏 ---------- */
.sidebar {
  width: 100px;
  background: #FFFFFF;
  border-right: 1px solid #EBEDF0;
  display: flex;
  flex-direction: column;
  padding: 10px 8px;
  flex-shrink: 0;
}

.sidebar-brand {
  padding: 6px 12px 16px;
  font-size: 13px;
  font-weight: 600;
  color: #1F2937;
  letter-spacing: 0.3px;
  user-select: none;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sidebar-footer {
  margin-top: auto;
  padding-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sidebar-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border-radius: 6px;
  text-decoration: none;
  color: #6B7280;
  font-size: 13px;
  transition: all 0.15s;
  user-select: none;
}

.sidebar-item svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.sidebar-item:hover {
  background: #F3F4F6;
  color: #374151;
}

.sidebar-item.active {
  background: #E6F4FF;
  color: #1677FF;
  font-weight: 500;
}

.workspace {
  flex: 1;
  background: #F7F8FA;
  overflow-y: auto;
  padding: 28px 32px;
}

/* ---------- 页面结构 ---------- */
.page-head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.page-head h2 {
  font-size: 20px;
  font-weight: 600;
  color: #1F2937;
}

.link-btn {
  margin-left: auto;
  border: none;
  background: none;
  color: #1677FF;
  font-size: 13px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 5px;
  font-family: inherit;
}

.link-btn:hover {
  background: #E6F4FF;
}

.panel {
  background: #FFFFFF;
  border: 1px solid #EBEDF0;
  border-radius: 8px;
  padding: 20px 24px;
  margin-bottom: 16px;
}

.panel-title {
  font-size: 15px;
  font-weight: 600;
  color: #1F2937;
  margin-bottom: 16px;
}

.panel-foot {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 14px;
  margin-top: 14px;
}

.panel-foot-text {
  font-size: 13px;
  color: #6B7280;
}

.panel-foot-text strong {
  color: #1677FF;
  font-weight: 600;
}

.save-row {
  margin-top: 4px;
  display: flex;
  justify-content: flex-end;
}

.drop-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  background: #FFFFFF;
  border: 1.5px dashed #D9DDE3;
  border-radius: 8px;
  padding: 16px 20px;
  cursor: pointer;
  transition: all 0.15s;
  user-select: none;
}

.drop-bar:hover,
.drop-bar.drag-over {
  border-color: #1677FF;
  background: #F0F7FF;
}

.drop-bar svg {
  color: #B7BCC4;
  flex-shrink: 0;
}

.drop-bar-text {
  flex: 1;
  font-size: 13px;
  color: #6B7280;
}

.page-hint {
  font-size: 12px;
  color: #9CA3AF;
  margin-top: 8px;
  margin-bottom: 16px;
}

/* ---------- 行与表单 ---------- */
.field-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  margin-bottom: 10px;
}

.field-row:last-child {
  margin-bottom: 0;
}

.field-label {
  font-size: 12px;
  color: #6B7280;
  white-space: nowrap;
}

.field-value {
  font-size: 13px;
  color: #374151;
}

.field-sep {
  width: 1px;
  height: 18px;
  background: #EBEDF0;
  margin: 0 4px;
}

.mod-check {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: #374151;
  cursor: pointer;
}

.mod-check input[type="checkbox"] {
  accent-color: #1677FF;
}

.toolbar-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.search-wrap {
  position: relative;
  flex: 1;
  min-width: 160px;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: #9CA3AF;
  font-size: 14px;
}

.btn-xs {
  padding: 3px 10px;
  font-size: 11px;
}

.warn-box {
  margin-top: 10px;
  padding: 8px 12px;
  background: #FFFBE6;
  color: #D48806;
  border-radius: 6px;
  font-size: 12px;
}

.error-box {
  margin-top: 12px;
  padding: 10px 14px;
  background: #FFF1F0;
  border: 1px solid #FFCCC7;
  border-radius: 6px;
  font-size: 12px;
  color: #FF4D4F;
  white-space: pre-wrap;
}

.success-card {
  margin-top: 16px;
  padding: 14px 18px;
  background: #F6FFED;
  border: 1px solid #B7EB8F;
  border-radius: 8px;
}

/* ---------- 进度与统计 ---------- */
.progress-num {
  font-size: 26px;
  font-weight: 700;
  color: #1677FF;
  margin-top: 4px;
}

.stats-row {
  display: flex;
  gap: 24px;
  margin-top: 12px;
}

.stat-label {
  font-size: 11px;
  color: #9CA3AF;
}

.stat-value {
  font-size: 15px;
  font-weight: 600;
}

.ctrl-row {
  display: flex;
  gap: 10px;
  margin-top: 16px;
  flex-wrap: wrap;
}

.log-box {
  margin-top: 14px;
  max-height: 100px;
  overflow: auto;
  background: #F7F8FA;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 11px;
  font-family: monospace;
}

.current-pair {
  margin-top: 14px;
  background: #F7F8FA;
  border-radius: 8px;
  padding: 12px 16px;
}

.pair-row {
  display: flex;
  gap: 10px;
  padding: 3px 0;
  font-size: 13px;
  color: #374151;
  align-items: baseline;
}

.pair-label {
  font-size: 11px;
  color: #9CA3AF;
  flex-shrink: 0;
  width: 56px;
}

.pair-trans {
  color: #1677FF;
  font-weight: 500;
}

/* ---------- 文件列表 ---------- */
.file-list {
  max-height: 200px;
  overflow: auto;
}

.file-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid #F3F4F6;
}

.file-row:last-child {
  border-bottom: none;
}

.file-name {
  font-size: 13px;
  font-weight: 500;
}

.file-type {
  font-size: 11px;
  color: #9CA3AF;
}

/* ---------- 校正对照表 ---------- */
.src-cell {
  vertical-align: top;
}

.src-text {
  font-size: 13px;
  color: #374151;
  word-break: break-word;
}

.src-key {
  font-size: 11px;
  color: #B7BCC4;
  font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace;
  margin-top: 2px;
}

.zh-cell {
  cursor: pointer;
}

.zh-text {
  font-size: 13px;
  color: #1F2937;
  word-break: break-word;
}

.zh-empty {
  color: #B7BCC4;
  font-style: italic;
  font-size: 12px;
}

.zh-tag {
  display: inline-block;
  margin-left: 6px;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 10px;
  background: #EDE9FE;
  color: #7C3AED;
}

/* ---------- Shared styles ---------- */
.card {
  background: #FFFFFF;
  border: 1px solid #EBEDF0;
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
  background: #1677FF;
  color: #FFFFFF;
}

.btn-primary:hover:not(:disabled) {
  background: #0958D9;
}

.btn-secondary {
  background: #F3F4F6;
  color: #374151;
  border: 1px solid #D9DDE3;
}

.btn-secondary:hover:not(:disabled) {
  background: #E5E7EB;
}

.btn-success {
  background: #52C41A;
  color: #FFFFFF;
}

.btn-success:hover:not(:disabled) {
  background: #389E0D;
}

.btn-danger {
  background: #FF4D4F;
  color: #FFFFFF;
}

.btn-danger:hover:not(:disabled) {
  background: #D9363E;
}

.btn-warning {
  background: #FA8C16;
  color: #FFFFFF;
}

.btn-warning:hover:not(:disabled) {
  background: #D46B08;
}

.btn-lg {
  padding: 10px 26px;
  font-size: 14px;
}

.input {
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid #D9DDE3;
  background: #FFFFFF;
  color: #1F2937;
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s;
  font-family: inherit;
}

.input:focus {
  border-color: #1677FF;
  box-shadow: 0 0 0 3px rgba(22, 119, 255, 0.12);
}

.input::placeholder {
  color: #B7BCC4;
}

.select {
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid #D9DDE3;
  background: #FFFFFF;
  color: #1F2937;
  font-size: 13px;
  outline: none;
  cursor: pointer;
  font-family: inherit;
}

.select:focus {
  border-color: #1677FF;
}

.textarea {
  width: 100%;
  padding: 10px 12px;
  border-radius: 6px;
  border: 1px solid #D9DDE3;
  background: #FFFFFF;
  color: #1F2937;
  font-size: 13px;
  outline: none;
  resize: vertical;
  min-height: 80px;
  font-family: inherit;
}

.textarea:focus {
  border-color: #1677FF;
  box-shadow: 0 0 0 3px rgba(22, 119, 255, 0.12);
}

.progress-bar {
  width: 100%;
  height: 6px;
  background: #F0F1F3;
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #1677FF;
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
  background: #FAFAFA;
  color: #6B7280;
  font-weight: 500;
  border-bottom: 1px solid #EBEDF0;
  position: sticky;
  top: 0;
  font-size: 12px;
  letter-spacing: 0.3px;
}

.table td {
  padding: 10px 14px;
  border-bottom: 1px solid #F3F4F6;
  color: #374151;
}

.table tr:hover td {
  background: #FAFAFA;
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
  background: #F6FFED;
  color: #52C41A;
}

.tag-ai {
  background: #E6F4FF;
  color: #1677FF;
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

.collapse-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: #F7F8FA;
  border: 1px solid #EBEDF0;
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
  border-bottom: 2px solid #EBEDF0;
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
  color: #1677FF;
  border-bottom-color: #1677FF;
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
  accent-color: #1677FF;
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
