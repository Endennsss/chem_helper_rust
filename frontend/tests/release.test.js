import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, readdir, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { prepareRelease } from '../../scripts/prepare-release.mjs';

test('release checksum filenames survive GitHub upload unchanged', async () => {
  const root = await mkdtemp(join(tmpdir(),'chem-release-'));
  try {
    const input = join(root,'artifacts');
    const output = join(root,'dist');
    await mkdir(join(input,'windows'),{ recursive:true });
    await writeFile(join(input,'windows','Chem Helper Rust_1.0.0_x64-setup.exe'),'hello');
    await prepareRelease(input,output);
    assert.deepEqual((await readdir(output)).sort(),['Chem-Helper-Rust_1.0.0_x64-setup.exe','SHA256SUMS.txt']);
    assert.equal(await readFile(join(output,'SHA256SUMS.txt'),'utf8'),'2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824  Chem-Helper-Rust_1.0.0_x64-setup.exe\n');
  } finally { await rm(root,{ recursive:true }); }
});
test('ambiguous asset names fail instead of overwriting a different binary', async () => {
  const root = await mkdtemp(join(tmpdir(),'chem-release-'));
  try {
    const input = join(root,'artifacts');
    await mkdir(input);
    await writeFile(join(input,'A B.exe'),'first');
    await writeFile(join(input,'A-B.exe'),'second');
    await assert.rejects(prepareRelease(input,join(root,'dist')), /collision/);
  } finally { await rm(root,{ recursive:true }); }
});
