import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { AnimatePresence, animate, motion } from "motion/react";
import {
  Archive,
  FileAudio,
  FileImage,
  FileText,
  FileVideo,
  X,
  ChevronDown,
  Check,
} from "lucide-react";
export type L = (pt: string, en: string) => string;
export const bytes = (n: number) => {
  if (!Number.isFinite(n) || n <= 0) return "0 B";
  const u = Math.min(4, Math.floor(Math.log(n) / Math.log(1024)));
  return `${(n / 1024 ** u).toLocaleString(document.documentElement.lang || "en", { maximumFractionDigits: u ? 1 : 0 })} ${["B", "KB", "MB", "GB", "TB"][u]}`;
};
export const time = (n: number) =>
  `${Math.floor(n / 60)}:${Math.floor(n % 60)
    .toString()
    .padStart(2, "0")}`;
export function NumberValue({
  value,
  format = bytes,
}: {
  value: number;
  format?: (n: number) => string;
}) {
  const [n, set] = useState(value);
  const old = useRef(value);
  useEffect(() => {
    const a = animate(old.current, value, {
      duration: 0.32,
      onUpdate: (v) => {
        old.current = v;
        set(v);
      },
    });
    return () => a.stop();
  }, [value]);
  return <>{format(n)}</>;
}
export function Mark({
  large = false,
  progress,
}: {
  large?: boolean;
  progress?: number;
}) {
  const id = useId();
  return (
    <motion.svg
      className={large ? "mark large" : "mark"}
      viewBox="0 0 64 64"
      fill="none"
      aria-label="Reshape"
      whileHover={{ rotate: 8, scale: 1.04 }}
    >
      <defs>
        <linearGradient id={`${id}-warm`} x1="0" y1="0" x2="1" y2="1">
          <stop stopColor="#ffb49f" />
          <stop offset=".48" stopColor="#ff756f" />
          <stop offset="1" stopColor="#d64574" />
        </linearGradient>
        <linearGradient id={`${id}-cool`} x1="0" y1="0" x2="1" y2="1">
          <stop stopColor="#b8b8b8" />
          <stop offset=".5" stopColor="#666666" />
          <stop offset="1" stopColor="#202020" />
        </linearGradient>
      </defs>
      <motion.path
        fill={`url(#${id}-warm)`}
        d="M13 12C23 3 40 4 50 12C59 19 57 29 50 36L40 46L31 37L41 27C45 23 43 20 39 18C33 15 25 18 21 22L13 30C6 25 6 18 13 12Z"
        animate={{ y: progress === undefined ? 0 : progress * -0.015 }}
      />
      <motion.path
        fill={`url(#${id}-cool)`}
        d="M51 52C41 61 24 60 14 52C5 45 7 35 14 28L24 18L33 27L23 37C19 41 21 44 25 46C31 49 39 46 43 42L51 34C58 39 58 46 51 52Z"
        animate={{ y: progress === undefined ? 0 : progress * 0.015 }}
      />
    </motion.svg>
  );
}
export function FileIcon({ category }: { category: string }) {
  const C =
    category === "image"
      ? FileImage
      : category === "video"
        ? FileVideo
        : category === "audio"
          ? FileAudio
          : category === "archive"
            ? Archive
            : FileText;
  return <C size={23} strokeWidth={1.4} />;
}
export function Button({
  children,
  onClick,
  primary = false,
  disabled = false,
  title,
  className = "",
}: {
  children: ReactNode;
  onClick?: () => void;
  primary?: boolean;
  disabled?: boolean;
  title?: string;
  className?: string;
}) {
  return (
    <motion.button
      type="button"
      whileTap={disabled ? {} : { scale: 0.965, y: 1 }}
      whileHover={disabled ? {} : { y: -1.5 }}
      transition={{ type: "spring", stiffness: 520, damping: 30, mass: 0.55 }}
      className={`${primary ? "primary" : "button"} ${className}`}
      onClick={onClick}
      disabled={disabled}
      title={title}
      aria-label={title}
    >
      {children}
    </motion.button>
  );
}
export function Modal({
  title,
  children,
  close,
  wide = false,
}: {
  title: string;
  children: ReactNode;
  close: () => void;
  wide?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const saved = useRef(document.activeElement as HTMLElement | null);
  const closeRef = useRef(close);
  closeRef.current = close;
  useEffect(() => {
    ref.current?.focus();
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") closeRef.current();
      if (e.key === "Tab") {
        const items = ref.current?.querySelectorAll<HTMLElement>(
          'button:not(:disabled),input,select,textarea,[tabindex="0"]',
        );
        if (!items?.length) return;
        const first = items[0],
          last = items[items.length - 1];
        if (
          e.shiftKey &&
          (document.activeElement === first ||
            document.activeElement === ref.current)
        ) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener("keydown", handler);
    return () => {
      document.removeEventListener("keydown", handler);
      saved.current?.focus();
    };
  }, []);
  return (
    <motion.div
      className="scrim"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.16 }}
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) close();
      }}
    >
      <motion.div
        ref={ref}
        tabIndex={-1}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        className={`modal ${wide ? "wide" : ""}`}
        initial={{ y: 18, scale: 0.975 }}
        animate={{ y: 0, scale: 1 }}
        exit={{ y: 12, opacity: 0 }}
      >
        <header>
          <h2>{title}</h2>
          <Button
            title={document.documentElement.lang === "en" ? "Close" : "Fechar"}
            onClick={close}
          >
            <X size={19} />
          </Button>
        </header>
        {children}
      </motion.div>
    </motion.div>
  );
}
export function Picker({
  value,
  items,
  onChange,
  label,
}: {
  value: string;
  items: { value: string; label: string }[];
  onChange: (v: string) => void;
  label: string;
}) {
  const [open, set] = useState(false);
  return (
    <div
      className="picker"
      onKeyDown={(e) => {
        if (e.key === "Escape" && open) {
          e.stopPropagation();
          set(false);
        }
      }}
    >
      <Button onClick={() => set(!open)} title={label}>
        <span>{items.find((i) => i.value === value)?.label ?? value}</span>
        <ChevronDown size={14} />
      </Button>
      <AnimatePresence>
        {open && (
          <motion.div
            key="options"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.12 }}
          >
            <button
              className="picker-away"
              aria-label={
                document.documentElement.lang === "en"
                  ? "Close options"
                  : "Fechar opções"
              }
              onClick={() => set(false)}
            />
            <motion.div
              className="picker-menu"
              role="group"
              aria-label={label}
              initial={{ opacity: 0, y: -4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -4 }}
              transition={{ type: "spring", stiffness: 480, damping: 32 }}
            >
              {items.map((i) => (
                <button
                  key={i.value}
                  onClick={() => {
                    onChange(i.value);
                    set(false);
                  }}
                >
                  {i.label}
                  {i.value === value && <Check size={14} />}
                </button>
              ))}
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
export function Toggle({
  label,
  value,
  onChange,
}: {
  label: string;
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="toggle">
      <span>{label}</span>
      <input
        type="checkbox"
        checked={value}
        onChange={(e) => onChange(e.target.checked)}
      />
      <motion.span
        className="switch"
        aria-hidden="true"
        animate={{
          backgroundColor: value ? "var(--switch-on)" : "var(--line)",
        }}
        transition={{ duration: 0.18 }}
      >
        <motion.span
          className="switch-thumb"
          animate={{ x: value ? 18 : 0, scale: value ? 1 : 0.9 }}
          transition={{
            type: "spring",
            stiffness: 650,
            damping: 30,
            mass: 0.45,
          }}
        >
          <motion.span
            animate={{ opacity: value ? 1 : 0, scale: value ? 1 : 0.5 }}
          >
            <Check size={10} strokeWidth={3} />
          </motion.span>
        </motion.span>
      </motion.span>
    </label>
  );
}
