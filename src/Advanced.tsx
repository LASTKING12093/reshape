import { type ConversionOptions as Options, type InputFile } from "./types";
import { Picker, Toggle, bytes, type L } from "./ui";
export default function Advanced({
  file,
  o,
  patch,
  hardware,
  formats,
  l,
}: {
  file: InputFile;
  o: Options;
  patch: (v: Partial<Options>) => void;
  hardware: string[];
  formats: string[];
  l: L;
}) {
  const number = (
    key: keyof Options,
    label: string,
    min: number,
    max: number,
    nullable = false,
  ) => (
    <label className="field" key={key}>
      <span>{label}</span>
      <input
        type="number"
        min={min}
        max={max}
        value={(o[key] as number | null) ?? ""}
        placeholder={l("Original", "Original")}
        onChange={(e) =>
          patch({
            [key]:
              e.target.value === "" && nullable ? null : Number(e.target.value),
          })
        }
      />
    </label>
  );
  const image = file.category === "image",
    video = file.category === "video",
    audio = file.category === "audio";
  const choice = (key: keyof Options, label: string, items: string[]) => (
    <label className="field">
      <span>{label}</span>
      <Picker
        label={label}
        value={String(o[key] ?? "")}
        items={(key === "bitDepth" ? ["", ...items] : items).map((value) => ({
          value,
          label:
            value === ""
              ? l("Original", "Original")
              : value === "software"
                ? "CPU"
                : value === "auto"
                  ? l("Automático", "Auto")
                  : value.toUpperCase(),
        }))}
        onChange={(v) =>
          patch({ [key]: key === "bitDepth" ? (v ? Number(v) : null) : v })
        }
      />
    </label>
  );
  return (
    <div className="advanced-fields">
      {choice("format", l("Formato de saída", "Output format"), formats)}
      <div className="section-label">
        {l("Controle do resultado", "Output controls")}
      </div>
      {(image || (file.format === "pdf" && o.format === "pdf")) &&
        number(
          "quality",
          l("Qualidade das imagens · 1–100", "Image quality · 1–100"),
          1,
          100,
        )}
      {(image || video) && (
        <>
          <div className="field-pair">
            {number("width", l("Largura", "Width"), 1, 32768, true)}
            {number("height", l("Altura", "Height"), 1, 32768, true)}
          </div>
          <Toggle
            label={l("Manter proporção", "Keep aspect ratio")}
            value={o.keepAspect}
            onChange={(v) => patch({ keepAspect: v })}
          />
        </>
      )}
      {image && (
        <>
          <Toggle
            label={l("Preservar transparência", "Preserve transparency")}
            value={o.keepAlpha}
            onChange={(v) => patch({ keepAlpha: v })}
          />
          <label className="field">
            <span>{l("Fundo para JPEG", "JPEG background")}</span>
            <input
              type="color"
              value={o.background}
              onChange={(e) => patch({ background: e.target.value })}
            />
          </label>
          {number("dpi", "DPI", 36, 1200)}
          <Toggle
            label={l(
              "Sem perdas (formatos compatíveis)",
              "Lossless (compatible formats)",
            )}
            value={o.lossless}
            onChange={(v) => patch({ lossless: v })}
          />
        </>
      )}
      {video && (
        <>
          {o.targetBytes != null && (
            <p className="hint">
              {l("Orçamento de saída", "Output budget")}: {bytes(o.targetBytes)}
              .{" "}
              {l(
                "A compressão usa CPU em duas passagens. Alterar CRF, codec ou processamento passa para qualidade manual.",
                "Compression uses two CPU passes. Changing CRF, codec or processing switches to manual quality.",
              )}
            </p>
          )}
          {choice("codec", l("Codec de vídeo", "Video codec"), [
            "auto",
            "h264",
            "h265",
            "av1",
            "prores",
          ])}
          {choice("encoder", l("Processamento", "Processing"), [
            "auto",
            "software",
            ...hardware,
          ])}
          {hardware.length === 0 && (
            <p className="hint">
              {l(
                "Nenhum encoder GPU disponível. CPU pronta.",
                "No GPU encoder available. CPU ready.",
              )}
            </p>
          )}
          <div className="field-pair">
            {number("crf", "CRF · 0–51", 0, 51)}
            {number(
              "fps",
              l("Quadros por segundo", "Frames per second"),
              1,
              240,
              true,
            )}
          </div>
          {number(
            "audioBitrate",
            l("Áudio · kbps por faixa", "Audio · kbps per stream"),
            16,
            512,
          )}
          <Toggle
            label={l("Preservar legendas", "Preserve subtitles")}
            value={o.subtitles}
            onChange={(v) => patch({ subtitles: v })}
          />
          <Toggle
            label={l("Preservar capítulos", "Preserve chapters")}
            value={o.chapters}
            onChange={(v) => patch({ chapters: v })}
          />
          {o.mode === "convert" && (
            <Toggle
              label={l(
                "Copiar fluxos sem recodificar",
                "Copy streams without re-encoding",
              )}
              value={o.remux}
              onChange={(v) => patch({ remux: v })}
            />
          )}
        </>
      )}
      {audio && (
        <>
          {number("bitrate", l("Bitrate · kbps", "Bitrate · kbps"), 32, 512)}
          <div className="field-pair">
            {number("sampleRate", "Hz", 8000, 192000, true)}
            {number("channels", l("Canais", "Channels"), 1, 8, true)}
          </div>
          {choice("bitDepth", l("Profundidade (PCM)", "Bit depth (PCM)"), [
            "16",
            "24",
          ])}
          <Toggle
            label={l(
              "Normalizar volume (−16 LUFS)",
              "Normalize loudness (−16 LUFS)",
            )}
            value={o.normalize}
            onChange={(v) => patch({ normalize: v })}
          />
          <Toggle
            label={l("Preservar capa", "Preserve artwork")}
            value={o.keepArtwork}
            onChange={(v) => patch({ keepArtwork: v })}
          />
        </>
      )}
      {file.category === "archive" && (
        <>
          {choice("archiveAction", l("Operação", "Operation"), [
            "repack",
            "extract",
          ])}
          {number("archiveLevel", l("Nível · 0–9", "Level · 0–9"), 0, 9)}
          <label className="field">
            <span>{l("Senha", "Password")}</span>
            <input
              type="password"
              value={o.password}
              onChange={(e) => patch({ password: e.target.value })}
            />
          </label>
        </>
      )}
      {file.format === "pdf" &&
        o.format !== "pdf" &&
        number(
          "dpi",
          l("Resolução das páginas · DPI", "Page resolution · DPI"),
          36,
          1200,
        )}
      {(image || audio || video) && (
        <Toggle
          label={l("Preservar metadados", "Preserve metadata")}
          value={o.keepMetadata}
          onChange={(v) => patch({ keepMetadata: v })}
        />
      )}
      <label className="field">
        <span>{l("Sufixo do arquivo", "Filename suffix")}</span>
        <input
          value={o.suffix}
          maxLength={64}
          placeholder="_reshape"
          onChange={(e) => patch({ suffix: e.target.value })}
        />
      </label>
      <p className="hint">
        {l(
          "Originais preservados. Conflitos recebem numeração automática.",
          "Originals are preserved. Conflicting names receive a number.",
        )}
      </p>
    </div>
  );
}
