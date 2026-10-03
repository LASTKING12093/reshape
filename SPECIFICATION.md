Quero que você desenvolva um aplicativo desktop completo para Windows que funcione como um CONVERSOR UNIVERSAL + COMPRESSOR UNIVERSAL de arquivos.

O objetivo é criar um aplicativo extremamente fácil de usar para usuários comuns, mas ao mesmo tempo muito poderoso para usuários avançados.

Não quero apenas um protótipo, mockup ou scaffolding. Quero um projeto real, funcional, testado, empacotável e com instalador Windows no final.

==================================================
1. OBJETIVO GERAL
==================================================

O usuário deve poder arrastar praticamente qualquer arquivo para o aplicativo e:

1. Converter para outros formatos compatíveis.
2. Comprimir o arquivo.
3. Redimensionar quando aplicável.
4. Definir qualidade.
5. Definir tamanho de saída aproximado.
6. Processar vários arquivos de uma vez.
7. Acompanhar progresso.
8. Receber o arquivo final sem precisar entender codecs ou ferramentas de linha de comando.

Todo processamento deve acontecer LOCALMENTE no computador.

Não usar serviços pagos ou APIs externas para fazer as conversões.

O usuário NÃO deve precisar instalar FFmpeg, ImageMagick, LibreOffice, Pandoc, 7-Zip ou qualquer outra dependência manualmente.

O instalador final deve preparar tudo necessário.

==================================================
2. PLATAFORMA E STACK
==================================================

Target inicial:

Windows 10 64-bit
Windows 11 64-bit

Use preferencialmente:

Tauri 2
Rust
React
TypeScript
Vite

Frontend moderno e extremamente rápido.

Backend Rust responsável por:

- filesystem;
- detecção de arquivos;
- gerenciamento de processos;
- conversões;
- filas;
- cancelamento;
- progresso;
- validação dos resultados;
- gerenciamento dos executáveis auxiliares.

Não usar Electron a menos que exista uma justificativa técnica extremamente forte.

==================================================
3. MOTORES DE CONVERSÃO
==================================================

Crie uma arquitetura de adapters/providers.

Não espalhe comandos diretamente pelo código.

Exemplo:

ConversionEngine
    ImageEngine
    AudioVideoEngine
    DocumentEngine
    ArchiveEngine
    PdfEngine
    TextEngine

Utilizar inicialmente:

FFmpeg + ffprobe
ImageMagick
LibreOffice headless
Pandoc
7-Zip CLI

Outras ferramentas podem ser adicionadas quando realmente necessárias.

Os executáveis devem preferencialmente ser distribuídos como sidecars/resources do aplicativo.

A aplicação deve saber automaticamente onde seus binários estão localizados.

Nunca depender do PATH global do Windows.

==================================================
4. IMAGENS
==================================================

Suportar o máximo possível, incluindo:

PNG
JPG
JPEG
WEBP
AVIF
HEIC
HEIF
BMP
TIFF
TIF
GIF
ICO

Suporte SVG quando tecnicamente apropriado.

IMPORTANTE:

Converter raster para SVG não deve simplesmente trocar a extensão.

Se vectorização real não estiver implementada, não oferecer essa conversão como se fosse equivalente.

Permitir:

formato de saída;
quality;
lossless quando disponível;
largura;
altura;
manter aspect ratio;
resize em porcentagem;
DPI;
remoção/preservação de metadata;
remoção/preservação de alpha;
background quando converter transparência para JPG;
compressão;
target file size;
preservar resolução;
preservar perfil de cores quando possível.

Presets:

Lossless
Maximum Quality
High Quality
Balanced
Small
Very Small
Custom

Target Size:

Se usuário selecionar, por exemplo:

500 KB

implementar algoritmo iterativo/binary search para encontrar a maior qualidade possível abaixo ou próxima do tamanho pedido.

Nunca fingir que tamanho lossless arbitrário é possível.

==================================================
5. ÁUDIO
==================================================

Formatos importantes:

WAV
FLAC
MP3
AAC
M4A
ALAC
OGG
OPUS
AIFF
WMA
AC3

Quando suportados pelo FFmpeg utilizado.

Permitir:

codec;
container;
bitrate;
VBR;
CBR;
sample rate;
bit depth;
mono/stereo;
número de canais;
normalização opcional;
metadata;
album art;
lossless;
target size.

