// Generate from an authenticated, freshly extracted engine tree, never a version rename.
import { readdir,readFile,writeFile } from 'node:fs/promises';
import { resolve,relative } from 'node:path';
import { createHash } from 'node:crypto';
const root=resolve(process.argv[2]||'src-tauri/binaries');
const lock=JSON.parse(await readFile('engines.lock.json','utf8'));const result={};
async function walk(dir){for(const entry of await readdir(dir,{withFileTypes:true})){
 const path=resolve(dir,entry.name);if(entry.isSymbolicLink())throw Error('Linked engine file');
 if(entry.isDirectory())await walk(path);else result[relative(root,path).replaceAll('\\','/')]=createHash('sha256').update(await readFile(path)).digest('hex');
}}
for(const entry of Object.values(lock).filter(e=>e.required))await walk(resolve(root,entry.id));
await writeFile('installer/engine-files.json',JSON.stringify(result,null,2)+'\n');
console.log(Object.keys(result).length+' extracted files pinned');
