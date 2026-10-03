# Matriz de formatos

Gerado por `node scripts/generate-registry.mjs`. Disponibilidade depende do health check dos motores. Lossless significa ausência de nova perda no conteúdo decodificado; metadados podem variar. PDF → PNG/JPG gera uma pasta de páginas. GIF e imagens multipágina podem exigir tratamento específico; não se promete preservar animação em formatos estáticos.

| Entrada | Saídas | Motor | Layout |
|---|---|---|---|
| png | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| jpg | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| jpeg | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| webp | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| avif | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| heic | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| heif | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| bmp | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| tiff | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| tif | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| gif | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| ico | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| svg | png, jpg, webp, avif, bmp, tiff, gif, ico, pdf | imagemagick | preserved |
| wav | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| flac | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| mp3 | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| aac | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| m4a | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| alac | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| ogg | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| opus | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| aiff | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| wma | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| ac3 | wav, flac, mp3, aac, m4a, ogg, opus, aiff, wma, ac3 | ffmpeg | preserved |
| mp4 | mp4, mov, mkv, avi, webm, mp3, wav, flac, m4a, opus | ffmpeg | preserved |
| mov | mp4, mov, mkv, avi, webm, mp3, wav, flac, m4a, opus | ffmpeg | preserved |
| mkv | mp4, mov, mkv, avi, webm, mp3, wav, flac, m4a, opus | ffmpeg | preserved |
| avi | mp4, mov, mkv, avi, webm, mp3, wav, flac, m4a, opus | ffmpeg | preserved |
| webm | mp4, mov, mkv, avi, webm, mp3, wav, flac, m4a, opus | ffmpeg | preserved |
| m4v | mp4, mov, mkv, avi, webm, mp3, wav, flac, m4a, opus | ffmpeg | preserved |
| ts | mp4, mov, mkv, avi, webm, mp3, wav, flac, m4a, opus | ffmpeg | preserved |
| mts | mp4, mov, mkv, avi, webm, mp3, wav, flac, m4a, opus | ffmpeg | preserved |
| wmv | mp4, mov, mkv, avi, webm, mp3, wav, flac, m4a, opus | ffmpeg | preserved |
| flv | mp4, mov, mkv, avi, webm, mp3, wav, flac, m4a, opus | ffmpeg | preserved |
| doc | docx, odt, rtf, txt, html, pdf | libreoffice | may-change |
| docx | docx, odt, rtf, txt, html, pdf | libreoffice | may-change |
| odt | docx, odt, rtf, txt, html, pdf | libreoffice | may-change |
| rtf | docx, odt, rtf, txt, html, pdf | libreoffice | may-change |
| xls | xlsx, ods, csv, pdf | libreoffice | may-change |
| xlsx | xlsx, ods, csv, pdf | libreoffice | may-change |
| ods | xlsx, ods, csv, pdf | libreoffice | may-change |
| csv | xlsx, ods, csv, pdf | libreoffice | may-change |
| ppt | pptx, odp, pdf | libreoffice | may-change |
| pptx | pptx, odp, pdf | libreoffice | may-change |
| odp | pptx, odp, pdf | libreoffice | may-change |
| md | html, md, docx, odt, txt | pandoc | may-change |
| html | html, md, docx, odt, txt | pandoc | may-change |
| txt | html, md, docx, odt, txt | pandoc | may-change |
| pdf | pdf, png, jpg | pdf | preserved |
| zip | zip, 7z, tar, tar.gz, gz, bz2, xz | 7zip | preserved |
| 7z | zip, 7z, tar, tar.gz, gz, bz2, xz | 7zip | preserved |
| tar | zip, 7z, tar, tar.gz, gz, bz2, xz | 7zip | preserved |
| tar.gz | zip, 7z, tar, tar.gz, gz, bz2, xz | 7zip | preserved |
| gz | zip, 7z, tar, tar.gz, gz, bz2, xz | 7zip | preserved |
| bz2 | zip, 7z, tar, tar.gz, gz, bz2, xz | 7zip | preserved |
| xz | zip, 7z, tar, tar.gz, gz, bz2, xz | 7zip | preserved |
| binary | zip, 7z, tar, tar.gz, gz, bz2, xz | 7zip | preserved |
