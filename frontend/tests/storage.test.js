import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createPersistentStorage } from '../src/services/persistent-storage.js';

test('hydrates favorites before reads and preserves them across restart', async () => {
  const folder = await mkdtemp(join(tmpdir(), 'chem-storage-'));
  const path = join(folder, 'state.json');
  await writeFile(path, '{}');
  const native = async (command, args) => {
    if (command === 'load_user_state') return { state: JSON.parse(await readFile(path, 'utf8')) };
    if (command === 'save_user_state') return writeFile(path, JSON.stringify(args.state));
    throw new Error('Unknown native command');
  };
  try {
    const first = createPersistentStorage(native);
    assert.throws(() => first.getItem('favoriteItems'));
    await first.initialize();
    first.setItem('favoriteItems', '[{"name":"Аммиак","quantity":30}]');
    await first.flush();
    const second = createPersistentStorage(native);
    await second.initialize();
    assert.deepEqual(JSON.parse(second.getItem('favoriteItems')), [{ name:'Аммиак', quantity:30 }]);
  } finally { await rm(folder, { recursive:true }); }
});

test('serializes writes so slow old snapshots cannot overwrite newer favorites', async () => {
  let persisted = {};
  let activeWrites = 0;
  const native = async (command, { state } = {}) => {
    if (command === 'load_user_state') return { state: {} };
    assert.equal(++activeWrites, 1);
    await new Promise(resolve => setTimeout(resolve, state.favoriteItems === 'old' ? 15 : 1));
    persisted = state;
    activeWrites--;
  };
  const storage = createPersistentStorage(native);
  await storage.initialize();
  storage.setItem('favoriteItems', 'old');
  storage.setItem('favoriteItems', 'latest');
  storage.setItem('animationsEnabled', 'false');
  await storage.flush();
  assert.deepEqual(persisted, { favoriteItems: 'latest', animationsEnabled:'false' });
});

test('failed native writes are reported and retried before allowing close', async () => {
  let diskUnavailable = true;
  let persisted;
  let reported = '';
  const native = async (command, args) => {
    if (command === 'load_user_state') return { state: {} };
    if (diskUnavailable) throw new Error('disk unavailable');
    persisted = args.state;
  };
  const storage = createPersistentStorage(native, error => { reported = error.message; });
  await storage.initialize();
  storage.setItem('favoriteItems', 'favorite');
  await assert.rejects(storage.flush(), /disk unavailable/);
  assert.equal(reported, 'disk unavailable');
  diskUnavailable = false;
  await storage.flush();
  assert.equal(persisted.favoriteItems, 'favorite');
});

test('closing waits for saves queued while an earlier save is finishing', async () => {
  const releases = [];
  let persisted;
  const storage = createPersistentStorage(async (command, args) => {
    if (command === 'load_user_state') return { state: {} };
    await new Promise(resolve => releases.push(resolve));
    persisted = args.state.favoriteItems;
  });
  await storage.initialize();
  storage.setItem('favoriteItems', 'old');
  await new Promise(resolve => setImmediate(resolve));
  let closed = false;
  const closing = storage.flush().then(() => { closed = true; });
  storage.setItem('favoriteItems', 'new');
  releases.shift()();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(closed, false, 'must not close with the newest save pending');
  releases.shift()();
  await closing;
  assert.equal(persisted, 'new');
});
