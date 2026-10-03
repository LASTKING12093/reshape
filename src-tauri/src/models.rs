use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputFile {
    pub path: String,
    pub name: String,
    pub format: String,
    pub category: String,
    pub size: u64,
    pub duration: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub sample_rate: Option<String>,
    pub channels: Option<u32>,
    pub bit_depth: Option<u32>,
    #[serde(default)]
    pub media: MediaDetails,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MediaDetails {
    pub codec: Option<String>,
    pub audio_codec: Option<String>,
    pub bitrate: Option<u64>,
    pub fps: Option<f64>,
    pub hdr: bool,
    pub audio_streams: u32,
    pub subtitle_codecs: Vec<String>,
    pub pages: Option<u32>,
    pub frames: Option<u32>,
    pub alpha: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConversionOptions {
    pub format: String,
    pub preset: String,
    pub mode: String,
    pub output_dir: String,
    pub quality: u8,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub percent: Option<u32>,
    pub keep_aspect: bool,
    pub keep_metadata: bool,
    pub keep_alpha: bool,
    pub background: String,
    pub dpi: u32,
    pub target_bytes: Option<u64>,
    pub bitrate: u32,
    pub audio_bitrate: u32,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub bit_depth: Option<u8>,
    pub normalize: bool,
    pub fps: Option<u32>,
    pub codec: String,
    pub encoder: String,
    pub crf: u8,
    pub threads: u8,
    pub subtitles: bool,
    pub chapters: bool,
    pub archive_level: u8,
    pub password: String,
    pub archive_action: String,
    pub lossless: bool,
    pub merge_paths: Vec<String>,
    pub keep_artwork: bool,
    pub suffix: String,
    pub remux: bool,
}
impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            format: "webp".into(),
            preset: "high".into(),
            mode: "convert".into(),
            output_dir: String::new(),
            quality: 88,
            width: None,
            height: None,
            percent: None,
            keep_aspect: true,
            keep_metadata: true,
            keep_alpha: true,
            background: "#ffffff".into(),
            dpi: 96,
            target_bytes: None,
            bitrate: 192,
            audio_bitrate: 128,
            sample_rate: None,
            channels: None,
            bit_depth: None,
            normalize: false,
            fps: None,
            codec: "auto".into(),
            encoder: "software".into(),
            crf: 23,
            threads: 2,
            subtitles: false,
            chapters: true,
            archive_level: 5,
            password: String::new(),
            archive_action: "repack".into(),
            lossless: false,
            merge_paths: Vec::new(),
            keep_artwork: true,
            suffix: String::new(),
            remux: false,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionJob {
    pub id: String,
    pub input: InputFile,
    pub options: ConversionOptions,
    pub status: String,
    pub progress: Option<f64>,
    pub speed: Option<String>,
    pub elapsed: f64,
    pub remaining: Option<f64>,
    pub output: Option<String>,
    pub output_size: Option<u64>,
    pub error: Option<String>,
    pub diagnostic: Option<String>,
    pub created: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub theme: String,
    pub language: String,
    pub output_dir: String,
    pub keep_metadata: bool,
    pub concurrency: usize,
    pub history_enabled: bool,
    pub temp_dir: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            language: "en".into(),
            output_dir: String::new(),
            keep_metadata: true,
            concurrency: 2,
            history_enabled: true,
            temp_dir: String::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineCapability {
    pub id: String,
    pub ready: bool,
    pub version: String,
    pub integrity: bool,
}

#[cfg(test)]
mod language_tests {
    use super::Settings;

    #[test]
    fn english_is_default_and_saved_portuguese_survives_migration() {
        assert_eq!(Settings::default().language, "en");
        let old: Settings = serde_json::from_str(r#"{"theme":"dark"}"#).unwrap();
        assert_eq!(old.language, "en");
        let saved: Settings = serde_json::from_str(r#"{"language":"pt"}"#).unwrap();
        assert_eq!(saved.language, "pt");
        let restored: Settings =
            serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap();
        assert_eq!(restored.language, "pt");
    }
}