Presets de áudio:

Original / Lossless
Studio
Maximum
High
Balanced
Small
Very Small
Custom

Exemplos:

WAV -> FLAC deve permitir conversão lossless.

FLAC -> WAV também.

WAV -> MP3 obviamente será lossy.

A interface deve deixar isso claro.

Preservar tags e artwork quando tecnicamente possível.

==================================================
6. VÍDEO
==================================================

Containers importantes:

MP4
MOV
MKV
AVI
WEBM
M4V
TS
MTS
WMV
FLV

Codecs quando disponíveis:

H.264
H.265 / HEVC
AV1
VP9
ProRes

Áudio:

AAC
Opus
MP3
FLAC
PCM
outros compatíveis.

Permitir:

resolução;
original;
2160p;
1440p;
1080p;
720p;
480p;
custom;

FPS;
codec;
CRF / quality;
bitrate;
audio bitrate;
audio codec;
preservação de subtitles;
preservação de chapters;
preservação de metadata;
hardware encoding quando disponível;
software encoding;
target file size.

Detectar automaticamente:

NVIDIA NVENC
Intel Quick Sync
AMD AMF

Mas não exigir hardware encoding.

Software encoding deve sempre existir como fallback.

Para Target Size de vídeo:

usar duração via ffprobe.

Calcular bitrate alvo levando em consideração:

target bytes;
duration;
audio bitrate;
container overhead.

Quando apropriado, utilizar two-pass encoding para aproximar melhor o tamanho selecionado.

Não prometer precisão absoluta.

Mostrar algo como:

Target: 100 MB
Expected: ~98–102 MB

quando apropriado.

==================================================
7. DOCUMENTOS
==================================================

Suportar conversões sensatas entre formatos como:

DOC
DOCX
ODT
RTF
TXT
HTML
MD

XLS
XLSX
ODS
CSV

PPT
PPTX
ODP

PDF

Usar principalmente LibreOffice headless para documentos Office.

Usar Pandoc para formatos onde ele é adequado.

Muito importante:

Não mostrar combinações absurdas simplesmente porque alguma ferramenta aceita tecnicamente.

Gerar uma matriz explícita de formatos de entrada -> formatos de saída suportados.

Conversions com possível perda de layout devem receber um pequeno indicador:

Perfect
Mostly preserved
May change layout

Não precisa usar exatamente esses textos.

==================================================
8. PDF
==================================================

Quero suporte amplo a PDF.

Implementar quando possível:

DOCX -> PDF
PPTX -> PDF
XLSX -> PDF
ODT -> PDF
imagens -> PDF
múltiplas imagens -> um PDF
PDF -> imagens
PDF compression

Para compressão PDF, oferecer:

Lossless Optimization
Balanced
Small
Very Small
Custom

Se alguma dependência necessária tiver licença que afete distribuição comercial/closed-source, NÃO ignore isso.

Documente claramente em THIRD_PARTY_LICENSES.md.

Prefira alternativas permissivas quando tecnicamente adequadas.

==================================================
9. ARQUIVOS COMPACTADOS
==================================================

Suportar:

ZIP
7Z
TAR
TAR.GZ
GZ
BZ2
XZ

Permitir:

criar arquivo;
extrair;
recompactar;
converter/reempacotar ZIP -> 7Z etc.;
nível de compressão;
password quando suportado;
preservação da estrutura interna.

Não excluir o arquivo original.

==================================================
10. COMPRESSOR
==================================================

Criar uma aba dedicada:

Compress

Ao adicionar um arquivo, detectar automaticamente o tipo e apresentar os métodos de compressão possíveis.

MODOS:

Lossless
Smart
Target Size
Custom

LOSSLESS:

Nunca causar degradação de conteúdo.

SMART:

Priorizar redução de tamanho com diferença perceptiva mínima.

TARGET SIZE:

Usuário informa:

500 KB
2 MB
20 MB
100 MB
etc.

O app tenta chegar o mais perto possível.

Se não for possível mantendo as restrições selecionadas:

explicar.

Exemplo:

"500 KB cannot be reached using lossless compression. Estimated minimum: 2.3 MB."

Oferecer:

Switch to Smart Compression

Não alterar automaticamente para modo lossy sem autorização do usuário.

==================================================
11. INTERFACE
==================================================

Quero uma UI premium, minimalista e extremamente intuitiva.

