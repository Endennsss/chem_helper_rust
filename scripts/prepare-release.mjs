import { readdir, mkdir, copyFile, readFile, writeFile } from 'node:fs/promises';
import { basename, join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';

async function collect(directory) {
  const result = [];
  for (const entry of await readdir(directory, { withFileTypes:true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) result.push(...await collect(path));
    else if (entry.isFile() && /\.(exe|deb|AppImage)$/.test(entry.name)) result.push(path);
  }
  return result;
}
export async function prepareRelease(input, output) {
  const files = await collect(input);
  if (!files.length) throw new Error('No release binaries found');
  const assets = new Map();
  for (const file of files) {
    const name = basename(file).replace(/[^A-Za-z0-9._-]/g, '-');
    if (assets.has(name)) throw new Error(`Release filename collision: ${name}`);
    assets.set(name,file);
  }
  await mkdir(output, { recursive:true });
  if ((await readdir(output)).length) throw new Error('Release output must be empty');
  const sums = [];
  for (const name of [...assets.keys()].sort()) {
    const target = join(output, name);
    await copyFile(assets.get(name),target);
    const digest = createHash('sha256').update(await readFile(target)).digest('hex');
    sums.push(`${digest}  ${name}\n`);
  }
  await writeFile(join(output,'SHA256SUMS.txt'),sums.join(''));
}
if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  await prepareRelease(process.argv[2] || 'release-assets',process.argv[3] || 'dist');
}
