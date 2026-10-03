import fs from 'node:fs';import path from 'node:path';
const meta=JSON.parse(fs.readFileSync('.cache/cargo-metadata.json'));const missing=JSON.parse(fs.readFileSync('.cache/missing-license-files.json'));let results=[];
for(const p of meta.packages.filter(p=>p.name!=='selectors' && missing.some(s=>s.startsWith(p.name+' ')))){
 const vcs=JSON.parse(fs.readFileSync(path.join(path.dirname(p.manifest_path),'.cargo_vcs_info.json')));let found=0;
 for(const name of ['LICENSE','LICENSE-MIT','LICENSE-APACHE','LICENSE.txt','COPYING','LICENSE-MPL-2.0','LICENSE-APACHE-2.0','LICENSE-MIT.txt']){
  const url=p.repository.replace('https://github.com/','https://raw.githubusercontent.com/')+'/'+vcs.git.sha1+'/'+name;
  const r=await fetch(url);if(!r.ok)continue;const text=await r.text();if(!/permission|license|redistribution/i.test(text))continue;
  fs.appendFileSync('src-tauri/legal/DEPENDENCY_LICENSES.txt','\n\n## '+p.name+' '+p.version+' — '+name+'\nSource: '+url+'\n\n'+text);found++;results.push({name:p.name,url});
 }
 if(!found)throw Error('No upstream license found: '+p.name);
}
fs.writeFileSync('.cache/upstream-license-sources.json',JSON.stringify(results,null,2));console.log(results);


const mpl = await (await fetch('https://www.mozilla.org/media/MPL/2.0/index.txt')).text();
if(!mpl.includes('Mozilla Public License'))throw Error('Invalid MPL text');
fs.appendFileSync('src-tauri/legal/DEPENDENCY_LICENSES.txt','\n\n## selectors 0.38.0 — MPL-2.0\nUnmodified source: https://crates.io/crates/selectors/0.38.0\n\n'+mpl);