Referências de filosofia:

Apple
Raycast
Arc
Linear

NÃO copiar literalmente nenhum deles.

Visual:

clean;
moderno;
poucos elementos;
muito whitespace;
animações suaves;
bordas discretas;
hierarquia visual forte;
dark mode;
light mode;
system theme.

Tela principal:

sidebar ou navigation bar com:

Convert
Compress
Queue
History
Settings

Área central:

uma grande drop zone.

Texto aproximadamente:

Drop files here

ou botão:

Choose Files

Aceitar drag and drop de múltiplos arquivos e pastas.

Depois do drop:

mostrar cards/rows contendo:

ícone;
nome;
formato;
dimensões/duração quando aplicável;
tamanho original;
formato de saída;
preset;
tamanho estimado;
status.

Exemplo:

song.wav
WAV • 24-bit • 48 kHz • 78.4 MB

Convert to:
FLAC

Preset:
Lossless

Expected:
~42 MB

[Convert]

Não sobrecarregar a interface com controles técnicos.

==================================================
12. SIMPLE MODE E ADVANCED MODE
==================================================

Por padrão usar SIMPLE MODE.

Simple Mode deve mostrar somente:

formato;
preset;
tamanho/resolução quando relevante;
local de saída;
Convert.

Advanced Mode pode mostrar:

codec;
bitrate;
CRF;
sample rate;
bit depth;
FPS;
metadata;
colorspace;
pixel format;
encoder;
threads;
hardware acceleration;
etc.

==================================================
13. BATCH PROCESSING
==================================================

Permitir selecionar:

1 arquivo
10 arquivos
100 arquivos

ao mesmo tempo.

Fila de conversão.

Mostrar:

Pending
Processing
Completed
Failed
Cancelled

Permitir:

Pause Queue
Resume
Cancel
Retry
Remove
Clear completed

Definir limite de tarefas simultâneas.

Não iniciar 20 encodes pesados simultaneamente.

Criar scheduler inteligente considerando CPU.

==================================================
14. PROGRESSO
==================================================

Implementar progresso real.

Para FFmpeg usar saída apropriada para machine-readable progress.

Para outros engines, determinar progresso quando possível.

Caso uma ferramenta não ofereça progresso confiável:

mostrar estado indeterminado em vez de inventar porcentagem.

Mostrar:

porcentagem;
tempo transcorrido;
estimativa restante quando confiável;
velocidade;
arquivo atual.

==================================================
15. DETECÇÃO DE ARQUIVO
==================================================

Não confiar exclusivamente na extensão.

Utilizar:

extensão;
MIME;
magic bytes;
ffprobe;
ImageMagick identify;
outros probes quando necessário.

Se arquivo estiver incorretamente nomeado, tentar identificar conteúdo real.

Exemplo:

imagem chamada file.dat

se for claramente PNG:

identificar como PNG.

==================================================
16. COMPATIBILITY MATRIX
==================================================

Criar uma estrutura central, declarativa e testável contendo:

Input format
Output formats
Engine
Lossless support
Target-size support
Metadata support
Advanced options

Algo como:

FormatCapabilityRegistry

Toda UI deve consultar essa fonte.

Não espalhar arrays independentes de formatos pela aplicação.

==================================================
17. SEGURANÇA
==================================================

Muito importante:

Nunca construir comandos concatenando strings de usuário diretamente.

Passar argumentos aos processos como arrays seguros.

Validar caminhos.

Suportar espaços, Unicode e caracteres especiais em filenames.

Evitar command injection.

Criar pasta temporária única por job.

Limpar temporários após conclusão/cancelamento/falha.

Nunca sobrescrever original por padrão.

Se output já existir:

file.jpg
file (1).jpg
file (2).jpg

Ou perguntar conforme configuração do usuário.

==================================================
18. PRIVACIDADE
==================================================

Conversões locais.

Nenhum arquivo deve ser enviado para servidores.

A aplicação deve funcionar offline.

Não adicionar telemetry por padrão.

==================================================
19. HISTÓRICO
==================================================

Guardar localmente:

arquivo;
input format;
output format;
old size;
new size;
redução percentual;
data;
status;
output path.

Botões:

Open File
Open Folder
Convert Again
Remove History

Não guardar cópias desnecessárias dos arquivos.

