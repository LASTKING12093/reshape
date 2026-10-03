import {mkdirSync,writeFileSync,copyFileSync,rmSync} from 'node:fs';import {spawnSync}from'node:child_process';import{resolve,dirname}from'node:path';
mkdirSync('test-fixtures',{recursive:true});
const root=resolve('src-tauri/binaries');
function run(exe,args){const r=spawnSync(resolve(root,exe),args,{encoding:'utf8',windowsHide:true});if(r.status!==0)throw Error(r.stderr||r.stdout||r.error);}
run('imagemagick/magick.exe',['-size','96x64','gradient:#7660ce-#eed8bd','-depth','8','test-fixtures/imagem ação 日本.png']);
run('imagemagick/magick.exe',['-size','96x64','gradient:#7660ce-#eed8bd','-depth','16','test-fixtures/16bit.png']);
run('imagemagick/magick.exe',['test-fixtures/imagem ação 日本.png','test-fixtures/photo.jpg']);
run('ffmpeg/ffmpeg.exe',['-y','-f','lavfi','-i','sine=frequency=440:duration=2','-c:a','pcm_s24le','test-fixtures/som ação 日本.wav']);
run('ffmpeg/ffmpeg.exe',['-y','-i','test-fixtures/som ação 日本.wav','test-fixtures/sound.flac']);
run('ffmpeg/ffmpeg.exe',['-y','-f','lavfi','-i','testsrc2=size=160x120:rate=15:duration=2','-f','lavfi','-i','sine=frequency=440:duration=2','-c:v','libx264','-pix_fmt','yuv420p','-c:a','aac','-shortest','test-fixtures/video.mp4']);
writeFileSync('test-fixtures/document.md','# Forma\n\nDocumento de teste original. Ação, coração, 日本.\n\n| Formato | Estado |\n|---|---|\n| PDF | Local |\n');
run('pandoc/pandoc.exe',['test-fixtures/document.md','-o','test-fixtures/document.docx']);
for(const format of ['odt','rtf','pptx'])run('pandoc/pandoc.exe',['test-fixtures/document.md','-o','test-fixtures/document.'+format]);
const sheetRoot=resolve('test-fixtures/xlsx-source');
const sheetFiles={
 '[Content_Types].xml':'<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>',
 '_rels/.rels':'<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>',
 'xl/workbook.xml':'<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Example" sheetId="1" r:id="rId1"/></sheets></workbook>',
 'xl/_rels/workbook.xml.rels':'<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>',
 'xl/worksheets/sheet1.xml':'<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>Reshape fixture</t></is></c><c r="B1"><v>42</v></c></row></sheetData></worksheet>'
};
for(const [name,xml]of Object.entries(sheetFiles)){const path=resolve(sheetRoot,name);mkdirSync(dirname(path),{recursive:true});writeFileSync(path,xml);}
rmSync('test-fixtures/document.xlsx',{force:true});
run('7zip/7z.exe',['a','-tzip',resolve('test-fixtures/document.xlsx'),sheetRoot+'/*']);
rmSync(sheetRoot,{recursive:true,force:true});
writeFileSync('test-fixtures/tiny.txt','a');writeFileSync('test-fixtures/invalid.png','not an image');
copyFileSync('test-fixtures/imagem ação 日本.png','test-fixtures/misnamed.dat');
run('7zip/7z.exe',['a','-tzip',resolve('test-fixtures/archive.zip'),resolve('test-fixtures/tiny.txt'),resolve('test-fixtures/document.md')]);
writeFileSync('test-fixtures/LICENSE.txt','These fixtures were generated specifically for Forma. CC0-1.0. No third-party content.');
console.log('Fixtures ready.');
run('ffmpeg/ffmpeg.exe',['-y','-f','lavfi','-i','anoisesrc=color=pink:duration=12:sample_rate=44100:seed=42','-c:a','flac','-sample_fmt','s32','test-fixtures/compress-1.4mb.flac']);
