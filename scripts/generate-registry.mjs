import {writeFileSync} from 'node:fs';
const rows=[];
function add(inputs,outputs,engine,category,lossless=[],target=false,metadata=true,layout='preserved') {
 for(const input of inputs.split(' ')) rows.push({input,outputs:outputs.split(' '),engine,category,losslessOutputs:lossless,targetSize:target,metadata,layout,advanced:category==='image'?['quality','resize','dpi','alpha','metadata']:category==='audio'?['bitrate','sampleRate','channels','bitDepth','normalize','metadata']:category==='video'?['codec','crf','bitrate','resolution','fps','subtitles','chapters','metadata']:category==='archive'?['level','password']:[]});
}
add('png jpg jpeg webp avif heic heif bmp tiff tif gif ico svg','png jpg webp avif bmp tiff gif ico pdf','imagemagick','image',['png','webp','bmp','tiff'],true);
add('wav flac mp3 aac m4a alac ogg opus aiff wma ac3','wav flac mp3 aac m4a ogg opus aiff wma ac3','ffmpeg','audio',['wav','flac','aiff'],true);
add('mp4 mov mkv avi webm m4v ts mts wmv flv','mp4 mov mkv avi webm mp3 wav flac m4a opus','ffmpeg','video',[],true);
add('doc docx odt rtf','docx odt rtf txt html pdf','libreoffice','document',[],false,true,'may-change');
add('xls xlsx ods csv','xlsx ods csv pdf','libreoffice','spreadsheet',[],false,true,'may-change');
add('ppt pptx odp','pptx odp pdf','libreoffice','presentation',[],false,true,'may-change');
add('md html txt','html md docx odt txt','pandoc','text',[],false,false,'may-change');
add('pdf','pdf png jpg','pdf','pdf',['pdf'],false,true);
add('zip 7z tar tar.gz gz bz2 xz','zip 7z tar tar.gz gz bz2 xz','7zip','archive',['zip','7z','tar','tar.gz','gz','bz2','xz'],false,false);
add('binary','zip 7z tar tar.gz gz bz2 xz','7zip','file',['zip','7z','tar','tar.gz','gz','bz2','xz'],false,false);
for(const row of rows){row.archiveOutputs=['zip','7z','tar','tar.gz','gz','bz2','xz'];}
writeFileSync('capabilities.json',JSON.stringify(rows,null,2)+'\n');
writeFileSync('SUPPORTED_FORMATS.md','# Matriz de formatos\n\nGerado por `node scripts/generate-registry.mjs`. Disponibilidade depende do health check dos motores. Lossless significa ausência de nova perda no conteúdo decodificado; metadados podem variar. PDF → PNG/JPG gera uma pasta de páginas. GIF e imagens multipágina podem exigir tratamento específico; não se promete preservar animação em formatos estáticos.\n\n| Entrada | Saídas | Motor | Layout |\n|---|---|---|---|\n'+rows.map(r=>`| ${r.input} | ${r.outputs.join(', ')} | ${r.engine} | ${r.layout} |`).join('\n')+'\n');