==================================================
20. CONFIGURAÇÕES
==================================================

Settings:

Theme
Language
Default output directory
Keep metadata
Overwrite behavior
Concurrency
Hardware acceleration
Default image preset
Default audio preset
Default video preset
Temporary directory
Clear temporary files
History
Check converter engines
About

Idiomas inicialmente:

English
Português do Brasil

Arquitetura preparada para i18n.

==================================================
21. TAMANHO ESTIMADO
==================================================

Antes da conversão, quando tecnicamente possível:

mostrar estimativa do output.

Não fingir precisão.

Use:

~42 MB

e não:

42.0381 MB

Para codecs baseados em bitrate:

calcular utilizando duração.

Para quality/CRF:

usar heurísticas e informar que é estimativa.

==================================================
22. COMPARAÇÃO DE TAMANHO
==================================================

Depois da conversão:

Completed

78.4 MB -> 41.8 MB

Saved 36.6 MB
46.7% smaller

Quando output for maior:

mostrar isso claramente.

==================================================
23. METADATA
==================================================

Opção global:

Preserve metadata

e opção:

Remove metadata

Para mídia preservar quando possível:

title
artist
album
track
year
cover art
EXIF
creation time

dependendo do formato.

==================================================
24. OUTPUT FOLDER
==================================================

Opções:

Same folder as source
Downloads
Custom folder

Também permitir:

Remember my choice.

==================================================
25. DRAG AND DROP
==================================================

Drag:

arquivo;
vários arquivos;
pasta inteira.

Ao arrastar pasta:

escanear recursivamente opcionalmente.

Mostrar confirmação quando houver muitos arquivos.

==================================================
26. FILE ASSOCIATIONS
==================================================

Preparar arquitetura para futuramente permitir:

Right Click -> Convert with [App]

e associações opcionais no Windows.

Se for simples implementar corretamente no installer, pode implementar.

Não sequestrar associações de arquivo automaticamente.

==================================================
27. CONTEXT MENU
==================================================

Se possível adicionar opcionalmente ao Explorer:

Convert with [App]
Compress with [App]

Somente mediante opção do usuário/installer.

==================================================
28. INSTALADOR
==================================================

Quero no final um instalador Windows:

AppName_x64-setup.exe

Preferencialmente usando o bundler/NSIS do Tauri.

O usuário deve:

1. baixar setup;
2. abrir;
3. instalar;
4. abrir aplicativo;
5. converter arquivos.

Nada de:

"agora instale FFmpeg"
"adicione ImageMagick ao PATH"
"instale Python"
"instale Node"

Essas são dependências de desenvolvimento, não do usuário final.

Dependências runtime devem estar:

embutidas;
instaladas automaticamente;
ou gerenciadas pelo app.

==================================================
29. SIDECARS / BINARIES
==================================================

Criar estrutura organizada, por exemplo:

src-tauri/
    binaries/
        ffmpeg/
        imagemagick/
        7zip/
        pandoc/
        libreoffice/
        ...

ou outra estrutura melhor.

Criar sistema central:

EngineManager

que:

localiza executáveis;
verifica versão;
verifica integridade;
informa disponibilidade;
executa self-test.

Settings deve possuir:

Converter Engines

FFmpeg      Ready
ImageMagick Ready
LibreOffice Ready
Pandoc      Ready
7-Zip       Ready

==================================================
30. LICENÇAS
==================================================

Não ignore licenciamento.

Crie:

THIRD_PARTY_LICENSES.md

e mantenha as licenças exigidas pelas ferramentas distribuídas.

Antes de decidir empacotar determinada ferramenta, verificar licença atual.

Se determinada ferramenta criar implicações importantes para distribuição closed-source/comercial, documentar e buscar alternativa apropriada.

==================================================
31. LOGGING
==================================================

Criar sistema de logs.

Logs normais não devem assustar usuário.

Interface deve mostrar erro simples:

Conversion failed

Details

Nos detalhes:

engine;
exit code;
comando sanitizado;
stderr relevante.

Criar opção:

Copy diagnostic information

para facilitar debugging.

Nunca incluir informações privadas desnecessárias.

==================================================
32. CANCELAMENTO
==================================================

Cancel deve realmente encerrar:

processo;
subprocessos;
pipes;
temporários.

No Windows cuidar de process tree.

Não deixar ffmpeg.exe rodando em background após cancelamento.

