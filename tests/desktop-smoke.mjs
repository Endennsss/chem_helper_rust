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
  assert.equal(await execute('return Number(document.querySelector(".library-panel .panel-heading b").textContent)'),386);
  assert.ok(await execute('return document.querySelector("#recipe-category").options.length >= 14'));
  await execute('const select=document.querySelector("#recipe-category"); select.value="Напитки"; select.dispatchEvent(new Event("change",{bubbles:true}))');
  await waitFor('return document.querySelectorAll(".library-list button").length === 138');
  await execute('const select=document.querySelector("#recipe-category"); select.value=""; select.dispatchEvent(new Event("change",{bubbles:true}))');
  await waitFor('return document.querySelectorAll(".library-list button").length === 386');
  await execute('document.querySelector("[data-reaction-id=Fresium]").click()');
  await execute('document.querySelector(".composer-panel .primary-action").click()');
  await waitFor('return document.querySelector(".guide-list")?.textContent.includes("≤ 300K")');
  // Invalid whole-batch input must not poison an existing valid queue.
  await execute('document.querySelector("[data-reaction-id=CreateDough]").click()');
  await execute('const input=document.querySelector("#quantity"); input.value="1.5"; input.dispatchEvent(new Event("input",{bubbles:true}))');
  await execute('document.querySelector(".composer-panel .primary-action").click()');
  await delay(300);
  assert.equal(await execute('return document.querySelectorAll(".queue-row").length'),1,'Fractional batch input must leave valid queue unchanged');
  assert.ok(await execute('return !!document.querySelector(".step-row")'),'Valid calculation must remain visible');
  await execute('const input=document.querySelector("#material"); input.value=""; input.dispatchEvent(new Event("input",{bubbles:true})); const quantity=document.querySelector("#quantity"); quantity.value="30"; quantity.dispatchEvent(new Event("input",{bubbles:true}))');
  await execute('document.querySelector(".command-strip .ghost-action").click()');
  await waitFor('return !document.querySelector(".queue-row")');
  await execute('document.querySelector("[data-reaction-id=BloodBreakdown]").click()');
  await execute('document.querySelector(".composer-panel .primary-action").click()');
  await waitFor('return document.querySelector(".guide-list")?.textContent.includes("Центрифуга") && document.querySelectorAll(".reaction-outputs .ingredient-grid span").length >= 5');
  await execute('document.querySelector("[data-reaction-id=Foam]").click()');
  await waitFor('return document.querySelector("label[for=quantity]").textContent.includes("Число порций")');
  await execute('const input=document.querySelector("#quantity"); input.value="2"; input.dispatchEvent(new Event("input",{bubbles:true}))');
  await execute('document.querySelector(".composer-panel .primary-action").click()');
  await waitFor('return document.querySelector(".queue-row")?.textContent.includes("2реакц.") && !!document.querySelector(".step-row")');
  assert.ok(await execute('return document.querySelector(".command-strip").textContent.includes("30u + 2 порц.")'),'Volume and batches must be counted separately');
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
  await waitFor('return document.querySelectorAll(".queue-row").length === 2 && !!document.querySelector(".step-row")');
  console.log('Native desktop: full 386-reaction offline catalog, 13 categories, cold synthesis, multi-output/equipment, batch units, favorite restart and screenshot OK');
} finally { await quit(); }
