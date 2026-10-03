use crate::models::ConversionOptions;
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub category: String,
    pub favorite: bool,
    pub options: ConversionOptions,
}
pub fn sanitize(mut presets: Vec<Preset>) -> Result<Vec<Preset>, String> {
    if presets.len() > 100 {
        return Err("Máximo de 100 presets".into());
    }
    let mut ids = std::collections::HashSet::new();
    for p in &mut presets {
        if p.name.trim().is_empty() || p.name.chars().count() > 80 || !ids.insert(p.id.clone()) {
            return Err("Nome ou identificador de preset inválido".into());
        }
        p.name = p.name.trim().into();
        // Secrets and file-specific destinations never become reusable settings.
        p.options.password.clear();
        p.options.merge_paths.clear();
        p.options.output_dir.clear();
        let d = ConversionOptions::default();
        if !["image", "video"].contains(&p.category.as_str()) {
            p.options.width = None;
            p.options.height = None;
            p.options.percent = None;
        }
        if p.category != "video" {
            p.options.encoder = d.encoder;
            p.options.codec = d.codec;
            p.options.crf = d.crf;
            p.options.fps = None;
            p.options.remux = false;
        }
        if !["video", "audio"].contains(&p.category.as_str()) {
            p.options.bitrate = d.bitrate;
            p.options.audio_bitrate = d.audio_bitrate;
            p.options.normalize = false;
            p.options.channels = None;
            p.options.sample_rate = None;
            p.options.bit_depth = None;
        }
        crate::conversion::validate_options(&p.options)?;
    }
    Ok(presets)
}
