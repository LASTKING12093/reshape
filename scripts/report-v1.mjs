import {readFileSync,writeFileSync,copyFileSync,createReadStream,statSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
const smoke=JSON.parse(readFileSync('test-output/installed-smoke-v1/smoke-report.json','utf8'));
if(!smoke.success || smoke.tests.length !== 11 || smoke.tests.filter(t=>t.preset).length !== 3) throw Error('V1 installed smoke evidence required');
copyFileSync('release/Forma_x64-setup.exe','release/Forma_V1_x64-setup.exe');
const installer=resolve('release/Forma_V1_x64-setup.exe');
const digest=createHash('sha256');for await (const b of createReadStream(installer))digest.update(b);const sha=digest.digest('hex');
const rows=smoke.tests.filter(t=>t.preset).map(t=>`| ${ {high:'Leve',balanced:'Equilibrada',small:'Máxima'}[t.preset]} | ${t.inputBytes.toLocaleString('pt-BR')} | ${t.outputBytes.toLocaleString('pt-BR')} | ${(100-t.outputBytes/t.inputBytes*100).toFixed(1)}% |`).join('\n');
const report=`# Forma V1 — 1.0.0

## 1. Atualização

Nova interface com navegação horizontal, área central de arquivos, três intensidades, opções técnicas recolhidas, indicação do formato antes da execução e resultados com economia medida. Ícone Windows corrigido: transparência em todos os tamanhos do ICO, incorporada ao executável e ao instalador.

## 2. Compressão corrigida

A versão anterior não usava o modo inteligente para ajustar o motor e podia continuar com FLAC sem perda. Agora a compressão é planejada no backend por categoria, mesmo quando chegam opções antigas inconsistentes. Áudio usa MP3 a 192/128/96 kbps; imagens de 8 bits usam WebP; vídeo usa MP4; PDF é otimizado e documentos são compactados em ZIP. A interface avisa sobre perda de qualidade. O modo sem perda permanece separado.

Resultados iguais ou maiores são descartados antes da publicação, exibindo **Sem redução** e oferecendo abrir o original. Não se anuncia economia inexistente. Nenhum original é sobrescrito.

## 3. Reprodução do caso FLAC

Arquivo sintético CC0 de 12 segundos, 24 bits, próximo de 1,4 MB. Resultados medidos pelo executável **instalado**:

| Intensidade | FLAC original (bytes) | MP3 (bytes) | Redução |
|---|---:|---:|---:|
${rows}

Esses números são do arquivo de teste; não garantem a mesma redução para qualquer áudio. MP3 tem perda. Um FLAC já otimizado pode não reduzir sem perda.

## 4. Validação

43 testes automatizados passaram (3 da matriz, 8 unitários Rust, 32 integrações). TypeScript e build Vite passaram. O fluxo FLAC→MP3 foi executado pelo diálogo nativo e pelos botões da interface Windows, obtendo 188,6 KB na opção equilibrada. Após a instalação, todos os oito componentes ficaram prontos com integridade válida e os 11 casos do smoke test passaram, incluindo três intensidades e o caso sem redução.

## 5. Arquivos da entrega

- [Instalador V1](release/Forma_V1_x64-setup.exe): ${installer}
- Executável instalado: ${resolve('test-output/installed-app/forma.exe')}
- Build: ${resolve('src-tauri/target/release/forma.exe')} (requer a pasta binaries adjacente).
- [Evidência do teste instalado](release/smoke-report-v1.json).
- [Matriz de formatos](SUPPORTED_FORMATS.md).

## 6. Motores

${smoke.engines.map(e=>`- ${e.id}: ${e.version} — integridade válida.`).join('\n')}

## 7. Instalação

O instalador V1 foi executado com sucesso sobre a instalação de teste anterior. Inclui WebView2 offline e motores. O identificador do aplicativo permanece com.forma.converter para conservar configurações e histórico. O instalador antigo deve ser substituído pelo V1; um atalho fixado anteriormente pode precisar ser fixado novamente se o Windows mantiver a imagem antiga em cache.

## 8. SHA-256 e tamanho

SHA-256: ${sha}

${statSync(installer).size} bytes (${(statSync(installer).size/1048576).toFixed(2)} MiB). Consulte [SHA256SUMS.txt](release/SHA256SUMS.txt).

## 9. Limites

Esta atualização trata do visual, ícone e fluxo de compressão. Não significa que todos os 54 itens da especificação original foram implementados. Persistem limitações em codecs, controles avançados e recuperação de fila após crash. O instalador não é assinado e não foi certificado em VMs limpas de todas as versões Windows/GPU. [Limitações](KNOWN_LIMITATIONS.md) e [licenças](THIRD_PARTY_LICENSES.md).

## 10. Reprodução do build

Veja [BUILD.md](BUILD.md). Execute npm ci, prepare os motores e rode npm run package. Nesta atualização, a suíte foi executada antes de scripts/build-release.ps1 -SkipTests; depois foi validado o executável instalado com --smoke-test. Os artefatos de testes anteriores não foram usados como prova do smoke V1.
`;
writeFileSync('RELEASE_REPORT.md',report);
writeFileSync('release/SHA256SUMS.txt',`${sha}  Forma_V1_x64-setup.exe\n${sha}  Forma_x64-setup.exe\n`);
copyFileSync('test-output/installed-smoke-v1/smoke-report.json','release/smoke-report-v1.json');
console.log(JSON.stringify({installer,sha256:sha,tests:smoke.tests.length}));
