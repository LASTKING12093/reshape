import test from 'node:test';import assert from 'node:assert/strict';import {readFileSync} from 'node:fs';
const rows=JSON.parse(readFileSync('capabilities.json','utf8'));
test('unique inputs and nonempty capabilities',()=>{assert.equal(new Set(rows.map(r=>r.input)).size,rows.length);for(const r of rows)assert.ok(r.outputs.length>0)});
test('no fake vectorization',()=>{for(const r of rows.filter(r=>r.category==='image'))assert.ok(!r.outputs.includes('svg'))});
test('required matrix routes',()=>{for(const [i,o] of [['png','jpg'],['jpg','png'],['png','webp'],['wav','flac'],['wav','mp3'],['flac','wav'],['mp4','webm'],['mp4','mp3'],['docx','pdf'],['md','html'],['zip','7z']])assert.ok(rows.find(r=>r.input===i)?.outputs.includes(o))});
