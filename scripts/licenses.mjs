import{mkdirSync,writeFileSync,readFileSync,copyFileSync,existsSync}from'node:fs';
mkdirSync('src-tauri/binaries/licenses',{recursive:true});
const notices={
 'FFmpeg-GPLv3.txt':'https://raw.githubusercontent.com/FFmpeg/FFmpeg/n8.1/COPYING.GPLv3',
 'Poppler-COPYING.txt':'https://raw.githubusercontent.com/tsdgeos/poppler_mirror/master/COPYING',
 'QPDF-LICENSE.txt':'https://raw.githubusercontent.com/qpdf/qpdf/v12.4.2/LICENSE.txt'
};
for(const [name,url]of Object.entries(notices)){const r=await fetch(url);if(!r.ok)throw Error(`${r.status} ${url}`);writeFileSync('src-tauri/binaries/licenses/'+name,await r.text())}
for(const [src,dst] of [['imagemagick/License.txt','ImageMagick.txt'],['imagemagick/NOTICE.txt','ImageMagick-NOTICE.txt'],['7zip/License.txt','7-Zip.txt'],['pandoc/COPYRIGHT.txt','Pandoc.txt'],['libreoffice/license.txt','LibreOffice.txt'],['libreoffice/NOTICE','LibreOffice-NOTICE.txt']])if(existsSync('src-tauri/binaries/'+src))copyFileSync('src-tauri/binaries/'+src,'src-tauri/binaries/licenses/'+dst);
copyFileSync('THIRD_PARTY_LICENSES.md','src-tauri/binaries/licenses/THIRD_PARTY_LICENSES.md');