==================================================
33. TESTES
==================================================

Crie testes unitários e integration tests.

Criar diretório:

test-fixtures/

com pequenos arquivos de teste, quando licenciamento permitir.

Testar pelo menos:

PNG -> JPG
JPG -> PNG
PNG -> WEBP
WAV -> FLAC
WAV -> MP3
FLAC -> WAV
MP4 -> WEBM
MP4 -> MP3
DOCX -> PDF
MD -> HTML
ZIP -> 7Z

Testar:

filenames com espaços;
acentos;
Unicode;
arquivos muito pequenos;
arquivos inválidos;
formato incompatível;
cancelamento;
output já existente.

==================================================
34. VALIDAÇÃO
==================================================

Após qualquer conversão:

não considere sucesso simplesmente porque exit code foi 0.

Validar:

output existe;
output não está vazio;
tipo é esperado;
probe funciona quando aplicável.

Para áudio lossless:

quando tecnicamente possível testar igualdade do conteúdo PCM decodificado.

Para imagem lossless:

quando apropriado validar pixels após decode.

==================================================
35. ERROR HANDLING
==================================================

Exemplos de mensagens:

Unsupported conversion.

The selected codec isn't available in this build.

Not enough disk space.

The source file appears to be corrupted.

The requested lossless target size cannot be achieved.

Conversion was cancelled.

Não mostrar apenas:

Error code 1

para usuário comum.

==================================================
36. DISK SPACE
==================================================

Antes de operações pesadas:

estimar espaço temporário necessário.

Detectar espaço livre.

Evitar começar conversão gigantesca sem espaço suficiente.

==================================================
37. PERFORMANCE
==================================================

Não carregar arquivos gigantes inteiros na RAM.

Usar streaming/processes adequadamente.

UI deve permanecer responsiva durante conversão.

Nunca bloquear main thread.

==================================================
38. ACCESSIBILITY
==================================================

Keyboard navigation.

Focus states.

Screen reader labels.

Contraste apropriado.

Esc para fechar modais.

Enter para ações principais quando apropriado.

==================================================
39. ARQUITETURA
==================================================

Quero projeto limpo e modular.

Sugestão conceitual:

src/
    components/
    features/
        convert/
        compress/
        queue/
        history/
        settings/
    hooks/
    stores/
    types/
    utils/
    i18n/

src-tauri/src/
    commands/
    engines/
        ffmpeg/
        imagemagick/
        libreoffice/
        pandoc/
        sevenzip/
    jobs/
    queue/
    formats/
    probing/
    filesystem/
    settings/
    history/
    errors/

Não precisa seguir literalmente se existir arquitetura melhor.

==================================================
40. DOMAIN MODEL
==================================================

Criar modelos claros:

InputFile
MediaInfo
ConversionJob
ConversionOptions
ConversionPreset
OutputTarget
ProgressUpdate
EngineCapability
FormatCapability
ConversionResult
CompressionMode

Usar tipos fortes.

Evitar any no TypeScript.

==================================================
41. SIMPLE PRESETS
==================================================

Não usar nomes técnicos como única escolha para usuário comum.

Imagem:

Lossless
Maximum
High
Balanced
Small

Audio:

Lossless
Maximum
High
Balanced
Small

Video:

Maximum
High
Balanced
Small
Very Small

Internamente mapear isso para parâmetros técnicos.

==================================================
42. SMART DEFAULTS
==================================================

Exemplos:

PNG -> JPG:
quality alta por padrão.

WAV -> FLAC:
lossless.

WAV -> MP3:
MP3 VBR de alta qualidade.

MOV -> MP4:
H.264 + AAC com parâmetros modernos.

Large video -> smaller MP4:
usar preset Balanced.

Sempre evitar configurações absurdas.

==================================================
43. MULTIPLE FILES
==================================================

Se todos forem mesmo tipo:

permitir aplicar configuração para todos.

Se forem tipos diferentes:

cada arquivo pode receber seu próprio formato.

Também permitir ações por grupo.

Exemplo:

20 PNGs -> WEBP

Apply to all.

==================================================
44. RESULTADOS
==================================================

Após batch:

20 files completed
Original: 1.42 GB
Output: 612 MB
Saved: 808 MB

Open Folder

==================================================
45. NÃO IMPLEMENTAR CONVERSÕES FALSAS
==================================================

