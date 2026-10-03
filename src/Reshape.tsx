import { messageText } from "./messages";
import { useCallback, useEffect, useState, useRef } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import {
  AnimatePresence,
  LayoutGroup,
  MotionConfig,
  motion,
} from "motion/react";
import {
  ArrowRight,
  ArrowDown,
  Plus,
  X,
  FolderOpen,
  Settings2,
  Clock3,
  ListTodo,
  SlidersHorizontal,
  Bookmark,
  Search,
  LoaderCircle,
  Check,
  Star,
  Trash2,
  Upload,
  ChevronRight,
} from "lucide-react";
import {
  defaults,
  type Snapshot,
  type Draft,
  type InputFile,
  type Plan,
  type ConversionOptions,
  type ConversionJob,
  type SavedPreset,
  type Settings,
} from "./types";
import {
  Button,
  Mark,
  FileIcon,
  Modal,
  Picker,
  Toggle,
  NumberValue,
  bytes,
  time,
  type L,
} from "./ui";
import Advanced from "./Advanced";
import Comparison, { Thumbnail } from "./Comparison";
import Jobs from "./Jobs";
const initial: Snapshot = {
  jobs: [],
  history: [],
  presets: [],
  registry: [],
  hardware: [],
  engines: [],
  paused: false,
  settings: {
    theme: "dark",
    language: "en",
    outputDir: "",
    keepMetadata: true,
    concurrency: 2,
    historyEnabled: true,
    tempDir: "",
  },
};
type Planned = { plan?: Plan; error?: string; signature: string };
export default function Reshape() {
  const [data, setData] = useState(initial),
    [ready, setReady] = useState(false),
    [mode, setMode] = useState<"convert" | "compress">("compress"),
    [drafts, setDrafts] = useState<Draft[]>([]),
    [selected, setSelected] = useState(""),
    [page, setPage] = useState<"work" | "queue" | "history">("work"),
    [overlay, setOverlay] = useState(""),
    [advanced, setAdvanced] = useState(false),
    [drag, setDrag] = useState(false),
    [busy, setBusy] = useState(false),
    [message, setMessage] = useState(""),
    [plans, setPlans] = useState<Record<string, Planned>>({}),
    [planning, setPlanning] = useState(false),
    [comparison, setComparison] = useState<ConversionJob | null>(null),
    [formatSearch, setFormatSearch] = useState(""),
    [presetName, setPresetName] = useState(""),
    [settings, setSettings] = useState<Settings>(initial.settings);
  const l: L = (pt, en) => (data.settings.language === "en" ? en : pt);
  const current = drafts.find((d) => d.file.path === selected) ?? drafts[0];
  const planFor = (d: Draft) => {
    const p = plans[d.file.path];
    return p?.signature === JSON.stringify(d.options) ? p : undefined;
  };
  const planned = current ? planFor(current) : undefined;
  const actual = planned?.plan?.options ?? current?.options;
  const notify = useCallback((s: string) => setMessage(s), []);
  const lastSnapshot = useRef("");
  const refreshing = useRef(false);
  const refresh = useCallback(async () => {
    if (!isTauri()) {
      setReady(true);
      return;
    }
    if (refreshing.current) return;
    refreshing.current = true;
    try {
      const next = await invoke<Snapshot>("snapshot");
      const signature = JSON.stringify(next);
      if (signature !== lastSnapshot.current) {
        lastSnapshot.current = signature;
        document.documentElement.lang =
          next.settings.language === "en" ? "en" : "pt-BR";
        setData(next);
      }
      setReady(true);
    } catch (e) {
      setMessage(String(e));
    } finally {
      refreshing.current = false;
    }
  }, []);
  useEffect(() => {
    void refresh();
    const t = setInterval(() => void refresh(), 800);
    return () => clearInterval(t);
  }, [refresh]);
  useEffect(() => {
    document.documentElement.dataset.theme = data.settings.theme;
    document.documentElement.lang =
      data.settings.language === "en" ? "en" : "pt-BR";
  }, [data.settings.theme, data.settings.language]);
  useEffect(() => {
    if (!message) return;
    const t = setTimeout(() => setMessage(""), 12000);
    return () => clearTimeout(t);
  }, [message]);
  const importFiles = useCallback(
    async (paths: string[]) => {
      if (!paths.length) return;
      setBusy(true);
      try {
        const r = await invoke<{
          files: InputFile[];
          errors: string[];
          truncated: boolean;
        }>("inspect", { paths, recursive: true });
        setDrafts((old) => [
          ...old,
          ...r.files
            .filter((f) => !old.some((d) => d.file.path === f.path))
            .map((file) => {
              const cap = data.registry.find((r) => r.input === file.format);
              const preferred =
                file.category === "image"
                  ? "webp"
                  : file.category === "audio"
                    ? "flac"
                    : file.category === "video"
                      ? "mp4"
                      : file.format === "pdf"
                        ? "pdf"
                        : (cap?.outputs[0] ?? "zip");
              return {
                file,
                options: {
                  ...defaults,
                  format: preferred,
                  mode: mode === "compress" ? "smart" : "convert",
                  preset: "balanced",
                  quality: 72,
                  crf: 27,
                  bitrate: 128,
                  encoder: "auto",
                  outputDir: data.settings.outputDir,
                  keepMetadata: data.settings.keepMetadata,
                  subtitles: file.media?.subtitleCodecs.length ? true : false,
                },
              };
            }),
        ]);
        setPage("work");
        if (r.errors.length) notify(r.errors.join("\n"));
        if (r.truncated)
          notify(
            l(
              "Importação limitada a 1.000 arquivos.",
              "Import limited to 1,000 files.",
            ),
          );
      } catch (e) {
        notify(String(e));
      } finally {
        setBusy(false);
      }
    },
    [data.registry, data.settings, mode, notify],
  );
  const importRef = useRef(importFiles);
  importRef.current = importFiles;
  const startupRead = useRef(false);
  useEffect(() => {
    if (!ready || !data.registry.length || startupRead.current || !isTauri())
      return;
    startupRead.current = true;
    void invoke<string[]>("startup_paths")
      .then((paths) => {
        if (paths.length) void importRef.current(paths);
      })
      .catch((e) => notify(String(e)));
  }, [ready, data.registry.length, notify]);
  useEffect(() => {
    if (!isTauri()) return;
    const listen = getCurrentWebview().onDragDropEvent((e) => {
      setDrag(e.payload.type === "enter" || e.payload.type === "over");
      if (e.payload.type === "drop") void importRef.current(e.payload.paths);
    });
    return () => {
      void listen.then((f) => f());
    };
  }, []);
  useEffect(() => {
    const paste = (e: ClipboardEvent) => {
      if ((e.target as HTMLElement)?.closest("input,textarea")) return;
      const text = e.clipboardData?.getData("text/plain") ?? "";
      const paths = text
        .split(/\r?\n/)
        .map((p) => p.trim().replace(/^"|"$/g, ""))
        .filter((p) => /^(?:[A-Za-z]:[\\/]|\\\\)/.test(p));
      if (paths.length) {
        e.preventDefault();
        void importRef.current(paths);
      }
    };
    document.addEventListener("paste", paste);
    return () => document.removeEventListener("paste", paste);
  }, []);
  const hardwareKey = data.hardware.join(",");
  useEffect(() => {
    if (
      !current ||
      !planned?.plan ||
      planning ||
      data.jobs.some((j) => j.status === "processing")
    )
      return;
    const f = current.file,
      p = planned.plan;
    if (f.category !== "image" || p.estimate.method === "measured") return;
    let live = true;
    const t = setTimeout(() => {
      void invoke<Plan["estimate"]>("refine_estimate", {
        input: f,
        options: p.options,
      })
        .then((estimate) => {
          if (live)
            setPlans((old) => ({
              ...old,
              [f.path]: { ...old[f.path], plan: { ...p, estimate } },
            }));
        })
        .catch(() => {});
    }, 900);
    return () => {
      live = false;
      clearTimeout(t);
    };
  }, [current?.file.path, JSON.stringify(actual), planning]);
  useEffect(() => {
    let live = true;
    if (!drafts.length) {
      setPlans({});
      setPlanning(false);
      return;
    }
    setPlanning(true);
    const t = setTimeout(() => {
      void (async () => {
        const next: Record<string, Planned> = {};
        for (const d of drafts) {
          if (!live) return;
          try {
            next[d.file.path] = {
              signature: JSON.stringify(d.options),
              plan: await invoke<Plan>("transformation_plan", {
                input: d.file,
                options: d.options,
              }),
            };
          } catch (e) {
            next[d.file.path] = {
              error: String(e),
              signature: JSON.stringify(d.options),
            };
          }
        }
        if (live) {
          setPlans(next);
          setPlanning(false);
        }
      })();
    }, 180);
    return () => {
      live = false;
      clearTimeout(t);
    };
  }, [drafts, hardwareKey]);
  const choose = async (directory = false) => {
    try {
      const r = await open({ multiple: !directory, directory });
      if (r) await importFiles(Array.isArray(r) ? r : [r]);
    } catch (e) {
      notify(String(e));
    }
  };
  const changeMode = (next: "convert" | "compress") => {
    if (next === mode && page === "work") return;
    setMode(next);
    setPage("work");
    setDrafts((ds) =>
      ds.map((d) => ({
        ...d,
        options: {
          ...d.options,
          mode: next === "compress" ? "smart" : "convert",
          targetBytes: null,
          lossless: false,
          remux: false,
        },
      })),
    );
  };
  const patch = (v: Partial<ConversionOptions>) => {
    if (!current) return;
    setDrafts((ds) =>
      ds.map((d) =>
        d.file.path === current.file.path
          ? {
              ...d,
              options: {
                ...(planFor(d)?.plan?.options ?? d.options),
                encoder: d.options.encoder,
                ...(v.crf !== undefined ||
                v.encoder !== undefined ||
                v.codec !== undefined ||
                v.format !== undefined
                  ? { targetBytes: null }
                  : {}),
                ...v,
                mode: mode === "convert" ? "convert" : "custom",
                preset: "custom",
              },
            }
          : d,
      ),
    );
  };
  const canCopy = (format: string) =>
    current?.file.category === "video" &&
    mode === "convert" &&
    (format === "mkv" ||
      (["mp4", "mov"].includes(format) &&
        ["h264", "hevc", "av1", "mpeg4"].includes(
          current.file.media?.codec ?? "",
        ) &&
        [null, undefined, "aac", "mp3", "alac", "ac3"].includes(
          current.file.media?.audioCodec,
        ) &&
        !current.file.media?.subtitleCodecs.some((s) => s !== "mov_text")));
  const selectFormat = (format: string) =>
    patch({
      format,
      remux: canCopy(format),
      lossless: false,
      width: null,
      height: null,
      fps: null,
    });
  const preset = (p: string) =>
    setDrafts((ds) =>
      ds.map((d) => ({
        ...d,
        options: {
          ...d.options,
          mode: mode === "compress" ? "smart" : "convert",
          preset: p,
          quality: p === "high" ? 85 : p === "small" ? 50 : 72,
          crf: p === "high" ? 23 : p === "small" ? 32 : 27,
          bitrate: p === "high" ? 192 : p === "small" ? 96 : 128,
          targetBytes: null,
          lossless: false,
          remux: false,
        },
      })),
    );
  const start = async () => {
    if (busy || planning) return;
    setBusy(true);
    const sent = new Set<string>();
    const errors: string[] = [];
    for (const d of drafts) {
      try {
        await invoke("enqueue", { input: d.file, options: d.options });
        sent.add(d.file.path);
      } catch (e) {
        errors.push(`${d.file.name}: ${e}`);
      }
    }
    setDrafts((ds) => ds.filter((d) => !sent.has(d.file.path)));
    if (sent.size) setPage("queue");
    if (errors.length) notify(errors.join("\n"));
    setBusy(false);
    await refresh();
  };
  const combine = async (format: "pdf" | "zip") => {
    if (!current || busy) return;
    setBusy(true);
    try {
      await invoke("enqueue", {
        input: drafts[0].file,
        options: {
          ...defaults,
          mode: "convert",
          format,
          archiveAction: "create",
          outputDir: current.options.outputDir,
          mergePaths: drafts.slice(1).map((d) => d.file.path),
        },
      });
      setDrafts([]);
      setPage("queue");
      await refresh();
    } catch (e) {
      notify(String(e));
    } finally {
      setBusy(false);
    }
  };
  const outputFolder = async () => {
    try {
      const dir = await open({ directory: true });
      if (typeof dir === "string") {
        setDrafts((ds) =>
          ds.map((d) => ({ ...d, options: { ...d.options, outputDir: dir } })),
        );
        await invoke("save_settings", {
          settings: { ...data.settings, outputDir: dir },
        });
        await refresh();
      }
    } catch (e) {
      notify(String(e));
    }
  };
  const savePresets = async (presets: SavedPreset[]) => {
    try {
      await invoke("save_presets", { presets });
      await refresh();
    } catch (e) {
      notify(String(e));
    }
  };
  const allPlans = drafts.map((d) => planFor(d)?.plan);
  const total = drafts.reduce((n, d) => n + d.file.size, 0);
  const low = allPlans.reduce((n, p) => n + (p?.estimate.low ?? 0), 0),
    high = allPlans.reduce((n, p) => n + (p?.estimate.high ?? 0), 0);
  const center = (low + high) / 2;
  const planError = drafts.some((d) => planFor(d)?.error);
  const keepsOriginal =
    allPlans.length > 0 &&
    allPlans.every((p) => p?.estimate.method === "original");
  const cap = current
    ? data.registry.find((r) => r.input === current.file.format)
    : undefined;
  const formats = [
    ...new Set([...(cap?.outputs ?? []), ...(cap?.archiveOutputs ?? [])]),
  ];
  const activeJobs = data.jobs.filter((j) =>
    ["pending", "processing", "cancelling"].includes(j.status),
  );
  const closeOverlay = useCallback(() => setOverlay(""), []),
    closeComparison = useCallback(() => setComparison(null), []);
  const recommended = planned?.plan?.recommendations ?? formats.slice(0, 3);
  return (
    <MotionConfig
      reducedMotion="never"
      transition={{ type: "spring", stiffness: 380, damping: 34, mass: 0.9 }}
    >
      <LayoutGroup>
        <div className="app-shell">
          <header className="app-header">
            <button
              className="wordmark"
              onClick={() => setPage("work")}
              aria-label={l("Reshape início", "Reshape home")}
            >
              <Mark />
              <span>reshape</span>
              <small>2.0.4</small>
            </button>
            <div className="mode-switch" aria-label={l("Modo", "Mode")}>
              {(["convert", "compress"] as const).map((m) => (
                <button
                  key={m}
                  aria-pressed={mode === m}
                  onClick={() => changeMode(m)}
                >
                  {mode === m && (
                    <motion.span
                      layoutId="mode-selector"
                      className="selection"
                    />
                  )}
                  <span>
                    {m === "convert"
                      ? l("Converter", "Convert")
                      : l("Comprimir", "Compress")}
                  </span>
                </button>
              ))}
            </div>
            <nav>
              <Button
                title={l("Fila", "Queue")}
                onClick={() => setPage(page === "queue" ? "work" : "queue")}
                className={page === "queue" ? "active" : ""}
              >
                <ListTodo size={19} />
                {activeJobs.length > 0 && (
                  <span className="count">{activeJobs.length}</span>
                )}
              </Button>
              <Button
                title={l("Histórico", "History")}
                onClick={() => setPage(page === "history" ? "work" : "history")}
              >
                <Clock3 size={18} />
              </Button>
              <Button
                title={l("Configurações", "Settings")}
                onClick={() => {
                  setSettings(data.settings);
                  setOverlay("settings");
                }}
              >
                <Settings2 size={18} />
              </Button>
            </nav>
          </header>
          <main>
            <AnimatePresence mode="wait">
              {!ready ? (
                <motion.div key="start" className="quiet-empty">
                  <Mark large />
                  <p>
                    {l("Preparando seu espaço…", "Preparing your workspace…")}
                  </p>
                </motion.div>
              ) : page !== "work" ? (
                <motion.div
                  key={page}
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -8 }}
                >
                  <Jobs
                    data={data}
                    l={l}
                    compare={setComparison}
                    notify={notify}
                    history={page === "history"}
                  />
                </motion.div>
              ) : (
                <motion.div
                  key="work"
                  layout="position"
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -6 }}
                  className={`work ${drafts.length ? "loaded" : ""}`}
                >
                  <motion.section
                    layout="position"
                    layoutId="workspace"
                    className={`workspace ${drag ? "drag-over" : ""} ${drafts.length ? "has-files" : ""}`}
                    style={{ borderRadius: 28 }}
                  >
                    {!current ? (
                      <motion.div
                        key="empty"
                        className="empty"
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                      >
                        <div className="brand-atmosphere" aria-hidden="true">
                          <i />
                          <i />
                          <i />
                        </div>
                        <motion.div
                          layoutId="file-object"
                          className="empty-symbol"
                          whileHover={{ rotate: -3, y: -4 }}
                        >
                          <Mark large />
                        </motion.div>
                        <h1>
                          {drag
                            ? l("Solte aqui.", "Drop them here.")
                            : l(
                                "Seus arquivos.\nNovas possibilidades.",
                                "Your files.\nReimagined.",
                              )}
                        </h1>
                        <p>
                          {l(
                            "Arraste seus arquivos para começar.",
                            "Drop your files to get started.",
                          )}
                        </p>
                        <Button
                          primary
                          onClick={() => void choose()}
                          disabled={busy}
                        >
                          <Plus size={18} />
                          {busy
                            ? l("Lendo arquivos…", "Reading files…")
                            : l("Escolher arquivos", "Choose files")}
                        </Button>
                        <button
                          className="text-button"
                          onClick={() => void choose(true)}
                        >
                          {l("ou adicionar uma pasta", "or add a folder")}
                          <ChevronRight size={13} />
                        </button>
                        <span className="privacy">
                          {l(
                            "Tudo acontece no seu computador.",
                            "Everything happens on your computer.",
                          )}
                        </span>
                      </motion.div>
                    ) : (
                      <>
                        <div className="workspace-top">
                          <span className="eyebrow">
                            {drafts.length > 1
                              ? `${drafts.length} ${l("ARQUIVOS", "FILES")}`
                              : l("SEU ARQUIVO", "YOUR FILE")}
                          </span>
                          <div>
                            {drafts.length > 1 && (
                              <Button
                                disabled={busy}
                                onClick={() => void combine("zip")}
                              >
                                {l("Agrupar ZIP", "Bundle ZIP")}
                              </Button>
                            )}
                            {drafts.length > 1 &&
                              drafts.every(
                                (d) => d.file.category === "image",
                              ) && (
                                <Button
                                  disabled={busy}
                                  onClick={() => void combine("pdf")}
                                >
                                  {l("Juntar PDF", "Merge PDF")}
                                </Button>
                              )}
                            <Button
                              title={l("Adicionar arquivos", "Add files")}
                              onClick={() => void choose()}
                            >
                              <Plus size={17} />
                            </Button>
                            <Button
                              title={l("Adicionar pasta", "Add folder")}
                              onClick={() => void choose(true)}
                            >
                              <FolderOpen size={17} />
                            </Button>
                            <Button
                              title={l("Remover todos", "Remove all")}
                              onClick={() => setDrafts([])}
                            >
                              <X size={17} />
                            </Button>
                          </div>
                        </div>
                        <div className="file-area">
                          {drafts.length > 1 && (
                            <motion.div layoutScroll className="batch-rail">
                              {drafts.map((d) => (
                                <button
                                  className={
                                    current.file.path === d.file.path
                                      ? "selected"
                                      : ""
                                  }
                                  key={d.file.path}
                                  onClick={() => setSelected(d.file.path)}
                                >
                                  <FileIcon category={d.file.category} />
                                  <span>
                                    <b>{d.file.name}</b>
                                    <small>
                                      {bytes(d.file.size)}
                                      {plans[d.file.path]?.error ? " · !" : ""}
                                    </small>
                                  </span>
                                </button>
                              ))}
                            </motion.div>
                          )}
                          <motion.div
                            key={current.file.path}
                            className="file-focus"
                            initial={{ opacity: 0, y: 6 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ duration: 0.22, ease: "easeOut" }}
                          >
                            <motion.div
                              layoutId="file-object"
                              className={`file-preview ${current.file.category}`}
                            >
                              <FileIcon category={current.file.category} />
                              <Thumbnail file={current.file} />
                              <span className="format-stamp">
                                {current.file.format.toUpperCase()}
                              </span>
                            </motion.div>
                            <h1 title={current.file.name}>
                              {current.file.name}
                            </h1>
                            <p className="file-meta">
                              {bytes(current.file.size)}
                              {current.file.width &&
                                ` · ${current.file.width} × ${current.file.height}`}
                              {current.file.duration &&
                                ` · ${time(current.file.duration)}`}
                              {current.file.media?.pages &&
                                ` · ${current.file.media.pages} ${l("páginas", "pages")}`}
                              {current.file.media?.hdr && " · HDR"}
                            </p>
                            {drafts.length > 1 && (
                              <button
                                className="text-button"
                                onClick={() =>
                                  setDrafts((ds) =>
                                    ds.filter(
                                      (d) => d.file.path !== current.file.path,
                                    ),
                                  )
                                }
                              >
                                {l("Remover este arquivo", "Remove this file")}
                              </button>
                            )}
                          </motion.div>
                        </div>
                        <div className="transform-controls">
                          <motion.div
                            layout="position"
                            className="control-heading"
                          >
                            <span className="eyebrow">
                              {mode === "compress"
                                ? l(
                                    "ENCONTRE O EQUILÍBRIO",
                                    "FIND YOUR BALANCE",
                                  )
                                : l("CONVERTER PARA", "CONVERT TO")}
                            </span>
                            <div>
                              <Button
                                title={l("Presets salvos", "Saved presets")}
                                onClick={() => setOverlay("presets")}
                              >
                                <Bookmark size={16} />
                              </Button>
                              <Button
                                onClick={() => setAdvanced(!advanced)}
                                className={advanced ? "active" : ""}
                              >
                                <SlidersHorizontal size={15} />
                                {l("Avançado", "Advanced")}
                              </Button>
                            </div>
                          </motion.div>
                          {mode === "convert" && (
                            <div className="formats">
                              {recommended.slice(0, 4).map((f) => (
                                <button
                                  key={f}
                                  className={
                                    actual?.format === f ? "chosen" : ""
                                  }
                                  onClick={() => selectFormat(f)}
                                >
                                  {f.toUpperCase()}
                                </button>
                              ))}
                              <button
                                onClick={() => {
                                  setFormatSearch("");
                                  setOverlay("formats");
                                }}
                              >
                                {l("Mais formatos", "More formats")}
                                <Plus size={14} />
                              </button>
                            </div>
                          )}
                          <div className="presets">
                            {[
                              [
                                "high",
                                l("Melhor qualidade", "Best quality"),
                                l("Mais detalhe", "More detail"),
                              ],
                              [
                                "balanced",
                                l("Equilibrado", "Balanced"),
                                l("O ponto ideal", "The sweet spot"),
                              ],
                              [
                                "small",
                                l("Menor tamanho", "Smallest"),
                                l("Mais compressão", "More compression"),
                              ],
                            ].map(([key, name, sub]) => (
                              <button
                                key={key}
                                aria-pressed={current.options.preset === key}
                                onClick={() => preset(key)}
                              >
                                {current.options.preset === key && (
                                  <motion.span
                                    layoutId="preset-selector"
                                    className="preset-selection"
                                  />
                                )}
                                <span>
                                  <strong>{name}</strong>
                                  <small>{sub}</small>
                                </span>
                                {current.options.preset === key && (
                                  <Check size={15} />
                                )}
                              </button>
                            ))}
                          </div>
                          {mode === "compress" && (
                            <div className="compression-kind">
                              {!drafts.some(
                                (d) => d.file.category === "video",
                              ) && (
                                <button
                                  className={
                                    current.options.mode === "lossless"
                                      ? "selected"
                                      : ""
                                  }
                                  onClick={() =>
                                    setDrafts((ds) =>
                                      ds.map((d) => ({
                                        ...d,
                                        options: {
                                          ...d.options,
                                          mode:
                                            d.options.mode === "lossless"
                                              ? "smart"
                                              : "lossless",
                                          lossless:
                                            d.options.mode !== "lossless",
                                          remux: false,
                                        },
                                      })),
                                    )
                                  }
                                >
                                  {l("Sem perdas", "Lossless")}
                                </button>
                              )}
                              {["audio", "video", "image"].includes(
                                current.file.category,
                              ) && (
                                <button
                                  className={
                                    current.options.mode === "target"
                                      ? "selected"
                                      : ""
                                  }
                                  onClick={() =>
                                    setDrafts((ds) =>
                                      ds.map((d) =>
                                        d.file.path === current.file.path
                                          ? {
                                              ...d,
                                              options: {
                                                ...d.options,
                                                mode:
                                                  d.options.mode === "target"
                                                    ? "smart"
                                                    : "target",
                                                targetBytes: Math.round(
                                                  d.file.size * 0.6,
                                                ),
                                                lossless: false,
                                                remux: false,
                                              },
                                            }
                                          : d,
                                      ),
                                    )
                                  }
                                >
                                  {l("Tamanho alvo", "Target size")}
                                </button>
                              )}
                              <span>
                                {actual?.format.toUpperCase()}{" "}
                                {actual?.lossless
                                  ? `· ${l("sem perdas", "lossless")}`
                                  : ["image", "audio", "video"].includes(
                                        current.file.category,
                                      )
                                    ? `· ${l("com perdas", "lossy")}`
                                    : ""}
                              </span>
                            </div>
                          )}
                          {current.options.mode === "target" && (
                            <label className="target-field">
                              {l("Tamanho aproximado", "Approximate size")}
                              <input
                                type="number"
                                min="0.01"
                                step="0.1"
                                value={
                                  (current.options.targetBytes ?? 0) / 1048576
                                }
                                onChange={(e) =>
                                  setDrafts((ds) =>
                                    ds.map((d) =>
                                      d.file.path === current.file.path
                                        ? {
                                            ...d,
                                            options: {
                                              ...d.options,
                                              targetBytes: Math.round(
                                                Number(e.target.value) *
                                                  1048576,
                                              ),
                                            },
                                          }
                                        : d,
                                    ),
                                  )
                                }
                              />
                              <span>MB</span>
                            </label>
                          )}
                          <AnimatePresence initial={false}>
                            {advanced && actual && (
                              <motion.div
                                key="advanced"
                                initial={{ height: 0, opacity: 0 }}
                                animate={{ height: "auto", opacity: 1 }}
                                exit={{ height: 0, opacity: 0 }}
                                transition={{
                                  height: {
                                    duration: 0.32,
                                    ease: [0.22, 1, 0.36, 1],
                                  },
                                  opacity: { duration: 0.18 },
                                }}
                                style={{ overflow: "clip" }}
                              >
                                <Advanced
                                  formats={formats}
                                  file={current.file}
                                  o={{
                                    ...actual,
                                    encoder: current.options.encoder,
                                  }}
                                  patch={patch}
                                  hardware={data.hardware}
                                  l={l}
                                />
                              </motion.div>
                            )}
                          </AnimatePresence>
                          {planned?.error && (
                            <p className="notice error-text" role="alert">
                              {messageText(planned.error)}
                            </p>
                          )}
                          {planned?.plan?.warnings.map((w) => (
                            <p className="plan-note" key={w}>
                              {messageText(w)}
                            </p>
                          ))}
                          <motion.div
                            layout="position"
                            className="estimate"
                            aria-busy={planning}
                          >
                            <div>
                              <span className="eyebrow">
                                {drafts.length > 1
                                  ? l("LOTE COMPLETO", "ENTIRE BATCH")
                                  : l("TAMANHO ESTIMADO", "ESTIMATED SIZE")}
                              </span>
                              <div className="size-line">
                                <span>{bytes(total)}</span>
                                <ArrowRight size={21} />
                                <strong>
                                  {planning ? (
                                    <span className="calculating">
                                      {l("Calculando…", "Calculating…")}
                                    </span>
                                  ) : allPlans.every(Boolean) ? (
                                    <>
                                      {!keepsOriginal && "~"}
                                      <NumberValue value={center} />
                                    </>
                                  ) : (
                                    "—"
                                  )}
                                </strong>
                              </div>
                              <p>
                                {keepsOriginal && !planning
                                  ? l(
                                      "Sem redução prevista. Seu original será mantido se o resultado não for menor.",
                                      "No reduction expected. Your original is kept if the result is not smaller.",
                                    )
                                  : !planning && allPlans.every(Boolean)
                                    ? `${bytes(low)} – ${bytes(high)} · ${allPlans.some((p) => p?.estimate.method === "heuristic") ? l("estimativa ampla; depende do conteúdo", "broad estimate; depends on content") : allPlans.every((p) => p?.estimate.method === "measured") ? l("medição local refinada", "refined local measurement") : l("estimativa aproximada", "approximate estimate")}`
                                    : l(
                                        "Planejando a transformação localmente.",
                                        "Planning your transformation locally.",
                                      )}
                              </p>
                            </div>
                            {!planning &&
                              allPlans.every(Boolean) &&
                              center > 0 &&
                              center < total && (
                                <div className="estimate-saving">
                                  <strong>
                                    ~
                                    <NumberValue
                                      value={(1 - center / total) * 100}
                                      format={(v) => `${Math.round(v)}%`}
                                    />
                                  </strong>
                                  <small>{l("menor", "smaller")}</small>
                                </div>
                              )}
                          </motion.div>
                          <div className="destination">
                            <FolderOpen size={15} />
                            <button
                              onClick={() => void outputFolder()}
                              title={current.options.outputDir}
                            >
                              {current.options.outputDir
                                ? current.options.outputDir
                                : l(
                                    "Mesma pasta do original",
                                    "Same folder as original",
                                  )}
                            </button>
                            {current.options.outputDir && (
                              <button
                                title={l(
                                  "Usar pasta original",
                                  "Use source folder",
                                )}
                                onClick={() => {
                                  setDrafts((ds) =>
                                    ds.map((d) => ({
                                      ...d,
                                      options: { ...d.options, outputDir: "" },
                                    })),
                                  );
                                  void invoke("save_settings", {
                                    settings: {
                                      ...data.settings,
                                      outputDir: "",
                                    },
                                  }).catch((e) => notify(String(e)));
                                }}
                              >
                                <X size={12} />
                              </button>
                            )}
                            <span>
                              {l(
                                "Originais preservados",
                                "Originals preserved",
                              )}
                            </span>
                          </div>
                          <Button
                            primary
                            className="start-button"
                            onClick={() => void start()}
                            disabled={
                              busy ||
                              planning ||
                              planError ||
                              !allPlans.every(Boolean)
                            }
                          >
                            {busy ? (
                              <LoaderCircle className="spin" size={18} />
                            ) : (
                              <ArrowDown size={18} />
                            )}{" "}
                            {mode === "compress"
                              ? l("Comprimir", "Compress")
                              : l("Converter", "Convert")}
                            {drafts.length > 1 &&
                              ` ${drafts.length} ${l("arquivos", "files")}`}
                            <ArrowRight size={18} />
                          </Button>
                        </div>
                      </>
                    )}
                  </motion.section>
                  {drag && (
                    <div className="drop-caption">
                      <Upload size={22} />
                      {l("Solte para adicionar", "Drop to add")}
                    </div>
                  )}
                </motion.div>
              )}
            </AnimatePresence>
          </main>
          <footer className="app-footer">
            <span>
              <i />
              {l("Local. Privado. Seu.", "Local. Private. Yours.")}
            </span>
            <button onClick={() => setOverlay("about")}>Reshape 2.0.4</button>
            {activeJobs.length > 0 && (
              <button className="queue-pill" onClick={() => setPage("queue")}>
                <Mark />
                {activeJobs.length} {l("na fila", "in queue")}
                <ChevronRight size={14} />
              </button>
            )}
          </footer>
          <AnimatePresence>
            {message && (
              <motion.div
                className="toast"
                role="status"
                initial={{ y: 25, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                exit={{ y: 12, opacity: 0 }}
              >
                <span>{messageText(message)}</span>
                <Button
                  title={l("Fechar", "Close")}
                  onClick={() => setMessage("")}
                >
                  <X size={16} />
                </Button>
              </motion.div>
            )}
          </AnimatePresence>
          <AnimatePresence>
            {overlay === "formats" && current && (
              <Modal
                title={l("Escolher formato", "Choose format")}
                close={closeOverlay}
              >
                <label className="search">
                  <Search size={18} />
                  <input
                    autoFocus
                    placeholder={l("Buscar formato", "Search formats")}
                    value={formatSearch}
                    onChange={(e) => setFormatSearch(e.target.value)}
                  />
                </label>
                <div className="format-list">
                  {formats
                    .filter((f) => f.includes(formatSearch.toLowerCase()))
                    .map((f) => (
                      <button
                        key={f}
                        onClick={() => {
                          selectFormat(f);
                          setOverlay("");
                        }}
                      >
                        <strong>{f.toUpperCase()}</strong>
                        <span>
                          {cap?.losslessOutputs.includes(f)
                            ? l(
                                "Pode preservar sem perdas",
                                "Can preserve losslessly",
                              )
                            : l("Conversão local", "Local conversion")}
                        </span>
                        <ArrowRight size={15} />
                      </button>
                    ))}
                </div>
              </Modal>
            )}
            {overlay === "presets" && current && (
              <Modal
                title={l("Seus presets", "Your presets")}
                close={closeOverlay}
              >
                <p className="hint">
                  {l(
                    "Configurações desta categoria. Pastas e senhas não são salvas.",
                    "Settings for this category. Folders and passwords are not saved.",
                  )}
                </p>
                <div className="save-preset">
                  <input
                    placeholder={l("Nome do novo preset", "New preset name")}
                    maxLength={80}
                    value={presetName}
                    onChange={(e) => setPresetName(e.target.value)}
                  />
                  <Button
                    primary
                    disabled={!presetName.trim() || !actual}
                    onClick={() => {
                      void savePresets([
                        ...(data.presets ?? []),
                        {
                          id: crypto.randomUUID(),
                          name: presetName,
                          category: current.file.category,
                          favorite: false,
                          options: actual!,
                        },
                      ]);
                      setPresetName("");
                    }}
                  >
                    <Plus size={17} />
                    {l("Salvar", "Save")}
                  </Button>
                </div>
                {[...(data.presets ?? [])]
                  .filter((p) => p.category === current.file.category)
                  .sort((a, b) => Number(b.favorite) - Number(a.favorite))
                  .map((p) => (
                    <div className="saved-preset" key={p.id}>
                      <input
                        aria-label={l("Nome do preset", "Preset name")}
                        defaultValue={p.name}
                        onBlur={(e) => {
                          if (e.target.value !== p.name)
                            void savePresets(
                              (data.presets ?? []).map((v) =>
                                v.id === p.id
                                  ? { ...v, name: e.target.value }
                                  : v,
                              ),
                            );
                        }}
                      />
                      <div>
                        <Button
                          title={l("Favoritar", "Favorite")}
                          onClick={() =>
                            void savePresets(
                              (data.presets ?? []).map((v) =>
                                v.id === p.id
                                  ? { ...v, favorite: !v.favorite }
                                  : v,
                              ),
                            )
                          }
                        >
                          <Star
                            size={15}
                            fill={p.favorite ? "currentColor" : "none"}
                          />
                        </Button>
                        <Button
                          onClick={() => {
                            setDrafts((ds) =>
                              ds.map((d) =>
                                d.file.category === p.category
                                  ? {
                                      ...d,
                                      options: {
                                        ...p.options,
                                        outputDir: d.options.outputDir,
                                      },
                                    }
                                  : d,
                              ),
                            );
                            setMode(
                              p.options.mode === "convert"
                                ? "convert"
                                : "compress",
                            );
                            setOverlay("");
                          }}
                        >
                          {l("Aplicar", "Apply")}
                        </Button>
                        <Button
                          title={l(
                            "Atualizar com configuração atual",
                            "Update with current settings",
                          )}
                          onClick={() =>
                            void savePresets(
                              (data.presets ?? []).map((v) =>
                                v.id === p.id ? { ...v, options: actual! } : v,
                              ),
                            )
                          }
                        >
                          <Check size={15} />
                        </Button>
                        <Button
                          title={l("Excluir", "Delete")}
                          onClick={() =>
                            void savePresets(
                              (data.presets ?? []).filter((v) => v.id !== p.id),
                            )
                          }
                        >
                          <Trash2 size={15} />
                        </Button>
                      </div>
                    </div>
                  ))}
              </Modal>
            )}
            {overlay === "settings" && (
              <Modal
                title={l("Configurações", "Settings")}
                close={closeOverlay}
              >
                <div className="settings-fields">
                  <label className="field">
                    <span>{l("Aparência", "Appearance")}</span>
                    <Picker
                      label={l("Aparência", "Theme")}
                      value={settings.theme}
                      items={[
                        { value: "dark", label: l("Escuro", "Dark") },
                        { value: "light", label: l("Claro", "Light") },
                        { value: "system", label: l("Sistema", "System") },
                      ]}
                      onChange={(v) => setSettings({ ...settings, theme: v })}
                    />
                  </label>
                  <label className="field">
                    <span>{l("Idioma", "Language")}</span>
                    <Picker
                      label={l("Idioma", "Language")}
                      value={settings.language}
                      items={[
                        { value: "pt", label: "Português" },
                        { value: "en", label: "English" },
                      ]}
                      onChange={(v) =>
                        setSettings({ ...settings, language: v })
                      }
                    />
                  </label>
                  <label className="field">
                    <span>{l("Tarefas simultâneas", "Concurrent tasks")}</span>
                    <Picker
                      label={l("Tarefas simultâneas", "Concurrency")}
                      value={String(settings.concurrency)}
                      items={[1, 2, 3, 4].map((n) => ({
                        value: String(n),
                        label: String(n),
                      }))}
                      onChange={(v) =>
                        setSettings({ ...settings, concurrency: Number(v) })
                      }
                    />
                  </label>
                  <p className="hint">
                    {l(
                      "Vídeos são processados um por vez para limitar carga e memória.",
                      "Videos process one at a time to limit CPU and memory load.",
                    )}
                  </p>
                  <Toggle
                    label={l(
                      "Preservar metadados por padrão",
                      "Preserve metadata by default",
                    )}
                    value={settings.keepMetadata}
                    onChange={(v) =>
                      setSettings({ ...settings, keepMetadata: v })
                    }
                  />
                  <Toggle
                    label={l("Salvar histórico", "Save history")}
                    value={settings.historyEnabled}
                    onChange={(v) =>
                      setSettings({ ...settings, historyEnabled: v })
                    }
                  />
                  <Button
                    onClick={() =>
                      void invoke("clear_temporary")
                        .then((v) =>
                          notify(
                            `${v} ${l("pastas temporárias removidas", "temporary folders removed")}`,
                          ),
                        )
                        .catch((e) => notify(String(e)))
                    }
                  >
                    {l("Limpar arquivos temporários", "Clear temporary files")}
                  </Button>
                  <Button
                    onClick={() =>
                      void invoke("clear_history").then(() => {
                        void refresh();
                        notify(l("Histórico apagado", "History cleared"));
                      })
                    }
                  >
                    {l("Apagar histórico", "Clear history")}
                  </Button>
                  <Button
                    primary
                    onClick={() =>
                      void invoke("save_settings", { settings })
                        .then(() => {
                          void refresh();
                          setOverlay("");
                        })
                        .catch((e) => notify(String(e)))
                    }
                  >
                    {l("Salvar preferências", "Save preferences")}
                  </Button>
                </div>
              </Modal>
            )}
            {overlay === "about" && (
              <Modal title="Reshape 2.0.4" close={closeOverlay}>
                <div className="about-mark">
                  <Mark large />
                  <p>
                    {l(
                      "Uma nova forma para seus arquivos.",
                      "A new shape for your files.",
                    )}
                  </p>
                </div>
                <p className="hint">
                  {l(
                    "Conversão e compressão locais. Seus arquivos não são enviados para serviços externos.",
                    "Local conversion and compression. Your files are not sent to external services.",
                  )}
                </p>
                <div className="engine-list">
                  {data.engines.map((e) => (
                    <div key={e.id}>
                      <span>{e.id}</span>
                      <small>
                        {e.ready && e.integrity
                          ? l("Verificado", "Verified")
                          : l("Indisponível", "Unavailable")}
                      </small>
                    </div>
                  ))}
                </div>
                <p className="hint">
                  Tauri · React · Motion (MIT) · FFmpeg (GPL) · ImageMagick ·
                  7-Zip · LibreOffice · Pandoc · qpdf · Poppler.{" "}
                  {l(
                    "O código original do Reshape é MIT. Os motores mantêm suas próprias licenças e são baixados durante a instalação.",
                    "Reshape's original code is MIT. Processing engines retain their own licenses and are downloaded during setup.",
                  )}
                </p>
                <Button onClick={() => void invoke("open_licenses").catch((e) => notify(String(e)))}>
                  {l("Componentes de código aberto", "Open-source components")}
                </Button>
                <Button
                  onClick={() =>
                    void invoke("check_engines")
                      .then(() => {
                        void refresh();
                        notify(
                          l("Verificação concluída", "Verification complete"),
                        );
                      })
                      .catch((e) => notify(String(e)))
                  }
                >
                  {l("Verificar motores", "Verify engines")}
                </Button>
              </Modal>
            )}
            {comparison && (
              <Comparison job={comparison} close={closeComparison} l={l} />
            )}
          </AnimatePresence>
        </div>
      </LayoutGroup>
    </MotionConfig>
  );
}
