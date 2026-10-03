export interface InputFile {
  media?: {
    codec: string | null;
    audioCodec: string | null;
    bitrate: number | null;
    fps: number | null;
    hdr: boolean;
    audioStreams: number;
    subtitleCodecs: string[];
    pages: number | null;
    frames: number | null;
    alpha: boolean;
  };
  path: string;
  name: string;
  format: string;
  category: string;
  size: number;
  duration: number | null;
  width: number | null;
  height: number | null;
  sampleRate: string | null;
  channels: number | null;
  bitDepth: number | null;
}
export interface FormatCapability {
  archiveOutputs: string[];
  input: string;
  outputs: string[];
  engine: string;
  category: string;
  losslessOutputs: string[];
  targetSize: boolean;
  metadata: boolean;
  layout: string;
  advanced: string[];
}
export interface ConversionOptions {
  suffix: string;
  remux: boolean;
  mergePaths: string[];
  keepArtwork: boolean;
  format: string;
  preset: string;
  mode: string;
  outputDir: string;
  quality: number;
  width: number | null;
  height: number | null;
  percent: number | null;
  keepAspect: boolean;
  keepMetadata: boolean;
  keepAlpha: boolean;
  background: string;
  dpi: number;
  targetBytes: number | null;
  bitrate: number;
  audioBitrate: number;
  sampleRate: number | null;
  channels: number | null;
  bitDepth: number | null;
  normalize: boolean;
  fps: number | null;
  codec: string;
  encoder: string;
  crf: number;
  threads: number;
  subtitles: boolean;
  chapters: boolean;
  archiveLevel: number;
  password: string;
  archiveAction: string;
  lossless: boolean;
}
export interface ConversionJob {
  id: string;
  input: InputFile;
  options: ConversionOptions;
  status: string;
  progress: number | null;
  speed: string | null;
  elapsed: number;
  remaining: number | null;
  output: string | null;
  outputSize: number | null;
  error: string | null;
  diagnostic: string | null;
  created: number;
}
export interface Settings {
  theme: string;
  language: string;
  outputDir: string;
  keepMetadata: boolean;
  concurrency: number;
  historyEnabled: boolean;
  tempDir: string;
}
export interface EngineCapability {
  id: string;
  ready: boolean;
  version: string;
  integrity: boolean;
}
export interface Snapshot {
  presets?: SavedPreset[];
  hardware: string[];
  jobs: ConversionJob[];
  history: ConversionJob[];
  settings: Settings;
  paused: boolean;
  engines: EngineCapability[];
  registry: FormatCapability[];
}
export interface Draft {
  file: InputFile;
  options: ConversionOptions;
}
export const defaults: ConversionOptions = {
  suffix: "",
  remux: false,
  mergePaths: [],
  keepArtwork: true,
  format: "webp",
  preset: "high",
  mode: "convert",
  outputDir: "",
  quality: 88,
  width: null,
  height: null,
  percent: null,
  keepAspect: true,
  keepMetadata: true,
  keepAlpha: true,
  background: "#ffffff",
  dpi: 96,
  targetBytes: null,
  bitrate: 192,
  audioBitrate: 128,
  sampleRate: null,
  channels: null,
  bitDepth: null,
  normalize: false,
  fps: null,
  codec: "auto",
  encoder: "software",
  crf: 23,
  threads: 2,
  subtitles: false,
  chapters: true,
  archiveLevel: 5,
  password: "",
  archiveAction: "repack",
  lossless: false,
};
export interface SavedPreset {
  id: string;
  name: string;
  category: string;
  favorite: boolean;
  options: ConversionOptions;
}
export interface Plan {
  options: ConversionOptions;
  estimate: { low: number; high: number; method: string };
  warnings: string[];
  recommendations: string[];
}
