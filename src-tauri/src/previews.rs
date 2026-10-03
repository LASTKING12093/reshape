//! Bounded previews of real local files. Never sends source files over the network.
use crate::{
    engines::EngineManager,
    models::InputFile,
    process::{self, Cancel},
};
use base64::Engine;
use std::{fs, path::Path};
pub fn render(
    f: &InputFile,
    position: f64,
    engines: &EngineManager,
    root: &Path,
) -> Result<String, String> {
    let _lock = crate::estimation::PREVIEW_WORKER
        .lock()
        .map_err(|_| "Preview worker unavailable")?;
    if !position.is_finite() || position < 0. {
        return Err("Invalid preview position".into());
    }
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let temp = tempfile::Builder::new()
        .prefix("reshape-preview-")
        .tempdir_in(root)
        .map_err(|e| e.to_string())?;
    let src = temp.path().join(format!("source.{}", f.format));
    // Neutral names for ImageMagick's path syntax; links avoid copying large sources.
    if fs::hard_link(&f.path, &src).is_err() {
        if f.size > 256 * 1024 * 1024 {
            return Err(
                "Prévia indisponível entre volumes para arquivos maiores que 256 MB.".into(),
            );
        }
        fs::copy(&f.path, &src).map_err(|e| e.to_string())?;
    }
    let output = temp.path().join("preview.png");
    let source = src.to_string_lossy().to_string();
    let dest = output.to_string_lossy().to_string();
    let (engine, args): (&str, Vec<String>) = if f.category == "image" {
        (
            "imagemagick",
            vec![
                "-limit".into(),
                "memory".into(),
                "128MiB".into(),
                "-limit".into(),
                "map".into(),
                "256MiB".into(),
                format!("{source}[0]"),
                "-auto-orient".into(),
                "-thumbnail".into(),
                "1600x1200>".into(),
                "-colorspace".into(),
                "sRGB".into(),
                dest,
            ],
        )
    } else if f.format == "pdf" {
        let page = (position as u32).max(1).min(f.media.pages.unwrap_or(1));
        (
            "pdf",
            vec![
                "-f".into(),
                page.to_string(),
                "-l".into(),
                page.to_string(),
                "-singlefile".into(),
                "-scale-to".into(),
                "1600".into(),
                "-png".into(),
                source,
                temp.path().join("preview").to_string_lossy().into(),
            ],
        )
    } else if f.category == "video" {
        (
            "ffmpeg",
            vec![
                "-v".into(),
                "error".into(),
                "-nostdin".into(),
                "-protocol_whitelist".into(),
                "file,pipe,data,crypto".into(),
                "-ss".into(),
                position
                    .min(f.duration.unwrap_or(1.).max(0.1) - 0.1)
                    .to_string(),
                "-i".into(),
                source,
                "-frames:v".into(),
                "1".into(),
                "-vf".into(),
                "scale=1280:720:force_original_aspect_ratio=decrease".into(),
                dest,
            ],
        )
    } else {
        return Err("Este tipo de arquivo não tem prévia visual.".into());
    };
    process::run_limited(
        &engines.executable(engine)?,
        &args,
        temp.path(),
        &Cancel::default(),
        None,
        30,
        |_, _| {},
    )?;
    let bytes = fs::read(output).map_err(|e| e.to_string())?;
    if bytes.len() > 12 * 1024 * 1024 {
        return Err("Preview exceeds memory limit".into());
    }
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
