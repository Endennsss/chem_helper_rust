// Run against tauri-driver on port 4444 with an isolated XDG_DATA_HOME.
// Build with: npm run tauri -- build --debug --no-bundle
import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import { writeFile } from 'node:fs/promises';

const base = process.env.WEBDRIVER_URL || 'http://127.0.0.1:4444';
const application = resolve(process.argv[2] || 'target/debug/chem-helper-rust');
let session;
async function request(method, path, data) {
  const response = await fetch(`${base}${path}`, { method, headers:{'Content-Type':'application/json'}, body: data === undefined ? undefined : JSON.stringify(data) });
  const result = await response.json();
  if (!response.ok || result.value?.error) throw new Error(JSON.stringify(result));
  return result.value;
}
const execute = script => request('POST', `/session/${session}/execute/sync`, { script, args:[] });
const delay = ms => new Promise(resolve => setTimeout(resolve,ms));
async function waitFor(script) {
  const deadline = Date.now()+60000;
  while (Date.now()<deadline) {
    if (await execute(script)) return;
    await delay(100);
  }
  throw new Error('Desktop condition timed out: '+script);
}
async function launch() {
  const result = await request('POST','/session',{ capabilities:{ alwaysMatch:{browserName:'wry','tauri:options':{application}} }});
  session=result.sessionId;
  await waitFor('return !!document.querySelector(".library-list button") && !document.querySelector(".splash-screen")');
}
async function quit() {
  if (session) { await request('DELETE',`/session/${session}`); session=undefined; }
}
try {
  await launch();
  await execute('document.querySelector(".library-list button").click()');
  await execute('document.querySelector(".composer-panel .primary-action").click()');
  await waitFor('return !!document.querySelector(".queue-row") && !!document.querySelector(".step-row")');
  await execute('document.querySelector(".favorite-toggle").click()');
  await waitFor('return document.querySelectorAll(".favorite-row").length === 1');
  const favorite = await execute('return document.querySelector(".favorite-row .row-main").textContent.trim()');
  assert.ok(favorite.length>0);
  const screenshot = await request('GET',`/session/${session}/screenshot`);
  await writeFile(process.env.SMOKE_SCREENSHOT || '/tmp/chem-helper-rust-desktop.png', Buffer.from(screenshot,'base64'));
  // The next session uses the same persistent native app-data directory.
  await quit();
  await launch();
  const restored = await execute('return document.querySelector(".favorite-row .row-main")?.textContent.trim()');
  assert.equal(restored,favorite);
  await waitFor('return document.querySelectorAll(".queue-row").length === 1 && !!document.querySelector(".step-row")');
  console.log('Native desktop: catalog, queue, calculation, favorite, restart persistence and screenshot OK');
} finally { await quit(); }
