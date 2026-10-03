# Reshape — identidade e interação

Pesquisa realizada em 02/10/2026: [Inspora](https://www.inspora.design/), coleções Product/Motion/Branding/3D e Temperature as a material; [Motion](https://motion.dev/examples), App Store/shared layout/reorder/presence e documentação React; [Bencho](https://bencho.dev/), especialmente comparação de imagens, reorder, upload e controles táteis; [Logosystem](https://logosystem.co/), coleções de espaço negativo, formas mínimas e marcas animadas. Nenhum ativo, logo ou código premium foi copiado.

A composição mantém um objeto central, controles progressivos e números que explicam o resultado. A revisão 2.0.2 segue a correção do usuário: fundo OLED #000000, superfícies neutras, tipografia branca e coral restrito à marca. O estado vazio retoma o objeto central e a escala contida da interface V2, sem faixas decorativas coloridas. Tipografia Segoe UI Variable/Segoe UI do sistema, sem distribuir fontes da Apple.

O novo símbolo original entrelaça duas faixas curvas em coral e grafite. As duas metades sugerem passagem entre formas; o negativo central mantém leitura em tamanhos pequenos. Master em `src-tauri/icons/reshape-mark.svg`; `icon.svg` acrescenta a base do ícone Windows. ICO em 16, 20, 24, 32, 48, 64, 128 e 256 pixels.

Motion for React usa molas de rigidez 380, amortecimento 34 e massa 0,9. Seletores compartilham layoutId; o objeto inicial e o arquivo carregado mantêm continuidade espacial; opções avançadas expandem a superfície; a fila usa Reorder; modais/toasts têm presença; números interpolam por 320 ms. O símbolo contrai pelo progresso real. Não há animação contínua em repouso. A preferência de movimento reduzido do sistema não desativa o sistema, conforme solicitado; não há flashes.

A janela conserva controles nativos de Windows para arraste, snap e DPI. Foco visível, diálogos com Escape e contenção de Tab e controles rotulados são mantidos. Zoom/pan da comparação é compartilhado entre as duas imagens.

Revisão de fluidez: snapshots idênticos não provocam renderizações; chamadas não se sobrepõem. Diálogos preservam foco entre atualizações; interruptores usam mola independente (650/30/0,45), menus animam entrada e saída, números retomam do valor visual atual. As artes são vetoriais/CSS e não exigem animação contínua em repouso. O instalador usa o mesmo símbolo, arte grafite/coral, superfícies escuras e textos próprios. Inglês é padrão em instalações novas; português continua selecionável.

O painel de estimativa tem hierarquia própria e espaço estável. Transições estruturais animam posição sem escalar texto; troca de arquivo usa fade/deslocamento curto. As seleções, interruptores, expansão e modais mantêm molas e continuidade. A estimativa só pertence às opções que a geraram; resultados antigos não aparecem durante a troca.
