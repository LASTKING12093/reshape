# Reshape 2.0: arquitetura e plano de execução

Aplicativo Windows x64, Tauri 2 + Rust + React/TypeScript. Todas as conversões são locais. Não há telemetria, servidor de conversão, shell arbitrário ou dependência do PATH do usuário.

## Etapas
1. Matriz declarativa e modelos de domínio; preparar recursos dos motores.
2. Probing, adapters, execução isolada, cancelamento e validação.
3. Scheduler limitado, persistência de histórico/configurações.
4. Interface simples/avançada, seleção/drop, fila e diagnósticos.
5. Fixtures próprias, testes unitários e conversões de integração.
6. Build de produção, NSIS offline, smoke test e hashes.

## Fronteiras
- `capabilities.json`: única matriz de entrada/saída, consumida por Rust e UI via IPC; documentação gerada.
- `src-tauri/src/engines.rs`: resoluções absolutas, inventário e montagem de argumentos tipados.
- `process.rs`: processo sem shell, pipes drenados, progresso FFmpeg e Job Object Windows com kill-on-close.
- `conversion.rs`: staging por job, target-size, validação e publicação sem sobrescrita.
- `queue.rs`: scheduler, cancelamento, histórico e persistência local.
- `probing.rs`: assinatura, extensão e probes reais.
- `src`: interface sem acesso direto a executáveis ou filesystem.

Cada tarefa usa diretório temporário exclusivo. O resultado é validado antes de ser publicado com criação exclusiva. Não se usa `-y` sobre arquivos do usuário. Jobs não retomam automaticamente após crash; histórico marca interrupção. O cancelamento mata a árvore antes da limpeza do staging.

Motores externos são agregados independentes, não bibliotecas ligadas ao aplicativo. Licenças e fontes correspondentes exigem revisão antes de redistribuição pública; o build local não substitui esse processo. Não há associação automática de arquivos.

## Adapters compostos

Arquivamento é uma capacidade transversal declarada em `archiveOutputs`: cada formato pode ser embalado sem modificar seu conteúdo. TAR.GZ usa duas etapas e valida a listagem de cada camada. PDF é roteado entre QPDF (otimização) e Poppler (rasterização), evitando Ghostscript como runtime adicional. Agrupamento de imagens usa cópias com nomes neutros no staging e saída PDF única.

O inventário de hardware executa um encode H.264 de um frame para cada backend candidato; a presença de um encoder em `ffmpeg -encoders` sozinha não é tratada como prova de GPU funcional. Falha de encoder de hardware dispara tentativa de software. Target size de vídeo usa duas passagens em software e reserva margem para áudio/container.

`smoke.rs` permite exercitar o mesmo backend e recursos do executável instalado via `--smoke-test`. Não é um motor substituto e não inicia a janela normal. `commands.rs` mantém a superfície IPC explícita; a interface não tem permissão para executar comandos genéricos.
## Reshape V2

A UI principal está em `src/Reshape.tsx`; módulos `ui`, `Advanced`, `Jobs` e `Comparison` compartilham componentes e tipos. `main.tsx` carrega apenas o tema novo. O frontend solicita planos ao backend, sem duplicar parâmetros de compressão.

Fluxo: `probing` → `formats` → `planning` → `estimation` → `queue` → `conversion/engines/process` → `output`. `previews` renderiza arquivos reais; `presets` valida e remove segredos e propriedades específicas de arquivos. `errors` produz mensagens curtas com diagnóstico separado.

A sondagem registra codec, bitrate, fps, HDR, faixas de áudio, tipos de legendas, páginas, frames e alpha. Planejamento valida incompatibilidades antes da fila. Áudio target e vídeo target reservam 3% para contêiner e todas as faixas de áudio. Remux evita reencode quando explicitamente adequado. A interface seleciona remux ao trocar para contêiner compatível; editar qualidade volta a recodificar.

Estimação retorna intervalo e método, nunca precisão falsa. Refinamento de imagens e prévias compartilham um worker serial, com memória limitada, diretórios temporários e timeout. Não há leitura integral de vídeos grandes para a UI: só PNGs de prévia limitados. Processamento usa hard links de entrada quando possível e cópia em streaming entre volumes. Publicação usa hard link atômico ou temporário no volume de destino seguido de persistência sem substituição.

A fila limita vídeos simultâneos a um e respeita limite total de 1–4 tarefas. `pending.json` recupera tarefas pausadas após interrupção; senhas não persistem. `presets.json`, `settings.json` e `history.json` permanecem sob o identificador legado `com.forma.converter`.

O template NSIS mantém as chaves privadas antigas para reconhecimento de atualização, mas grava DisplayName/ícone/versão Reshape e substitui o executável por reshape.exe. O hook remove apenas atalhos antigos Forma. A licença do template Tauri é MIT/Apache-2.0; origem documentada em THIRD_PARTY_LICENSES.md.

