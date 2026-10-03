// Developer cache only. The shipped installer uses EngineSetup.Download.
import { createWriteStream, createReadStream, existsSync, mkdirSync, readFileSync, renameSync, rmSync, statSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { pipeline } from 'node:stream/promises';
mkdirSync('.cache/downloads', { recursive: true });
const lock = JSON.parse(readFileSync('engines.lock.json', 'utf8'));
async function valid(path, entry) {
  if (!existsSync(path) || statSync(path).size !== entry.bytes) return false;
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex') === entry.sha256;
}
for (const [name, entry] of Object.entries(lock)) {
  if (entry.id === 'webview2') continue;
  if (!/^[a-z0-9.-]+$/i.test(name) || !entry.url.startsWith('https://')) throw Error('Invalid download manifest');
  const path = '.cache/downloads/' + name, part = path + '.part';
  if (await valid(path, entry)) continue;
  rmSync(part, { force: true });
  try {
    console.log('Downloading', entry.name, entry.version);
    const response = await fetch(entry.url, { signal: AbortSignal.timeout(30 * 60 * 1000) });
    if (!response.ok || !response.url.startsWith('https://')) throw Error('Supplier download failed: ' + response.status);
    await pipeline(response.body, createWriteStream(part, { flags: 'wx' }));
    if (!await valid(part, entry)) throw Error('Size or SHA-256 mismatch: ' + name);
    rmSync(path, { force: true });
    renameSync(part, path);
  } finally { rmSync(part, { force: true }); }
}
