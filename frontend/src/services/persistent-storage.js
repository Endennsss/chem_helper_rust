export function createPersistentStorage(invoke, onError = () => {}) {
  let state = null;
  let pending = Promise.resolve();
  let lastError = null;
  const copy = () => JSON.parse(JSON.stringify(state));
  function enqueue(snapshot) {
    pending = pending.then(() => invoke('save_user_state', { state: snapshot })).then(
      () => { lastError = null; },
      (error) => { lastError = error; onError(error); },
    );
  }
  return {
    async initialize() {
      const result = await invoke('load_user_state');
      state = result.state;
      return result.warning;
    },
    getItem(key) {
      if (state === null) throw new Error('Хранилище ещё не загружено');
      const value = state[key];
      return value == null ? null : typeof value === 'string' ? value : JSON.stringify(value);
    },
    setItem(key, value) {
      if (state === null) throw new Error('Хранилище ещё не загружено');
      state[key] = String(value);
      enqueue(copy());
    },
    async flush() {
      let retried = false;
      for (;;) {
        const batch = pending;
        await batch;
        if (batch !== pending) continue;
        if (lastError) {
          if (retried) throw lastError;
          retried = true;
          enqueue(copy());
          continue;
        }
        return;
      }
    },
  };
}
