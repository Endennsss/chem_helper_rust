import { createApp, nextTick } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { openUrl } from '@tauri-apps/plugin-opener';
import App from './app/App.vue';
import { storage, initializeStorage, flushStorage } from './services/storage';
import './styles/app.css';

async function start() {
  const warning = await initializeStorage();
  document.documentElement.classList.toggle('no-animations', storage.getItem('animationsEnabled') === 'false');
  createApp(App).mount('#app');
  if (warning) window.dispatchEvent(new CustomEvent('storage-error', { detail: warning }));
  document.addEventListener('click', event => {
    const link = event.target.closest('a[href]');
    if (link && /^https?:\/\//.test(link.href)) {
      event.preventDefault();
      openUrl(link.href).catch(error => window.dispatchEvent(new CustomEvent('storage-error', { detail: String(error) })));
    }
  });
  const windowHandle = getCurrentWindow();
  let closing = false;
  await windowHandle.onCloseRequested(async event => {
    event.preventDefault();
    if (closing) return;
    closing = true;
    const root = document.getElementById('app');
    root.inert = true;
    try {
      await nextTick();
      await flushStorage();
      await windowHandle.destroy();
    } catch (error) {
      root.inert = false;
      closing = false;
      window.dispatchEvent(new CustomEvent('storage-error', { detail: `Не удалось сохранить данные. Закрытие отменено: ${error}` }));
    }
  });
}
start().catch(error => {
  const message = document.createElement('p');
  message.textContent = `Не удалось запустить приложение: ${error}. Для разработки используйте npm run tauri dev.`;
  document.getElementById('app').replaceChildren(message);
});
