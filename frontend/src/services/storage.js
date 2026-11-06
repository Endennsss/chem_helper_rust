import { invoke } from '@tauri-apps/api/core';
import { createPersistentStorage } from './persistent-storage';
export const storage = createPersistentStorage(invoke, error => {
  window.dispatchEvent(new CustomEvent('storage-error', { detail: String(error) }));
});
export const initializeStorage = () => storage.initialize();
export const flushStorage = () => storage.flush();
