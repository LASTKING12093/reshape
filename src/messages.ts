// Native diagnostics stay untouched; product messages follow the selected language.
const english: Record<string, string> = {
  "Compressão com perdas. O arquivo original é preservado.":
    "Lossy compression. Your original file is preserved.",
  "Para comprimir, escolha um tamanho alvo menor que o original.":
    "To compress, choose a target size smaller than the original.",
  "Este vídeo já tem uma taxa muito baixa para esse tamanho. Escolha Melhor qualidade ou um alvo maior, abaixo do original.":
    "This video already has a very low bitrate for that size. Choose Higher quality or a larger target below the original size.",
  "Prévia indisponível entre volumes para arquivos maiores que 256 MB.":
    "Preview is unavailable across drives for files larger than 256 MB.",
  "Somente a primeira faixa de áudio será extraída; vídeo e demais faixas não entram no resultado.":
    "Only the first audio track will be extracted. Video and other tracks are not included.",
  "MP3 aceita até dois canais. Em Avançado, escolha 2 canais ou use FLAC para preservar o áudio multicanal.":
    "MP3 supports up to two channels. Choose 2 channels in Advanced, or use FLAC to keep multichannel audio.",
  "MP3: escolha bitrate de 32 a 320 kbps.":
    "MP3: choose a bitrate between 32 and 320 kbps.",
  "Remux incompatível. Use MKV sem redimensionar ou desative a cópia de fluxos.":
    "Incompatible remux. Use MKV without resizing, or turn off stream copy.",
  "Este vídeo é HDR. Use Converter → MKV → Copiar fluxos para preservar HDR. A compressão HDR ainda não está disponível.":
    "This video is HDR. Use Convert → MKV → Copy streams to preserve HDR. HDR compression is not available yet.",
  "As legendas serão removidas. Ative Preservar legendas em Avançado.":
    "Subtitles will be removed. Turn on Preserve subtitles in Advanced to keep them.",
  "Legendas gráficas exigem saída MKV para preservação.":
    "Use MKV to preserve image-based subtitles.",
  "Vídeo e áudio serão recodificados; HDR exige cópia de fluxos. A taxa de quadros é mantida quando não definida.":
    "Video and audio will be re-encoded. HDR requires stream copy. Frame rate is preserved unless changed.",
  "Imagem animada ou multipágina: escolha GIF, WebP, TIFF ou PDF para preservar todos os quadros.":
    "Animated or multipage image: choose GIF, WebP, TIFF or PDF to preserve every frame.",
  "JPEG remove transparência usando a cor de fundo escolhida.":
    "JPEG replaces transparency with your chosen background color.",
  "Áudio com perdas. Volume permanece igual, salvo normalização explícita.":
    "Lossy audio. Loudness stays unchanged unless you enable normalization.",
  "Otimiza estrutura e imagens, preservando texto e vetores. PDFs já otimizados podem não diminuir.":
    "Optimizes structure and images while keeping text and vectors. Already optimized PDFs may not shrink.",
  "Duração indisponível": "Duration unavailable",
  "Tamanho alvo fora da faixa MP3 (32–320 kbps). Ajuste o tamanho ou escolha outro formato.":
    "Target size is outside the MP3 range (32–320 kbps). Adjust it or choose another format.",
  "Tamanho alvo aproximado; inclui reserva de 3% para o contêiner.":
    "Approximate target size, including a 3% allowance for the container.",
  "A saída provavelmente não será menor. Se isso ocorrer, o original será mantido.":
    "The output may not be smaller. If so, your original will be kept.",
  "A compressão de vídeo usa perda de qualidade. Escolha uma intensidade ou use Converter.":
    "Video compression is lossy. Choose a quality preset or use Convert.",
  "Informe um tamanho maior que zero.":
    "Enter a target size greater than zero.",
  "Tamanho desejado disponível para imagens de 8 bits, áudio e vídeo. Use compressão automática para este arquivo.":
    "Target size is available for 8-bit images, audio and video. Use automatic compression for this file.",
  "Sufixo de arquivo inválido.": "Invalid filename suffix.",
  "Máximo de 100 presets": "You can save up to 100 presets.",
  "Nome ou identificador de preset inválido":
    "Invalid preset name or identifier.",
  "Este tipo de arquivo não tem prévia visual.":
    "This file type has no visual preview.",
  "Espaço insuficiente. Escolha outra pasta de saída ou libere espaço no disco temporário.":
    "Not enough disk space. Choose another output folder or free space on the temporary disk.",
  "Sem permissão para gravar. Escolha uma pasta acessível e verifique se o arquivo está aberto em outro programa.":
    "Cannot write to this location. Choose an accessible folder and check whether another app has the file open.",
  "Arquivo protegido. Confira a senha e o tipo de proteção em Avançado.":
    "Protected file. Check the password and protection type in Advanced.",
  "O motor não conseguiu processar este arquivo. Tente outro formato ou uma configuração mais simples. O diagnóstico está em Detalhes.":
    "The engine could not process this file. Try another format or simpler settings. Diagnostics are available in Details.",
};
export function messageText(text: string): string {
  if (document.documentElement.lang !== "en") return text;
  let translated = text;
  for (const [pt, en] of Object.entries(english))
    translated = translated.replaceAll(pt, en);
  return translated;
}
