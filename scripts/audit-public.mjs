// Read-only privacy gate. Findings contain filenames and categories, never matched secrets.
import { readdir, readFile, stat } from 'node:fs/promises';
import { resolve, relative, extname } from 'node:path';
const root = resolve(process.argv[2] || '.');
const excluded = new Set(['.git', '.cache', 'node_modules', 'dist', 'target', 'binaries', 'release', 'test-output', 'test-fixtures', 'gen']);
const findings = [];
let files = 0;
const machinePaths = [process.cwd(), process.env.USERPROFILE].filter(Boolean)
  .flatMap(p => [p, p.replaceAll('\\', '/')]);
const forbiddenNames = /^(?:\.env(?:\..+)?|id_rsa.*|id_ed25519.*|.*\.(?:pfx|p12|pem|key|dmp|log))$/i;
async function scan(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    if (excluded.has(entry.name) || /^RELEASE_REPORT/.test(entry.name)) continue;
    const path = resolve(dir, entry.name), file = relative(root, path).replaceAll('\\', '/');
    if (entry.isSymbolicLink()) { findings.push({ file, kind: 'symlink-requires-review' }); continue; }
    if (entry.isDirectory()) { await scan(path); continue; }
    if (forbiddenNames.test(entry.name) && entry.name !== '.env.example') findings.push({ file, kind: 'private-or-generated-file' });
    if ((await stat(path)).size > 10_000_000) findings.push({ file, kind: 'large-source-file' });
    const bytes = await readFile(path); files++;
    if (machinePaths.some(p => bytes.includes(Buffer.from(p)) || bytes.includes(Buffer.from(p, 'utf16le')))) findings.push({ file, kind: 'personal-machine-path' });
    if (['.exe','.dll','.pdb','.msi','.zip','.7z'].includes(extname(path))) findings.push({ file, kind: 'binary-in-source' });
    const text = bytes.toString('utf8');
    if (/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/.test(text)) findings.push({ file, kind: 'private-key' });
    if (/\b(?:192\.168\.\d{1,3}\.\d{1,3}|10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])\.\d{1,3}\.\d{1,3})\b/.test(text)) findings.push({ file, kind: 'private-ip-needs-review' });
  }
}
await scan(root);
console.log(JSON.stringify({ files, findings }, null, 2));
if (findings.length) process.exitCode = 1;
