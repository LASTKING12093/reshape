import { messageText } from "./messages";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, useMotionValue } from "motion/react";
import { LoaderCircle, Minus, Plus } from "lucide-react";
import { type ConversionJob, type InputFile } from "./types";
import { Button, Modal, bytes, time, type L } from "./ui";
const cache = new Map<string, string>();
export async function previewImage(file: InputFile, position = 0) {
  const key = `${file.path}:${file.size}:${position}`;
  if (cache.has(key)) return cache.get(key)!;
  const image = await invoke<string>("preview", { input: file, position });
  if (cache.size >= 24) cache.delete(cache.keys().next().value!);
  cache.set(key, image);
  return image;
}
export function Thumbnail({ file }: { file: InputFile }) {
  const [src, set] = useState("");
  useEffect(() => {
    let live = true;
    set("");
    if (["image", "video", "pdf"].includes(file.category))
      void previewImage(file)
        .then((v) => {
          if (live) set(v);
        })
        .catch(() => {});
    return () => {
      live = false;
    };
  }, [file.path, file.size]);
  return src ? (
    <motion.img
      initial={{ opacity: 0, scale: 1.02 }}
      animate={{ opacity: 1, scale: 1 }}
      src={src}
      alt={file.name}
    />
  ) : null;
}
export default function Comparison({
  job,
  close,
  l,
}: {
  job: ConversionJob;
  close: () => void;
  l: L;
}) {
  const panX = useMotionValue(0),
    panY = useMotionValue(0);
  const [output, setOutput] = useState<InputFile | null>(null),
    [images, setImages] = useState<string[]>([]),
    [error, setError] = useState(""),
    [busy, setBusy] = useState(true),
    [position, setPosition] = useState(0),
    [split, setSplit] = useState(50),
    [zoom, setZoom] = useState(1);
  const video = job.input.category === "video",
    pdf = job.input.format === "pdf";
  useEffect(() => {
    let live = true;
    void invoke<{ files: InputFile[]; errors: string[] }>("inspect", {
      paths: [job.output],
      recursive: false,
    })
      .then((r) => {
        if (live) {
          if (r.files[0]) setOutput(r.files[0]);
          else setError(r.errors.join("\n"));
        }
      })
      .catch((e) => setError(String(e)));
    return () => {
      live = false;
    };
  }, [job.output]);
  useEffect(() => {
    if (!output) return;
    let live = true;
    setBusy(true);
    const timer = setTimeout(() => {
      void (async () => {
        try {
          const a = await previewImage(
            job.input,
            pdf ? position + 1 : position,
          );
          const b = await previewImage(output, pdf ? position + 1 : position);
          if (live) {
            setImages([a, b]);
            setError("");
          }
        } catch (e) {
          if (live) setError(String(e));
        } finally {
          if (live) setBusy(false);
        }
      })();
    }, 200);
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, [output, position, job.input, pdf]);
  return (
    <Modal title={l("Antes e depois", "Before & after")} close={close} wide>
      <div className="compare-meta">
        <div>
          <small>{l("ORIGINAL", "ORIGINAL")}</small>
          <strong>{bytes(job.input.size)}</strong>
          <span>
            {job.input.width} × {job.input.height}
          </span>
        </div>
        <div>
          <small>{l("RESULTADO", "RESULT")}</small>
          <strong>{bytes(job.outputSize ?? 0)}</strong>
          <span>
            {output?.width} × {output?.height}
          </span>
        </div>
        <b>
          {((1 - (job.outputSize ?? 0) / job.input.size) * 100).toFixed(1)}%{" "}
          {l("menor", "smaller")}
        </b>
      </div>
      <div className="comparison">
        <motion.div
          className="comparison-pan"
          drag={zoom > 1}
          dragConstraints={{ left: -300, right: 300, top: -200, bottom: 200 }}
          style={{ scale: zoom, x: panX, y: panY }}
        >
          {images.length === 2 && (
            <img src={images[1]} alt={l("Resultado", "Output")} />
          )}
        </motion.div>
        <div
          className="original-clip"
          style={{ clipPath: `inset(0 ${100 - split}% 0 0)` }}
        >
          <motion.div
            className="comparison-pan"
            style={{ scale: zoom, x: panX, y: panY }}
          >
            {images.length === 2 && (
              <img src={images[0]} alt={l("Original", "Original")} />
            )}
          </motion.div>
        </div>
        <div
          className="compare-line"
          style={{ left: `${split}%` }}
          role="slider"
          tabIndex={0}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={Math.round(split)}
          aria-label={l("Divisão antes e depois", "Before after split")}
          onPointerDown={(e) => {
            e.stopPropagation();
            e.currentTarget.setPointerCapture(e.pointerId);
          }}
          onPointerMove={(e) => {
            if (e.currentTarget.hasPointerCapture(e.pointerId)) {
              const r = e.currentTarget.parentElement!.getBoundingClientRect();
              setSplit(
                Math.max(
                  0,
                  Math.min(100, ((e.clientX - r.left) / r.width) * 100),
                ),
              );
            }
          }}
          onKeyDown={(e) => {
            if (["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) {
              e.preventDefault();
              setSplit(
                e.key === "Home"
                  ? 0
                  : e.key === "End"
                    ? 100
                    : Math.max(
                        0,
                        Math.min(
                          100,
                          split + (e.key === "ArrowRight" ? 2 : -2),
                        ),
                      ),
              );
            }
          }}
        >
          <span>↔</span>
        </div>
        <label className="before-label">{l("Original", "Original")}</label>
        <label className="after-label">Reshape</label>
        {busy && (
          <div className="preview-loading">
            <LoaderCircle className="spin" size={20} />
          </div>
        )}
      </div>
      {error && <p className="error-text">{messageText(error)}</p>}
      <div className="compare-tools">
        <Button
          onClick={() => {
            const next = Math.max(1, zoom - 0.5);
            setZoom(next);
            if (next === 1) {
              panX.set(0);
              panY.set(0);
            }
          }}
          title={l("Diminuir zoom", "Zoom out")}
        >
          <Minus size={16} />
        </Button>
        <span>{zoom * 100}%</span>
        <Button
          onClick={() => setZoom(Math.min(3, zoom + 0.5))}
          title={l("Aumentar zoom", "Zoom in")}
        >
          <Plus size={16} />
        </Button>
        <span className="hint">
          {l(
            "Arraste a divisão. Com zoom, arraste a imagem.",
            "Drag the divider. When zoomed, drag the image.",
          )}
        </span>
      </div>
      {(pdf || video) && (
        <label className="timeline">
          <span>
            {pdf ? `${l("Página", "Page")} ${position + 1}` : time(position)}
          </span>
          <input
            type="range"
            value={position}
            min="0"
            step={pdf ? 1 : 0.1}
            max={
              pdf
                ? Math.max(
                    0,
                    Math.min(
                      job.input.media?.pages ?? 1,
                      output?.media?.pages ?? 1,
                    ) - 1,
                  )
                : Math.max(
                    0,
                    Math.min(job.input.duration ?? 1, output?.duration ?? 1) -
                      0.1,
                  )
            }
            onChange={(e) => setPosition(Number(e.target.value))}
          />
        </label>
      )}
      <p className="hint">
        {video
          ? l(
              "Quadros reais no mesmo instante. A prévia não reproduz vídeo; use Abrir arquivo para assistir.",
              "Actual frames at the same timestamp. This preview does not play video; open the file to watch.",
            )
          : l(
              "Prévia limitada a 1600 px. Os arquivos originais permanecem intactos.",
              "Preview limited to 1600 px. Original files remain intact.",
            )}
      </p>
    </Modal>
  );
}