Nunca:

trocar extensão e chamar isso de conversão.

Nunca oferecer:

JPG -> SVG

como verdadeira vectorização se não houver vectorizer.

Nunca fingir:

lossless

quando houve recodificação lossy.

Nunca afirmar:

exact target size

quando encoder só permite aproximação.

==================================================
46. EXTENSIBILIDADE
==================================================

Quero conseguir adicionar novos engines futuramente.

Exemplo:

trait ConversionEngine

supports(...)
probe(...)
build_job(...)
execute(...)
cancel(...)
validate(...)

ou arquitetura Rust equivalente.

Adicionar novo formato não deve exigir refazer metade do aplicativo.

==================================================
47. PRIMEIRO BOOT
==================================================

Na primeira execução:

executar pequeno health check dos engines.

Não mostrar configuração complicada.

Se tudo estiver correto:

abrir direto na tela principal.

==================================================
48. INSTALAÇÃO OFFLINE
==================================================

Preferência forte:

o setup final deve funcionar offline.

Portanto, sempre que licença e tamanho permitirem:

incluir runtime dependencies no instalador.

Se WebView2/runtime adicional precisar ser tratado, usar estratégia adequada do Tauri.

Documentar decisão.

==================================================
49. BUILD AUTOMATION
==================================================

Criar scripts simples:

dev
test
lint
build
package

Exemplo:

npm run dev
npm run test
npm run tauri build

Também criar um script PowerShell:

scripts/build-release.ps1

que:

valida dependências de desenvolvimento;
executa testes;
gera frontend;
gera Tauri;
gera installer;
mostra caminho final.

==================================================
50. DOCUMENTAÇÃO
==================================================

Criar:

README.md
ARCHITECTURE.md
SUPPORTED_FORMATS.md
BUILD.md
THIRD_PARTY_LICENSES.md
TESTING.md

README deve explicar o produto.

BUILD deve permitir que outro desenvolvedor construa o projeto do zero.

SUPPORTED_FORMATS deve ser gerado ou sincronizado com a capability registry sempre que possível.

==================================================
51. DEFINIÇÃO DE PRONTO
==================================================

Não considere projeto terminado enquanto não houver:

UI funcional;
drag and drop;
detecção de formatos;
conversão real;
compressão real;
fila;
progresso;
cancelamento;
settings;
history;
error handling;
múltiplos engines;
testes;
build de produção;
instalador Windows.

Quero conseguir instalar a aplicação e executar conversões reais.

==================================================
52. FORMA DE TRABALHO
==================================================

Você tem autonomia para tomar decisões técnicas.

Não me interrompa constantemente para perguntar decisões pequenas.

Quando houver mais de uma solução razoável:

pesquise/documente;
escolha a que tenha melhor equilíbrio entre qualidade, estabilidade, licença, manutenção e experiência do usuário.

Antes de implementar:

1. inspecione o ambiente;
2. crie um plano técnico;
3. crie architecture docs;
4. monte capability matrix;
5. implemente por etapas;
6. teste cada engine;
7. integre UI;
8. execute testes;
9. faça build;
10. gere installer;
11. faça smoke test do build final.

Não pare após criar arquivos vazios ou TODOs.

Não marque funcionalidade como concluída antes de realmente testá-la.

==================================================
53. PRIORIDADE DE IMPLEMENTAÇÃO
==================================================

Prioridade máxima:

Images
Audio
Video

Em seguida:

Documents
PDF
Archives

Mas construa desde o início a arquitetura universal para que todas usem a mesma experiência de usuário.

==================================================
54. RESULTADO FINAL
==================================================

Quando terminar, quero um relatório contendo:

1. O que foi implementado.
2. Formatos suportados.
3. Engines utilizados.
4. Arquitetura.
5. Testes executados.
6. Problemas conhecidos.
7. Local do executável.
8. Local do instalador.
9. SHA-256 do instalador.
10. Como reproduzir o build.

O objetivo final é que eu tenha um verdadeiro aplicativo de conversão universal para Windows, extremamente simples para tarefas como:

PNG -> JPG
WAV -> FLAC
WAV -> MP3
MOV -> MP4
DOCX -> PDF
PDF -> JPG
ZIP -> 7Z

mas poderoso o suficiente para usuários avançados controlarem qualidade, tamanho, codecs e outros parâmetros.