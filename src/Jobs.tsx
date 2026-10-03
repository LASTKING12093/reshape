import { messageText } from "./messages";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, Reorder } from "motion/react";
import {
  FolderOpen,
  ExternalLink,
  Copy,
  Columns2,
  RotateCcw,
  X,
  GripVertical,
  Pause,
  Play,
  Trash2,
} from "lucide-react";
import type { ConversionJob, Snapshot } from "./types";
import { Button, FileIcon, Mark, NumberValue, bytes, time, type L } from "./ui";
export default function Jobs({
  data,
  l,
  compare,
  notify,
  history = false,
}: {
  data: Snapshot;
  l: L;
  compare: (job: ConversionJob) => void;
  notify: (v: string) => void;
  history?: boolean;
}) {
  const pending = data.jobs.filter((j) => j.status === "pending");
  const [order, setOrder] = useState(pending.map((j) => j.id));
  useEffect(() => {
    setOrder(pending.map((j) => j.id));
  }, [pending.map((j) => j.id).join(",")]);
  const control = (action: string, id?: string) =>
    void invoke("control", { action, id }).catch((e) => notify(String(e)));
  const row = (j: ConversionJob) => {
    const active = ["processing", "cancelling"].includes(j.status),
      done = ["completed", "unchanged"].includes(j.status);
    const labels: Record<string, string> = {
      pending: l("Na fila", "Queued"),
      processing: l("Processando", "Processing"),
      cancelling: l("Cancelando…", "Cancelling…"),
      completed: l("Concluído", "Done"),
      unchanged: l(
        "Original mantido · não ficou menor",
        "Original kept · no reduction",
      ),
      cancelled: l("Cancelado", "Cancelled"),
      failed: l("Não foi possível concluir", "Could not finish"),
    };
    return (
      <motion.article layout className={`job ${j.status}`} key={j.id}>
        <div className="job-leading">
          {j.status === "pending" ? (
            <GripVertical size={16} />
          ) : active ? (
            <Mark progress={j.progress ?? 0} />
          ) : (
            <FileIcon category={j.input.category} />
          )}
        </div>
        <div className="job-body">
          <div className="job-heading">
            <strong title={j.input.name}>{j.input.name}</strong>
            <span className="job-status">{labels[j.status]}</span>
          </div>
          <div className="job-measure">
            <span>{bytes(j.input.size)}</span>
            <span>→</span>
            <b>
              {done ? (
                <NumberValue value={j.outputSize ?? 0} />
              ) : (
                j.options.format.toUpperCase()
              )}
            </b>
            {done && (j.outputSize ?? 0) < j.input.size && (
              <span className="saving">
                −{Math.round((1 - (j.outputSize ?? 0) / j.input.size) * 100)}%
              </span>
            )}
            {active && (
              <span>
                {j.progress === null
                  ? l("Aguardando o motor…", "Waiting for engine…")
                  : `${Math.floor(j.progress)}%`}{" "}
                · {time(j.elapsed)} {j.speed && `· ${j.speed}`}{" "}
                {j.remaining !== null &&
                  `· ~${time(j.remaining)} ${l("restantes", "left")}`}
              </span>
            )}
          </div>
          {active && (
            <div
              className={`process-track ${j.progress === null ? "indeterminate" : ""}`}
            >
              <motion.div
                animate={{
                  scaleX:
                    j.progress === null
                      ? 0.18
                      : Math.max(0.01, j.progress / 100),
                }}
              />
            </div>
          )}
          {j.error && <p className="error-text">{messageText(j.error)}</p>}
          <div className="job-actions">
            {done && (
              <>
                <Button
                  onClick={() =>
                    void invoke("open_result", {
                      id: j.id,
                      folder: false,
                    }).catch((e) => notify(String(e)))
                  }
                >
                  <ExternalLink size={14} />
                  {l("Abrir", "Open")}
                </Button>
                <Button
                  title={l("Mostrar na pasta", "Show in folder")}
                  onClick={() =>
                    void invoke("open_result", {
                      id: j.id,
                      folder: true,
                    }).catch((e) => notify(String(e)))
                  }
                >
                  <FolderOpen size={15} />
                </Button>
                <Button
                  title={l("Copiar caminho", "Copy path")}
                  onClick={() =>
                    void navigator.clipboard
                      .writeText(j.output ?? "")
                      .then(() => notify(l("Caminho copiado", "Path copied")))
                      .catch((e) => notify(String(e)))
                  }
                >
                  <Copy size={14} />
                </Button>
                {["image", "video", "pdf"].includes(j.input.category) &&
                  j.output &&
                  !j.options.mergePaths.length &&
                  !(j.input.format === "pdf" && j.options.format !== "pdf") && (
                    <Button onClick={() => compare(j)}>
                      <Columns2 size={14} />
                      {l("Comparar", "Compare")}
                    </Button>
                  )}
              </>
            )}
            {!history && ["failed", "cancelled"].includes(j.status) && (
              <Button onClick={() => control("retry", j.id)}>
                <RotateCcw size={14} />
                {l("Tentar novamente", "Retry")}
              </Button>
            )}
            {j.diagnostic && (
              <details>
                <summary>{l("Detalhes", "Details")}</summary>
                <pre>{j.diagnostic}</pre>
              </details>
            )}
            {!history && ["pending", "processing"].includes(j.status) && (
              <Button
                title={l("Cancelar arquivo", "Cancel file")}
                onClick={() => control("cancel", j.id)}
              >
                <X size={15} />
              </Button>
            )}
          </div>
        </div>
      </motion.article>
    );
  };
  return (
    <div className="jobs">
      <div className="queue-toolbar">
        <div>
          <h2>
            {history
              ? l("Histórico", "History")
              : l("Seus arquivos, em movimento.", "Your files, in motion.")}
          </h2>
          <p>
            {history
              ? l(
                  "Resultados salvos neste computador.",
                  "Results saved on this computer.",
                )
              : l(
                  "Você pode continuar adicionando arquivos.",
                  "You can keep adding files.",
                )}
          </p>
        </div>
        {!history && (
          <div className="toolbar-actions">
            <Button onClick={() => control(data.paused ? "resume" : "pause")}>
              {data.paused ? <Play size={15} /> : <Pause size={15} />}{" "}
              {data.paused
                ? l("Retomar", "Resume")
                : l("Pausar fila", "Pause queue")}
            </Button>
            <Button
              onClick={() => control("retryFailed")}
              title={l("Repetir falhas", "Retry failed")}
            >
              <RotateCcw size={16} />
            </Button>
            <Button
              onClick={() => control("cancelAll")}
              title={l("Cancelar todos", "Cancel all")}
            >
              <X size={16} />
            </Button>
            <Button
              onClick={() => control("clear")}
              title={l("Limpar concluídos", "Clear finished")}
            >
              <Trash2 size={16} />
            </Button>
          </div>
        )}
      </div>
      {data.paused && !history && (
        <p className="notice">
          {l(
            "Fila pausada. Os arquivos pendentes aguardam sua ação.",
            "Queue paused. Pending files are waiting for you.",
          )}
        </p>
      )}
      {history ? (
        data.history.map(row)
      ) : (
        <>
          {data.jobs
            .filter((j) => ["processing", "cancelling"].includes(j.status))
            .map(row)}
          <Reorder.Group
            axis="y"
            values={order}
            onReorder={setOrder}
            className="pending-list"
          >
            {order.map((id) => {
              const j = pending.find((j) => j.id === id);
              return j ? (
                <Reorder.Item
                  key={id}
                  value={id}
                  onDragEnd={() =>
                    void invoke("reorder", { ids: order }).catch((e) =>
                      notify(String(e)),
                    )
                  }
                >
                  {row(j)}
                </Reorder.Item>
              ) : null;
            })}
          </Reorder.Group>
          {[...data.jobs]
            .reverse()
            .filter(
              (j) =>
                !["pending", "processing", "cancelling"].includes(j.status),
            )
            .map(row)}
        </>
      )}
      {(history ? data.history : data.jobs).length === 0 && (
        <div className="quiet-empty">
          <Mark />
          <p>{l("Nada por aqui ainda.", "Nothing here yet.")}</p>
        </div>
      )}
    </div>
  );
}
