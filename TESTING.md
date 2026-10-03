# Testes e validação

## V1 — regressão de compressão

43 verificações passaram: 3 testes da matriz, 8 unitários Rust e 32 integrações. Os cinco novos testes cobrem FLAC de 1.465.285 bytes nas três intensidades, opções antigas inconsistentes, preservação sem perda, alvo inválido e resultado sem redução na fila. A interface Windows foi usada para comprimir o FLAC de teste em MP3 equilibrado: 193.141 bytes, 86,8% de redução. O teste instalado executa também as três intensidades e o caso sem redução.

## Automatizados

`node scripts/fixtures.mjs` gera mídia sintética e documentos originais CC0. `npm test` executa a matriz de formatos e testes Rust. As integrações não são ignoradas quando falta um motor: falham explicitamente.

### Conversões cobertas

PNG → JPG; JPG → PNG; PNG → WebP; WAV → FLAC; WAV → MP3; FLAC → WAV; MP4 → WebM; MP4 → MP3; DOCX → PDF; MD → HTML; ZIP → 7Z; imagem → PDF; PDF → PNG/JPG; otimização PDF; imagens → PDF multipágina; criação/extração 7Z com senha; ZIP → TAR.GZ → extração; vídeo com duas passagens.

### Comportamento coberto

- Assinatura PNG com extensão `.dat`.
- Nomes com espaços, acentos e japonês.
- Imagem inválida, texto mínimo, conversão incompatível.
- Comparação PCM e pixels em conversões sem perda.
- Rejeição de PNG 16-bit → WebP lossless.
- Alvo de imagem via busca de qualidade; alvo sem perda impossível.
- Cancelamento de processo FFmpeg real.
- Traversal em nomes de arquivo compactado.
- Publicação com nome existente preservando o original.
- Fila pausada, cancelamento pendente, retomada e registro no histórico.
- Limpeza dos temporários em sucesso/falha.

## Interface

A janela de produção foi inspecionada no Windows em tema escuro. Um WAV 24-bit com Unicode foi selecionado pelo diálogo nativo e convertido para FLAC pela interface; a fila exibiu concluído, tamanhos e economia. O teste do instalador usa o modo de smoke test do mesmo executável instalado, não um substituto Python/Node do backend.

## O que não foi comprovado

Não há uma matriz de VMs limpas Windows 10/11, teste de toda combinação de entradas/codecs, fuzzing de decoders, benchmark com centenas de vídeos grandes, teste de todos os modelos de GPU ou certificação de acessibilidade. Resultados de smoke test e relatório final devem ser consultados; não confundir caminhos declarados na matriz com cobertura exaustiva de todas as variantes de arquivos.


## QA V2

`src-tauri/tests/reshape_v2.rs` cobre preservação de codec/duração no remux, proteção HDR, orçamento de múltiplas faixas, imagem real/estimativa medida, proteção de animação, limpeza de segredos de presets, retomada pausada/reordenação/cancelamento global, fallback NVENC e publicação sem sobrescrita.

Argumentos de QA `--qa-scale 1|1.25|1.5|2` ajustam apenas o zoom WebView da janela desta execução. Não alteram DPI do Windows. `--open <caminhos>` permite carregar fixtures reais no aplicativo. A inspeção do render em 100/125/150/200% não substitui testes em monitores físicos com DPIs distintos.
