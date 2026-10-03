import { readFile } from 'node:fs/promises';
if (!process.argv[2]) throw new Error('Pass the production executable to inspect.');
const bytes = await readFile(process.argv[2]);
const paths = [process.cwd(), process.env.USERPROFILE].filter(Boolean)
  .flatMap(p => [p, p.replaceAll('\\', '/')]);
const found = paths.some(p => bytes.includes(Buffer.from(p)) || bytes.includes(Buffer.from(p, 'utf16le')));
console.log(JSON.stringify({ personalBuildPathFound: found }));
if (found) process.exitCode = 1;
