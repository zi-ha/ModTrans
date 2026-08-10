export type ToastType = 'success' | 'error';

export function showToast(message: string, type: ToastType = 'success') {
  const el = document.createElement('div');
  const bg = type === 'success' ? '#16A34A' : '#DC2626';
  el.style.cssText = `position:fixed;top:20px;right:20px;background:${bg};color:#fff;padding:10px 20px;border-radius:6px;z-index:9999;font-size:14px;box-shadow:0 2px 8px rgba(0,0,0,0.15);`;
  el.textContent = message;
  document.body.appendChild(el);
  setTimeout(() => el.remove(), 2000);
}
